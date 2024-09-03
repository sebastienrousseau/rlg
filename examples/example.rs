// example.rs
// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(missing_docs)]
/// Importing necessary items from external crates
use dtt::datetime::DateTime;
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg::{
    // Importing macros for logging
    macro_fatal_log,
    macro_log_if,
    macro_log_with_metadata,
    macro_print_log,
    macro_trace_log,
};

/// Main function
fn main() {
    // Creating a new DateTime instance
    let date = DateTime::new();
    // Extracting ISO 8601 format from DateTime
    let iso = date.to_string();

    // Defining log formats to iterate over
    let log_formats = vec![
        LogFormat::CLF,
        LogFormat::JSON,
        LogFormat::CEF,
        LogFormat::ELF,
        LogFormat::GELF,
        LogFormat::W3C,
    ];

    // Iterating over each log format
    for format in log_formats {
        // Creating a new log entry
        let log = Log::new(
            // Unique ID for log entry
            "12345678-1234-1234-1234-1234567890ab",
            // Timestamp in ISO 8601 format
            &iso,
            // Log level
            &LogLevel::INFO,
            // Log component
            "SystemTrayEvent",
            // Log message
            "Showing main window",
            // Log format
            &format,
        );
        // Printing success message for log creation
        println!("🦀 Log::new(): ✅ {}", log);
    }

    // Example usage of macro_trace_log
    let trace_log =
        macro_trace_log!(&iso, "TraceComponent", "This is a trace log");
    // Printing success message for trace log creation
    println!("🦀 macro_trace_log: ✅ {}", trace_log);

    // Example usage of macro_fatal_log
    let fatal_log =
        macro_fatal_log!(&iso, "FatalComponent", "This is a fatal log");
    // Printing success message for fatal log creation
    println!("🦀 macro_fatal_log: ✅ {}", fatal_log);

    // Example usage of macro_log_if
    let should_log = true;
    let log_to_check = Log::new(
        // Unique ID for log entry
        "12345678-1234-1234-1234-1234567890ab",
        // Timestamp in ISO 8601 format
        &iso,
        // Log level
        &LogLevel::INFO,
        // Log component
        "ConditionalComponent",
        // Log message
        "Log to check for conditional logging",
        // Log format
        &LogFormat::CLF,
    );
    // Logging only if condition is true
    macro_log_if!(should_log, log_to_check);

    // Example usage of macro_log_with_metadata
    let metadata = "Additional metadata";
    let log_with_metadata = macro_log_with_metadata!(
        // Unique ID for log entry
        "12345678-1234-1234-1234-1234567890ab",
        // Timestamp in ISO 8601 format
        &iso,
        // Log level
        &LogLevel::INFO,
        // Log component
        "ComponentWithMetadata",
        // Log message
        "Log with metadata",
        // Log format
        &LogFormat::JSON
    );
    // Printing success message for log with metadata creation
    println!(
        "🦀 macro_log_with_metadata: ✅ {} | Metadata: {}",
        log_with_metadata, metadata
    );

    // Example usage of macro_print_log
    let log_to_print = Log::new(
        // Unique ID for log entry
        "12345678-1234-1234-1234-1234567890ab",
        // Timestamp in ISO 8601 format
        &iso,
        // Log level
        &LogLevel::INFO,
        // Log component
        "PrintLogComponent",
        // Log message
        "Log to be printed",
        // Log format
        &LogFormat::CLF,
    );
    // Printing log
    macro_print_log!(log_to_print);
}
