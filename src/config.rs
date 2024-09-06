// config.rs
// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Configuration module for RustLogs (RLG).
//!
//! This module provides structures and implementations for managing
//! the configuration of the RustLogs library. It includes functionality
//! for loading, saving, and manipulating configuration settings, as well
//! as handling environment variables, error management, and log rotation.

use crate::LogLevel;
use config::{
    Config as ConfigSource, ConfigError as SourceConfigError,
    File as ConfigFile,
};
use envy;
use log::{error, info, warn};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env, fmt,
    fs::{self, OpenOptions},
    net::{SocketAddr, ToSocketAddrs},
    num::NonZeroU64,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};
use thiserror::Error;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;

const CURRENT_CONFIG_VERSION: &str = "1.0";

/// Custom error types for configuration management.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Error occurred while parsing an environment variable.
    #[error("Environment variable parse error: {0}")]
    EnvVarParseError(#[from] envy::Error),

    /// Error occurred while parsing the configuration file.
    #[error("Configuration parsing error: {0}")]
    ConfigParseError(#[from] SourceConfigError),

    /// Invalid file path was provided for configuration.
    #[error("Invalid file path: {0}")]
    InvalidFilePath(String),

    /// Error reading from a file.
    #[error("File read error: {0}")]
    FileReadError(String),

    /// Error writing to a file.
    #[error("File write error: {0}")]
    FileWriteError(String),

    /// Error validating the configuration settings.
    #[error("Configuration validation error: {0}")]
    ValidationError(String),

    /// Configuration version mismatch.
    #[error("Configuration version error: {0}")]
    VersionError(String),

    /// Required field is missing in the configuration.
    #[error("Missing required field: {0}")]
    MissingFieldError(String),

    /// Error setting up the file watcher.
    #[error("Watcher error: {0}")]
    WatcherError(#[from] notify::Error),
}

/// Enum representing log rotation options.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Serialize,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
)]
pub enum LogRotation {
    /// Size-based log rotation.
    Size(NonZeroU64),
    /// Time-based log rotation.
    Time(NonZeroU64),
    /// Date-based log rotation.
    Date,
    /// Count-based log rotation.
    Count(u32),
}

impl FromStr for LogRotation {
    type Err = ConfigError;

    /// Parses a string into a `LogRotation` enum variant.
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice representing the log rotation type and associated value.
    ///
    /// # Returns
    ///
    /// A `Result<LogRotation, ConfigError>` indicating the log rotation option or an error.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.trim().splitn(2, ':').collect();
        match parts[0].to_lowercase().as_str() {
            "size" => parse_nonzero_u64(parts.get(1).copied(), "size")
                .map(LogRotation::Size),
            "time" => parse_nonzero_u64(parts.get(1).copied(), "time")
                .map(LogRotation::Time),
            "date" => Ok(LogRotation::Date),
            "count" => {
                let count = parts
                    .get(1)
                    .ok_or_else(|| ConfigError::ValidationError("Missing count value for log rotation".to_string()))?
                    .parse()
                    .map_err(|_| ConfigError::ValidationError(format!("Invalid count value for log rotation: '{}'", parts[1])))?;
                if count == 0 {
                    Err(ConfigError::ValidationError(
                        "Log rotation count must be greater than 0"
                            .to_string(),
                    ))
                } else {
                    Ok(LogRotation::Count(count))
                }
            }
            _ => Err(ConfigError::ValidationError(format!(
                "Invalid log rotation option: '{}'",
                s
            ))),
        }
    }
}

/// Helper function to parse a `NonZeroU64` from a string value.
fn parse_nonzero_u64(
    value: Option<&str>,
    context: &str,
) -> Result<NonZeroU64, ConfigError> {
    let size = value
        .ok_or_else(|| {
            ConfigError::ValidationError(format!(
                "Missing {} value for log rotation",
                context
            ))
        })?
        .parse::<u64>()
        .map_err(|_| {
            ConfigError::ValidationError(format!(
                "Invalid {} value for log rotation",
                context
            ))
        })?;

    NonZeroU64::new(size).ok_or_else(|| {
        ConfigError::ValidationError(format!(
            "{} value must be greater than 0",
            context
        ))
    })
}

/// Enum representing different logging destinations.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum LoggingDestination {
    /// Log to a file.
    File(PathBuf),
    /// Log to standard output.
    Stdout,
    /// Log to a network destination.
    Network(String), // Expects format like "127.0.0.1:8080" or "example.com:8080"
}

