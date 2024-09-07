// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # RustLogs (RLG) Log Level Examples
//!
//! This example demonstrates the usage of the `LogLevel` enum in RustLogs (RLG).
//! It covers displaying, parsing, numeric conversions, validation, and error handling
//! for different log levels such as `DEBUG`, `INFO`, `ERROR`, and more.

#![allow(missing_docs)]

use rlg::log_level::{LogLevel, ParseLogLevelError};
use std::convert::TryInto;
use std::str::FromStr;

/// Entry point for the RustLogs log level examples.
///
/// This function runs various examples demonstrating log level handling,
/// including string parsing, numeric conversions, validation, and error handling.
///
/// # Errors
///
/// Returns an error if any of the example functions fail.
pub(crate) fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 RustLogs Log Level Examples 🦀\n");

    log_level_display_example();
    log_level_parsing_example()?;
    log_level_try_into_example()?;
    log_level_includes_example();
    log_level_case_insensitivity_example()?;
    log_level_error_handling_example();
    log_level_numeric_conversion_example()?;

    println!("\n🎉 All examples completed successfully!");

    Ok(())
}

/// Demonstrates displaying log levels using the `LogLevel` enum.
///
/// This function prints the string representation of various log levels.
fn log_level_display_example() {
    println!("🦀  **Log Level Display Example**");
    println!("---------------------------------------------");

    let levels = [
        LogLevel::ALL,
        LogLevel::DEBUG,
        LogLevel::INFO,
        LogLevel::ERROR,
        LogLevel::FATAL,
        LogLevel::NONE,
        LogLevel::TRACE,
        LogLevel::VERBOSE,
        LogLevel::WARN,
        LogLevel::CRITICAL,
        LogLevel::DISABLED,
    ];

    for level in &levels {
        println!("    Log level: {}", level);
    }
}

/// Demonstrates parsing log levels from string representations.
///
/// This function parses various log levels and handles invalid inputs.
///
/// # Errors
///
/// Returns an error if parsing fails for invalid log level strings.
fn log_level_parsing_example() -> Result<(), Box<dyn std::error::Error>>
{
    println!("\n🦀  **Log Level Parsing Example**");
    println!("---------------------------------------------");

    let valid_levels = ["DEBUG", "INFO", "ERROR", "WARN", "CRITICAL"];
    let invalid_level = "Invalid";

    for &level_str in &valid_levels {
        let parsed_level = LogLevel::from_str(level_str)?;
        println!(
            "    Parsed log level: {} -> {:?}",
            level_str, parsed_level
        );
    }

    match LogLevel::from_str(invalid_level) {
        Err(e) => println!(
            "    ✅  Correctly failed to parse invalid log level: {}",
            e
        ),
        _ => {
            println!("    ❌  Unexpected success for invalid log level")
        }
    }

    Ok(())
}

/// Demonstrates converting from `String` to `LogLevel` using `TryInto`.
///
/// This function converts string representations of log levels to `LogLevel` enum variants.
///
/// # Errors
///
/// Returns an error if conversion fails for invalid log level strings.
fn log_level_try_into_example() -> Result<(), Box<dyn std::error::Error>>
{
    println!("\n🦀  **Log Level TryInto Example**");
    println!("---------------------------------------------");

    let valid_levels = vec![
        "ALL".to_string(),
        "DEBUG".to_string(),
        "INFO".to_string(),
        "ERROR".to_string(),
    ];

    for level in valid_levels {
        let log_level: LogLevel = level.clone().try_into()?;
        println!(
            "    Successfully converted '{}' to {:?}",
            level, log_level
        );
    }

    // Handle invalid level
    let invalid_level: Result<LogLevel, _> =
        "Invalid".to_string().try_into();
    if let Err(e) = invalid_level {
        println!(
            "    ✅  Correctly failed to convert invalid log level: {}",
            e
        );
    }

    Ok(())
}

/// Demonstrates the `includes` method of the `LogLevel` enum.
///
/// This function checks if a log level includes another log level, based on numeric precedence.
fn log_level_includes_example() {
    println!("\n🦀  **Log Level Includes Example**");
    println!("---------------------------------------------");

    // Print numeric values for debugging
    println!("    ALL: {}", LogLevel::ALL.to_numeric());
    println!("    ERROR: {}", LogLevel::ERROR.to_numeric());

    // Assuming ALL should include all levels
    assert!(
        LogLevel::ALL.includes(LogLevel::ERROR),
        "ALL should include ERROR"
    );

    // ERROR should include DEBUG if ERROR has a higher numeric value
    assert!(
        LogLevel::ERROR.includes(LogLevel::DEBUG),
        "ERROR should include DEBUG"
    );

    // DEBUG should not include ERROR, because DEBUG is lower in severity
    assert!(
        !LogLevel::DEBUG.includes(LogLevel::ERROR),
        "DEBUG should not include ERROR"
    );

    // WARN includes DEBUG if WARN has a higher numeric value
    assert!(
        LogLevel::WARN.includes(LogLevel::DEBUG),
        "WARN should include DEBUG"
    );

    println!("    ✅  Log level includes checks passed.");
}

/// Demonstrates case insensitivity when parsing log levels.
///
/// This function shows that log levels can be parsed regardless of case.
///
/// # Errors
///
/// Returns an error if parsing fails for valid log level strings (with mixed case).
fn log_level_case_insensitivity_example(
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🦀  **Log Level Case Insensitivity Example**");
    println!("---------------------------------------------");

    assert_eq!(LogLevel::from_str("debug")?, LogLevel::DEBUG);
    assert_eq!(LogLevel::from_str("INFO")?, LogLevel::INFO);
    assert_eq!(LogLevel::from_str("Trace")?, LogLevel::TRACE);

    println!("    ✅  Case insensitivity checks passed.");

    Ok(())
}

/// Demonstrates handling of errors related to log level parsing.
///
/// This function shows how to handle invalid log level strings and errors.
fn log_level_error_handling_example() {
    println!("\n🦀  **Log Level Error Handling Example**");
    println!("---------------------------------------------");

    let invalid_level = "Invalid";
    match LogLevel::from_str(invalid_level) {
        Err(e) => println!(
            "    ✅  Correctly handled invalid log level: {}",
            e
        ),
        _ => {
            println!("    ❌  Unexpected success for invalid log level")
        }
    }

    let error = ParseLogLevelError::new(invalid_level);
    println!("    ✅  ParseLogLevelError: {}", error);
}

/// Demonstrates numeric conversions for log levels.
///
/// This function converts log levels to their numeric representations and back.
fn log_level_numeric_conversion_example(
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🦀  **Log Level Numeric Conversion Example**");
    println!("---------------------------------------------");

    // Convert log levels to numeric values
    assert_eq!(LogLevel::ALL.to_numeric(), 0);
    assert_eq!(LogLevel::INFO.to_numeric(), 6);
    assert_eq!(LogLevel::ERROR.to_numeric(), 8);

    // Convert from numeric values to log levels
    assert_eq!(LogLevel::from_numeric(0), Some(LogLevel::ALL));
    assert_eq!(LogLevel::from_numeric(8), Some(LogLevel::ERROR));
    assert_eq!(LogLevel::from_numeric(11), None); // Invalid numeric value

    println!("    ✅  Log level numeric conversions passed.");

    Ok(())
}
