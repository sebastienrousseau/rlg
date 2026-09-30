// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml. All rights reserved.

//! One command line for the three MCP transports.
//!
//! Every server in the suite is started the same way:
//!
//! ```text
//! <server>                                   # stdio, for a client that spawns it
//! <server> --transport streamable-http       # HTTP on 127.0.0.1:8000, path /mcp
//! <server> --transport sse --port 8001       # the older HTTP+SSE transport
//! ```
//!
//! Over streamable HTTP the SDK speaks both current protocol revisions
//! on one endpoint: `2026-07-28` (stateless, `server/discover`,
//! per-request `_meta`) and `2025-11-25` (`initialize` handshake,
//! `Mcp-Session-Id`). Responses stream as server-sent events; a `GET`
//! on the same path opens the server-to-client event stream.
//! `--transport sse` serves the `2024-11-05` HTTP+SSE transport for
//! clients that still expect it: `GET /sse` opens the event stream,
//! whose first event names the `/messages/?sessionId=...` endpoint the
//! client posts to.
//!
//! The listener binds the loopback interface unless told otherwise.
//! There is no authentication here: put the server behind a gateway
//! you trust before binding a routable address.
//!
//! This file depends only on the SDK and on a [`ServerHandler`]
//! passed in, so it is copied verbatim into every Rust server of the
//! suite. Nothing in it knows what the tools are.

use std::collections::HashMap;
use std::fmt;
use std::io;
use std::pin::Pin;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use axum::Router;
use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use futures::channel::mpsc;
use futures::{SinkExt, Stream, StreamExt};
use rmcp::model::{ClientJsonRpcMessage, ServerJsonRpcMessage};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService,
};
use rmcp::{ServerHandler, ServiceExt};
use serde::Deserialize;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

/// Where the listeners bind unless told otherwise.
pub const DEFAULT_HOST: &str = "127.0.0.1";
/// The port the HTTP transports listen on unless told otherwise.
pub const DEFAULT_PORT: u16 = 8000;
/// The streamable HTTP endpoint.
pub const STREAMABLE_HTTP_PATH: &str = "/mcp";
/// Where the legacy transport's event stream is opened.
pub const SSE_PATH: &str = "/sse";
/// Where the legacy transport's client messages are posted.
pub const MESSAGE_PATH: &str = "/messages/";

/// How the server talks to its client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    /// JSON-RPC over the process's own stdin and stdout.
    Stdio,
    /// The current HTTP transport, both protocol revisions.
    StreamableHttp,
    /// The 2024-11-05 HTTP+SSE transport.
    Sse,
}

impl Transport {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "stdio" => Some(Self::Stdio),
            "streamable-http" => Some(Self::StreamableHttp),
            "sse" => Some(Self::Sse),
            _ => None,
        }
    }
}

/// What the command line asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// The transport to serve.
    pub transport: Transport,
    /// The interface the HTTP transports bind.
    pub host: String,
    /// The port the HTTP transports bind. Zero asks the system for a
    /// free one, which is what a test wants.
    pub port: u16,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            transport: Transport::Stdio,
            host: DEFAULT_HOST.to_owned(),
            port: DEFAULT_PORT,
        }
    }
}

/// The outcome of reading the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Serve with these options.
    Serve(Options),
    /// `--help`: print the usage text and exit.
    Help,
    /// `--version`: print the version and exit.
    Version,
}

/// The usage text, for `--help` and for a usage error.
#[must_use]
pub fn usage(name: &str) -> String {
    format!(
        "Usage: {name} [--transport <stdio|streamable-http|sse>] \
         [--host <address>] [--port <number>]\n\
         \n\
         Options:\n\
         \x20 --transport <name>  stdio (default), streamable-http, or sse\n\
         \x20 --host <address>    interface for the HTTP transports \
         (default {DEFAULT_HOST})\n\
         \x20 --port <number>     port for the HTTP transports \
         (default {DEFAULT_PORT})\n\
         \x20 --version           print the version and exit\n\
         \x20 --help              print this text and exit\n\
         \n\
         streamable-http serves {STREAMABLE_HTTP_PATH} (MCP 2025-11-25 and \
         2026-07-28).\n\
         sse serves {SSE_PATH} and {MESSAGE_PATH} (MCP 2024-11-05).\n\
         Neither transport authenticates: keep them on the loopback \
         interface or\n\
         behind a gateway you trust."
    )
}

