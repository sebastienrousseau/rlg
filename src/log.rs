// log.rs
// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{Config, LogFormat, LogLevel, RlgError, RlgResult};
use dtt::datetime::DateTime;
use hostname;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Write as FmtWrite},
    io,
};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use vrd::random::Random;

/// The `Log` struct provides an easy way to log a message to the console.
/// It contains a set of defined fields to create a simple log message with a readable output format.
#[derive(
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    Eq,
    Hash,
)]
pub struct Log {
    /// The session ID for the log entry.
    pub session_id: String,
    /// The time the log entry was created.
    pub time: String,
    /// The log level of the message.
    pub level: LogLevel,
    /// The component that generated the log message.
    pub component: String,
    /// The description of the log message.
    pub description: String,
    /// The format of the log message.
    pub format: LogFormat,
}

impl Default for Log {
    fn default() -> Log {
        Log {
            session_id: String::default(),
            time: String::default(),
            level: LogLevel::INFO,
            component: String::default(),
            description: String::default(),
            format: LogFormat::CLF,
        }
    }
}

impl Log {
    /// Logs a message asynchronously using a pre-allocated buffer to reduce memory allocation.
    ///
    /// This function formats the log message according to the specified log format and writes it to
    /// both the log file and standard output. It ensures that the log file is flushed after every write
    /// to guarantee data persistence.
    ///
    /// # Returns
    /// * `RlgResult<()>` - Result with `Ok(())` if the logging succeeds, or `RlgError` if any errors occur.
    pub async fn log(&self) -> RlgResult<()> {
        let mut log_message = String::with_capacity(256);

        // Format the log message based on the specified log format.
        let write_result = match self.format {
        LogFormat::CLF => writeln!(
            log_message,
            "SessionID={} Timestamp={} Description={} Level={} Component={} Format=CLF",
            self.session_id, self.time, self.description, self.level, self.component
        ),
        LogFormat::JSON => writeln!(
            log_message,
            "{{\"SessionID\":\"{}\",\"Timestamp\":\"{}\",\"Level\":\"{}\",\"Component\":\"{}\",\"Description\":\"{}\",\"Format\":\"JSON\"}}",
            self.session_id, self.time, self.level, self.component, self.description
        ),
        LogFormat::CEF => writeln!(
            log_message,
            "CEF:0|{}|{}|{}|{}|{}|CEF",
            self.session_id, self.time, self.level, self.component, self.description
        ),
        _ => writeln!(log_message, "Unsupported format"),  // Handle unsupported formats
    };

        write_result.map_err(|e| {
            RlgError::FormattingError(format!(
                "Formatting error: {}",
                e
            ))
        })?;

        // Extract the log file path from the configuration.
        let log_file_path;
        {
            let config = Config::load_async(None::<&str>)
                .await
                .map_err(|e| {
                    RlgError::IoError(io::Error::new(
                        io::ErrorKind::Other,
                        e,
                    ))
                })?;
            log_file_path = config.read().log_file_path.clone();
        }

        // Open the log file for appending, or create it if it does not exist.
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)
            .await
            .map_err(|e| {
                RlgError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to open log file: {}", e),
                ))
            })?;

        file.write_all(log_message.as_bytes()).await.map_err(|e| {
            RlgError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to write to log file: {}", e),
            ))
        })?;

        file.flush().await.map_err(|e| {
            RlgError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to flush log file: {}", e),
            ))
        })?;

        Ok(())
    }

    /// Creates a new log entry with provided details.
    pub fn new(
        session_id: &str,
        time: &str,
        level: &LogLevel,
        component: &str,
        description: &str,
        format: &LogFormat,
    ) -> Self {
        Self {
            session_id: session_id.to_string(),
            time: time.to_string(),
            level: *level,
            component: component.to_string(),
            description: description.to_string(),
            format: *format,
        }
    }

    /// Writes a log entry to the log file using the provided details.
    pub async fn write_log_entry(
        log_level: LogLevel,
        process: &str,
        message: &str,
        log_format: LogFormat,
    ) -> RlgResult<()> {
        let config = Config::load_async(None::<&str>).await?;

        // Open or create the log file
        let log_file_path = config.read().log_file_path.clone();
        let mut log_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&log_file_path)
            .await
            .map_err(|e| {
                RlgError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Failed to open or create log file '{}': {}",
                        log_file_path.display(),
                        e
                    ),
                ))
            })?;

        // Create the log entry
        let log_entry = Log::new(
            &Random::default().int(0, 1_000_000_000).to_string(),
            &DateTime::new().to_string(),
            &log_level,
            process,
            message,
            &log_format,
        );

        // Format the log entry according to the specified log format
        let formatted_entry = log_entry.to_string();

        // Write the formatted log entry to the file asynchronously
        log_file
            .write_all(formatted_entry.as_bytes())
            .await
            .map_err(|e| {
                RlgError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to write log entry: {}", e),
                ))
            })?;

        // Optionally, flush the file to ensure all data is written
        log_file.flush().await.map_err(|e| {
            RlgError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to flush log file: {}", e),
            ))
        })?;

        Ok(())
    }
}

impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.format {
            LogFormat::CLF => write!(
                f,
                "SessionID={} Timestamp={} Description={} Level={} Component={}",
                self.session_id, self.time, self.description, self.level, self.component
            ),
            LogFormat::JSON => write!(
                f,
                "{{\"SessionID\":\"{}\",\"Timestamp\":\"{}\",\"Level\":\"{}\",\"Component\":\"{}\",\"Description\":\"{}\",\"Format\":\"JSON\"}}",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::CEF => write!(
                f,
                "CEF:0|{}|{}|{}|{}|{}|CEF",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::ELF => write!(
                f,
                "ELF:0|{}|{}|{}|{}|{}|ELF",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::W3C => write!(
                f,
                "W3C:0|{}|{}|{}|{}|{}|W3C",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::GELF => write!(
                f,
                r#"{{
                    "version": "1.1",
                    "host": "{}",
                    "short_message": "{}",
                    "level": "{:?}",
                    "timestamp": "{}",
                    "component": "{}",
                    "session_id": "{}"
                }}"#,
                self.component, self.description, self.level, self.time, self.component, self.session_id
            ),
            LogFormat::ApacheAccessLog => write!(
                f,
                "{} - - [{}] \"{}\" {} {}",
                hostname::get().map_err(|_| fmt::Error)?.to_string_lossy(),
                self.time,
                self.description,
                self.level,
                self.component
            ),
            LogFormat::Logstash => write!(
                f,
                r#"{{
                    "@timestamp": "{}",
                    "level": "{}",
                    "component": "{}",
                    "message": "{}"
                }}"#,
                self.time, self.level, self.component, self.description
            ),
            LogFormat::Log4jXML => write!(
                f,
                r#"<log4j:event logger="{}" timestamp="{}" level="{}" thread="{}"><log4j:message>{}</log4j:message></log4j:event>"#,
                self.component, self.time, self.level, self.session_id, self.description
            ),
            LogFormat::NDJSON => write!(
                f,
                r#"{{
                    "timestamp": "{}",
                    "level": "{}",
                    "component": "{}",
                    "message": "{}"
                }}"#,
                self.time, self.level, self.component, self.description
            ),
        }
    }
}
