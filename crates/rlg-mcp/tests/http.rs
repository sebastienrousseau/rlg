// http.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The two HTTP transports.
//!
//! The session tests serve [`rlg_mcp::transport`] in-process — the
//! code the binary runs — on a free port, and talk HTTP/1.1 to it
//! over a plain socket. In-process, the transport's lines count
//! towards coverage, which a spawned binary's cannot. Two tests still run the real binary: one
//! that it starts and announces its address, one that a port in use
//! is reported. The client here is deliberately small and literal — one
//! request, one response, chunked bodies decoded by hand — because
//! what is being checked is the wire format a client will see, and a
//! client library would hide it.

#![allow(missing_docs)]

use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::Duration;

use serde_json::{Value, json};

use rlg_mcp::transport;

const ERROR: &str = r#"{"session_id":2,"time":"t","level":"ERROR","component":"db","description":"boom","format":"JSON","attributes":{}}"#;

fn log_file() -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    writeln!(f, "{ERROR}").unwrap();
    f.flush().unwrap();
    f
}

/// A server running in-process on its own runtime, stopped when
/// dropped.
struct Server {
    addr: String,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

/// Held while a server picks a port and starts listening, so two
/// tests starting at once cannot be handed the same free port.
static STARTING: std::sync::Mutex<()> = std::sync::Mutex::new(());

impl Server {
    fn start(transport: &str) -> Self {
        let _starting = STARTING
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Another process can still take the port between release and
        // bind; a failed bind is reported back and retried.
        for _ in 0..5 {
            if let Some(server) = Self::try_start(transport) {
                return server;
            }
        }
        panic!("no free port for the {transport} server after 5 tries");
    }

