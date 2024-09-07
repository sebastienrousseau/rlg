// Copyright © 2023-2024 Rust Utilities Library. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # Rust Utilities Library Usage Examples
//!
//! This program demonstrates the usage of various utilities in the Rust Utilities Library (RLG),
//! including sanitization of log messages, file operations (writability checks, truncation),
//! file size formatting, and datetime parsing. Each function provides examples with detailed output.

#![allow(missing_docs)]

use rlg::utils::*;
use std::error::Error;
use tempfile::tempdir;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

/// Entry point for the Rust Utilities Library examples.
///
/// This function runs the example functions demonstrating sanitization, file operations,
/// file size formatting, and datetime parsing.
///
/// # Errors
///
/// Returns an error if any of the example functions fail.
#[tokio::main]
pub(crate) async fn main() -> Result<(), Box<dyn Error>> {
    println!("🦀 Rust Utilities Library Usage Examples 🦀\n");

    sanitize_log_message_example();
    file_operations_example().await?;
    format_file_size_example();
    parse_datetime_example()?;

    println!("\n🎉 All examples completed successfully!");

    Ok(())
}

/// Demonstrates the sanitization of log messages.
///
/// This function showcases how `sanitize_log_message` can remove unwanted characters
/// (e.g., newlines, carriage returns, control characters) from a log message.
fn sanitize_log_message_example() {
    println!("🦀  **Log Message Sanitization Example**");
    println!("---------------------------------------------");

    let input = "Hello\nWorld\r\u{0007}";
    let sanitized = sanitize_log_message(input);
    println!("    Original: {}\n    Sanitized: {}", input, sanitized);
}

/// Demonstrates various file operations: checking if a file is writable, truncating a file,
/// and checking if a directory is writable.
///
/// This function uses `is_file_writable`, `truncate_file`, and `is_directory_writable`.
///
/// # Errors
///
/// Returns an error if any file operation fails.
async fn file_operations_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀  **File Operations Example**");
    println!("---------------------------------------------");

    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("test.log");

    // Example 1: Check if file is writable
    println!("\n🔍  Checking if file is writable:");
    if is_file_writable(&file_path).await.unwrap() {
        println!("    ✅  File is writable: {}", file_path.display());
    } else {
        println!(
            "    ❌  File is not writable: {}",
            file_path.display()
        );
    }

    // Example 2: Truncate a file
    println!("\n✂️  Truncating file content:");
    let mut file = File::create(&file_path).await?;
    file.write_all(b"Hello, World!").await?;
    truncate_file(&file_path, 5).await?;
    let content = fs::read_to_string(&file_path).await?;
    println!("    ✅  Truncated content: {}", content);

    // Example 3: Check if directory is writable
    println!("\n📂  Checking if directory is writable:");
    if is_directory_writable(temp_dir.path()).await.unwrap() {
        println!(
            "    ✅  Directory is writable: {}",
            temp_dir.path().display()
        );
    } else {
        println!(
            "    ❌  Directory is not writable: {}",
            temp_dir.path().display()
        );
    }

    Ok(())
}

/// Demonstrates formatting of file sizes using `format_file_size`.
///
/// This function prints formatted file sizes for different byte values.
fn format_file_size_example() {
    println!("\n🦀  **File Size Formatting Example**");
    println!("---------------------------------------------");

    let sizes = vec![1023, 1024, 1024 * 1024, 1024 * 1024 * 1024];

    for size in sizes {
        let formatted = format_file_size(size);
        println!(
            "    Size: {} bytes -> Formatted: {}",
            size, formatted
        );
    }
}

/// Demonstrates parsing a datetime string using `parse_datetime`.
///
/// This function attempts to parse valid and invalid ISO 8601 date-time strings.
///
/// # Errors
///
/// Returns an error if parsing fails for valid date-time strings.
fn parse_datetime_example() -> Result<(), Box<dyn Error>> {
    println!("\n🦀  **DateTime Parsing Example**");
    println!("---------------------------------------------");

    let valid_datetime = "2023-05-17T15:30:45Z";
    let invalid_datetime = "invalid datetime";

    println!("\n📅  Parsing valid datetime string:");
    match parse_datetime(valid_datetime) {
        Ok(_) => println!(
            "    ✅  Parsed valid datetime: {}",
            valid_datetime
        ),
        Err(_) => println!(
            "    ❌  Failed to parse valid datetime: {}",
            valid_datetime
        ),
    }

    println!("\n📅  Parsing invalid datetime string:");
    match parse_datetime(invalid_datetime) {
        Ok(_) => println!(
            "    ❌  Unexpected success for invalid datetime: {}",
            invalid_datetime
        ),
        Err(_) => println!(
            "    ✅  Correctly failed to parse invalid datetime: {}",
            invalid_datetime
        ),
    }

    Ok(())
}
