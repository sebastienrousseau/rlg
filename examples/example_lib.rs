// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # RustLogs (RLG) Library Usage Examples
//!
//! This example demonstrates the usage of various components of the RustLogs (RLG) library,
//! including log levels, log formats, and macros for logging functionality.

#![allow(missing_docs)]

use rlg::{log::Log, log_format::LogFormat, log_level::LogLevel};
use rlg::{
    macro_error_log, macro_info_log, macro_log_if,
    macro_log_with_metadata, macro_print_log, macro_set_log_format_clf,
    macro_warn_log, VERSION,
};

/// Entry point for the RustLogs library examples.
///
/// This function demonstrates various logging formats, log levels, and macro functionality.
pub(crate) fn main() {
    println!("🦀 RustLogs Library Usage Examples 🦀\n");

    log_common_format_example();
    log_display_example();
    log_level_display_example();
    log_version_example();
    log_format_example();
    log_macros_example();

    println!("\n🎉 All examples completed successfully!");
}

/// Demonstrates logging in Common Log Format (CLF).
///
/// This function creates a log entry and prints it in CLF format.
fn log_common_format_example() {
    println!("🦀  **Log Common Format Example**");
    println!("---------------------------------------------");

    let log = Log::new(
        "session_123",
        "2022-01-01T00:00:00Z",
        &LogLevel::ERROR,
        "component_a",
        "description_a",
        &LogFormat::CLF,
    );
    println!("    ✅  Log created in CLF format:\n    {}", log);
}

/// Demonstrates the display formatting of a log entry.
///
/// This function creates a log and prints its string representation.
fn log_display_example() {
    println!("\n🦀  **Log Display Example**");
    println!("---------------------------------------------");

    let log = Log::new(
        "12345678-1234-1234-1234-1234567890ab",
        "2023-01-23 14:03:00.000+0000",
        &LogLevel::ERROR,
        "TestComponent",
        "This is a test log message",
        &LogFormat::CLF,
    );
    println!("    ✅  Log created:\n    {}", log);
}

/// Demonstrates logging at various log levels.
///
/// This function creates log entries for different levels and displays their string representations.
fn log_level_display_example() {
    println!("\n🦀  **Log Level Display Example**");
    println!("---------------------------------------------");

    let levels = [
        LogLevel::ALL,
        LogLevel::DEBUG,
        LogLevel::INFO,
        LogLevel::WARN,
        LogLevel::ERROR,
        LogLevel::FATAL,
        LogLevel::TRACE,
        LogLevel::VERBOSE,
        LogLevel::CRITICAL,
        LogLevel::NONE,
        LogLevel::DISABLED,
    ];

    for level in &levels {
        println!("    Log Level: {} -> {:?}", level, level);
    }
}

/// Demonstrates how to retrieve and display the `VERSION` constant.
///
/// This function verifies that the `VERSION` constant matches the package version.
fn log_version_example() {
    println!("\n🦀  **Log Version Example**");
    println!("---------------------------------------------");

    println!("    Library version: {}", VERSION);
    assert_eq!(VERSION, env!("CARGO_PKG_VERSION"), "Version mismatch");
}

/// Demonstrates log formatting in various formats (JSON, CLF, CEF, etc.).
///
/// This function creates log entries and prints them in different log formats.
fn log_format_example() {
    println!("\n🦀  **Log Format Example**");
    println!("---------------------------------------------");

    let formats = [
        LogFormat::CLF,
        LogFormat::JSON,
        LogFormat::CEF,
        LogFormat::ELF,
        LogFormat::W3C,
        LogFormat::GELF,
        LogFormat::ApacheAccessLog,
        LogFormat::Logstash,
        LogFormat::Log4jXML,
        LogFormat::NDJSON,
    ];

    for format in &formats {
        let log = Log::new(
            "session_123",
            "2023-01-23T14:03:00.000+0000",
            &LogLevel::INFO,
            "TestComponent",
            "This is a test log message",
            format,
        );
        println!("    Log in {} format:\n    {}", format, log);
    }
}

/// Demonstrates the usage of various logging macros.
///
/// This function creates log entries using macros such as `macro_log!`, `macro_info_log!`, and others.
fn log_macros_example() {
    println!("\n🦀  **Logging Macros Example**");
    println!("---------------------------------------------");

    // Info log
    let info_log = macro_info_log!(
        "2023-01-01",
        "AppComponent",
        "Info log message"
    );
    println!("    ✅  Info log created:\n    {:?}", info_log);

    // Warn log
    let warn_log = macro_warn_log!(
        "2023-01-01",
        "AppComponent",
        "Warning log message"
    );
    println!("    ✅  Warn log created:\n    {:?}", warn_log);

    // Error log
    let error_log = macro_error_log!(
        "2023-01-01",
        "AppComponent",
        "Error log message"
    );
    println!("    ✅  Error log created:\n    {:?}", error_log);

    // Conditional logging
    println!("\n    Conditional logging:");
    macro_log_if!(true, info_log); // Will log
    macro_log_if!(false, warn_log); // Will not log
    println!("    ✅  Conditional logging executed.");

    // Log with metadata
    let log_with_metadata = macro_log_with_metadata!(
        "session_id",
        "2023-01-01",
        &LogLevel::INFO,
        "AppComponent",
        "Log with metadata",
        &LogFormat::JSON
    );
    println!(
        "\n    ✅  Log with metadata created:\n    {}",
        log_with_metadata
    );

    // Set log format
    let mut log = macro_info_log!(
        "2023-01-01",
        "AppComponent",
        "Message with format"
    );
    macro_set_log_format_clf!(log);
    println!(
        "\n    ✅  Log format changed to CLF:\n    {:?}",
        log.format
    );

    // Print log
    println!("\n    Printing log:");
    macro_print_log!(info_log);
    println!("    ✅  Log printed.");
}