    /// Serve on a port the system just handed out and released: the
    /// in-process twin of `--port 0`. `None` if the bind failed.
    fn try_start(transport: &str) -> Option<Self> {
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .and_then(|l| l.local_addr())
            .expect("free port")
            .port();
        let args =
            ["--transport", transport, "--port", &port.to_string()]
                .map(String::from);
        let transport::Command::Serve(options) =
            transport::parse(args).expect("valid arguments")
        else {
            panic!("expected a serve command");
        };
        let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
        let (failed_tx, failed) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            let runtime =
                tokio::runtime::Runtime::new().expect("runtime");
            runtime.block_on(async move {
                tokio::select! {
                    served = transport::serve(&options, rlg_mcp::LogServer::new) => {
                        let _ = failed_tx.send(served);
                    }
                    _ = stopped => {}
                }
            });
        });
        let addr = format!("127.0.0.1:{port}");
        for _ in 0..100 {
            if let Ok(result) = failed.try_recv() {
                let _ = thread.join();
                assert!(result.is_err(), "server stopped on its own");
                return None;
            }
            if TcpStream::connect(&addr).is_ok() {
                return Some(Self {
                    addr,
                    stop: Some(stop),
                    thread: Some(thread),
                });
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("server did not start listening on {addr}");
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// One HTTP/1.1 exchange, with the body read incrementally so an
/// event stream can be consumed as it arrives.
struct Http {
    status: u16,
    headers: Vec<(String, String)>,
    reader: BufReader<TcpStream>,
    chunked: bool,
    remaining: Option<usize>,
    /// Decoded body bytes not yet handed out.
    pending: Vec<u8>,
}

impl Http {
    fn send(
        addr: &str,
        method: &str,
        path: &str,
        headers: &[(&str, &str)],
        body: &str,
    ) -> Self {
        let mut stream = TcpStream::connect(addr).expect("connect");
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .expect("timeout");
        let mut request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n"
        );
        for (name, value) in headers {
            let _ = write!(request, "{name}: {value}\r\n");
        }
        let _ = write!(
            request,
            "Content-Length: {}\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(request.as_bytes()).expect("write");

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        let _ = reader.read_line(&mut line).expect("status line");
        let status: u16 = line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| panic!("bad status line {line:?}"));
        let mut headers = Vec::new();
        loop {
            line.clear();
            let _ = reader.read_line(&mut line).expect("header");
            let trimmed = line.trim_end();
            if trimmed.is_empty() {
                break;
            }
            let (name, value) =
                trimmed.split_once(':').expect("header");
            headers.push((
                name.to_ascii_lowercase(),
                value.trim().to_owned(),
            ));
        }
        let header = |name: &str| {
            headers
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.clone())
        };
        let chunked = header("transfer-encoding")
            .is_some_and(|v| v.contains("chunked"));
        let remaining =
            header("content-length").and_then(|v| v.parse().ok());
        Self {
            status,
            headers,
            reader,
            chunked,
            remaining,
            pending: Vec::new(),
        }
    }

    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// Pull the next piece of body into `pending`. False at the end.
    fn fill(&mut self) -> bool {
        if self.chunked {
            let mut size = String::new();
            let _ =
                self.reader.read_line(&mut size).expect("chunk size");
            let size = usize::from_str_radix(size.trim(), 16)
                .expect("hex size");
            if size == 0 {
                return false;
            }
            let mut chunk = vec![0; size + 2];
            self.reader.read_exact(&mut chunk).expect("chunk");
            chunk.truncate(size);
            self.pending.extend_from_slice(&chunk);
            true
        } else {
            let want = self.remaining.unwrap_or(usize::MAX).min(4096);
            if want == 0 {
                return false;
            }
            let mut buf = vec![0; want];
            let n = self.reader.read(&mut buf).expect("read");
            if n == 0 {
                return false;
            }
            self.pending.extend_from_slice(&buf[..n]);
            if let Some(r) = self.remaining.as_mut() {
                *r -= n;
            }
            true
        }
    }

    /// The whole body, for a response that ends.
    fn body(mut self) -> String {
        while self.fill() {}
        String::from_utf8(self.pending).expect("utf-8")
    }

    /// The next `event` with a `data` field, as (event name, data).
    ///
    /// Keep-alive comments carry no data and are skipped.
    fn next_event(&mut self) -> (String, String) {
        loop {
            let text =
                String::from_utf8_lossy(&self.pending).into_owned();
            if let Some(end) = text.find("\n\n") {
                let block = text[..end].to_owned();
                let _ = self.pending.drain(..end + 2);
                let mut event = "message".to_owned();
                let mut data = Vec::new();
                for line in block.lines() {
                    if let Some(v) = line.strip_prefix("event:") {
                        v.trim().clone_into(&mut event);
                    } else if let Some(v) = line.strip_prefix("data:") {
                        data.push(v.trim().to_owned());
                    }
                }
                // A priming event carries an id and no data; a
                // keep-alive is a comment. Neither is a message.
                if data.iter().any(|d| !d.is_empty()) {
                    return (event, data.join("\n"));
                }
                continue;
            }
            assert!(
                self.fill(),
                "stream ended before an event arrived"
            );
        }
    }

    /// The next JSON-RPC message on the stream.
    fn next_message(&mut self) -> Value {
        let (event, data) = self.next_event();
        assert_eq!(
            event, "message",
            "unexpected event {event}: {data}"
        );
        serde_json::from_str(&data)
            .unwrap_or_else(|e| panic!("{e}: {data}"))
    }
}

const ACCEPT_BOTH: (&str, &str) =
    ("Accept", "application/json, text/event-stream");
const JSON: (&str, &str) = ("Content-Type", "application/json");

fn initialize(version: &str) -> String {
    json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": version, "capabilities": {},
        "clientInfo": {"name": "test", "version": "0"}}})
    .to_string()
}

fn stateless(id: u32, method: &str, params: Value) -> String {
    let mut params = params;
    // Both keys are required by the revision; a request naming only
    // its version is refused.
    params["_meta"] = json!({
        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
        "io.modelcontextprotocol/clientCapabilities": {}
    });
    json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
        .to_string()
}

// --- Streamable HTTP, 2025-11-25: initialize and a session ------------

