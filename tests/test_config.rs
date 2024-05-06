// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(test)]
mod tests {
    use rlg::{
        config::{Config, LogRotation, LoggingDestination},
        log_level::LogLevel,
    };
    use std::{env, path::PathBuf, str::FromStr};

    // Tests for LogLevel enum parsing
    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("INFO").unwrap(), LogLevel::INFO);
        assert_eq!(LogLevel::from_str("debug").unwrap(), LogLevel::DEBUG);
        assert_eq!(LogLevel::from_str("NONE").unwrap(), LogLevel::NONE);
        assert_eq!(
            LogLevel::from_str("INVALID").unwrap_err(),
            "Invalid log level: INVALID"
        );
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
        // Prepare environment variables for testing
        env::set_var("LOG_FILE_PATH", "/path/to/log");
        env::set_var("LOG_LEVEL", "ERROR");
        env::set_var("LOG_ROTATION", "size");
        env::set_var("LOG_FORMAT", "%level - %message");
        env::set_var("LOG_DESTINATIONS", "file, stdout");

        // Load the config
        let config = Config::load().unwrap();

        // Check if the loaded config matches the expected values
        assert_eq!(config.log_file_path, PathBuf::from("/path/to/log"));
        assert_eq!(config.log_level, LogLevel::ERROR);
        assert_eq!(
            config.log_rotation.unwrap(),
            LogRotation::BySize(1024 * 1024)
        ); // Default rotation size: 1MB
        assert_eq!(config.log_format, "%level - %message");
        assert_eq!(
            config.logging_destinations,
            vec![
                LoggingDestination::File(PathBuf::from("/path/to/log")),
                LoggingDestination::Stdout
            ]
        );
    }

    // Test for displaying log file path
    #[test]
    fn test_config_log_file_path_display() {
        let config = Config {
            log_file_path: PathBuf::from("/path/to/log"),
            log_level: LogLevel::INFO,
            log_rotation: None,
            log_format: "%level - %message".to_string(),
            logging_destinations: vec![],
        };
        assert_eq!(config.log_file_path_display(), "/path/to/log");
    }

    // Test loading Config with invalid values
    #[test]
    fn test_config_load_with_invalid_values() {
        env::set_var("LOG_LEVEL", "INVALID");
        env::set_var("LOG_ROTATION", "INVALID");

        let result = Config::load();
        assert!(result.is_err());
        let error_message = result.unwrap_err();
        assert!(
            error_message.contains("Invalid log level")
                || error_message.contains("Invalid log rotation option")
        );
    }

    // Test loading Config with default values
    #[test]
    fn test_config_load_with_defaults() {
        // Clear environment variables
        env::remove_var("LOG_FILE_PATH");
        env::remove_var("LOG_LEVEL");
        env::remove_var("LOG_ROTATION");
        env::remove_var("LOG_FORMAT");
        env::remove_var("LOG_DESTINATIONS");

        let config = Config::load().unwrap();

        assert_eq!(config.log_file_path, PathBuf::from("RLG.log"));
        assert_eq!(config.log_level, LogLevel::INFO);
        assert_eq!(config.log_rotation, None);
        assert_eq!(config.log_format, "%level - %message");
        assert_eq!(
            config.logging_destinations,
            vec![LoggingDestination::File(PathBuf::from("RLG.log"))]
        );
    }
}
