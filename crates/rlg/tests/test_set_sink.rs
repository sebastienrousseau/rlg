// test_set_sink.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! `set_sink` before the first record decides where the engine
//! writes; after the flusher has started it is refused. Its own test
//! binary, so no other test has started the flusher first.

#![allow(missing_docs)]
#![cfg(not(miri))]

use rlg::engine::{ENGINE, set_sink};
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::sink::PlatformSink;

#[test]
fn set_sink_routes_records_and_is_refused_once_started() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out.log");
    let file = std::fs::File::create(&path).unwrap();
    assert!(set_sink(PlatformSink::File(file)).is_ok());

    Log::info("to the chosen sink")
        .format(LogFormat::JSON)
        .fire();
    ENGINE.shutdown();

    let written = std::fs::read_to_string(&path).unwrap();
    assert!(written.contains("to the chosen sink"), "{written:?}");
    assert!(matches!(
        set_sink(PlatformSink::Stdout),
        Err(PlatformSink::Stdout)
    ));
}
