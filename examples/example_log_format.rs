// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # RustLogs (RLG) Log Format Examples
//!
//! This example demonstrates the usage of the `LogFormat` struct in RustLogs (RLG).
//! It covers formatting, parsing from strings, validation of log messages, and handling errors
//! for different log formats, such as CLF, JSON, CEF, W3C, and others.

#![allow(missing_docs)]

use rlg::log_format::LogFormat;
use std::str::FromStr;

/// Entry point for the RustLogs log format examples.
///
/// This function runs various examples demonstrating log format handling,
/// including string parsing, formatting, validation, and error management.
///
/// # Errors
///
/// Returns an error if any of the example functions fail.
pub(crate) fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 RustLogs Log Format Examples 🦀\n");

    log_format_display_example();
    log_format_parsing_example()?;
    log_format_validation_example();
    log_format_format_log_example()?;
    log_format_error_handling_example();

    println!("\n🎉 All examples completed successfully!");

    Ok(())
}

/// Demonstrates the display of log formats using the `LogFormat` enum.
///
/// This function prints the string representation of various log formats.
fn log_format_display_example() {
    println!("🦀  **Log Format Display Example**");
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
        println!("    Log format: {}", format);
    }
}

/// Demonstrates parsing of log formats from string representations.
///
/// This function parses various log formats and handles invalid inputs.
///
/// # Errors
///
/// Returns an error if parsing fails for invalid log format strings.
fn log_format_parsing_example() -> Result<(), Box<dyn std::error::Error>>
{
    println!("\n🦀  **Log Format Parsing Example**");
    println!("---------------------------------------------");

    let valid_formats = ["CLF", "JSON", "CEF", "ELF", "W3C", "GELF"];
    let invalid_format = "InvalidFormat";

    for &format_str in &valid_formats {
        let parsed_format = LogFormat::from_str(format_str)?;
        println!(
            "    Parsed log format: {} -> {:?}",
            format_str, parsed_format
        );
    }

    match LogFormat::from_str(invalid_format) {
        Ok(_) => println!(
            "    ❌  Unexpected success for invalid log format: {}",
            invalid_format
        ),
        Err(e) => println!(
            "    ✅  Correctly failed to parse invalid log format: {}",
            e
        ),
    }

    Ok(())
}

/// Demonstrates the validation of log messages using different log formats.
///
/// This function validates log messages for formats such as CLF, JSON, CEF, and others.
fn log_format_validation_example() {
    println!("\n🦀  **Log Format Validation Example**");
    println!("---------------------------------------------");

    // Valid log messages
    let valid_clf_log = "127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326";
    let valid_json_log = "{\"key\":\"value\"}";
    let valid_cef_log =
        "CEF:0|security|threat|1.0|100|Something happened|5|msg=hello";
    let valid_w3c_log = "#Fields: date time c-ip cs-method cs-uri-stem sc-status\n2024-01-01 12:34:56 192.168.0.1 GET /index.html 200";

    assert!(
        LogFormat::CLF.validate(valid_clf_log),
        "Valid CLF log failed validation"
    );
    assert!(
        LogFormat::JSON.validate(valid_json_log),
        "Valid JSON log failed validation"
    );
    assert!(
        LogFormat::CEF.validate(valid_cef_log),
        "Valid CEF log failed validation"
    );
    assert!(
        LogFormat::W3C.validate(valid_w3c_log),
        "Valid W3C log failed validation"
    );

    println!("    ✅  Successfully validated valid log messages.");

    // Invalid log messages
    assert!(
        !LogFormat::CLF.validate("Invalid CLF log"),
        "Invalid CLF log should have failed"
    );
    assert!(
        !LogFormat::JSON.validate("Invalid JSON"),
        "Invalid JSON log should have failed"
    );
    assert!(
        !LogFormat::CEF.validate("Invalid CEF log"),
        "Invalid CEF log should have failed"
    );

    println!(
        "    ✅  Correctly handled invalid log message validations."
    );
}

/// Demonstrates formatting of log messages using different log formats.
///
/// This function formats log messages using supported formats such as CLF, JSON, CEF, and others.
///
/// # Errors
///
/// Returns an error if the log formatting fails (e.g., for invalid JSON).
fn log_format_format_log_example(
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🦀  **Log Format Formatting Example**");
    println!("---------------------------------------------");

    // Valid formatting
    let clf_log = "127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326";
    let json_log = "{\"key\":\"value\"}";
    let cef_log =
        "CEF:0|security|threat|1.0|100|Something happened|5|msg=hello";

    assert_eq!(LogFormat::CLF.format_log(clf_log)?, clf_log);
    println!("    ✅  CLF log formatted successfully.");

    let formatted_json = LogFormat::JSON.format_log(json_log)?;
    println!(
        "    ✅  JSON log formatted successfully:\n    {}",
        formatted_json
    );

    assert_eq!(LogFormat::CEF.format_log(cef_log)?, cef_log);
    println!("    ✅  CEF log formatted successfully.");

    // Invalid JSON formatting
    let invalid_json_log = "Invalid JSON";
    if LogFormat::JSON.format_log(invalid_json_log).is_err() {
        println!(
            "    ✅  Correctly failed to format invalid JSON log."
        );
    }

    Ok(())
}

/// Demonstrates handling of errors related to log format parsing and formatting.
///
/// This function shows how to handle invalid log format strings and log message formatting failures.
fn log_format_error_handling_example() {
    println!("\n🦀  **Log Format Error Handling Example**");
    println!("---------------------------------------------");

    // Parsing error
    let invalid_format_str = "InvalidFormat";
    let result = LogFormat::from_str(invalid_format_str);
    if result.is_err() {
        println!(
            "    ✅  Correctly handled invalid log format: {}",
            invalid_format_str
        );
    }

    // Formatting error for invalid JSON
    let invalid_json = "Invalid JSON";
    let format_result = LogFormat::JSON.format_log(invalid_json);
    if let Err(e) = format_result {
        println!(
            "    ✅  Correctly handled log formatting error: {}",
            e
        );
    }
}
