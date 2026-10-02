// tests.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Unit tests for `transport.rs`.

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
