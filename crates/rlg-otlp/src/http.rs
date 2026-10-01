// http.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The minimal HTTP/1.1 client protocol the async exporter needs,
//! with no I/O of its own.
//!
//! An OTLP export is one `POST` whose only interesting answer is the
//! status code, so the exchange is kept as small as it can be: one
//! connection per request with `Connection: close`, a
//! `Content-Length` body, and a response read only as far as its
//! status line. There is no TLS. The exporter talks plain `http://`
//! to a local OpenTelemetry Collector or sidecar, which owns TLS and
//! authentication towards the backend.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::io;

/// The most response header bytes read before giving up on a
/// collector that never finishes its status line and headers.
pub(crate) const MAX_RESPONSE_HEAD: usize = 16 * 1024;

/// Headers the client writes itself; a caller may not set them.
const RESERVED_HEADERS: [&str; 5] = [
    "connection",
    "content-length",
    "content-type",
    "host",
    "transfer-encoding",
];

/// A parsed `http://host[:port][/path]` collector address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Endpoint {
    /// Host name or IP address to connect to, without brackets.
    pub(crate) host: String,
    /// TCP port, 80 when the URL gives none.
    pub(crate) port: u16,
    /// `host[:port]` as written in the URL, for the `Host` header.
    authority: String,
    /// Request target: the path and query, `/` when empty.
    path: String,
}

impl Endpoint {
    /// Parse a collector URL. Only the `http` scheme is accepted.
    ///
    /// # Errors
    /// Returns a description of the problem for an `https` URL, a
    /// missing or unsupported scheme, user info, an empty host, or a
    /// bad port.
    pub(crate) fn parse(url: &str) -> Result<Self, String> {
        let rest = strip_http_scheme(url)?;
        let (authority, path) = rest
            .find(['/', '?'])
            .map_or((rest, "/"), |i| rest.split_at(i));
        if authority.contains('@') {
            return Err(format!(
                "{url}: user info is not supported; pass credentials as headers"
            ));
        }
        let (host, port) = split_host_port(authority)
            .ok_or_else(|| format!("{url}: invalid host or port"))?;
        let path = if path.starts_with('?') {
            format!("/{path}")
        } else {
            path.to_string()
        };
        Ok(Self {
            host,
            port,
            authority: authority.to_string(),
            path,
        })
    }
}

fn strip_http_scheme(url: &str) -> Result<&str, String> {
    let lower = url.get(..8).unwrap_or(url).to_ascii_lowercase();
    if lower.starts_with("https://") {
        return Err(format!(
            "{url}: https is not supported; send to a local \
             OpenTelemetry Collector over http and let it handle TLS"
        ));
    }
    if !lower.starts_with("http://") {
        return Err(format!(
            "{url}: the endpoint must start with http://"
        ));
    }
    Ok(&url[7..])
}

/// Split `host`, `host:port`, `[v6]` or `[v6]:port`.
fn split_host_port(authority: &str) -> Option<(String, u16)> {
    let (host, port) = match authority.strip_prefix('[') {
        Some(v6) => split_bracketed(v6)?,
        None => authority
            .split_once(':')
            .map_or((authority, None), |(h, p)| (h, Some(p))),
    };
    if host.is_empty() {
        return None;
    }
    let port = port.map_or(Some(80), |p| p.parse().ok())?;
    Some((host.to_string(), port))
}

/// Split `v6]` or `v6]:port`, the part of an authority after `[`.
fn split_bracketed(v6: &str) -> Option<(&str, Option<&str>)> {
    let (host, after) = v6.split_once(']')?;
    if after.is_empty() {
        return Some((host, None));
    }
    after.strip_prefix(':').map(|port| (host, Some(port)))
}

/// Reject a caller header that is not a valid field name, carries a
/// line break or NUL in its value, or is one the client writes.
///
/// # Errors
/// Returns a description of the first offending header.
pub(crate) fn validate_headers(
    headers: &HashMap<String, String>,
) -> Result<(), String> {
    for (name, value) in headers {
        if name.is_empty() || !name.bytes().all(is_token_byte) {
            return Err(format!("invalid header name {name:?}"));
        }
        if RESERVED_HEADERS
            .contains(&name.to_ascii_lowercase().as_str())
        {
            return Err(format!(
                "header {name:?} is set by the exporter"
            ));
        }
        if value.bytes().any(|b| matches!(b, b'\r' | b'\n' | 0)) {
            return Err(format!(
                "header {name:?} has a line break in its value"
            ));
        }
    }
    Ok(())
}