#[test]
fn streamable_http_holds_a_session_with_a_handshake() {
    let server = Server::start("streamable-http");
    let mut r = Http::send(
        &server.addr,
        "POST",
        "/mcp",
        &[ACCEPT_BOTH, JSON],
        &initialize("2025-11-25"),
    );
    assert_eq!(r.status, 200);
    assert!(
        r.header("content-type")
            .is_some_and(|c| c.starts_with("text/event-stream")),
        "{:?}",
        r.headers
    );
    let session =
        r.header("mcp-session-id").expect("session id").to_owned();
    let init = r.next_message();
    assert_eq!(init["id"], 1);
    assert_eq!(init["result"]["protocolVersion"], "2025-11-25");
    assert_eq!(init["result"]["serverInfo"]["name"], "rlg-mcp");
    drop(r);

    let session_header = ("Mcp-Session-Id", session.as_str());
    let version_header = ("MCP-Protocol-Version", "2025-11-25");
    let r = Http::send(
        &server.addr,
        "POST",
        "/mcp",
        &[ACCEPT_BOTH, JSON, session_header, version_header],
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    );
    assert_eq!(r.status, 202);

    let mut r = Http::send(
        &server.addr,
        "POST",
        "/mcp",
        &[ACCEPT_BOTH, JSON, session_header, version_header],
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
    );
    assert_eq!(r.status, 200);
    let list = r.next_message();
    let tools = list["result"]["tools"].as_array().expect("tools");
    assert_eq!(tools.len(), 4, "{list}");
    assert!(
        tools.iter().all(|t| t["outputSchema"].is_object()),
        "{list}"
    );
    drop(r);

    // GET opens the server-to-client stream for the session.
    let r = Http::send(
        &server.addr,
        "GET",
        "/mcp",
        &[
            ("Accept", "text/event-stream"),
            session_header,
            version_header,
        ],
        "",
    );
    assert_eq!(r.status, 200);
    assert!(
        r.header("content-type")
            .is_some_and(|c| c.starts_with("text/event-stream")),
        "{:?}",
        r.headers
    );
    drop(r);

    // A session id the server never issued is not found.
    let r = Http::send(
        &server.addr,
        "POST",
        "/mcp",
        &[
            ACCEPT_BOTH,
            JSON,
            ("Mcp-Session-Id", "nope"),
            version_header,
        ],
        r#"{"jsonrpc":"2.0","id":3,"method":"ping"}"#,
    );
    assert_eq!(r.status, 404);

    // DELETE ends the session.
    let r = Http::send(
        &server.addr,
        "DELETE",
        "/mcp",
        &[session_header, version_header],
        "",
    );
    assert!(r.status < 300, "{}", r.status);
}

// --- Streamable HTTP, 2026-07-28: stateless, per-request _meta ----------

#[test]
fn streamable_http_serves_stateless_requests_without_a_session() {
    let f = log_file();
    let path = f.path().to_str().unwrap();
    let server = Server::start("streamable-http");
    let version = ("MCP-Protocol-Version", "2026-07-28");
    // SEP-2243: this revision mirrors the method (and, for a tool call,
    // the tool name) in headers so a gateway can route without reading
    // the body. The SDK requires them.

    let mut r = Http::send(
        &server.addr,
        "POST",
        "/mcp",
        &[
            ACCEPT_BOTH,
            JSON,
            version,
            ("Mcp-Method", "server/discover"),
        ],
        &stateless(1, "server/discover", json!({})),
    );
    assert_eq!(r.status, 200);
    assert!(
        r.header("mcp-session-id").is_none(),
        "no session in this revision"
    );
    let discover = r.next_message();
    let versions = discover["result"]["supportedVersions"]
        .as_array()
        .expect("supportedVersions");
    assert!(versions.contains(&json!("2026-07-28")), "{discover}");
    assert!(versions.contains(&json!("2025-11-25")), "{discover}");
    assert_eq!(
        discover["result"]["_meta"]["io.modelcontextprotocol/serverInfo"]
            ["name"],
        "rlg-mcp"
    );
    drop(r);

    let mut r = Http::send(
        &server.addr,
        "POST",
        "/mcp",
        &[ACCEPT_BOTH, JSON, version, ("Mcp-Method", "tools/list")],
        &stateless(2, "tools/list", json!({})),
    );
    assert_eq!(r.status, 200);
    let list = r.next_message();
    assert_eq!(
        list["result"]["tools"].as_array().map(Vec::len),
        Some(4)
    );
    drop(r);

    let call_headers = [
        ACCEPT_BOTH,
        JSON,
        version,
        ("Mcp-Method", "tools/call"),
        ("Mcp-Name", "summarize_errors"),
    ];
    let mut r = Http::send(
        &server.addr,
        "POST",
        "/mcp",
        &call_headers,
        &stateless(
            3,
            "tools/call",
            json!({"name": "summarize_errors", "arguments": {"path": path}}),
        ),
    );
    assert_eq!(r.status, 200);
    let call = r.next_message();
    assert_eq!(call["result"]["isError"], false, "{call}");
    assert_eq!(
        call["result"]["structuredContent"]["total"], 1,
        "{call}"
    );
    drop(r);

    // A tool failure is a result the model can read.
    let mut r = Http::send(
        &server.addr,
        "POST",
        "/mcp",
        &call_headers,
        &stateless(
            4,
            "tools/call",
            json!({"name": "summarize_errors", "arguments": {"path": "/definitely/does/not/exist.ndjson"}}),
        ),
    );
    let call = r.next_message();
    assert_eq!(call["result"]["isError"], true, "{call}");
    drop(r);
}