/// Read the command line.
///
/// `args` excludes the program name. Both `--flag value` and
/// `--flag=value` are accepted.
///
/// # Errors
///
/// A flag that is unknown, missing its value, or given a value that
/// does not parse. The message says which.
pub fn parse<I>(args: I) -> Result<Command, String>
where
    I: IntoIterator,
    I::Item: Into<String>,
{
    let mut options = Options::default();
    let mut args = args.into_iter().map(Into::into);
    while let Some(arg) = args.next() {
        let (flag, inline) = match arg.split_once('=') {
            Some((flag, value)) => (flag.to_owned(), Some(value.to_owned())),
            None => (arg, None),
        };
        let value = |args: &mut dyn Iterator<Item = String>| {
            inline
                .clone()
                .or_else(|| args.next())
                .ok_or_else(|| format!("{flag} needs a value"))
        };
        match flag.as_str() {
            "--help" | "-h" => return Ok(Command::Help),
            "--version" | "-V" => return Ok(Command::Version),
            "--transport" => {
                let name = value(&mut args)?;
                options.transport =
                    Transport::parse(&name).ok_or_else(|| {
                        format!(
                            "unknown transport `{name}`; choose stdio, \
                         streamable-http or sse"
                        )
                    })?;
            }
            "--host" => options.host = value(&mut args)?,
            "--port" => {
                let text = value(&mut args)?;
                options.port = text
                    .parse()
                    .map_err(|_| format!("`{text}` is not a port number"))?;
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    Ok(Command::Serve(options))
}

/// Serve `factory`'s handler as `name` according to the command line.
///
/// This is the whole of `main`: parse, serve, and turn the outcome
/// into an exit status. A usage error prints the usage text and exits
/// with 2; a transport failure prints the reason and exits with 1.
pub fn run<H, F, I>(name: &str, version: &str, args: I, factory: F) -> ExitCode
where
    H: ServerHandler,
    F: Fn() -> H + Send + Sync + 'static,
    I: IntoIterator,
    I::Item: Into<String>,
{
    let options = match parse(args) {
        Ok(Command::Serve(options)) => options,
        Ok(Command::Help) => {
            println!("{}", usage(name));
            return ExitCode::SUCCESS;
        }
        Ok(Command::Version) => {
            println!("{name} {version}");
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("{name}: {message}\n\n{}", usage(name));
            return ExitCode::from(2);
        }
    };
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(e) => {
            eprintln!("{name}: cannot start the async runtime: {e}");
            return ExitCode::FAILURE;
        }
    };
    match runtime.block_on(serve(&options, factory)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{name}: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Serve the handler over the chosen transport until the client goes
/// away (stdio) or the process is interrupted (HTTP).
///
/// # Errors
///
/// The listener could not bind, or the transport failed underneath the
/// session.
pub async fn serve<H, F>(options: &Options, factory: F) -> io::Result<()>
where
    H: ServerHandler,
    F: Fn() -> H + Send + Sync + 'static,
{
    match options.transport {
        Transport::Stdio => serve_stdio(factory()).await,
        Transport::StreamableHttp => {
            let listener = bind(options).await?;
            announce(&listener, STREAMABLE_HTTP_PATH)?;
            serve_streamable_http(listener, options, factory).await
        }
        Transport::Sse => {
            let listener = bind(options).await?;
            announce(&listener, SSE_PATH)?;
            serve_sse(listener, factory).await
        }
    }
}

async fn serve_stdio<H: ServerHandler>(handler: H) -> io::Result<()> {
    use rmcp::service::ServerInitializeError as Init;
    let running = match handler.serve(rmcp::transport::stdio()).await {
        Ok(running) => running,
        // The client hung up, or spoke before the handshake, and there
        // is nobody left to report to: a session that never began is
        // the normal end of a stdio server, not a failure of it.
        Err(Init::ConnectionClosed(_) | Init::ExpectedInitializeRequest(_)) => {
            return Ok(());
        }
        Err(e) => return Err(io::Error::other(format!("stdio session: {e}"))),
    };
    // Likewise once it is under way: the client closing the pipe is
    // how a stdio session ends.
    let _ = running
        .waiting()
        .await
        .map_err(|e| io::Error::other(format!("stdio session: {e}")))?;
    Ok(())
}

async fn bind(options: &Options) -> io::Result<TcpListener> {
    TcpListener::bind((options.host.as_str(), options.port))
        .await
        .map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "cannot listen on {}:{}: {e}",
                    options.host, options.port
                ),
            )
        })
}

/// Say where the server is, on stderr so stdout stays free.
///
/// A test starts the server on port 0 and reads the port from here.
fn announce(listener: &TcpListener, path: &str) -> io::Result<()> {
    let addr = listener.local_addr()?;
    eprintln!("listening on http://{addr}{path}");
    Ok(())
}

