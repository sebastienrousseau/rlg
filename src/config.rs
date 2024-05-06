// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::LogLevel;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, env, path::PathBuf};

/// Enum representing different log rotation options.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum LogRotation {
    BySize(u64), // Log rotation by size (in bytes)
    ByTime(u64), // Log rotation by time (in seconds)
}

/// Enum representing different logging destinations.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum LoggingDestination {
    File(PathBuf),   // Log to a file with the specified path
    Stdout,          // Log to standard output (stdout)
    Network(String), // Log to a network endpoint (e.g., syslog server)
}

/// Configuration struct for logging system.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Config {
    /// Path and name of the log file.
    pub log_file_path: PathBuf,
    /// Global log level for the application.
    pub log_level: LogLevel,
    /// Log rotation configuration.
    pub log_rotation: Option<LogRotation>,
    /// Default log format for the application.
    pub log_format: String,
    /// Logging destinations.
    pub logging_destinations: Vec<LoggingDestination>,
}

impl Config {
    /// Loads configuration from environment variables or defaults.
    pub fn load() -> Result<Config, String> {
        let log_file_path = env::var("LOG_FILE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("RLG.log"));

        let log_level = env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "INFO".into())
            .parse()
            .map_err(|e| format!("Invalid log level: {}", e))?;

        let log_rotation = match env::var("LOG_ROTATION") {
            Ok(rotation_str) => Some(match rotation_str.as_str() {
                "size" => LogRotation::BySize(1024 * 1024), // Default rotation size: 1MB
                "time" => LogRotation::ByTime(86400),       // Default rotation time: 24 hours
                _ => return Err("Invalid log rotation option".to_string()),
            }),
            Err(_) => None,
        };

        let log_format = env::var("LOG_FORMAT").unwrap_or_else(|_| "%level - %message".into());

        let logging_destinations = env::var("LOG_DESTINATIONS")
            .unwrap_or_else(|_| "file".into())
            .split(',')
            .map(|dest| match dest.trim().to_lowercase().as_str() {
                "file" => Ok(LoggingDestination::File(log_file_path.clone())),
                "stdout" => Ok(LoggingDestination::Stdout),
                "network" => Ok(LoggingDestination::Network("127.0.0.1:514".to_string())), // Default syslog server
                _ => Err(format!("Invalid logging destination: {}", dest)),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Config {
            log_file_path,
            log_level,
            log_rotation,
            log_format,
            logging_destinations,
        })
    }
    /// Returns a reference to the log file path.
    pub fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Config: log_file_path='{}', log_level='{:?}', log_rotation='{:?}', log_format='{}', logging_destinations='{:?}'",
            self.log_file_path.display(),
            self.log_level,
            self.log_rotation,
            self.log_format,
            self.logging_destinations
        )
    }
    /// Convert log_file_path to a displayable representation
    pub fn log_file_path_display(&self) -> Cow<str> {
        self.log_file_path.to_string_lossy()
    }
}
