// utils.rs
// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::error::RlgResult;
use dtt::datetime::DateTime;
use std::path::Path;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncSeekExt, AsyncWriteExt, AsyncReadExt};

/// Generates a timestamp string in ISO 8601 format.
///
/// # Returns
///
/// A `String` containing the current timestamp in ISO 8601 format.
///
/// # Examples
///
/// ```
/// use rlg::utils::generate_timestamp;
///
/// let timestamp = generate_timestamp();
/// println!("Current timestamp: {}", timestamp);
/// ```
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
pub fn sanitize_log_message(message: &str) -> String {
    message
        .replace('\n', " ")
        .replace('\r', " ")
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
/// # Examples
///
/// ```
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
/// # Examples
///
/// ```
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
pub async fn truncate_file(path: &Path, size: u64) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .await?;

    let file_size = file.metadata().await?.len();

    if size < file_size {
        // Read the content
        let mut content = Vec::new();
        file.read_to_end(&mut content).await?;

        // Truncate the content
        content.truncate(size as usize);

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
pub fn format_file_size(size: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
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
/// # Examples
///
/// ```
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
/// # Examples
///
/// ```
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
