// redact.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! `Config`'s `Debug` and `diff` output, without `env_vars` values.
//!
//! `env_vars` holds environment values (`expand_env_vars` copies real
//! ones in), which can be secrets. Printing a `Config` or a diff must
//! not put them into a log line, so only the names are shown.

use super::Config;
use std::collections::{BTreeSet, HashMap};
use std::fmt;

/// The names of `env_vars`, sorted, each with a redacted value.
struct RedactedVars<'a>(&'a HashMap<String, String>);

impl fmt::Debug for RedactedVars<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: BTreeSet<&String> = self.0.keys().collect();
        f.debug_map()
            .entries(names.into_iter().map(|k| (k, "[REDACTED]")))
            .finish()
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("version", &self.version)
            .field("profile", &self.profile)
            .field("log_file_path", &self.log_file_path)
            .field("log_level", &self.log_level)
            .field("log_rotation", &self.log_rotation)
            .field("log_format", &self.log_format)
            .field("logging_destinations", &self.logging_destinations)
            .field("env_vars", &RedactedVars(&self.env_vars))
            .finish()
    }
}

/// Which `env_vars` names were added, removed or changed between `a`
/// and `b`, without their values; `None` when nothing differs.
pub(super) fn env_vars_diff(
    a: &HashMap<String, String>,
    b: &HashMap<String, String>,
) -> Option<String> {
    let names: BTreeSet<&String> = a.keys().chain(b.keys()).collect();
    let changes: Vec<String> = names
        .into_iter()
        .filter_map(|k| match (a.get(k), b.get(k)) {
            (Some(x), Some(y)) if x == y => None,
            (Some(_), Some(_)) => Some(format!("changed {k}")),
            (None, _) => Some(format!("added {k}")),
            (_, None) => Some(format!("removed {k}")),
        })
        .collect();
    (!changes.is_empty()).then(|| changes.join(", "))
}
