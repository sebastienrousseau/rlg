// utils.rs
// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::error::RlgResult;
use dtt::datetime::DateTime;
use std::path::Path;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncSeekExt;

/// Generates a timestamp string in ISO 8601 format.
pub fn generate_timestamp() -> String {
    DateTime::new().to_string()
}

/// Sanitizes a string for use in log messages.
///
/// This function replaces newlines and control characters with spaces.
pub fn sanitize_log_message(message: &str) -> String {
    message
        .replace('\n', " ")
        .replace('\r', " ")
        .replace(|c: char| c.is_control(), " ")
}

/// Checks if a file exists and is writable.
pub async fn is_file_writable(path: &Path) -> RlgResult<bool> {
    if path.exists() {
        let metadata = fs::metadata(path).await?;
        Ok(metadata.is_file() && !metadata.permissions().readonly())
    } else {
        // If the file doesn't exist, check if we can create it
        match fs::File::create(path).await {
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
/// * `path` - The path to the file to be truncated.
/// * `size` - The size (in bytes) to truncate the file to.
pub async fn truncate_file(
    path: &Path,
    size: u64,
) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).open(path).await?;

    // Truncate the file to the specified size.
    file.set_len(size).await?;

    // Optionally reposition the cursor to the end of the truncated content.
    file.seek(tokio::io::SeekFrom::Start(size)).await?;

    Ok(())
}

/// Formats a file size in a human-readable format.
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
pub fn parse_datetime(datetime_str: &str) -> RlgResult<DateTime> {
    DateTime::parse(datetime_str)
        .map_err(|e| crate::error::RlgError::custom(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    #[test]
    fn test_sanitize_log_message() {
        let input = "Hello\nWorld\r\u{0007}";
        let expected = "Hello World  ";
        assert_eq!(sanitize_log_message(input), expected);
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(1023), "1023.00 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[tokio::test]
    async fn test_is_file_writable() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.log");

        // Test non-existent file
        assert!(is_file_writable(&file_path).await.unwrap());

        // Test writable file
        File::create(&file_path).await.unwrap();
        assert!(is_file_writable(&file_path).await.unwrap());

        // Test read-only file
        let mut perms =
            fs::metadata(&file_path).await.unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&file_path, perms).await.unwrap();
        assert!(!is_file_writable(&file_path).await.unwrap());
    }

    #[tokio::test]
    async fn test_truncate_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.log");

        let mut file = File::create(&file_path).await.unwrap();
        file.write_all(b"Hello, World!").await.unwrap();

        truncate_file(&file_path, 5).await.unwrap();

        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "Hello");
    }

    #[test]
    fn test_parse_datetime() {
        let test_case = "2023-05-17T15:30:45Z";
        assert!(parse_datetime(test_case).is_ok());

        assert!(parse_datetime("invalid datetime").is_err());
    }
}