/// Resolve when the process is asked to stop.
async fn interrupted() {
    if tokio::signal::ctrl_c().await.is_err() {
        // No signal handler could be installed: stay up until killed.
        std::future::pending::<()>().await;
    }
}

async fn serve_streamable_http<H, F>(
    listener: TcpListener,
    options: &Options,
    factory: F,
) -> io::Result<()>
where
    H: ServerHandler,
    F: Fn() -> H + Send + Sync + 'static,
{
    let ct = CancellationToken::new();
    let mut config = StreamableHttpServerConfig::default()
        .with_cancellation_token(ct.child_token());
    // The SDK refuses a `Host` header it does not expect, which is
    // what stops a page in a browser from reaching a local server
    // through DNS rebinding. Loopback names are allowed by default;
    // an operator who binds another interface has chosen to be
    // reachable by it, so that name is allowed too. Binding every
    // interface means there is no name to check against.
    if options.host == "0.0.0.0" || options.host == "::" {
        config = config.disable_allowed_hosts();
    } else if !config.allowed_hosts.contains(&options.host) {
        config.allowed_hosts.push(options.host.clone());
    }
    let service = StreamableHttpService::new(
        move || Ok(factory()),
        LocalSessionManager::default().into(),
        config,
    );
    let router = Router::new().nest_service(STREAMABLE_HTTP_PATH, service);
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            interrupted().await;
            ct.cancel();
        })
        .await
}

// --- The 2024-11-05 HTTP+SSE transport ---------------------------------
//
// The SDK dropped the server side of this transport in 3.x. It is a
// small thing: one event stream per session, and a POST endpoint that
// feeds messages into it. The SDK's service runs over an in-memory
// pair of channels, exactly as it would over a socket.

/// One session's inbox: the channel its posted messages go down.
type Inbox = mpsc::Sender<ClientJsonRpcMessage>;

/// The live sessions, by id.
type Sessions = Arc<Mutex<HashMap<String, Inbox>>>;

/// What the two SSE handlers share.
struct SseState<H> {
    factory: Box<dyn Fn() -> H + Send + Sync>,
    sessions: Sessions,
    /// Cancelled when the server stops, ending every session.
    shutdown: CancellationToken,
}

/// `?sessionId=...` on the message endpoint.
#[derive(Debug, Deserialize)]
struct SessionQuery {
    #[serde(rename = "sessionId")]
    session_id: String,
}

async fn serve_sse<H, F>(listener: TcpListener, factory: F) -> io::Result<()>
where
    H: ServerHandler,
    F: Fn() -> H + Send + Sync + 'static,
{
    let shutdown = CancellationToken::new();
    let state = Arc::new(SseState {
        factory: Box::new(factory),
        sessions: Sessions::default(),
        shutdown: shutdown.clone(),
    });
    let router = Router::new()
        .route(SSE_PATH, get(open_stream::<H>))
        .route(MESSAGE_PATH, post(post_message::<H>))
        // Without the trailing slash too: clients differ on whether
        // they keep it, and a 404 over a slash is a poor way to fail.
        .route(MESSAGE_PATH.trim_end_matches('/'), post(post_message::<H>))
        .with_state(state);
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            interrupted().await;
            shutdown.cancel();
        })
        .await
}

/// `GET /sse`: start a session and stream its events.
///
/// The first event is `endpoint`, naming where this session's
/// messages are posted. Every message the server sends after that is a
/// `message` event carrying one JSON-RPC message.
async fn open_stream<H: ServerHandler>(
    State(state): State<Arc<SseState<H>>>,
) -> Response {
    let id = uuid::Uuid::new_v4().simple().to_string();
    let (inbox, from_client) = mpsc::channel::<ClientJsonRpcMessage>(32);
    let (to_client, outbox) = mpsc::channel::<ServerJsonRpcMessage>(32);
    let ct = state.shutdown.child_token();
    let _ = state
        .sessions
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(id.clone(), inbox);

    let handler = (state.factory)();
    let session_ct = ct.clone();
    drop(tokio::spawn(async move {
        // The service ends when its client stream closes -- the POST
        // side dropped -- or when the token is cancelled. Either way
        // there is nobody to report to.
        if let Ok(running) = handler
            .serve_with_ct((to_client, from_client), session_ct)
            .await
        {
            let _ = running.waiting().await;
        }
    }));

    let endpoint = Event::default()
        .event("endpoint")
        .data(format!("{MESSAGE_PATH}?sessionId={id}"));
    let messages = outbox.map(|message| {
        serde_json::to_string(&message)
            .map(|json| Event::default().event("message").data(json))
    });
    let events = futures::stream::once(async { Ok(endpoint) }).chain(messages);
    let stream = SessionStream {
        events: Box::pin(events),
        guard: SessionGuard {
            id,
            sessions: Arc::clone(&state.sessions),
            ct,
        },
    };
    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}

