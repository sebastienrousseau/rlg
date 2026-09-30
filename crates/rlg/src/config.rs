// config.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! TOML-based configuration: loading, validation, diffing, and hot-reload.
//!
//! Load from a file with [`Config::load`][crate::config::Config::load],
//! or build programmatically via
//! [`Config::default`][crate::config::Config::default] and
//! [`Config::set`][crate::config::Config::set]. Serialize back to TOML
//! with [`Config::save_to_file`][crate::config::Config::save_to_file].
//!
//! Enable the `tokio` feature for async loading and file-watcher hot-reload.

use crate::LogLevel;
use config::{
    Config as ConfigSource, ConfigError as SourceConfigError,
    File as ConfigFile,
};
use envy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    fs::{self, OpenOptions},
    num::NonZeroU64,
    path::{Path, PathBuf},
    sync::Arc,
};
use thiserror::Error;

const CURRENT_CONFIG_VERSION: &str = "1.0";

#[cfg(feature = "tokio")]
mod hot_reload;
mod log_rotation;
mod set;

#[cfg(feature = "tokio")]
pub use hot_reload::HOT_RELOAD_POLL_INTERVAL;
pub use log_rotation::LogRotation;
use set::SETTERS;

/// Configuration error variants.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to parse an environment variable.
    #[error("Environment variable parse error: {0}")]
    EnvVarParseError(#[from] envy::Error),

    /// Failed to parse the configuration file.
    #[error("Configuration parsing error: {0}")]
    ConfigParseError(#[from] SourceConfigError),

    /// The provided config file path is invalid or inaccessible.
    #[error("Invalid file path: {0}")]
    InvalidFilePath(String),

    /// File read failed.
    #[error("File read error: {0}")]
    FileReadError(String),

    /// File write failed.
    #[error("File write error: {0}")]
    FileWriteError(String),

    /// Validation failed for a configuration field.
    #[error("Configuration validation error: {0}")]
    ValidationError(String),

    /// Config file version does not match the expected version.
    #[error("Configuration version error: {0}")]
    VersionError(String),

    /// A required field is missing from the configuration.
    #[error("Missing required field: {0}")]
    MissingFieldError(String),

    /// The watched file could not be read (requires `tokio`).
    #[cfg(feature = "tokio")]
    #[error("Watcher error: {0}")]
    WatcherError(std::io::Error),
}

impl From<crate::commons::config::ConfigError> for ConfigError {
    fn from(err: crate::commons::config::ConfigError) -> Self {
        Self::ValidationError(err.to_string())
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
    /// Loads configuration from a file or falls back to defaults.
    ///
    /// This is the synchronous variant. See [`Config::load_async`] for the
    /// async equivalent (requires the `tokio` feature).
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file cannot be read,
    /// parsed, or if the version is unsupported.
    pub fn load<P: AsRef<Path>>(
        config_path: Option<P>,
    ) -> Result<Arc<RwLock<Self>>, ConfigError> {
        let config = match config_path {
            Some(path) => Self::from_toml(
                &fs::read_to_string(path.as_ref())
                    .map_err(|e| read_error(&e))?,
            )?,
            None => Self::default(),
        };
        config.activate()
    }

    /// Loads configuration from a file or environment variables (async).
    ///
    /// Requires the `tokio` feature.
    ///
    /// # Errors
    ///
    /// This function returns an error if the configuration file cannot be read,
    /// parsed, or if the version is unsupported.
    #[cfg(feature = "tokio")]
    pub async fn load_async<P: AsRef<Path>>(
        config_path: Option<P>,
    ) -> Result<Arc<RwLock<Self>>, ConfigError> {
        // Owned before the first await, so the future does not borrow
        // the caller's path.
        let path = config_path.map(|p| p.as_ref().to_path_buf());
        let config = match path {
            Some(path) => Self::from_toml(
                &tokio::fs::read_to_string(&path)
                    .await
                    .map_err(|e| read_error(&e))?,
            )?,
            None => Self::default(),
        };
        config.activate()
    }

    /// Parse a TOML configuration and check its version.
    fn from_toml(contents: &str) -> Result<Self, ConfigError> {
        let source = ConfigSource::builder()
            .add_source(ConfigFile::from_str(
                contents,
                config::FileFormat::Toml,
            ))
            .build()?;
        let version: String = source.get("version")?;
        if version != CURRENT_CONFIG_VERSION {
            return Err(ConfigError::VersionError(format!(
                "Unsupported configuration version: {version}"
            )));
        }
        Ok(source.try_deserialize()?)
    }

    /// Validate, create the log paths, and share the result.
    fn activate(self) -> Result<Arc<RwLock<Self>>, ConfigError> {
        self.validate()?;
        self.ensure_paths()?;
        Ok(Arc::new(RwLock::new(self)))
    }

    /// Saves the current configuration to a file in TOML format.
    ///
    /// This matches the TOML format expected by [`Config::load`].
    ///
    /// # Errors
    ///
    /// This function returns an error if the file cannot be written or
    /// if serialization fails.
    pub fn save_to_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), ConfigError> {
        let config_string =
            toml::to_string_pretty(self).map_err(|e| {
                ConfigError::FileWriteError(format!(
                    "Failed to serialize config to TOML: {e}"
                ))
            })?;
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
        let (_, setter) = SETTERS
            .iter()
            .find(|(name, _)| *name == key)
            .ok_or_else(|| {
                ConfigError::ValidationError(format!(
                    "Unknown configuration key: {key}"
                ))
            })?;
        setter(self, val)
    }

    /// Validates the configuration settings.
    ///
    /// # Errors
    ///
    /// This function returns an error if any configuration setting is invalid.
    pub fn validate(&self) -> Result<(), ConfigError> {
        let mut v = crate::commons::validation::Validator::new();
        v.check("version", || not_empty(&self.version))
            .check("profile", || not_empty(&self.profile))
            .check("log_format", || not_empty(&self.log_format));
        // Path and destination checks are not string validations, and
        // fail on their own.
        self.validate_destinations()?;
        for (key, value) in &self.env_vars {
            v.check(&format!("env_var_key_{key}"), || not_empty(key));
            v.check(&format!("env_var_val_{key}"), || not_empty(value));
        }
        v.finish().map_err(|errors| {
            let msgs: Vec<String> = errors
                .iter()
                .map(|(f, e)| format!("{f}: {e}"))
                .collect();
            ConfigError::ValidationError(msgs.join("; "))
        })
    }

    fn validate_destinations(&self) -> Result<(), ConfigError> {
        if self.log_file_path.as_os_str().is_empty() {
            return Err(ConfigError::ValidationError(
                "Log file path cannot be empty".into(),
            ));
        }
        if self.logging_destinations.is_empty() {
            return Err(ConfigError::ValidationError(
                "At least one logging destination must be specified"
                    .into(),
            ));
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

/// A trimmed value must not be empty, for [`Config::validate`].
fn not_empty(
    value: &str,
) -> crate::commons::validation::ValidationResult<()> {
    crate::commons::validation::validate_not_empty(value.trim())
        .map(|_| ())
}

/// A config file that could not be read.
fn read_error(e: &std::io::Error) -> ConfigError {
    ConfigError::FileReadError(e.to_string())
}

#[cfg(all(test, not(miri)))]
mod tests;