#[test]
fn streamable_http_refuses_what_the_stateless_revision_forbids() {
    let server = Server::start("streamable-http");
    let version = ("MCP-Protocol-Version", "2026-07-28");

    // So is a tool the server does not have: in this revision the
    // SDK's `-32602` would be an HTTP 400, which no model reads.
    let mut r = Http::send(
        &server.addr,
        "POST",
        "/mcp",
        &[
            ACCEPT_BOTH,
            JSON,
            version,
            ("Mcp-Method", "tools/call"),
            ("Mcp-Name", "no_such_tool"),
        ],
        &stateless(
            5,
            "tools/call",
            json!({"name": "no_such_tool", "arguments": {}}),
        ),
    );
    assert_eq!(r.status, 200);
    let call = r.next_message();
    assert_eq!(call["result"]["isError"], true, "{call}");
    assert!(
        call["result"]["content"][0]["text"]
            .as_str()
            .is_some_and(|t| t.contains("Unknown tool: no_such_tool")),
        "{call}"
    );
    drop(r);

    // A request naming its revision without the rest of the required
    // metadata is refused before any handler runs.
    let r = Http::send(
        &server.addr,
        "POST",
        "/mcp",
        &[ACCEPT_BOTH, JSON, version, ("Mcp-Method", "tools/list")],
        r#"{"jsonrpc":"2.0","id":5,"method":"tools/list","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28"}}}"#,
    );
    assert_eq!(r.status, 400);
    assert!(r.body().contains("clientCapabilities"));

    // Without a session, GET has nothing to stream: 405.
    let r = Http::send(
        &server.addr,
        "GET",
        "/mcp",
        &[("Accept", "text/event-stream"), version],
        "",
    );
    assert_eq!(r.status, 405);

    // A body that is not JSON is refused at the HTTP layer. The SDK
    // says 415, an unsupported media type, which is how it reads a
    // body that claims to be JSON and is not.
    let r = Http::send(
        &server.addr,
        "POST",
        "/mcp",
        &[ACCEPT_BOTH, JSON, version],
        "{not json",
    );
    assert_eq!(r.status, 415);
}

#[test]
fn streamable_http_mirrors_routing_headers() {
    // SEP-2243: a request may carry its method and tool name as
    // headers for routing. A header that disagrees with the body is a
    // lie one of the two is telling, and is refused.
    let server = Server::start("streamable-http");
    let version = ("MCP-Protocol-Version", "2026-07-28");
    let mut r = Http::send(
        &server.addr,
        "POST",
        "/mcp",
        &[ACCEPT_BOTH, JSON, version, ("Mcp-Method", "prompts/list")],
        &stateless(1, "prompts/list", json!({})),
    );
    assert_eq!(r.status, 200);
    let list = r.next_message();
    assert_eq!(
        list["result"]["prompts"][0]["name"],
        "triage_error_spike"
    );
    drop(r);

    let r = Http::send(
        &server.addr,
        "POST",
        "/mcp",
        &[ACCEPT_BOTH, JSON, version, ("Mcp-Method", "tools/list")],
        &stateless(2, "server/discover", json!({})),
    );
    assert_eq!(
        r.status, 400,
        "a mismatched Mcp-Method must be refused"
    );
    assert!(r.body().contains("-32020"));
}

// --- The 2024-11-05 HTTP+SSE transport -----------------------------------

