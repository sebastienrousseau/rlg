// config.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Configuration module for RustLogs (RLG).

use crate::LogLevel;
use config::{
    Config as ConfigSource, ConfigError as SourceConfigError,
    File as ConfigFile,
};
use envy;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env, fmt,
    fs::{self, OpenOptions},
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.trim().splitn(2, ':').collect();
        match parts[0].to_lowercase().as_str() {
            "size" => {
                let size_str = parts.get(1).ok_or_else(|| {
                    ConfigError::ValidationError(
                        "Missing size value for log rotation"
                            .to_string(),
                    )
                })?;
                let size = size_str.parse::<u64>().map_err(|_| ConfigError::ValidationError(format!("Invalid size value for log rotation: '{size_str}'")))?;
                Ok(Self::Size(NonZeroU64::new(size).ok_or_else(
                    || {
                        ConfigError::ValidationError(
                            "Log rotation size must be greater than 0"
                                .to_string(),
                        )
                    },
                )?))
            }
            "time" => {
                let time_str = parts.get(1).ok_or_else(|| {
                    ConfigError::ValidationError(
                        "Missing time value for log rotation"
                            .to_string(),
                    )
                })?;
                let time = time_str.parse::<u64>().map_err(|_| ConfigError::ValidationError(format!("Invalid time value for log rotation: '{time_str}'")))?;
                Ok(Self::Time(NonZeroU64::new(time).ok_or_else(
                    || {
                        ConfigError::ValidationError(
                            "Log rotation time must be greater than 0"
                                .to_string(),
                        )
                    },
                )?))
            }
            "date" => Ok(Self::Date),
            "count" => {
                let count = parts
                    .get(1)
                    .ok_or_else(|| ConfigError::ValidationError("Missing count value for log rotation".to_string()))?
                    .parse::<usize>()
                    .map_err(|_| ConfigError::ValidationError(format!("Invalid count value for log rotation: '{0}'", parts[1])))?;
                if count == 0 {
                    Err(ConfigError::ValidationError(
                        "Log rotation count must be greater than 0"
                            .to_string(),
                    ))
                } else {
                    Ok(Self::Count(
                        count.try_into().unwrap_or(u32::MAX),
                    ))
                }
            }
            _ => Err(ConfigError::ValidationError(format!(
                "Invalid log rotation option: '{s}'"
            ))),
        }
    }
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
    Network(String),
}

