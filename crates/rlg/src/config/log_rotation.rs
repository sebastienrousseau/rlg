// log_rotation.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The [`LogRotation`] policy and its `size:N` / `time:N` / `date` /
//! `count:N` string form.

use super::ConfigError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::NonZeroU64;
use std::str::FromStr;

/// Log rotation policy variants.
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
        let trimmed = s.trim();
        let (kind, value) = trimmed
            .split_once(':')
            .map_or((trimmed, None), |(k, v)| (k, Some(v)));
        match kind.to_lowercase().as_str() {
            "size" => rotation_amount(value, "size").map(Self::Size),
            "time" => rotation_amount(value, "time").map(Self::Time),
            "date" => Ok(Self::Date),
            "count" => rotation_count(value).map(Self::Count),
            _ => Err(ConfigError::ValidationError(format!(
                "Invalid log rotation option: '{s}'"
            ))),
        }
    }
}

/// The non-zero number after `size:` or `time:`; `what` names it in
/// the error messages.
fn rotation_amount(
    value: Option<&str>,
    what: &str,
) -> Result<NonZeroU64, ConfigError> {
    let invalid = |msg: String| ConfigError::ValidationError(msg);
    let text = value.ok_or_else(|| {
        invalid(format!("Missing {what} value for log rotation"))
    })?;
    let n = text.parse::<u64>().map_err(|_| {
        invalid(format!(
            "Invalid {what} value for log rotation: '{text}'"
        ))
    })?;
    NonZeroU64::new(n).ok_or_else(|| {
        invalid(format!("Log rotation {what} must be greater than 0"))
    })
}

/// The non-zero count after `count:`, saturating at `u32::MAX`.
fn rotation_count(value: Option<&str>) -> Result<u32, ConfigError> {
    let invalid = |msg: String| ConfigError::ValidationError(msg);
    let text = value.ok_or_else(|| {
        invalid("Missing count value for log rotation".to_string())
    })?;
    let n = text.parse::<usize>().map_err(|_| {
        invalid(format!(
            "Invalid count value for log rotation: '{text}'"
        ))
    })?;
    if n == 0 {
        return Err(invalid(
            "Log rotation count must be greater than 0".to_string(),
        ));
    }
    Ok(n.try_into().unwrap_or(u32::MAX))
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
