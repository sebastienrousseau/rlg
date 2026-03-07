// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # RustLogs (RLG) Library Examples
//!
//! This example demonstrates the usage of the RustLogs (RLG) library,
//! including creating log entries, formatting logs, and using the fluent API.

use rlg::VERSION;
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;

/// Entry point for the RustLogs library examples.
///
/// This function demonstrates various logging formats, log levels, and fluent API.
pub(crate) fn main() {
    println!("🦀 RustLogs Library Usage Examples 🦀\n");

    log_common_format_example();
    log_display_example();
    log_level_display_example();
    log_version_example();
    log_format_example();
    log_fluent_api_example();

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
        let log = Log::build(
            LogLevel::INFO,
            &format!("Log message in {} format", format),
        )
        .session_id("12345")
        .time("2023-01-01T12:00:00Z")
        .component("system")
        .format(format);
        println!("{}", log);
    }
}

/// Demonstrates using the `Display` trait for logs.
fn log_display_example() {
    println!("\n🦀  **Log Display Example**");
    println!("---------------------------------------------");

    let log = Log::build(LogLevel::DEBUG, "This is a debug message")
        .session_id("67890")
        .time("2023-01-01T13:00:00Z")
        .component("app")
        .format(LogFormat::CLF);
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

    let log = Log::build(LogLevel::INFO, "User logged in")
        .session_id("54321")
        .time("2023-01-01T14:00:00Z")
        .component("auth")
        .format(log_format);
    println!("Log Entry: {}", log);
}

/// Demonstrates the fluent API for creating log entries.
fn log_fluent_api_example() {
    println!("\n🦀  **Fluent API Examples**");
    println!("---------------------------------------------");

    // Info log
    let info_log =
        Log::info("Information message").component("app-component");
    println!("Info Log: {}", info_log);

    // Error log
    let error_log =
        Log::error("Error occurred").component("api-component");
    println!("Error Log: {}", error_log);

    // Log with attributes
    let attr_log = Log::debug("Database query executed")
        .component("db-component")
        .with("query_time_ms", 42)
        .with("rows_affected", 5)
        .format(LogFormat::JSON);
    println!("Attribute Log: {}", attr_log);

    // Conditional fire
    let should_log = true;
    if should_log {
        Log::info("Conditional log fired").fire();
    }

    // Set format via fluent API
    let log = Log::default();
    let log = log.format(LogFormat::CLF);
    println!("Log format after fluent API: {}", log.format);
}
