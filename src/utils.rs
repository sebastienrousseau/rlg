// utils.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::error::RlgResult;
use dtt::datetime::DateTime;

#[cfg(feature = "tokio")]
use std::path::Path;

#[cfg(feature = "tokio")]
use tokio::fs::{self, File, OpenOptions};
#[cfg(feature = "tokio")]
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

/// Generates a timestamp string in ISO 8601 format.
///
/// # Returns
///
/// A `String` containing the current timestamp in ISO 8601 format.
///
/// # Examples
///
/// ```rust,no_run
/// use rlg::utils::generate_timestamp;
///
/// let timestamp = generate_timestamp();
/// println!("Current timestamp: {}", timestamp);
/// ```
#[must_use]
pub fn generate_timestamp() -> String {
    DateTime::new().to_string()
}

/// Sanitizes a string for use in log messages.
///
/// This function replaces newlines and control characters with spaces.
///
/// # Arguments
///
/// * `message` - A string slice that holds the message to be sanitized.
///
/// # Returns
///
/// A `String` with sanitized content.
///
/// # Examples
///
/// ```
/// use rlg::utils::sanitize_log_message;
///
/// let message = "Hello\nWorld\r\u{0007}";
/// let sanitized = sanitize_log_message(message);
/// assert_eq!(sanitized, "Hello World  ");
/// ```
#[must_use]
pub fn sanitize_log_message(message: &str) -> String {
    message
        .replace(['\n', '\r'], " ")
        .replace(|c: char| c.is_control(), " ")
}

/// Checks if a file exists and is writable.
///
/// # Arguments
///
/// * `path` - A reference to a `Path` that holds the file path to check.
///
/// # Returns
///
/// A `RlgResult<bool>` which is `Ok(true)` if the file exists and is writable,
/// `Ok(false)` otherwise, or an error if the operation fails.
///
/// # Errors
///
/// This function returns an error if the file metadata cannot be read.
///
/// # Examples
///
/// ```no_run
/// use rlg::utils::is_file_writable;
/// use std::path::Path;
///
/// #[tokio::main]
/// async fn main() -> rlg::error::RlgResult<()> {
///     let path = Path::new("example.log");
///     let is_writable = is_file_writable(&path).await?;
///     println!("Is file writable: {}", is_writable);
///     Ok(())
/// }
/// ```
#[cfg(feature = "tokio")]
pub async fn is_file_writable(path: &Path) -> RlgResult<bool> {
    if path.exists() {
        let metadata = fs::metadata(path).await?;
        Ok(metadata.is_file() && !metadata.permissions().readonly())
    } else {
        // If the file doesn't exist, check if we can create it
        match File::create(path).await {
            Ok(_) => {
                fs::remove_file(path).await?;
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }
}

/// Truncates the file at the given path to the specified size.
///
/// # Arguments
///
/// * `path` - A reference to a `Path` that holds the file path to truncate.
/// * `size` - The size (in bytes) to truncate the file to.
///
/// # Returns
///
/// A `std::io::Result<()>` which is `Ok(())` if the operation succeeds,
/// or an error if it fails.
///
/// # Errors
///
/// This function returns an error if the file cannot be opened, or if
/// the seek or write operations fail.
///
/// # Examples
///
/// ```no_run
/// use rlg::utils::truncate_file;
/// use std::path::Path;
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     let path = Path::new("example.log");
///     truncate_file(&path, 1024).await?;
///     println!("File truncated successfully");
///     Ok(())
/// }
/// ```
#[cfg(feature = "tokio")]
pub async fn truncate_file(
    path: &Path,
    size: u64,
) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .await?;

    let file_size = file.metadata().await?.len();

    if size < file_size {
        // Read the content
        // SAFETY: Casting size to usize is safe here as we're truncating to a size that fits in memory for this operation.
        #[allow(clippy::cast_possible_truncation)]
        let mut content = vec![0; size as usize];
        file.read_exact(&mut content).await?;

        // Seek to the beginning of the file
        file.seek(std::io::SeekFrom::Start(0)).await?;

        // Write the truncated content
        file.write_all(&content).await?;
    }

    // Set the file length
    file.set_len(size).await?;

    Ok(())
}

/// Formats a file size in a human-readable format.
///
/// # Arguments
///
/// * `size` - The file size in bytes.
///
/// # Returns
///
/// A `String` containing the formatted file size.
///
/// # Examples
///
/// ```
/// use rlg::utils::format_file_size;
///
/// let size = 1_500_000;
/// let formatted = format_file_size(size);
/// assert_eq!(formatted, "1.43 MB");
/// ```
#[must_use]
pub fn format_file_size(size: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    // SAFETY: Loss of precision is acceptable for human-readable file size formatting.
    #[allow(clippy::cast_precision_loss)]
    let mut size_f = size as f64;
    let mut unit_index = 0;

    while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_index += 1;
    }

    format!("{size_f:.2} {unit}", unit = UNITS[unit_index])
}

/// Parses a datetime string in ISO 8601 format.
///
/// # Arguments
///
/// * `datetime_str` - A string slice containing the datetime in ISO 8601 format.
///
/// # Returns
///
/// A `RlgResult<DateTime>` which is `Ok(DateTime)` if parsing succeeds,
/// or an error if parsing fails.
///
/// # Errors
///
/// This function returns an error if the datetime string cannot be parsed.
///
/// # Examples
///
/// ```rust,no_run
/// use rlg::utils::parse_datetime;
///
/// let datetime_str = "2024-08-29T12:00:00Z";
/// match parse_datetime(datetime_str) {
///     Ok(dt) => println!("Parsed datetime: {}", dt),
///     Err(e) => eprintln!("Failed to parse datetime: {}", e),
/// }
/// ```
pub fn parse_datetime(datetime_str: &str) -> RlgResult<DateTime> {
    DateTime::parse(datetime_str)
        .map_err(|e| crate::error::RlgError::custom(e.to_string()))
}

/// Generates a highly unique, 16-character pseudo-random hex string suitable for OTLP span IDs.
///
/// # Returns
/// A `String` containing the span ID.
#[must_use]
pub fn generate_span_id() -> String {
    crate::commons::id::generate_random_hex()[..16].to_string()
}

/// Generates a highly unique, 32-character pseudo-random hex string suitable for OTLP trace IDs.
///
/// # Returns
/// A `String` containing the trace ID.
#[must_use]
pub fn generate_trace_id() -> String {
    crate::commons::id::generate_random_hex()
}

/// Checks if a directory is writable.
///
/// # Arguments
///
/// * `path` - A reference to a `Path` that holds the directory path to check.
///
/// # Returns
///
/// A `RlgResult<bool>` which is `Ok(true)` if the directory is writable,
/// `Ok(false)` otherwise, or an error if the operation fails.
///
/// # Errors
///
/// This function returns an error if the temporary file used for testing writability cannot be removed.
///
/// # Examples
///
/// ```no_run
/// use rlg::utils::is_directory_writable;
/// use std::path::Path;
///
/// #[tokio::main]
/// async fn main() -> rlg::error::RlgResult<()> {
///     let path = Path::new(".");
///     let is_writable = is_directory_writable(&path).await?;
///     println!("Is directory writable: {}", is_writable);
///     Ok(())
/// }
/// ```
#[cfg(feature = "tokio")]
pub async fn is_directory_writable(path: &Path) -> RlgResult<bool> {
    if !path.is_dir() {
        return Ok(false);
    }

    let test_file = path.join(".rlg_write_test");
    match File::create(&test_file).await {
        Ok(_) => {
            fs::remove_file(&test_file).await?;
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}
