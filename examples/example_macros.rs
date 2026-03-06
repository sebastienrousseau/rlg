// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # RustLogs (RLG) Macros Examples
//!
//! This example demonstrates the usage of various macros in the RustLogs (RLG) library,
//! including logging at different levels, adding metadata, and log formatting.

#![allow(deprecated)]

#[allow(unused_imports)]
use dtt::datetime::DateTime;
#[allow(unused_imports)]
use rlg::log_format::LogFormat;
#[allow(unused_imports)]
use rlg::log_level::LogLevel;
use rlg::{
    macro_error_log, macro_fatal_log, macro_info_log, macro_log,
    macro_log_if, macro_log_with_metadata, macro_set_log_format_clf,
    macro_warn_log,
};

/// Entry point for the RustLogs macros examples.
///
/// This function demonstrates the usage of various macros for logging at different levels,
/// applying metadata, and conditional logging.
pub(crate) fn main() {
    println!("🦀 RustLogs Macros Examples 🦀\n");

    basic_macro_log_example();
    info_log_macro_example();
    warn_log_macro_example();
    error_log_macro_example();
    trace_log_macro_example();
    fatal_log_macro_example();
    conditional_log_example();
    log_with_metadata_example();
    log_with_format_example();
    unicode_log_example();

    println!("\n🎉 All examples completed successfully!");
}

/// Demonstrates basic usage of the `macro_log!` macro.
fn basic_macro_log_example() {
    println!("🦀  **Basic Macro Log Example**");
    println!("---------------------------------------------");

    let log = macro_log!(
        "session_id",
        "2022-01-01T12:00:00Z",
        &LogLevel::INFO,
        "component",
        "This is an info message",
        &LogFormat::JSON
    );
    log.fire();
}

/// Demonstrates usage of the `macro_info_log!` macro.
fn info_log_macro_example() {
    println!("\n🦀  **Info Log Macro Example**");
    println!("---------------------------------------------");

    let log =
        macro_info_log!("2022-01-01T12:00:00Z", "component", "message");
    log.fire();
}

/// Demonstrates usage of the `macro_warn_log!` macro.
fn warn_log_macro_example() {
    println!("\n🦀  **Warn Log Macro Example**");
    println!("---------------------------------------------");

    let log =
        macro_warn_log!("2022-01-01T12:00:00Z", "component", "message");
    log.fire();
}

/// Demonstrates usage of the `macro_error_log!` macro.
fn error_log_macro_example() {
    println!("\n🦀  **Error Log Macro Example**");
    println!("---------------------------------------------");

    let log = macro_error_log!(
        "2022-01-01T12:00:00Z",
        "component",
        "message"
    );
    log.fire();
}

/// Demonstrates usage of the `macro_trace_log!` macro.
fn trace_log_macro_example() {
    println!("\n🦀  **Trace Log Macro Example**");
    println!("---------------------------------------------");

    let log = rlg::macro_trace_log!(
        "2022-01-01T12:00:00Z",
        "component",
        "message"
    );
    log.fire();
}

/// Demonstrates usage of the `macro_fatal_log!` macro.
fn fatal_log_macro_example() {
    println!("\n🦀  **Fatal Log Macro Example**");
    println!("---------------------------------------------");

    let log = macro_fatal_log!(
        "2022-01-01T12:00:00Z",
        "component",
        "message"
    );
    log.fire();
}

/// Demonstrates usage of the `macro_log_if!` macro.
fn conditional_log_example() {
    println!("\n🦀  **Conditional Log Example**");
    println!("---------------------------------------------");

    let log =
        macro_info_log!("2022-01-01T12:00:00Z", "component", "message");
    macro_log_if!(true, log); // Should log
    println!("(Log with true condition should have fired)");

    let log2 =
        macro_warn_log!("2022-01-01T12:00:00Z", "component", "message");
    macro_log_if!(false, log2); // Should not log
    println!("(Log with false condition should not have fired)");
}

/// Demonstrates usage of the `macro_log_with_metadata!` macro.
fn log_with_metadata_example() {
    println!("\n🦀  **Log with Metadata Example**");
    println!("---------------------------------------------");

    let log_message = macro_log_with_metadata!(
        "session_id",
        "2022-01-01T12:00:00Z",
        LogLevel::INFO,
        "component",
        "message",
        LogFormat::JSON
    );
    println!("Log Message JSON: {}", log_message);
}

/// Demonstrates setting log format using `macro_set_log_format_clf!`.
fn log_with_format_example() {
    println!("\n🦀  **Log with Format Example**");
    println!("---------------------------------------------");

    let mut log =
        macro_info_log!("2022-01-01T12:00:00Z", "component", "message");
    macro_set_log_format_clf!(log);
    println!("Log Format set to: {}", log.format);
}

/// Demonstrates logging messages with Unicode characters.
fn unicode_log_example() {
    println!("\n🦀  **Unicode Log Example**");
    println!("---------------------------------------------");

    let log = macro_info_log!(
        "2022-01-01T12:00:00Z",
        "component",
        "こんにちは RustLogs! 🦀"
    );
    log.fire();
}
