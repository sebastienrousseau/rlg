// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{Config, LogFormat, LogLevel};
use dtt::DateTime;
use hostname;
use std::{
    fmt::{self, Write as FmtWrite},
    io::{self, stdout, Write},
};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use vrd::random::Random;

/// The `Log` struct provides an easy way to log a message to the console.
/// It contains a set of defined fields to create a simple log message with a readable output format.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Log {
    /// A string that holds a session ID. The session ID is a unique
    /// identifier for the current session. A random GUID (Globally
    /// Unique Identifier) is generated by default.
    pub session_id: String,
    /// A string that holds the timestamp in ISO 8601 format.
    pub time: String,
    /// A string that holds the level (INFO, WARN, ERROR, etc.).
    pub level: LogLevel,
    /// A string that holds the component name.
    pub component: String,
    /// A string that holds the description of the log message.
    pub description: String,
    /// A string that holds the log format.
    pub format: LogFormat,
}

impl Default for Log {
    /// This implementation allows the Log struct to be created with default values.
    /// It creates a new instance of the Log struct with empty strings for the session_id,
    /// time, component, and description fields, and LogLevel::INFO for the level field.
    /// This is useful when creating a new instance of the Log struct. It allows the struct
    /// to be created with default values, and then the fields can be set to the desired values.
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
    /// The message is formatted according to the specified log format and then written to a file.
    /// Additionally, the message is printed to the standard output and the output buffer is flushed.
    ///
    /// # Errors
    ///
    /// Returns an `io::Result<()>` indicating the outcome of the logging operation.
    /// An error is returned if there's an issue with string formatting or IO operations (file writing or flushing stdout).
    ///
    pub async fn log(&self) -> io::Result<()> {
        let mut log_message = String::with_capacity(256);

        let write_result = match self.format {
            LogFormat::CLF => write!(
                log_message,
                "SessionID={} Timestamp={} Description={} Level={} Component={} Format=CLF",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::JSON => write!(
                log_message,
                "{{\"SessionID\":\"{}\",\"Timestamp\":\"{}\",\"Level\":\"{}\",\"Component\":\"{}\",\"Description\":\"{}\"}} Format=JSON",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::CEF => write!(
                log_message,
                "CEF:0|{}|{}|{}|{}|{}|CEF",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::ELF => write!(
                log_message,
                "ELF:0|{}|{}|{}|{}|{}|ELF",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::W3C => write!(
                log_message,
                "W3C:0|{}|{}|{}|{}|{}|W3C",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::GELF => write!(
                log_message,
                "GELF:0|{}|{}|{}|{}|{}|GELF",
                self.session_id, self.time, self.level, self.component, self.description
            ),
            LogFormat::ApacheAccessLog => write!(
                log_message,
                "{} - - [{}] \"{}\" {} {}",
                hostname::get().unwrap().to_string_lossy(),
                self.time,
                self.description,
                self.level,
                self.component
            ),
            LogFormat::Logstash => write!(
                log_message,
                "{{\"@timestamp\":\"{}\",\"level\":\"{}\",\"component\":\"{}\",\"message\":\"{}\"}}",
                self.time,
                self.level,
                self.component,
                self.description
            ),
            LogFormat::Log4jXML => write!(
                log_message,
                "<log4j:event logger=\"{}\" timestamp=\"{}\" level=\"{}\" thread=\"{}\"><log4j:message>{}</log4j:message></log4j:event>",
                self.component,
                self.time,
                self.level,
                self.session_id,
                self.description
            ),
            LogFormat::NDJSON => write!(
                log_message,
                "{{\"timestamp\":\"{}\",\"level\":\"{}\",\"component\":\"{}\",\"message\":\"{}\"}}",
                self.time,
                self.level,
                self.component,
                self.description
            ),
        };

        // Handle potential formatting errors
        write_result.map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Formatting error: {}", e),
            )
        })?;

        // Attempt to write the log message to a file
        let config = Config::load()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let log_file_path_display = config.log_file_path_display();
        let mut file = tokio::fs::File::create(&config.log_file_path)
            .await
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Failed to open log file '{}': {}",
                        log_file_path_display, e
                    ),
                )
            })?;

        file.write_all(log_message.as_bytes()).await.map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to write to log file: {}", e),
            )
        })?;

        // Printing to stdout and flushing, with error handling if needed
        println!("{log_message}");
        stdout().flush().map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to flush stdout: {}", e),
            )
        })?;

        Ok(())
    }

    /// Creates a new log entry with provided details.
    ///
    /// # Parameters
    ///
    /// - `session_id`: A unique identifier for the session.
    /// - `time`: The timestamp in ISO 8601 format.
    /// - `level`: The logging level.
    /// - `component`: The component generating the log.
    /// - `description`: The log message.
    /// - `format`: The format for the log message.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `Log`.
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
    ///
    /// # Parameters
    ///
    /// - `log_level`: The severity level of the log.
    /// - `process`: The process name generating the log.
    /// - `message`: The log message.
    /// - `log_format`: The format of the log message.
    ///
    /// # Returns
    ///
    /// A `std::io::Result<()>` indicating the success or failure of writing the log entry.
    pub async fn write_log_entry(
        log_level: LogLevel,
        process: &str,
        message: &str,
        log_format: LogFormat,
    ) -> io::Result<()> {
        // Load configuration
        let config = Config::load().map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to load config: {}", e),
            )
        })?;

        // Open or create the log file
        let log_file_path = config.log_file_path.clone();
        let mut log_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&log_file_path)
            .await
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Failed to open or create log file '{}': {}",
                        log_file_path.display(),
                        e
                    ),
                )
            })?;

        // Create the log entry
        let log_entry = Log::new(
            &Random::default().int(0, 1_000_000_000).to_string(),
            &DateTime::new().iso_8601,
            &log_level,
            process,
            message,
            &log_format,
        );

        // Format the log entry according to the specified log format
        let formatted_entry = log_format
            .format_log(&log_entry.to_string())
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to format log entry: {}", e),
                )
            })?;

        // Write the formatted log entry to the file asynchronously
        log_file
            .write_all(formatted_entry.as_bytes())
            .await
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to write log entry: {}", e),
                )
            })?;

        // Optionally, you can flush the file to ensure all data is written
        log_file.flush().await.map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to flush log file: {}", e),
            )
        })?;

        Ok(())
    }
}

