// rotation.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Log rotation policies: size, time, date, and count-based.
//!
//! Wrap a file sink with [`RotatingFile`][crate::rotation::RotatingFile]
//! to enforce automatic rotation.
//! On rotation, the current file is renamed with a timestamp suffix and
//! a fresh file is opened at the original path.

use crate::config::LogRotation;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// File writer that enforces a [`LogRotation`] policy.
#[derive(Debug)]
pub struct RotatingFile {
    /// Current open file handle.
    file: File,
    /// Path to the current log file.
    path: PathBuf,
    /// Rotation policy to enforce.
    policy: LogRotation,
    /// Bytes written to the current file.
    bytes_written: u64,
    /// Events written to the current file (for count-based rotation).
    events_written: u32,
    /// Time when the current file was opened (for time-based rotation).
    opened_at: Instant,
    /// Date string when the current file was opened (for date-based rotation).
    opened_date: String,
}

impl RotatingFile {
    /// Open (or create) a log file with the given rotation policy.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if the file cannot be opened or created.
    pub fn open(path: &Path, policy: LogRotation) -> io::Result<Self> {
        let file =
            OpenOptions::new().create(true).append(true).open(path)?;
        let bytes_written = file.metadata().map_or(0, |m| m.len());
        Ok(Self {
            file,
            path: path.to_path_buf(),
            policy,
            bytes_written,
            events_written: 0,
            opened_at: Instant::now(),
            opened_date: today_date_string(),
        })
    }

    /// Write a batch of bytes, then rotate if the policy threshold is met.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if the write or file rotation fails.
    pub fn write_batch(
        &mut self,
        data: &[u8],
        event_count: u32,
    ) -> io::Result<()> {
        self.file.write_all(data)?;
        self.bytes_written += data.len() as u64;
        self.events_written += event_count;

        if self.should_rotate() {
            self.rotate()?;
        }
        Ok(())
    }

    /// Checks whether the current file should be rotated.
    fn should_rotate(&self) -> bool {
        match self.policy {
            LogRotation::Size(max_bytes) => {
                self.bytes_written >= max_bytes.get()
            }
            LogRotation::Time(seconds) => {
                self.opened_at.elapsed().as_secs() >= seconds.get()
            }
            LogRotation::Date => {
                today_date_string() != self.opened_date
            }
            LogRotation::Count(max_events) => {
                self.events_written >= max_events
            }
        }
    }

    /// Rotates the current file by renaming it with a timestamp suffix
    /// and opening a new file at the original path.
    fn rotate(&mut self) -> io::Result<()> {
        // Flush and drop the current file handle.
        self.file.flush()?;

        // Build the rotated file name.
        let timestamp = chrono_like_timestamp();
        let rotated_name = if let Some(ext) = self.path.extension() {
            let stem = self.path.with_extension("");
            PathBuf::from(format!(
                "{}.{timestamp}.{}",
                stem.display(),
                ext.to_string_lossy()
            ))
        } else {
            PathBuf::from(format!(
                "{}.{timestamp}",
                self.path.display()
            ))
        };

        fs::rename(&self.path, &rotated_name)?;

        // Open a new file at the original path.
        self.file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        self.bytes_written = 0;
        self.events_written = 0;
        self.opened_at = Instant::now();
        self.opened_date = today_date_string();

        Ok(())
    }
}

/// Returns today's date as `YYYY-MM-DD`.
fn today_date_string() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    // Simple date calculation (no leap-second precision needed for rotation).
    let days = secs / 86400;
    let (year, month, day) = days_to_ymd(days);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Returns a compact timestamp for rotated file names: `YYYYMMDD-HHMMSS`.
fn chrono_like_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let days = secs / 86400;
    let (year, month, day) = days_to_ymd(days);
    let day_secs = secs % 86400;
    let h = day_secs / 3600;
    let m = (day_secs % 3600) / 60;
    let s = day_secs % 60;
    format!("{year:04}{month:02}{day:02}-{h:02}{m:02}{s:02}")
}

