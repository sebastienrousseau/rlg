// transport.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

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
//! This module (`transport.rs` and `transport/`) depends only on the
//! SDK and on a [`ServerHandler`] passed in, so it is copied verbatim
//! into every Rust server of the suite. Nothing in it knows what the
//! tools are.

use std::io;
use std::process::ExitCode;

use axum::Router;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService,
};
use rmcp::{ServerHandler, ServiceExt};
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
        let (flag, inline) = split_flag(arg);
        match flag.as_str() {
            "--help" | "-h" => return Ok(Command::Help),
            "--version" | "-V" => return Ok(Command::Version),
            other => apply_flag(&mut options, other, inline, &mut args)?,
        }
    }
    Ok(Command::Serve(options))
}

/// Split `--flag=value` at its first `=`; a bare `--flag` has no
/// inline value.
fn split_flag(arg: String) -> (String, Option<String>) {
    match arg.split_once('=') {
        Some((flag, value)) => (flag.to_owned(), Some(value.to_owned())),
        None => (arg, None),
    }
}

/// Set the option `flag` names, from its inline value or else the next
/// argument. An unknown flag fails before any argument is read.
fn apply_flag(
    options: &mut Options,
    flag: &str,
    inline: Option<String>,
    rest: &mut dyn Iterator<Item = String>,
) -> Result<(), String> {
    if !matches!(flag, "--transport" | "--host" | "--port") {
        return Err(format!("unknown argument `{flag}`"));
    }
    let value = inline
        .or_else(|| rest.next())
        .ok_or_else(|| format!("{flag} needs a value"))?;
    match flag {
        "--transport" => options.transport = parse_transport(&value)?,
        "--port" => options.port = parse_port(&value)?,
        _ => options.host = value,
    }
    Ok(())
}

fn parse_transport(name: &str) -> Result<Transport, String> {
    Transport::parse(name).ok_or_else(|| {
        format!(
            "unknown transport `{name}`; choose stdio, streamable-http or sse"
        )
    })
}

fn parse_port(text: &str) -> Result<u16, String> {
    text.parse()
        .map_err(|_| format!("`{text}` is not a port number"))
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

mod sse;
use sse::serve_sse;

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
    fn flags_split_on_the_first_equals_and_unknown_ones_stop_parsing() {
        // An unknown flag fails before anything after it is read, even a
        // flag that would otherwise win.
        assert!(
            parse(["--bogus", "--help"]).is_err_and(|e| e.contains("--bogus"))
        );
        assert!(parse(["--bogus=1"]).is_err_and(|e| e.contains("`--bogus`")));
        // Only the first `=` separates the value.
        match parse(["--transport=sse", "--host=a=b", "--port=0"]) {
            Ok(Command::Serve(options)) => {
                assert_eq!(options.transport, Transport::Sse);
                assert_eq!(options.host, "a=b");
                assert_eq!(options.port, 0);
            }
            other => panic!("expected serve, got {other:?}"),
        }
        // A value-taking flag reads the next argument as its value.
        assert!(
            parse(["--host", "--help"]).is_ok_and(|c| matches!(
                c,
                Command::Serve(Options { ref host, .. }) if host == "--help"
            ))
        );
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