impl fmt::Display for Log {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.format {
            LogFormat::CLF => {
                write!(
                    f,
                    "SessionID={} Timestamp={} Description={} Level={} Component={}",
                    self.session_id, self.time, self.description, self.level, self.component
                )
                .expect("Unable to write log message");
                Ok(())
            }
            LogFormat::JSON => {
                write!(
                f,
                "{{\"SessionID\":\"{}\",\"Timestamp\":\"{}\",\"Level\":\"{}\",\"Component\":\"{}\",\"Description\":\"{}\",\"Format\":\"JSON\"}}",
                self.session_id, self.time, self.level, self.component, self.description)
                .expect("Unable to write log message");
                Ok(())
            }
            LogFormat::CEF => {
                write!(
                    f,
                    "CEF:0|{}|{}|{}|{}|{}|CEF",
                    self.session_id,
                    self.time,
                    self.level,
                    self.component,
                    self.description
                )
                .expect("Unable to write log message");
                Ok(())
            }
            LogFormat::ELF => {
                write!(
                    f,
                    "ELF:0|{}|{}|{}|{}|{}|ELF",
                    self.session_id,
                    self.time,
                    self.level,
                    self.component,
                    self.description
                )
                .expect("Unable to write log message");
                Ok(())
            }
            LogFormat::W3C => {
                write!(
                    f,
                    "W3C:0|{}|{}|{}|{}|{}|W3C",
                    self.session_id,
                    self.time,
                    self.level,
                    self.component,
                    self.description
                )
                // self.session_id, self.time, self.level, self.component, self.description)
                .expect("Unable to write log message");
                Ok(())
            }
            LogFormat::GELF => {
                write!(
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
                    self.component,
                    self.description,
                    self.level,
                    self.time,
                    self.component,
                    self.session_id
                )
                .expect("Unable to write log message");
                Ok(())
            }
            LogFormat::ApacheAccessLog => {
                write!(
                    f,
                    "{} - - [{}] \"{}\" {} {}",
                    hostname::get().unwrap().to_string_lossy(),
                    self.time,
                    self.description,
                    self.level,
                    self.component
                )
                .expect("Unable to write log message");
                Ok(())
            }
            LogFormat::Logstash => {
                write!(
                    f,
                    r#"{{
                            "@timestamp": "{}",
                            "level": "{}",
                            "component": "{}",
                            "message": "{}"
                        }}"#,
                    self.time,
                    self.level,
                    self.component,
                    self.description
                )
                .expect("Unable to write log message");
                Ok(())
            }
            LogFormat::Log4jXML => {
                write!(
                    f,
                    r#"<log4j:event logger="{}" timestamp="{}" level="{}" thread="{}"><log4j:message>{}</log4j:message></log4j:event>"#,
                    self.component,
                    self.time,
                    self.level,
                    self.session_id,
                    self.description
                )
                .expect("Unable to write log message");
                Ok(())
            }
            LogFormat::NDJSON => {
                write!(
                    f,
                    r#"{{
                            "timestamp": "{}",
                            "level": "{}",
                            "component": "{}",
                            "message": "{}"
                        }}"#,
                    self.time,
                    self.level,
                    self.component,
                    self.description
                )
                .expect("Unable to write log message");
                Ok(())
            }
        }
    }
}
