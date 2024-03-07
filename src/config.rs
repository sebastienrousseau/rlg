// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::env;

/// Configuration struct for logging system.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Config {
    /// Path and name of the log file.
    pub log_file_path: String,
}

impl Config {
    /// Loads configuration from environment variables or defaults.
    pub fn load() -> Config {
        let log_file_path = env::var("LOG_FILE_PATH").unwrap_or_else(|_| "RLG.log".into());
        Config { log_file_path }
    }
}
