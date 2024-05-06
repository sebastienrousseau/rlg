// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(test)]
mod tests {

    use rlg::{
        config::{Config, LogRotation, LoggingDestination},
        log_level::LogLevel::{self, DEBUG, INFO, NONE},
    };
    use std::{env, path::PathBuf, str::FromStr};

    // Tests for LogLevel enum parsing
    #[test]
    fn test_log_level_from_str() {
        assert_eq!(<LogLevel as FromStr>::from_str("INFO").unwrap(), INFO);
        assert_eq!(<LogLevel as FromStr>::from_str("debug").unwrap(), DEBUG);
        assert_eq!(<LogLevel as FromStr>::from_str("NONE").unwrap(), NONE);
    }

    // Tests for LogRotation enum parsing
    #[test]
    fn test_log_rotation_from_str() {
        assert_eq!(
            LogRotation::BySize(1024 * 1024),
            "size".parse::<LogRotation>().unwrap()
        );
        assert_eq!(
            LogRotation::ByTime(86400),
            "time".parse::<LogRotation>().unwrap()
        );
        assert!("invalid"
            .parse::<LogRotation>()
            .unwrap_err()
            .contains("Invalid log rotation option"));
    }

    // Tests for loading Config from environment variables
    #[test]
    fn test_config_load() {
        // Load the config
        let config = Config::load().unwrap();

        // Check if the loaded config matches the expected values
        assert_eq!(config.log_file_path, PathBuf::from("RLG.log"));
        // Check if log_rotation is None when not specified in environment variables
        assert_eq!(config.log_rotation, None);
        assert_eq!(config.log_format, "%level - %message");
        assert_eq!(
            config.logging_destinations,
            vec![LoggingDestination::File(PathBuf::from("RLG.log"))]
        );
    }

    // Test for displaying log file path
    #[test]
    fn test_config_log_file_path_display() {
        let config = Config {
            log_file_path: PathBuf::from("RLG.log"),
            log_level: INFO,
            log_rotation: None,
            log_format: "%level - %message".to_string(),
            logging_destinations: vec![],
        };
        assert_eq!(config.log_file_path_display(), "RLG.log");
    }

    #[test]
    fn test_config_load_with_invalid_values() {
        // Set invalid values for LOG_LEVEL and LOG_ROTATION
        env::set_var("LOG_LEVEL", "INVALID");
        env::set_var("LOG_ROTATION", "INVALID");

        // Load the configuration
        let result = Config::load();

        // Assert that the result is an error
        assert!(result.is_err());

        // Assert that the error message contains either "Invalid log level" or "Invalid log rotation option"
        let error_message = result.unwrap_err();
        assert!(
            error_message.contains("Invalid log level")
                || error_message.contains("Invalid log rotation option")
        );
    }

    // Test loading Config with default values
    #[test]
    fn test_config_load_with_defaults() {
        // Load the configuration
        let config = Config::load();
        assert_eq!(config.clone().unwrap().log_rotation, None);
    }

    #[test]
    fn test_log_rotation_clone_and_copy() {
        let rotation1 = LogRotation::BySize(1024 * 1024);
        let rotation2 = rotation1;
        assert_eq!(rotation1, rotation2);
    }
}
