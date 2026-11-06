// guard.rs
// Copyright © 2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Request checks the HTTP+SSE transport applies before it serves
//! anything.
//!
//! A page in a browser can reach a server on the loopback interface
//! by rebinding its own name to `127.0.0.1`. The browser then sends
//! that name in `Host`, and the page's own origin in `Origin`. So a
//! request is served only when its `Host` is one the server answers
//! to, any `Origin` it carries is local, and a posted message is
//! `application/json`, which a page cannot send cross-origin without
//! a preflight the server never answers.
//!
//! Ported from noyalib-mcp (commit 9e13b31).

use axum::http::header::{CONTENT_TYPE, HOST, ORIGIN};
use axum::http::uri::Authority;
use axum::http::{HeaderMap, StatusCode, Uri};

/// Why a request was refused: the status and a short reason, which
/// `IntoResponse` turns into the reply.
pub(super) type Refusal = (StatusCode, &'static str);

/// The names a server on the loopback interface answers to.
pub(super) const LOOPBACK_HOSTS: [&str; 3] = ["localhost", "127.0.0.1", "::1"];

/// The `Host` names a server bound to `bind_host` answers to, or
/// `None` when it binds every interface and there is no name to
/// check against.
pub(super) fn allowed_hosts(bind_host: &str) -> Option<Vec<String>> {
    if bind_host == "0.0.0.0" || bind_host == "::" {
        return None;
    }
    let mut hosts: Vec<String> = LOOPBACK_HOSTS.iter().map(|h| (*h).to_owned()).collect();
    let bind = normalize(bind_host);
    if !hosts.contains(&bind) {
        hosts.push(bind);
    }
    Some(hosts)
}

/// The browser origins a server bound to `bind_host` accepts: the
/// loopback names on any port, plus the bound name when it is a
/// specific one. In the form the SDK's `allowed_origins` takes.
pub(super) fn allowed_origins(bind_host: &str) -> Vec<String> {
    origin_names(bind_host)
        .iter()
        .flat_map(|name| {
            let name = if name.contains(':') {
                format!("[{name}]")
            } else {
                name.clone()
            };
            [format!("http://{name}:*"), format!("https://{name}:*")]
        })
        .collect()
}

/// The host names an `Origin` may carry: the `Host` names, or the
/// loopback ones when every interface is bound.
fn origin_names(bind_host: &str) -> Vec<String> {
    allowed_hosts(bind_host)
        .unwrap_or_else(|| LOOPBACK_HOSTS.iter().map(|h| (*h).to_owned()).collect())
}

/// The checks one listener applies.
#[derive(Debug, Clone)]
pub(super) struct Guard {
    /// `None` accepts any `Host`.
    hosts: Option<Vec<String>>,
    /// The names an `Origin` may carry: loopback, and the bound name.
    origins: Vec<String>,
}

impl Guard {
    /// The guard for a listener bound to `bind_host`.
    pub(super) fn for_bind_host(bind_host: &str) -> Self {
        Self {
            hosts: allowed_hosts(bind_host),
            origins: origin_names(bind_host),
        }
    }

    /// Refuse a request whose `Host` or `Origin` is foreign.
    pub(super) fn check(&self, headers: &HeaderMap) -> Result<(), Refusal> {
        if let Some(hosts) = &self.hosts {
            let host = headers
                .get(HOST)
                .and_then(|v| v.to_str().ok())
                .and_then(host_name);
            if !host.is_some_and(|h| hosts.contains(&h)) {
                return Err(forbidden("Host header is not allowed"));
            }
        }
        let Some(origin) = headers.get(ORIGIN) else {
            return Ok(());
        };
        let name = origin.to_str().ok().and_then(origin_host);
        if name.is_some_and(|n| self.origins.contains(&n)) {
            Ok(())
        } else {
            Err(forbidden("Origin header is not allowed"))
        }
    }
}

/// Refuse a posted message that is not `application/json`.
pub(super) fn require_json(headers: &HeaderMap) -> Result<(), Refusal> {
    let media = headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(';').next())
        .map(str::trim);
    if media.is_some_and(|m| m.eq_ignore_ascii_case("application/json")) {
        Ok(())
    } else {
        Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Content-Type must be application/json",
        ))
    }
}

fn forbidden(message: &'static str) -> Refusal {
    (StatusCode::FORBIDDEN, message)
}

/// `example.com:8080` or `[::1]:80` to its lowercase host name.
fn host_name(value: &str) -> Option<String> {
    Authority::try_from(value.trim())
        .ok()
        .map(|a| normalize(a.host()))
}

/// `http://example.com:8080` to its lowercase host name. `null` and
/// anything without a host have none.
fn origin_host(value: &str) -> Option<String> {
    let uri = Uri::try_from(value.trim()).ok()?;
    uri.scheme_str()?;
    uri.host().map(normalize)
}

fn normalize(host: &str) -> String {
    host.trim_matches(['[', ']']).to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            let _ = map.insert(*name, value.parse().expect("header"));
        }
        map
    }

    #[test]
    fn loopback_hosts_and_origins_pass() {
        let guard = Guard::for_bind_host("127.0.0.1");
        for host in ["127.0.0.1:8000", "localhost", "[::1]:1", "LOCALHOST:9"] {
            assert!(guard.check(&headers(&[("host", host)])).is_ok(), "{host}");
        }
        let local = headers(&[("host", "localhost"), ("origin", "http://[::1]:3000")]);
        assert!(guard.check(&local).is_ok());
    }

    #[test]
    fn foreign_hosts_and_origins_are_refused() {
        let guard = Guard::for_bind_host("127.0.0.1");
        for bad in [
            headers(&[("host", "evil.example")]),
            headers(&[]),
            headers(&[("host", "localhost"), ("origin", "http://evil.example")]),
            headers(&[("host", "localhost"), ("origin", "null")]),
            headers(&[("host", "localhost"), ("origin", "localhost")]),
        ] {
            assert!(guard.check(&bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn binding_every_interface_skips_the_host_check_but_not_origin() {
        let guard = Guard::for_bind_host("0.0.0.0");
        assert!(guard.check(&headers(&[("host", "box.lan")])).is_ok());
        let page = headers(&[("host", "box.lan"), ("origin", "http://evil.example")]);
        assert!(guard.check(&page).is_err());
    }

    #[test]
    fn a_named_bind_host_is_allowed_as_host_and_origin() {
        let guard = Guard::for_bind_host("Box.LAN");
        let ok = headers(&[("host", "box.lan:8000"), ("origin", "https://box.lan")]);
        assert!(guard.check(&ok).is_ok());
        assert!(allowed_origins("box.lan").contains(&"http://box.lan:*".to_owned()));
        assert!(allowed_origins("::").contains(&"http://[::1]:*".to_owned()));
    }

    #[test]
    fn only_json_is_accepted() {
        assert!(require_json(&headers(&[("content-type", "application/json")])).is_ok());
        assert!(
            require_json(&headers(&[(
                "content-type",
                "Application/JSON; charset=utf-8"
            )]))
            .is_ok()
        );
        assert!(require_json(&headers(&[("content-type", "text/plain")])).is_err());
        assert!(require_json(&headers(&[])).is_err());
    }
}
