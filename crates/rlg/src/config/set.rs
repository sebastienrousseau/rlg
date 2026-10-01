// set.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The per-key setters behind [`Config::set`].

use super::{Config, ConfigError, SourceConfigError};

/// Assigns one field from a JSON value, for [`Config::set`].
pub(super) type Setter =
    fn(&mut Config, serde_json::Value) -> Result<(), ConfigError>;

/// The keys [`Config::set`] accepts, each with its setter.
pub(super) const SETTERS: [(&str, Setter); 8] = [
    ("version", |c, v| {
        c.version = string_field(&v, "Invalid version format")?;
        Ok(())
    }),
    ("profile", |c, v| {
        c.profile = string_field(&v, "Invalid profile format")?;
        Ok(())
    }),
    ("log_format", |c, v| {
        c.log_format = string_field(&v, "Invalid log format")?;
        Ok(())
    }),
    ("log_file_path", |c, v| {
        c.log_file_path = typed_field(v)?;
        Ok(())
    }),
    ("log_level", |c, v| {
        c.log_level = typed_field(v)?;
        Ok(())
    }),
    ("log_rotation", |c, v| {
        c.log_rotation = typed_field(v)?;
        Ok(())
    }),
    ("logging_destinations", |c, v| {
        c.logging_destinations = typed_field(v)?;
        Ok(())
    }),
    ("env_vars", |c, v| {
        c.env_vars = typed_field(v)?;
        Ok(())
    }),
];

/// A string-valued field for [`Config::set`]; `err` if `val` is not a
/// string.
fn string_field(
    val: &serde_json::Value,
    err: &str,
) -> Result<String, ConfigError> {
    val.as_str()
        .map(str::to_string)
        .ok_or_else(|| ConfigError::ValidationError(err.to_string()))
}

/// A structured field for [`Config::set`], deserialised from `val`.
fn typed_field<T: serde::de::DeserializeOwned>(
    val: serde_json::Value,
) -> Result<T, ConfigError> {
    serde_json::from_value(val).map_err(|e| {
        ConfigError::ConfigParseError(SourceConfigError::Message(
            e.to_string(),
        ))
    })
}
