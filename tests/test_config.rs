// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(test)]
mod tests {
    use rlg::config::ConfigError;
    use rlg::{
        config::{Config, LogRotation},
        log_level::LogLevel,
    };
    use std::{env, path::PathBuf, str::FromStr};

    /// Tests for parsing different variants of the LogLevel enum from strings.
    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("INFO").unwrap(), LogLevel::INFO);
        assert_eq!(
            LogLevel::from_str("debug").unwrap(),
            LogLevel::DEBUG
        );
        assert_eq!(LogLevel::from_str("NONE").unwrap(), LogLevel::NONE);
    }

    /// Tests for correctly parsing valid LogRotation enum variants from strings.
    #[test]
    fn test_log_rotation_from_str_valid() {
        assert_eq!(
            LogRotation::BySize(1024 * 1024),
            "size".parse::<LogRotation>().unwrap()
        );
        assert_eq!(
            LogRotation::ByTime(86400),
            "time".parse::<LogRotation>().unwrap()
        );
    }

    /// Tests that parsing an invalid string as LogRotation returns the appropriate error.
    #[test]
    fn test_log_rotation_from_str_invalid() {
        let error = "invalid".parse::<LogRotation>().unwrap_err();
        assert!(
            matches!(error, ConfigError::RotationError(msg) if msg.contains("Invalid log rotation option"))
        );
    }

    /// Tests displaying the log file path from the Config struct.
    #[test]
    fn test_config_log_file_path_display() {
        let config = Config {
            log_file_path: PathBuf::from("RLG.log"),
            log_level: LogLevel::INFO,
            log_rotation: None,
            log_format: "%level - %message".to_string(),
            logging_destinations: vec![],
        };
        assert_eq!(config.log_file_path_display(), "RLG.log");
    }

    /// Tests loading the configuration with invalid environment variable values for LOG_LEVEL and LOG_ROTATION.
    #[test]
    fn test_config_load_with_invalid_values() {
        env::set_var("LOG_LEVEL", "INVALID");
        env::set_var("LOG_ROTATION", "INVALID");

        let result = Config::load();
        assert!(result.is_err());
        let error_message = format!("{}", result.unwrap_err());
        assert!(
            error_message.contains("Invalid log level")
                || error_message
                    .contains("Invalid log rotation option")
        );
    }

    /// Tests the cloning and copying capabilities of the LogRotation enum.
    #[test]
    fn test_log_rotation_clone_and_copy() {
        let rotation1 = LogRotation::BySize(1024 * 1024);
        let rotation2 = rotation1;
        assert_eq!(rotation1, rotation2);
    }
}
