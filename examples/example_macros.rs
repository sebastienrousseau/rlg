// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # RustLogs (RLG) Macros Examples
//!
//! This example demonstrates the usage of various macros in the RustLogs (RLG) library,
//! including logging at different levels, adding metadata, and log formatting.

#![allow(missing_docs)]

use rlg::{log_format::LogFormat, log_level::LogLevel};
use rlg::{
    macro_error_log, macro_fatal_log, macro_info_log, macro_log,
    macro_log_if, macro_log_with_metadata, macro_print_log,
    macro_set_log_format_clf, macro_trace_log, macro_warn_log,
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
///
/// This function logs a message with the provided session ID, timestamp, log level, component,
/// description, and log format.
fn basic_macro_log_example() {
    println!("🦀  **Basic Macro Log Example**");
    println!("---------------------------------------------");

    let log = macro_log!(
        "session_id",
        "2022-01-01T12:00:00Z",
        &LogLevel::INFO,
        "Component",
        "This is a log message",
        &LogFormat::JSON
    );

    println!("    ✅  Log created: {:?}", log);
}

/// Demonstrates logging an informational message using the `macro_info_log!` macro.
///
/// This function creates a log entry at the INFO level.
fn info_log_macro_example() {
    println!("\n🦀  **Info Log Macro Example**");
    println!("---------------------------------------------");

    let log = macro_info_log!(
        "2024-01-01T12:00:00Z",
        "AppComponent",
        "Informational log message"
    );

    println!("    ✅  Info log created: {:?}", log);
}

/// Demonstrates logging a warning message using the `macro_warn_log!` macro.
///
/// This function creates a log entry at the WARN level.
fn warn_log_macro_example() {
    println!("\n🦀  **Warn Log Macro Example**");
    println!("---------------------------------------------");

    let log = macro_warn_log!(
        "2024-01-01T12:00:00Z",
        "AppComponent",
        "Warning log message"
    );

    println!("    ✅  Warn log created: {:?}", log);
}

/// Demonstrates logging an error message using the `macro_error_log!` macro.
///
/// This function creates a log entry at the ERROR level.
fn error_log_macro_example() {
    println!("\n🦀  **Error Log Macro Example**");
    println!("---------------------------------------------");

    let log = macro_error_log!(
        "2024-01-01T12:00:00Z",
        "AppComponent",
        "Error log message"
    );

    println!("    ✅  Error log created: {:?}", log);
}

/// Demonstrates logging a trace message using the `macro_trace_log!` macro.
///
/// This function creates a log entry at the TRACE level.
fn trace_log_macro_example() {
    println!("\n🦀  **Trace Log Macro Example**");
    println!("---------------------------------------------");

    let log = macro_trace_log!(
        "2024-01-01T12:00:00Z",
        "AppComponent",
        "Trace log message"
    );

    println!("    ✅  Trace log created: {:?}", log);
}

/// Demonstrates logging a fatal error message using the `macro_fatal_log!` macro.
///
/// This function creates a log entry at the FATAL level.
fn fatal_log_macro_example() {
    println!("\n🦀  **Fatal Log Macro Example**");
    println!("---------------------------------------------");

    let log = macro_fatal_log!(
        "2024-01-01T12:00:00Z",
        "AppComponent",
        "Fatal error log message"
    );

    println!("    ✅  Fatal log created: {:?}", log);
}

/// Demonstrates conditional logging using the `macro_log_if!` macro.
///
/// This function logs a message only if a specified condition is true.
fn conditional_log_example() {
    println!("\n🦀  **Conditional Log Example**");
    println!("---------------------------------------------");

    let log = macro_info_log!(
        "2024-01-01T12:00:00Z",
        "ConditionalComponent",
        "This log will only appear if the condition is true"
    );

    macro_log_if!(true, log); // Logs if the condition is true
    macro_log_if!(false, log); // Will not log if the condition is false

    println!("    ✅  Conditional logging executed.");
}

/// Demonstrates adding metadata to logs using the `macro_log_with_metadata!` macro.
///
/// This function logs a message with additional metadata.
fn log_with_metadata_example() {
    println!("\n🦀  **Log with Metadata Example**");
    println!("---------------------------------------------");

    let log_message = macro_log_with_metadata!(
        "session123",
        "2024-01-01T12:00:00Z",
        &LogLevel::INFO,
        "MetadataComponent",
        "Log message with metadata",
        &LogFormat::JSON
    );

    println!("    ✅  Log with metadata created: {}", log_message);
}

/// Demonstrates changing the log format using the `macro_set_log_format_clf!` macro.
///
/// This function changes the log format to CLF (Common Log Format).
fn log_with_format_example() {
    println!("\n🦀  **Log Format Example**");
    println!("---------------------------------------------");

    let mut log = macro_info_log!(
        "2024-01-01T12:00:00Z",
        "AppComponent",
        "Log message"
    );
    println!("    Original format: {:?}", log.format);

    macro_set_log_format_clf!(log);
    println!("    ✅  Log format changed to CLF: {:?}", log.format);
}

/// Demonstrates logging messages that contain Unicode characters.
///
/// This function creates a log entry with special Unicode characters in the message.
fn unicode_log_example() {
    println!("\n🦀  **Unicode Log Example**");
    println!("---------------------------------------------");

    let log = macro_info_log!(
        "2024-01-01T12:00:00Z",
        "UnicodeComponent",
        "Unicode log: 你好, världen, 🌍"
    );

    println!("    ✅  Unicode log created: {:?}", log);
}
