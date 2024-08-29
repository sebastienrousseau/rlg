// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(test)]
mod tests {
    use rlg::config::{
        Config, ConfigError, LogRotation, LoggingDestination,
    };
    use rlg::log_level::LogLevel;
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
        assert_eq!(LogLevel::from_str("warn").unwrap(), LogLevel::WARN);
        assert_eq!(
            LogLevel::from_str("ERROR").unwrap(),
            LogLevel::ERROR
        );
        assert!(LogLevel::from_str("INVALID").is_err());
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

        let result = Config::load(None);

        // Check if result is an error
        assert!(result.is_err(), "Config::load() should fail on invalid environment variables");

        if let Err(e) = result {
            match e {
                ConfigError::ParseError(msg) => {
                    assert!(
                        msg.contains("Invalid log level"),
                        "Error should mention invalid log level"
                    );
                }
                _ => {
                    panic!("Expected ParseError for invalid log level")
                }
            }
        }
    }

    /// Tests the cloning and copying capabilities of the LogRotation enum.
    #[test]
    fn test_log_rotation_clone_and_copy() {
        let rotation1 = LogRotation::BySize(1024 * 1024);
        let rotation2 = rotation1;
        assert_eq!(rotation1, rotation2);
    }

    /// Tests the ConfigError enum variants.
    #[test]
    fn test_config_error() {
        let env_var_error =
            ConfigError::EnvVarError("Test error".to_string());
        let parse_error =
            ConfigError::ParseError("Test error".to_string());
        let invalid_path =
            ConfigError::InvalidPath("Test error".to_string());
        let rotation_error =
            ConfigError::RotationError("Test error".to_string());

        assert!(format!("{}", env_var_error)
            .contains("environment variable error"));
        assert!(format!("{}", parse_error).contains("parsing error"));
        assert!(format!("{}", invalid_path).contains("invalid path"));
        assert!(format!("{}", rotation_error)
            .contains("file rotation error"));
    }

    /// Tests the LoggingDestination enum variants.
    #[test]
    fn test_logging_destination() {
        let file_dest =
            LoggingDestination::File(PathBuf::from("test.log"));
        let stdout_dest = LoggingDestination::Stdout;
        let network_dest =
            LoggingDestination::Network("127.0.0.1:514".to_string());

        assert!(matches!(file_dest, LoggingDestination::File(_)));
        assert!(matches!(stdout_dest, LoggingDestination::Stdout));
        assert!(matches!(network_dest, LoggingDestination::Network(_)));
    }
}