#[test]
fn sse_opens_a_stream_and_answers_posts_on_it() {
    let f = log_file();
    let path = f.path().to_str().unwrap();
    let server = Server::start("sse");
    let mut stream = Http::send(&server.addr, "GET", "/sse", &[], "");
    assert_eq!(stream.status, 200);
    assert!(
        stream
            .header("content-type")
            .is_some_and(|c| c.starts_with("text/event-stream")),
        "{:?}",
        stream.headers
    );
    let (event, endpoint) = stream.next_event();
    assert_eq!(event, "endpoint");
    assert!(
        endpoint.starts_with("/messages/?sessionId="),
        "{endpoint}"
    );

    let r = Http::send(
        &server.addr,
        "POST",
        &endpoint,
        &[JSON],
        &initialize("2024-11-05"),
    );
    assert_eq!(r.status, 202);
    let init = stream.next_message();
    assert_eq!(init["id"], 1);
    assert_eq!(init["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(init["result"]["serverInfo"]["name"], "rlg-mcp");

    let r = Http::send(
        &server.addr,
        "POST",
        &endpoint,
        &[JSON],
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    );
    assert_eq!(r.status, 202);

    let r = Http::send(
        &server.addr,
        "POST",
        &endpoint,
        &[JSON],
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
    );
    assert_eq!(r.status, 202);
    let list = stream.next_message();
    assert_eq!(list["id"], 2);
    assert_eq!(
        list["result"]["tools"].as_array().map(Vec::len),
        Some(4)
    );

    let call = json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"tail_log","arguments":{"path":path}}}).to_string();
    let r = Http::send(&server.addr, "POST", &endpoint, &[JSON], &call);
    assert_eq!(r.status, 202);
    let call = stream.next_message();
    assert!(
        call["result"]["content"][0]["text"]
            .as_str()
            .is_some_and(|t| t.contains("boom")),
        "{call}"
    );

    // The endpoint without its trailing slash works too.
    let bare = endpoint.replacen("/messages/?", "/messages?", 1);
    let r = Http::send(
        &server.addr,
        "POST",
        &bare,
        &[JSON],
        r#"{"jsonrpc":"2.0","id":4,"method":"ping"}"#,
    );
    assert_eq!(r.status, 202);
    assert_eq!(stream.next_message()["id"], 4);
}

#[test]
fn sse_refuses_what_it_cannot_route() {
    let server = Server::start("sse");
    let r = Http::send(
        &server.addr,
        "POST",
        "/messages/?sessionId=unknown",
        &[JSON],
        r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#,
    );
    assert_eq!(r.status, 404);

    let mut stream = Http::send(&server.addr, "GET", "/sse", &[], "");
    let (_, endpoint) = stream.next_event();
    let r = Http::send(
        &server.addr,
        "POST",
        &endpoint,
        &[JSON],
        "{not json",
    );
    assert_eq!(r.status, 400);
    assert!(r.body().contains("invalid JSON-RPC"));

    // Hanging up ends the session: the endpoint stops existing.
    drop(stream);
    let mut gone = 0;
    for _ in 0..50 {
        let r = Http::send(
            &server.addr,
            "POST",
            &endpoint,
            &[JSON],
            r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#,
        );
        gone = r.status;
        if gone == 404 {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert_eq!(gone, 404, "the session outlived its stream");
}

#[test]
fn the_binary_announces_where_it_listens() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rlg-mcp"))
        .args(["--transport", "sse", "--port", "0"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("server starts");
    let mut line = String::new();
    let _ = BufReader::new(child.stderr.take().expect("stderr"))
        .read_line(&mut line)
        .expect("read");
    let _ = child.kill();
    let _ = child.wait();
    let addr = line
        .trim()
        .strip_prefix("listening on http://127.0.0.1:")
        .and_then(|rest| rest.strip_suffix("/sse"))
        .unwrap_or_else(|| panic!("no address in {line:?}"));
    assert!(addr.parse::<u16>().is_ok_and(|port| port > 0), "{line:?}");
}

#[test]
fn a_port_in_use_is_reported_not_swallowed() {
    let holder =
        std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = holder.local_addr().expect("addr").port().to_string();
    let out = Command::new(env!("CARGO_BIN_EXE_rlg-mcp"))
        .args(["--transport", "streamable-http", "--port", &port])
        .output()
        .expect("runs");
    assert_eq!(out.status.code(), Some(1));
    let text = String::from_utf8_lossy(&out.stderr);
    assert!(text.contains("cannot listen on"), "{text}");
    assert!(text.contains(&port), "{text}");
}