/// Configuration structure for the logging system.
#[derive(Debug, Clone, Serialize, Deserialize)]
// Allowed because Config contains no unsafe invariants that Deserialize could violate.
#[allow(clippy::unsafe_derive_deserialize)]
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
        Self {
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
    /// # Errors
    ///
    /// This function returns an error if the configuration file cannot be read,
    /// parsed, or if the version is unsupported.
    pub async fn load_async<P: AsRef<Path>>(
        config_path: Option<P>,
    ) -> Result<Arc<RwLock<Self>>, ConfigError> {
        let path_buf = config_path.map(|p| p.as_ref().to_path_buf());
        let config = if let Some(path) = path_buf {
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
                    "Unsupported configuration version: {version}"
                )));
            }
            config_source.try_deserialize()?
        } else {
            Self::default()
        };
        config.validate()?;
        config.ensure_paths()?;
        Ok(Arc::new(RwLock::new(config)))
    }

    /// Saves the current configuration to a file.
    ///
    /// # Errors
    ///
    /// This function returns an error if the file cannot be written.
    ///
    /// # Panics
    ///
    /// This function panics if serialization to JSON fails (unreachable for this struct).
    pub fn save_to_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), ConfigError> {
        let config_string = serde_json::to_string_pretty(self)
            .expect("Failed to serialize config");
        fs::write(path, config_string).map_err(|e| {
            ConfigError::FileWriteError(format!(
                "Failed to write config file: {e}"
            ))
        })?;
        Ok(())
    }

    /// Sets a value in the configuration based on the specified key.
    ///
    /// # Errors
    ///
    /// This function returns an error if the value cannot be serialized or if the key is unknown.
    pub fn set<T: Serialize>(
        &mut self,
        key: &str,
        value: T,
    ) -> Result<(), ConfigError> {
        let val = serde_json::to_value(value)
            .map_err(|e| ConfigError::ValidationError(e.to_string()))?;

        match key {
            "version" => {
                if let Some(s) = val.as_str() {
                    self.version = s.to_string();
                } else {
                    return Err(ConfigError::ValidationError(
                        "Invalid version format".to_string(),
                    ));
                }
            }
            "profile" => {
                if let Some(s) = val.as_str() {
                    self.profile = s.to_string();
                } else {
                    return Err(ConfigError::ValidationError(
                        "Invalid profile format".to_string(),
                    ));
                }
            }
            "log_file_path" => {
                self.log_file_path = serde_json::from_value(val)
                    .map_err(|e| {
                        ConfigError::ConfigParseError(
                            SourceConfigError::Message(e.to_string()),
                        )
                    })?;
            }
            "log_level" => {
                self.log_level =
                    serde_json::from_value(val).map_err(|e| {
                        ConfigError::ConfigParseError(
                            SourceConfigError::Message(e.to_string()),
                        )
                    })?;
            }
            "log_rotation" => {
                self.log_rotation = serde_json::from_value(val)
                    .map_err(|e| {
                        ConfigError::ConfigParseError(
                            SourceConfigError::Message(e.to_string()),
                        )
                    })?;
            }
            "log_format" => {
                if let Some(s) = val.as_str() {
                    self.log_format = s.to_string();
                } else {
                    return Err(ConfigError::ValidationError(
                        "Invalid log format".to_string(),
                    ));
                }
            }
            "logging_destinations" => {
                self.logging_destinations = serde_json::from_value(val)
                    .map_err(|e| {
                        ConfigError::ConfigParseError(
                            SourceConfigError::Message(e.to_string()),
                        )
                    })?;
            }
            "env_vars" => {
                self.env_vars =
                    serde_json::from_value(val).map_err(|e| {
                        ConfigError::ConfigParseError(
                            SourceConfigError::Message(e.to_string()),
                        )
                    })?;
            }
            _ => {
                #[cfg(not(test))]
                return Err(ConfigError::ValidationError(format!(
                    "Unknown configuration key: {key}"
                )));
                #[cfg(test)]
                {
                    let _ = key;
                    return Err(ConfigError::ValidationError(
                        "Unknown key".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Validates the configuration settings.
    ///
    /// # Errors
    ///
    /// This function returns an error if any configuration setting is invalid.
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

        for (key, value) in &self.env_vars {
            if key.trim().is_empty() {
                return Err(ConfigError::ValidationError(
                    "Environment variable key cannot be empty"
                        .to_string(),
                ));
            }
            if value.trim().is_empty() {
                return Err(ConfigError::ValidationError(format!(
                    "Value for environment variable '{key}' cannot be empty"
                )));
            }
        }

        Ok(())
    }

    /// Creates directories and log files required by the configuration.
    ///
    /// # Errors
    ///
    /// This function returns an error if the directories or files cannot be created.
    pub fn ensure_paths(&self) -> Result<(), ConfigError> {
        if let Some(LoggingDestination::File(path)) =
            self.logging_destinations.first()
        {
            if let Some(parent_dir) = path.parent() {
                fs::create_dir_all(parent_dir).map_err(|e| {
                    ConfigError::ValidationError(format!(
                        "Failed to create directory for log file: {e}"
                    ))
                })?;
            }
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .map_err(|e| {
                    ConfigError::ValidationError(format!(
                        "Log file is not writable: {e}"
                    ))
                })?;
        }
        Ok(())
    }

    /// Expands environment variables in the configuration values.
    #[must_use]
    pub fn expand_env_vars(&self) -> Self {
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
    /// # Errors
    ///
    /// This function returns an error if the watcher cannot be initialized.
    #[allow(clippy::incompatible_msrv)]
    pub fn hot_reload_async(
        config_path: &str,
        config: &Arc<RwLock<Self>>,
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

        let config_clone = config.clone();
        let path_owned = config_path.to_string();
        tokio::spawn(async move {
            let _watcher = watcher; // Keep watcher alive for the lifetime of the task
            loop {
                tokio::select! {
                    Some(res) = rx.recv() => {
                        if let Ok(Event { kind: EventKind::Modify(_), .. }) = res {
                            if let Ok(new_config) = Self::load_async(Some(&path_owned)).await {
                                let mut config_write = config_clone.write();
                                *config_write = new_config.read().clone();
                            }
                        }
                    }
                    _ = stop_rx.recv() => break,
                }
            }
        });
        Ok(stop_tx)
    }

    /// Compares two configurations and returns the differences.
    #[must_use]
    pub fn diff(
        config1: &Self,
        config2: &Self,
    ) -> HashMap<String, String> {
        let mut diffs = HashMap::new();
        macro_rules! config_diff_fields {
            ($c1:expr, $c2:expr, $diffs:expr;
             $( display $field:ident; )*
             $( debug $dfield:ident; )*
             $( path $pfield:ident; )*
            ) => {
                $(
                    if $c1.$field != $c2.$field {
                        $diffs.insert(
                            stringify!($field).to_string(),
                            format!("{} -> {}", $c1.$field, $c2.$field),
                        );
                    }
                )*
                $(
                    if $c1.$dfield != $c2.$dfield {
                        $diffs.insert(
                            stringify!($dfield).to_string(),
                            format!("{:?} -> {:?}", $c1.$dfield, $c2.$dfield),
                        );
                    }
                )*
                $(
                    if $c1.$pfield != $c2.$pfield {
                        $diffs.insert(
                            stringify!($pfield).to_string(),
                            format!("{} -> {}", $c1.$pfield.display(), $c2.$pfield.display()),
                        );
                    }
                )*
            };
        }
        config_diff_fields!(config1, config2, diffs;
            display version;
            display profile;
            display log_format;
            debug log_level;
            debug log_rotation;
            debug logging_destinations;
            debug env_vars;
            path log_file_path;
        );
        diffs
    }

    /// Overrides the current configuration with values from another configuration.
    #[must_use]
    pub fn override_with(&self, other: &Self) -> Self {
        let mut env_vars = self.env_vars.clone();
        env_vars.extend(other.env_vars.clone());
        Self {
            version: other.version.clone(),
            profile: other.profile.clone(),
            log_file_path: other.log_file_path.clone(),
            log_level: other.log_level,
            log_rotation: other.log_rotation,
            log_format: other.log_format.clone(),
            logging_destinations: other.logging_destinations.clone(),
            env_vars,
        }
    }
}

impl TryFrom<env::Vars> for Config {
    type Error = ConfigError;
    fn try_from(vars: env::Vars) -> Result<Self, Self::Error> {
        envy::from_iter(vars).map_err(ConfigError::EnvVarParseError)
    }
}

impl fmt::Display for LogRotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Size(size) => write!(f, "Size: {size} bytes"),
            Self::Time(seconds) => write!(f, "Time: {seconds} seconds"),
            Self::Date => write!(f, "Date-based rotation"),
            Self::Count(count) => write!(f, "Count: {count} logs"),
        }
    }
}

#[cfg(all(test, not(miri)))]
mod tests {
    use super::*;
    use parking_lot::RwLock;
    use std::sync::Arc;
    use tokio::time::{Duration, sleep};

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn test_config_hot_reload_async_full() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config = Config::default();
        config.save_to_file(&config_path).unwrap();

        let shared_config = Arc::new(RwLock::new(Config::default()));
        let stop_tx = Config::hot_reload_async(
            config_path.to_str().unwrap(),
            &shared_config,
        )
        .unwrap();

        // Trigger Modify
        let new_config = Config {
            profile: "modified".to_string(),
            ..Config::default()
        };
        new_config.save_to_file(&config_path).unwrap();

        sleep(Duration::from_millis(200)).await;

        let _ = stop_tx.send(()).await;
    }

    #[test]
    fn test_config_set_exhaustive() {
        let mut config = Config::default();
        assert!(config.set("version", 123).is_err());
        assert!(config.set("profile", 123).is_err());
        assert!(config.set("log_file_path", 123).is_err());
        assert!(config.set("log_level", 123).is_err());
        assert!(config.set("log_rotation", 123).is_err());
        assert!(config.set("log_format", 123).is_err());
        assert!(config.set("logging_destinations", 123).is_err());
        assert!(config.set("env_vars", 123).is_err());
        assert!(config.set("unknown_key", "value").is_err());
    }

    #[test]
    fn test_config_set_unknown_key() {
        let mut config = Config::default();
        let res = config.set("absolutely_unknown_key_123", "value");
        assert!(res.is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_config_save_to_file_fail_unit() {
        let config = Config::default();
        let dir_path = env::temp_dir();
        let res = config.save_to_file(&dir_path);
        assert!(res.is_err());
    }

    #[test]
    fn test_log_rotation_exhaustive() {
        assert!(LogRotation::from_str("count:0").is_err());
        assert!(LogRotation::from_str("size:0").is_err());
        assert!(LogRotation::from_str("time:0").is_err());
        assert!(LogRotation::from_str("invalid:xxx").is_err());
    }
}
