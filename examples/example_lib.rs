// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # RustLogs (RLG) Library Examples
//!
//! This example demonstrates the usage of the RustLogs (RLG) library,
//! including creating log entries, formatting logs, and using macros.

#![allow(deprecated)]

use dtt::datetime::DateTime;
#[allow(unused_imports)]
use rlg::config::Config;
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg::VERSION;
use rlg::{
    macro_error_log, macro_info_log, macro_log_if,
    macro_log_with_metadata, macro_set_log_format_clf,
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

/// Demonstrates creating log entries with common formats.
fn log_common_format_example() {
    println!("🦀  **Common Format Examples**");
    println!("---------------------------------------------");

    let formats = vec![
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

    for format in formats {
        let log = Log::new(
            "12345",
            "2023-01-01T12:00:00Z",
            &LogLevel::INFO,
            "system",
            &format!("Log message in {} format", format),
            &format,
        );
        println!("{}", log);
    }
}

/// Demonstrates using the `Display` trait for logs.
fn log_display_example() {
    println!("\n🦀  **Log Display Example**");
    println!("---------------------------------------------");

    let log = Log::new(
        "67890",
        "2023-01-01T13:00:00Z",
        &LogLevel::DEBUG,
        "app",
        "This is a debug message",
        &LogFormat::CLF,
    );
    println!("Formatted Log: {}", log);
}

/// Demonstrates displaying different log levels.
fn log_level_display_example() {
    println!("\n🦀  **Log Level Display Example**");
    println!("---------------------------------------------");

    let levels = vec![
        LogLevel::ALL,
        LogLevel::DEBUG,
        LogLevel::TRACE,
        LogLevel::VERBOSE,
        LogLevel::INFO,
        LogLevel::WARN,
        LogLevel::ERROR,
        LogLevel::FATAL,
        LogLevel::CRITICAL,
        LogLevel::NONE,
    ];

    for level in levels {
        println!("Log Level: {}", level);
    }
}

/// Displays the current version of the library.
fn log_version_example() {
    println!("\n🦀  **Library Version Example**");
    println!("---------------------------------------------");
    println!("RustLogs Version: {}", VERSION);
}

/// Demonstrates parsing and using log formats.
fn log_format_example() {
    println!("\n🦀  **Log Format Example**");
    println!("---------------------------------------------");

    let format_str = "json";
    let log_format = format_str.parse::<LogFormat>().unwrap();
    println!("Parsed Log Format: {}", log_format);

    let log = Log::new(
        "54321",
        "2023-01-01T14:00:00Z",
        &LogLevel::INFO,
        "auth",
        "User logged in",
        &log_format,
    );
    println!("Log Entry: {}", log);
}

/// Demonstrates the usage of macros for creating log entries.
fn log_macros_example() {
    println!("\n🦀  **Macro Usage Examples**");
    println!("---------------------------------------------");

    let time = DateTime::new().to_string();

    // info log macro
    let info_log =
        macro_info_log!(&time, "app-component", "Information message");
    println!("Info Log (Macro): {}", info_log);

    // error log macro
    let error_log =
        macro_error_log!(&time, "api-component", "Error occurred");
    println!("Error Log (Macro): {}", error_log);

    // log with metadata macro
    let metadata_log = macro_log_with_metadata!(
        "session-123",
        &time,
        LogLevel::DEBUG,
        "db-component",
        "Database query executed",
        LogFormat::JSON
    );
    println!("Metadata Log (Macro): {}", metadata_log);

    // conditional log macro
    macro_log_if!(true, info_log); // Will log
    macro_log_if!(false, error_log); // Will not log

    // set log format macro
    let mut log = Log::default();
    macro_set_log_format_clf!(log);
    println!("Log format after macro: {}", log.format);
}
