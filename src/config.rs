// log.rs
// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::LogLevel;
use serde::{Deserialize, Serialize};
use std::{env, fs, path::Path, path::PathBuf, str::FromStr};
use thiserror::Error;

/// Errors that can occur while constructing a configuration.
#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    Error,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
pub enum ConfigError {
    #[error("environment variable error: {0}")]
    /// Error message for environment variable errors.
    EnvVarError(String),
    #[error("parsing error: {0}")]
    /// Error message for parsing errors.
    ParseError(String),
    #[error("invalid path: {0}")]
    /// Error message for invalid path errors.
    InvalidPath(String),
    #[error("file rotation error: {0}")]
    /// Error message for file rotation errors.
    RotationError(String),
    #[error("file read error: {0}")]
    /// Error message for file read errors.
    FileReadError(String),
    #[error("file parse error: {0}")]
    /// Error message for file parsing errors.
    FileParseError(String),
}

/// Enum representing different log rotation options.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
pub enum LogRotation {
    /// Rotate log files based on size.
    BySize(u64),
    /// Rotate log files based on time.
    ByTime(u64),
    /// Rotate log files based on date.
    ByDate,
    /// Rotate log files based on count.
    ByFileCount(u32),
}

/// Enum representing different logging destinations.
#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
pub enum LoggingDestination {
    /// Log to a file.
    File(PathBuf),
    /// Log to standard output.
    Stdout,
    /// Log to a network destination.
    Network(String),
}

/// Struct representing the configuration for the logging system.
#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
pub struct Config {
    /// The path to the log file.
    pub log_file_path: PathBuf,
    /// The log level.
    pub log_level: LogLevel,
    /// The log rotation configuration.
    pub log_rotation: Option<LogRotation>,
    /// The log format.
    pub log_format: String,
    /// The logging destinations.
    pub logging_destinations: Vec<LoggingDestination>,
}

impl Config {
    /// Provides default values for the configuration, useful when environment variables are not set.
    fn default() -> Self {
        Self {
            log_file_path: PathBuf::from("RLG.log"),
            log_level: LogLevel::INFO,
            log_rotation: Some(LogRotation::BySize(10 * 1024 * 1024)), // Default to 10 MB
            log_format: "%level - %message".into(),
            logging_destinations: vec![LoggingDestination::File(
                PathBuf::from("RLG.log"),
            )],
        }
    }

    /// Returns a display-friendly string representation of the log file path.
    pub fn log_file_path_display(&self) -> String {
        self.log_file_path.display().to_string()
    }

    /// Loads configuration from a TOML or YAML file, or from environment variables if the file is not found.
    ///
    /// # Parameters
    ///
    /// - `config_path`: An optional path to the configuration file. If `None`, the method will try to load from environment variables.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Config, ConfigError>` containing the loaded configuration or an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use rlg::config::Config;
    /// let config = Config::load(Some("config.toml")).expect("Failed to load config");
    /// ```
    pub fn load(
        config_path: Option<&str>,
    ) -> Result<Self, ConfigError> {
        if let Some(path) = config_path {
            if Path::new(path).exists() {
                let extension = Path::new(path)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_lowercase();

                let config_str =
                    fs::read_to_string(path).map_err(|e| {
                        ConfigError::FileReadError(format!(
                        "Failed to read configuration file '{}': {}",
                        path, e
                    ))
                    })?;

                let config = match extension.as_str() {
                    "toml" => {
                        toml::from_str(&config_str).map_err(|e| {
                            ConfigError::FileParseError(format!(
                            "Failed to parse TOML configuration: {}",
                            e
                        ))
                        })?
                    }
                    "yaml" | "yml" => serde_yml::from_str(&config_str)
                        .map_err(|e| {
                            ConfigError::FileParseError(format!(
                            "Failed to parse YAML configuration: {}",
                            e
                        ))
                        })?,
                    _ => {
                        return Err(ConfigError::FileParseError(
                            "Unsupported configuration file format"
                                .to_string(),
                        ))
                    }
                };

                return Ok(config);
            }
        }

        // Fallback to environment variables if no config file is provided or it fails to load
        Self::from_env()
    }

    /// Loads configuration from environment variables or applies default values if variables are not set.
    pub fn from_env() -> Result<Self, ConfigError> {
        let log_file_path = env::var("LOG_FILE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| Self::default().log_file_path);

        let log_level = env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "INFO".into())
            .parse::<LogLevel>()
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        let log_rotation =
            env::var("LOG_ROTATION").ok().and_then(|r| r.parse().ok());

        let log_format = env::var("LOG_FORMAT")
            .unwrap_or_else(|_| Self::default().log_format);

        let logging_destinations = env::var("LOG_DESTINATIONS")
            .unwrap_or_else(|_| "file".into())
            .split(',')
            .map(|dest| match dest.trim().to_lowercase().as_str() {
                "file" => {
                    Ok(LoggingDestination::File(log_file_path.clone()))
                }
                "stdout" => Ok(LoggingDestination::Stdout),
                "network" => Ok(LoggingDestination::Network(
                    "127.0.0.1:514".to_string(),
                )),
                _ => Err(ConfigError::EnvVarError(format!(
                    "Invalid logging destination: {}",
                    dest
                ))),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            log_file_path,
            log_level,
            log_rotation,
            log_format,
            logging_destinations,
        })
    }
}

impl FromStr for LogRotation {
    /// Parses a string into a LogRotation enum variant.
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "size" => Ok(LogRotation::BySize(1024 * 1024)), // Default to 1 MB
            "time" => Ok(LogRotation::ByTime(86400)), // Default to 1 day
            "date" => Ok(LogRotation::ByDate),
            "count" => s
                .split(':')
                .nth(1)
                .and_then(|c| c.parse::<u32>().ok())
                .map(LogRotation::ByFileCount)
                .ok_or_else(|| {
                    ConfigError::RotationError(format!(
                        "Invalid rotation count option: {}",
                        s
                    ))
                }),
            _ => Err(ConfigError::RotationError(format!(
                "Invalid log rotation option: {}",
                s
            ))),
        }
    }
}