/// `tchar` from RFC 9110 §5.6.2.
const fn is_token_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric()
        || matches!(
            b,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

/// Encode a complete `POST` of a JSON `body` to `endpoint`, ready to
/// write to the socket. `headers` must have passed
/// [`validate_headers`].
pub(crate) fn encode_post(
    endpoint: &Endpoint,
    headers: &HashMap<String, String>,
    body: &str,
) -> Vec<u8> {
    let mut head = String::with_capacity(256);
    let _ = write!(
        head,
        "POST {} HTTP/1.1\r\nHost: {}\r\n\
         Content-Type: application/json\r\nContent-Length: {}\r\n\
         Connection: close\r\nUser-Agent: rlg-otlp/{}\r\n",
        endpoint.path,
        endpoint.authority,
        body.len(),
        env!("CARGO_PKG_VERSION"),
    );
    for (name, value) in headers {
        let _ = write!(head, "{name}: {value}\r\n");
    }
    head.push_str("\r\n");
    let mut request = head.into_bytes();
    request.extend_from_slice(body.as_bytes());
    request
}

/// The final status code of the response in `buf`, once its header
/// block is complete. Interim `1xx` responses are skipped.
///
/// Returns `Ok(None)` while more bytes are needed.
///
/// # Errors
/// Returns [`io::ErrorKind::InvalidData`] for a status line that is
/// not `HTTP/1.x NNN`.
pub(crate) fn parse_status(buf: &[u8]) -> io::Result<Option<u16>> {
    let mut rest = buf;
    while let Some(end) = find_head_end(rest) {
        let status = status_code(&rest[..end])?;
        if !(100..200).contains(&status) {
            return Ok(Some(status));
        }
        rest = &rest[end + 4..];
    }
    Ok(None)
}

fn find_head_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

fn status_code(head: &[u8]) -> io::Result<u16> {
    let line = head.split(|&b| b == b'\r').next().unwrap_or_default();
    let mut parts = line.splitn(3, |&b| b == b' ');
    let version = parts.next().unwrap_or_default();
    let code = parts.next().unwrap_or_default();
    let valid = version.starts_with(b"HTTP/1.")
        && code.len() == 3
        && code.iter().all(u8::is_ascii_digit);
    if !valid {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "malformed HTTP status line from collector",
        ));
    }
    Ok(code.iter().fold(0, |n, d| n * 10 + u16::from(d - b'0')))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(url: &str) -> Endpoint {
        Endpoint::parse(url).unwrap()
    }

    #[test]
    fn parses_host_port_and_path() {
        let e = parse("http://localhost:4318/v1/logs");
        assert_eq!((e.host.as_str(), e.port), ("localhost", 4318));
        assert_eq!(e.authority, "localhost:4318");
        assert_eq!(e.path, "/v1/logs");
    }

    #[test]
    fn defaults_port_and_path() {
        let e = parse("http://collector");
        assert_eq!((e.port, e.path.as_str()), (80, "/"));
        assert_eq!(parse("HTTP://collector?x=1").path, "/?x=1");
    }

    #[test]
    fn parses_bracketed_ipv6() {
        let e = parse("http://[::1]:4318/v1/logs");
        assert_eq!((e.host.as_str(), e.port), ("::1", 4318));
        assert_eq!(e.authority, "[::1]:4318");
        assert_eq!(parse("http://[::1]").port, 80);
    }

    #[test]
    fn rejects_https_with_a_pointer_to_the_collector() {
        let err = Endpoint::parse("https://api.example.com/v1/logs")
            .unwrap_err();
        assert!(err.contains("Collector"), "{err}");
        assert!(Endpoint::parse("HTTPS://x").is_err());
    }

    #[test]
    fn rejects_malformed_endpoints() {
        for url in [
            "localhost:4318",
            "ftp://x",
            "http://",
            "http://:80",
            "http://x:port",
            "http://x:70000",
            "http://user:pw@x",
            "http://[::1",
            "http://[::1]junk",
            "",
        ] {
            assert!(Endpoint::parse(url).is_err(), "{url}");
        }
    }

    #[test]
    fn validates_headers() {
        let ok = HashMap::from([("x-api-key".into(), "k".into())]);
        assert!(validate_headers(&ok).is_ok());
        for (name, value) in [
            ("bad name", "v"),
            ("", "v"),
            ("Content-Length", "1"),
            ("HOST", "evil"),
            ("x-inject", "a\r\nX-Evil: 1"),
            ("x-nul", "a\0b"),
        ] {
            let h = HashMap::from([(name.into(), value.into())]);
            assert!(validate_headers(&h).is_err(), "{name:?}");
        }
    }

    #[test]
    fn encodes_a_framed_post() {
        let e = parse("http://127.0.0.1:4318/v1/logs");
        let h = HashMap::from([("x-api-key".into(), "k".into())]);
        let req = String::from_utf8(encode_post(&e, &h, "{}")).unwrap();
        assert!(req.starts_with("POST /v1/logs HTTP/1.1\r\n"));
        assert!(req.contains("\r\nHost: 127.0.0.1:4318\r\n"));
        assert!(req.contains("\r\nContent-Length: 2\r\n"));
        assert!(req.contains("\r\nConnection: close\r\n"));
        assert!(req.contains("\r\nx-api-key: k\r\n"));
        assert!(req.ends_with("\r\n\r\n{}"));
    }

    #[test]
    fn parses_final_status_after_interim_responses() {
        let resp = b"HTTP/1.1 100 Continue\r\n\r\n\
                     HTTP/1.1 503 Service Unavailable\r\nRetry-After: 1\r\n\r\n";
        assert_eq!(parse_status(resp).unwrap(), Some(503));
        assert_eq!(
            parse_status(b"HTTP/1.0 200 OK\r\n\r\nbody").unwrap(),
            Some(200)
        );
    }

    #[test]
    fn waits_for_a_complete_head() {
        assert_eq!(parse_status(b"").unwrap(), None);
        assert_eq!(parse_status(b"HTTP/1.1 200 OK\r\n").unwrap(), None);
        assert_eq!(
            parse_status(b"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1")
                .unwrap(),
            None
        );
    }

    #[test]
    fn rejects_malformed_status_lines() {
        for resp in [
            &b"SSH-2.0-OpenSSH\r\n\r\n"[..],
            b"HTTP/2 200\r\n\r\n",
            b"HTTP/1.1 20 OK\r\n\r\n",
            b"HTTP/1.1 2x0 OK\r\n\r\n",
            b"\r\n\r\n",
        ] {
            let err = parse_status(resp).unwrap_err();
            assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        }
    }
}