/// Configuration structure for the logging system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Version of the configuration.
    #[serde(default = "default_version")]
    pub version: String,
    /// Profile name for the configuration.
    #[serde(default = "default_profile")]
    pub profile: String,
    /// Path to the log file.
    #[serde(default = "default_log_file_path")]
    pub log_file_path: PathBuf,
    /// Log level for the system.
    #[serde(default)]
    pub log_level: LogLevel,
    /// Log rotation settings.
    pub log_rotation: Option<LogRotation>,
    /// Log format string.
    #[serde(default = "default_log_format")]
    pub log_format: String,
    /// Logging destinations for the system.
    #[serde(default = "default_logging_destinations")]
    pub logging_destinations: Vec<LoggingDestination>,
    /// Environment variables for the system.
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
}

/// Default values for configuration fields.
fn default_version() -> String {
    CURRENT_CONFIG_VERSION.to_string()
}
fn default_profile() -> String {
    "default".to_string()
}
fn default_log_file_path() -> PathBuf {
    PathBuf::from("RLG.log")
}
fn default_log_format() -> String {
    "%level - %message".to_string()
}
fn default_logging_destinations() -> Vec<LoggingDestination> {
    vec![LoggingDestination::File(PathBuf::from("RLG.log"))]
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: default_version(),
            profile: default_profile(),
            log_file_path: default_log_file_path(),
            log_level: LogLevel::INFO,
            log_rotation: NonZeroU64::new(10 * 1024 * 1024)
                .map(LogRotation::Size),
            log_format: default_log_format(),
            logging_destinations: default_logging_destinations(),
            env_vars: HashMap::new(),
        }
    }
}

impl Config {
    /// Loads configuration from a file or environment variables.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rlg::config::Config;
    /// use std::sync::Arc;
    /// use parking_lot::RwLock;
    /// use std::env;
    /// use std::fs;
    ///
    /// // Create a temporary directory for testing
    /// let temp_dir = env::temp_dir();
    /// let log_file_path = temp_dir.join("test_RLG.log");
    ///
    /// // Ensure the file exists to avoid errors
    /// fs::File::create(&log_file_path).unwrap();
    ///
    /// // Load the configuration
    /// let mut config = Config::default();
    /// config.log_file_path = log_file_path;
    ///
    /// // Wrap in Arc<RwLock> for further use
    /// let config = Arc::new(RwLock::new(config));
    ///
    /// // Output the configuration
    /// let config = config.read();
    /// println!("Config version: {}", config.version);
    /// ```
    pub async fn load_async<P: AsRef<Path>>(
        config_path: Option<P>,
    ) -> Result<Arc<RwLock<Config>>, ConfigError> {
        let config = if let Some(path) = config_path {
            let mut file = File::open(&path).await.map_err(|e| {
                ConfigError::FileReadError(e.to_string())
            })?;
            let mut contents = String::new();
            file.read_to_string(&mut contents).await.map_err(|e| {
                ConfigError::FileReadError(e.to_string())
            })?;

            let config_source = ConfigSource::builder()
                .add_source(ConfigFile::from_str(
                    &contents,
                    config::FileFormat::Toml,
                ))
                .build()?;

            let version: String = config_source.get("version")?;
            if version != CURRENT_CONFIG_VERSION {
                return Err(ConfigError::VersionError(format!(
                    "Unsupported configuration version: {}",
                    version
                )));
            }

            config_source.try_deserialize()?
        } else {
            Config::default()
        };

        config.validate()?;
        Ok(Arc::new(RwLock::new(config)))
    }

