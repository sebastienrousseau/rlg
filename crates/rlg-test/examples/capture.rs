// capture.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Demonstrates capturing rlg records into an in-memory handle and
// asserting on them — the pattern downstream libraries use to test
// that their own code emits the right structured logs.

#![allow(missing_docs)]

use rlg::log::Log;
use rlg::log_level::LogLevel;
use rlg_test::{LogExt, assert_logged, capture};

fn business_logic_under_test(capture: &rlg_test::Capture) {
    Log::info("user authenticated")
        .component("auth")
        .with("user_id", 42_u64)
        .log_to(capture);

    Log::error("payment declined")
        .component("payments")
        .with("reason", "INSUFFICIENT_FUNDS")
        .log_to(capture);
}

fn main() {
    let capture = capture();
    business_logic_under_test(&capture);

    assert_logged!(capture, len == 2);
    assert_logged!(capture, level == LogLevel::ERROR);
    assert_logged!(capture, component "auth");
    assert_logged!(capture, contains "INSUFFICIENT_FUNDS");
    assert_logged!(capture, attribute "user_id" => 42_u64);

    println!("captured {} records — all assertions held", capture.len());
}