/// Converts days since Unix epoch to (year, month, day).
const fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU64;

    #[test]
    fn test_today_date_string_format() {
        let date = today_date_string();
        // YYYY-MM-DD
        assert_eq!(date.len(), 10);
        assert_eq!(&date[4..5], "-");
        assert_eq!(&date[7..8], "-");
    }

    #[test]
    fn test_chrono_like_timestamp_format() {
        let ts = chrono_like_timestamp();
        // YYYYMMDD-HHMMSS
        assert_eq!(ts.len(), 15);
        assert_eq!(&ts[8..9], "-");
    }

    #[test]
    fn test_days_to_ymd_epoch() {
        let (y, m, d) = days_to_ymd(0);
        assert_eq!((y, m, d), (1970, 1, 1));
    }

    #[test]
    fn test_rotating_file_size_based() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.log");
        let policy = LogRotation::Size(NonZeroU64::new(100).unwrap());
        let mut rf = RotatingFile::open(&path, policy).unwrap();
        // Write 50 bytes — no rotation
        rf.write_batch(&[b'A'; 50], 1).unwrap();
        assert!(path.exists());
        // Write 60 more bytes — triggers rotation
        rf.write_batch(&[b'B'; 60], 1).unwrap();
        // Original path should still exist (new file)
        assert!(path.exists());
        // There should be a rotated file
        let entries: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        assert!(entries.len() >= 2, "expected rotated file");
    }

    #[test]
    fn test_rotating_file_count_based() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("count.log");
        let policy = LogRotation::Count(3);
        let mut rf = RotatingFile::open(&path, policy).unwrap();
        rf.write_batch(b"event1\n", 1).unwrap();
        rf.write_batch(b"event2\n", 1).unwrap();
        rf.write_batch(b"event3\n", 1).unwrap(); // triggers rotation
        let entries: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        assert!(entries.len() >= 2, "expected rotated file");
    }

    #[test]
    fn test_rotating_file_no_extension() {
        // Drives the `else` branch in `rotate()` that handles a path
        // without an extension (lines 112-114 in coverage report).
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rawlogfile");
        let policy = LogRotation::Size(NonZeroU64::new(10).unwrap());
        let mut rf = RotatingFile::open(&path, policy).unwrap();
        rf.write_batch(b"0123456789X", 1).unwrap(); // > 10 bytes → rotate
        let entries: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        assert!(entries.len() >= 2);
        // The rotated name must start with the original stem.
        let names: Vec<_> = entries
            .iter()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert!(
            names.iter().any(|n| n.starts_with("rawlogfile.")),
            "no rotated file found in {names:?}"
        );
    }

    #[test]
    fn test_rotating_file_time_based_does_not_trigger_immediately() {
        // Exercises the Time policy branch in `should_rotate()`.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("time.log");
        let policy = LogRotation::Time(NonZeroU64::new(3600).unwrap());
        let mut rf = RotatingFile::open(&path, policy).unwrap();
        rf.write_batch(b"data\n", 1).unwrap();
        // Only the current file should exist; 3600s hasn't elapsed.
        let entries: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_rotating_file_date_based_does_not_trigger_immediately() {
        // Exercises the Date policy branch in `should_rotate()`.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("date.log");
        let mut rf =
            RotatingFile::open(&path, LogRotation::Date).unwrap();
        rf.write_batch(b"data\n", 1).unwrap();
        let entries: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_rotating_file_open_existing_picks_up_size() {
        // Pre-create a file with known content, then `open()` it.
        // Verifies `bytes_written` is populated from existing metadata.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("existing.log");
        fs::write(&path, b"already here\n").unwrap();
        let policy = LogRotation::Size(NonZeroU64::new(5).unwrap());
        let mut rf = RotatingFile::open(&path, policy).unwrap();
        // First write should trigger rotation because preexisting bytes
        // already exceed the threshold.
        rf.write_batch(b"x", 1).unwrap();
        let entries: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        assert!(entries.len() >= 2);
    }
}