    /// Retrieves a value from the configuration based on the specified key.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rlg::config::Config;
    ///
    /// let config = Config::default();
    /// let log_level: Option<String> = config.get("log_level");
    /// if let Some(level) = log_level {
    ///     println!("Log level: {}", level);
    /// }
    /// ```
    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let value = match key {
            "version" => serde_json::to_value(&self.version).ok()?,
            "profile" => serde_json::to_value(&self.profile).ok()?,
            "log_file_path" => {
                serde_json::to_value(&self.log_file_path).ok()?
            }
            "log_level" => serde_json::to_value(self.log_level).ok()?,
            "log_rotation" => {
                serde_json::to_value(self.log_rotation).ok()?
            }
            "log_format" => {
                serde_json::to_value(&self.log_format).ok()?
            }
            "logging_destinations" => {
                serde_json::to_value(&self.logging_destinations).ok()?
            }
            "env_vars" => serde_json::to_value(&self.env_vars).ok()?,
            _ => return None,
        };
        serde_json::from_value(value).ok()
    }

    /// Saves the current configuration to a file.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rlg::config::Config;
    ///
    /// let config = Config::default();
    /// config.save_to_file("config.json").unwrap();
    /// ```
    pub fn save_to_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), ConfigError> {
        let config_string = serde_json::to_string_pretty(self)
            .map_err(|e| {
                ConfigError::FileWriteError(format!(
                    "Failed to serialize config: {}",
                    e
                ))
            })?;

        fs::write(path, config_string).map_err(|e| {
            ConfigError::FileWriteError(format!(
                "Failed to write config file: {}",
                e
            ))
        })?;

        Ok(())
    }

    /// Sets a value in the configuration based on the specified key.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rlg::config::Config;
    ///
    /// let mut config = Config::default();
    /// config.set("log_format", "%level - %message").unwrap();
    /// println!("New log format: {}", config.log_format);
    /// ```
    pub fn set<T: Serialize>(
        &mut self,
        key: &str,
        value: T,
    ) -> Result<(), ConfigError> {
        let serialize_value =
            |v: T| -> Result<serde_json::Value, ConfigError> {
                serde_json::to_value(v).map_err(|e| {
                    ConfigError::ValidationError(e.to_string())
                })
            };

        match key {
            "version" => {
                self.version = serialize_value(value)?
                    .as_str()
                    .ok_or_else(|| {
                        ConfigError::ValidationError(
                            "Invalid version format".to_string(),
                        )
                    })?
                    .to_string()
            }
            "profile" => {
                self.profile = serialize_value(value)?
                    .as_str()
                    .ok_or_else(|| {
                        ConfigError::ValidationError(
                            "Invalid profile format".to_string(),
                        )
                    })?
                    .to_string()
            }
            "log_file_path" => {
                self.log_file_path =
                    serde_json::from_value(serialize_value(value)?)
                        .map_err(|e| {
                            ConfigError::ConfigParseError(
                                SourceConfigError::Message(
                                    e.to_string(),
                                ),
                            )
                        })?
            }
            "log_level" => {
                self.log_level =
                    serde_json::from_value(serialize_value(value)?)
                        .map_err(|e| {
                            ConfigError::ConfigParseError(
                                SourceConfigError::Message(
                                    e.to_string(),
                                ),
                            )
                        })?
            }
            "log_rotation" => {
                self.log_rotation =
                    serde_json::from_value(serialize_value(value)?)
                        .map_err(|e| {
                            ConfigError::ConfigParseError(
                                SourceConfigError::Message(
                                    e.to_string(),
                                ),
                            )
                        })?
            }
            "log_format" => {
                self.log_format = serialize_value(value)?
                    .as_str()
                    .ok_or_else(|| {
                        ConfigError::ValidationError(
                            "Invalid log format".to_string(),
                        )
                    })?
                    .to_string()
            }
            "logging_destinations" => {
                self.logging_destinations =
                    serde_json::from_value(serialize_value(value)?)
                        .map_err(|e| {
                            ConfigError::ConfigParseError(
                                SourceConfigError::Message(
                                    e.to_string(),
                                ),
                            )
                        })?
            }
            "env_vars" => {
                self.env_vars =
                    serde_json::from_value(serialize_value(value)?)
                        .map_err(|e| {
                            ConfigError::ConfigParseError(
                                SourceConfigError::Message(
                                    e.to_string(),
                                ),
                            )
                        })?
            }
            _ => {
                return Err(ConfigError::ValidationError(format!(
                    "Unknown configuration key: {}",
                    key
                )))
            }
        }
        Ok(())
    }

    /// Validates the configuration settings.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rlg::config::Config;
    /// use std::fs::{self, OpenOptions};
    /// use std::env;
    ///
    /// // Create a temporary directory for the log file
    /// let temp_dir = env::temp_dir();
    /// let log_file_path = temp_dir.join("test_RLG.log");
    ///
    /// // Ensure the log file exists and is writable
    /// OpenOptions::new().write(true).create(true).open(&log_file_path).unwrap();
    ///
    /// // Load default configuration and set the log file path
    /// let mut config = Config::default();
    /// config.log_file_path = log_file_path;
    ///
    /// // Validate the configuration
    /// config.validate().unwrap();
    /// println!("Configuration is valid!");
    /// ```
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.version.trim().is_empty() {
            return Err(ConfigError::ValidationError(
                "Version cannot be empty".to_string(),
            ));
        }

        if self.profile.trim().is_empty() {
            return Err(ConfigError::ValidationError(
                "Profile cannot be empty".to_string(),
            ));
        }

        if self.log_file_path.as_os_str().is_empty() {
            return Err(ConfigError::ValidationError(
                "Log file path cannot be empty".to_string(),
            ));
        }

        if let Some(rotation) = &self.log_rotation {
            match rotation {
                LogRotation::Size(size) if size.get() == 0 => {
                    return Err(ConfigError::ValidationError(
                        "Log rotation size must be greater than 0"
                            .to_string(),
                    ));
                }
                LogRotation::Time(time) if time.get() == 0 => {
                    return Err(ConfigError::ValidationError(
                        "Log rotation time must be greater than 0"
                            .to_string(),
                    ));
                }
                LogRotation::Count(count) if *count == 0 => {
                    return Err(ConfigError::ValidationError(
                        "Log rotation count must be greater than 0"
                            .to_string(),
                    ));
                }
                _ => {}
            }
        }

        if self.log_format.trim().is_empty() {
            return Err(ConfigError::ValidationError(
                "Log format cannot be empty".to_string(),
            ));
        }

        if self.logging_destinations.is_empty() {
            return Err(ConfigError::ValidationError(
                "At least one logging destination must be specified"
                    .to_string(),
            ));
        }

        for destination in &self.logging_destinations {
            if let LoggingDestination::Network(address) = destination {
                self.validate_network_address(address)?;
            }
        }

        for (key, value) in &self.env_vars {
            if key.trim().is_empty() {
                return Err(ConfigError::ValidationError(
                    "Environment variable key cannot be empty"
                        .to_string(),
                ));
            }
            if value.trim().is_empty() {
                return Err(ConfigError::ValidationError(format!("Value for environment variable '{}' cannot be empty", key)));
            }
        }

        // Check if log file is writable
        if let LoggingDestination::File(path) =
            &self.logging_destinations[0]
        {
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(path)
                .map_err(|e| {
                    ConfigError::ValidationError(format!(
                        "Log file is not writable: {}",
                        e
                    ))
                })?;
        }

        Ok(())
    }

    /// Validates a network address.
    fn validate_network_address(
        &self,
        address: &str,
    ) -> Result<(), ConfigError> {
        if address.trim().is_empty() {
            return Err(ConfigError::ValidationError(
                "Network logging destination address cannot be empty"
                    .to_string(),
            ));
        }

        if address.parse::<SocketAddr>().is_ok() {
            return Ok(());
        }

        address
            .to_socket_addrs()
            .map_err(|e| {
                ConfigError::ValidationError(format!(
                    "Invalid network address '{}': {}",
                    address, e
                ))
            })?
            .next()
            .ok_or_else(|| {
                ConfigError::ValidationError(format!(
                    "Could not resolve network address: '{}'",
                    address
                ))
            })?;

        Ok(())
    }

    /// Expands environment variables in the configuration values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rlg::config::Config;
    ///
    /// let config = Config::default();
    /// let expanded_config = config.expand_env_vars();
    /// println!("Expanded env vars: {:?}", expanded_config.env_vars);
    /// ```
    pub fn expand_env_vars(&self) -> Config {
        let mut new_config = self.clone();
        for (key, value) in &mut new_config.env_vars {
            if let Ok(env_value) = env::var(key) {
                *value = env_value;
            }
        }
        new_config
    }

    /// Hot-reloads configuration on file change.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rlg::config::Config;
    /// use std::sync::Arc;
    /// use parking_lot::RwLock;
    /// use std::fs;
    /// use std::env;
    ///
    /// # tokio_test::block_on(async {
    /// let temp_dir = env::temp_dir();
    /// let config_file_path = temp_dir.join("test_config.toml");
    ///
    /// // Create a simple config file to be watched
    /// let config_content = r#"
    /// version = "1.0"
    /// profile = "default"
    /// "#;
    /// fs::write(&config_file_path, config_content).unwrap();
    ///
    /// // Load default configuration and start watching the config file
    /// let config = Arc::new(RwLock::new(Config::default()));
    ///
    /// // Start hot reload with the temporary config file
    /// let _ = Config::hot_reload_async(config_file_path.to_str().unwrap(), config.clone()).await.unwrap();
    /// # });
    /// ```
    #[allow(clippy::incompatible_msrv)]
    pub async fn hot_reload_async(
        config_path: &str,
        config: Arc<RwLock<Config>>,
    ) -> Result<mpsc::Sender<()>, ConfigError> {
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);
        let (tx, mut rx) = mpsc::channel::<notify::Result<Event>>(100);

        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.blocking_send(res);
        })?;

        watcher.watch(
            Path::new(config_path),
            RecursiveMode::NonRecursive,
        )?;

        let config_path = config_path.to_string();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(res) = rx.recv() => {
                        match res {
                            Ok(Event { kind, .. }) => match kind {
                                EventKind::Modify(_) => {
                                    info!("Configuration file changed, reloading...");
                                    match Config::load_async(Some(&config_path)).await {
                                        Ok(new_config) => {
                                            let mut config_write = config.write();
                                            *config_write = new_config.read().clone();
                                            info!("Configuration reloaded successfully");
                                        }
                                        Err(e) => error!("Failed to reload configuration: {}", e),
                                    }
                                }
                                EventKind::Create(_) => info!("Configuration file created"),
                                EventKind::Remove(_) => warn!("Configuration file removed"),
                                _ => {}
                            },
                            Err(e) => error!("Watch error: {:?}", e),
                        }
                    }
                    _ = stop_rx.recv() => {
                        info!("Stopping configuration hot reload");
                        break;
                    }
                }
            }
        });

        Ok(stop_tx)
    }

    /// Compares two configurations and returns the differences.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rlg::config::Config;
    /// use std::collections::HashMap;
    ///
    /// let config1 = Config::default();
    /// let config2 = Config {
    ///     profile: "test".to_string(),
    ///     ..Config::default()
    /// };
    /// let differences = Config::diff(&config1, &config2);
    /// println!("Differences: {:?}", differences);
    /// ```
    pub fn diff(
        config1: &Config,
        config2: &Config,
    ) -> HashMap<String, String> {
        let mut differences = HashMap::new();

        if config1.version != config2.version {
            differences.insert(
                "version".to_string(),
                format!("{} -> {}", config1.version, config2.version),
            );
        }
        if config1.profile != config2.profile {
            differences.insert(
                "profile".to_string(),
                format!("{} -> {}", config1.profile, config2.profile),
            );
        }
        if config1.log_file_path != config2.log_file_path {
            differences.insert(
                "log_file_path".to_string(),
                format!(
                    "{} -> {}",
                    config1.log_file_path.display(),
                    config2.log_file_path.display()
                ),
            );
        }
        if config1.log_level != config2.log_level {
            differences.insert(
                "log_level".to_string(),
                format!(
                    "{:?} -> {:?}",
                    config1.log_level, config2.log_level
                ),
            );
        }
        if config1.log_rotation != config2.log_rotation {
            differences.insert(
                "log_rotation".to_string(),
                format!(
                    "{:?} -> {:?}",
                    config1.log_rotation, config2.log_rotation
                ),
            );
        }
        if config1.log_format != config2.log_format {
            differences.insert(
                "log_format".to_string(),
                format!(
                    "{} -> {}",
                    config1.log_format, config2.log_format
                ),
            );
        }
        if config1.logging_destinations != config2.logging_destinations
        {
            differences.insert(
                "logging_destinations".to_string(),
                format!(
                    "{:?} -> {:?}",
                    config1.logging_destinations,
                    config2.logging_destinations
                ),
            );
        }
        if config1.env_vars != config2.env_vars {
            differences.insert(
                "env_vars".to_string(),
                format!(
                    "{:?} -> {:?}",
                    config1.env_vars, config2.env_vars
                ),
            );
        }

        differences
    }

    /// Merges another configuration into the current configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rlg::config::Config;
    ///
    /// let config1 = Config::default();
    /// let config2 = Config {
    ///     profile: "test".to_string(),
    ///     ..Config::default()
    /// };
    /// let merged_config = config1.merge(&config2);
    /// println!("Merged config profile: {}", merged_config.profile);
    /// ```
    pub fn merge(&self, other: &Config) -> Config {
        Config {
            version: other.version.clone(),
            profile: other.profile.clone(),
            log_file_path: other.log_file_path.clone(),
            log_level: other.log_level,
            log_rotation: other.log_rotation,
            log_format: other.log_format.clone(),
            logging_destinations: other.logging_destinations.clone(),
            env_vars: self
                .env_vars
                .iter()
                .chain(other.env_vars.iter())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }
}

/// Implements `TryFrom` for environment variable parsing.
impl TryFrom<env::Vars> for Config {
    type Error = ConfigError;

    fn try_from(vars: env::Vars) -> Result<Self, Self::Error> {
        envy::from_iter(vars)
            .map_err(|e: envy::Error| ConfigError::EnvVarParseError(e))
    }
}

/// Implements `Display` trait for `LogRotation`.
impl fmt::Display for LogRotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogRotation::Size(size) => {
                write!(f, "Size: {} bytes", size.get())
            }
            LogRotation::Time(seconds) => {
                write!(f, "Time: {} seconds", seconds.get())
            }
            LogRotation::Date => write!(f, "Date-based rotation"),
            LogRotation::Count(count) => {
                write!(f, "Count: {} logs", count)
            }
        }
    }
}