/// `POST /messages/?sessionId=...`: one JSON-RPC message in, `202` out.
///
/// The reply, if any, goes down the session's event stream, which is
/// what makes this the older transport: the HTTP response carries
/// nothing.
async fn post_message<H: ServerHandler>(
    State(state): State<Arc<SseState<H>>>,
    Query(query): Query<SessionQuery>,
    body: Bytes,
) -> Response {
    let message: ClientJsonRpcMessage = match serde_json::from_slice(&body) {
        Ok(message) => message,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("invalid JSON-RPC: {e}"))
                .into_response();
        }
    };
    let inbox = state
        .sessions
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&query.session_id)
        .cloned();
    let Some(mut inbox) = inbox else {
        return (StatusCode::NOT_FOUND, "no such session").into_response();
    };
    match inbox.send(message).await {
        Ok(()) => StatusCode::ACCEPTED.into_response(),
        // The service has gone but the stream has not yet been torn
        // down: the session is over.
        Err(_) => (StatusCode::GONE, "session closed").into_response(),
    }
}

/// Ends the session when the event stream is dropped -- which is how
/// a client hangs up.
struct SessionGuard {
    id: String,
    sessions: Sessions,
    ct: CancellationToken,
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        let _ = self
            .sessions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&self.id);
        self.ct.cancel();
    }
}

/// The event stream of one session, with its guard attached.
struct SessionStream {
    events:
        Pin<Box<dyn Stream<Item = Result<Event, serde_json::Error>> + Send>>,
    #[allow(dead_code, reason = "held for its Drop")]
    guard: SessionGuard,
}

impl Stream for SessionStream {
    type Item = Result<Event, serde_json::Error>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.events.as_mut().poll_next(cx)
    }
}

impl<H> fmt::Debug for SseState<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SseState").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_arguments_means_stdio() {
        assert_eq!(
            parse(Vec::<String>::new()),
            Ok(Command::Serve(Options::default()))
        );
    }

    #[test]
    fn the_http_transports_take_host_and_port() {
        let want = Options {
            transport: Transport::StreamableHttp,
            host: "0.0.0.0".to_owned(),
            port: 9000,
        };
        assert_eq!(
            parse([
                "--transport",
                "streamable-http",
                "--host",
                "0.0.0.0",
                "--port",
                "9000"
            ]),
            Ok(Command::Serve(want.clone()))
        );
        // `--flag=value` is the same as `--flag value`.
        assert_eq!(
            parse([
                "--transport=streamable-http",
                "--host=0.0.0.0",
                "--port=9000"
            ]),
            Ok(Command::Serve(want))
        );
        assert_eq!(
            parse(["--transport", "sse"]),
            Ok(Command::Serve(Options {
                transport: Transport::Sse,
                ..Options::default()
            }))
        );
    }

    #[test]
    fn help_and_version_win_over_everything_else() {
        assert_eq!(parse(["--help"]), Ok(Command::Help));
        assert_eq!(parse(["-h"]), Ok(Command::Help));
        assert_eq!(parse(["--version"]), Ok(Command::Version));
        assert_eq!(parse(["--transport", "sse", "-V"]), Ok(Command::Version));
    }

    #[test]
    fn bad_arguments_are_named() {
        assert!(
            parse(["--transport", "carrier-pigeon"])
                .is_err_and(|e| e.contains("carrier-pigeon"))
        );
        assert!(
            parse(["--port", "eighty"]).is_err_and(|e| e.contains("eighty"))
        );
        assert!(parse(["--port", "70000"]).is_err_and(|e| e.contains("70000")));
        assert!(parse(["--port"]).is_err_and(|e| e.contains("needs a value")));
        assert!(parse(["--bogus"]).is_err_and(|e| e.contains("--bogus")));
    }

    #[test]
    fn the_usage_text_names_every_flag_and_path() {
        let text = usage("any-mcp");
        for needle in [
            "any-mcp",
            "--transport",
            "--host",
            "--port",
            "--version",
            "--help",
            STREAMABLE_HTTP_PATH,
            SSE_PATH,
            MESSAGE_PATH,
        ] {
            assert!(text.contains(needle), "usage lacks {needle}:\n{text}");
        }
    }
}
