// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Tests for the configuration module of RustLogs (RLG).
//!
//! This module contains comprehensive tests for the `Config` struct and related
//! functionality, including parsing, validation, environment variable handling,
//! and various configuration operations.

#[cfg(test)]
mod tests {
    use rlg::{
        config::{
            Config, ConfigError, LogRotation, LoggingDestination,
        },
        log_level::LogLevel,
    };
    use serde::Deserialize;
    use std::{
        collections::HashMap,
        env,
        num::NonZeroU64,
        path::PathBuf,
        str::FromStr,
    };
    use tempfile::tempdir;
    use tokio::{fs, io::AsyncWriteExt};

    /// Tests parsing different variants of the LogLevel enum from strings.
    #[test]
    fn test_log_level_from_str_basic() {
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
            version: "1.0".to_string(),
            profile: "test".to_string(),
            log_file_path: PathBuf::from("RLG.log"),
            log_level: LogLevel::INFO,
            log_rotation: None,
            log_format: "%level - %message".to_string(),
            logging_destinations: vec![],
            env_vars: HashMap::new(),
        };

        assert_eq!(
            config.log_file_path.display().to_string(),
            "RLG.log",
            "Log file path should be 'RLG.log'"
        );
        assert!(
            config.log_file_path.is_relative(),
            "The log file path should be a relative path"
        );
        assert_eq!(
            config.log_file_path.to_str().unwrap(),
            "RLG.log",
            "The string representation of the path should be 'RLG.log'"
        );
    }

    /// Tests loading configuration with invalid values for LOG_LEVEL and LOG_ROTATION.
    #[tokio::test]
    async fn test_config_load_with_invalid_values() {
        let temp_dir =
            tempdir().expect("Failed to create temp directory");
        let log_file_path = temp_dir.path().join("RLG.log");
        let mut log_file = fs::File::create(&log_file_path)
            .await
            .expect("Failed to create log file");
        log_file
            .write_all(b"This is a test log file")
            .await
            .expect("Failed to write to log file");

        let config_content = r#"
        version = "1.0"
        log_file_path = "RLG.log"
        log_format = "%level - %message"
    "#;

        let config_file_path = temp_dir.path().join("config.toml");
        fs::write(&config_file_path, config_content)
            .await
            .expect("Failed to write config file");

        env::set_var("LOG_LEVEL", "INVALID_LOG_LEVEL");
        env::set_var("LOG_ROTATION", "INVALID_LOG_ROTATION");

        let config_result =
            Config::load_async(Some(&config_file_path)).await;
        assert!(config_result.is_ok(), "Config load should not fail");

        let log_level_env =
            env::var("LOG_LEVEL").expect("LOG_LEVEL not set");
        let log_rotation_env =
            env::var("LOG_ROTATION").expect("LOG_ROTATION not set");

        assert!(
            log_level_env.parse::<LogLevel>().is_err(),
            "Expected LOG_LEVEL to be invalid"
        );
        assert!(
            log_rotation_env.parse::<LogRotation>().is_err(),
            "Expected LOG_ROTATION to be invalid"
        );

        env::remove_var("LOG_LEVEL");
        env::remove_var("LOG_ROTATION");

        fs::remove_file(log_file_path)
            .await
            .expect("Failed to remove log file");
    }

    /// Tests the cloning and copying capabilities of the LogRotation enum.
    #[test]
    fn test_log_rotation_clone_and_copy() {
        let size = NonZeroU64::new(1024 * 1024)
            .expect("Failed to create NonZeroU64 instance");
        let rotation1 = LogRotation::Size(size);
        let rotation2 = rotation1;
        assert_eq!(rotation1, rotation2);
    }

    /// Tests the ConfigError enum variants.
    #[test]
    fn test_config_error() {
        let env_var_error = ConfigError::EnvVarParseError(
            envy::Error::MissingValue("Test error"),
        );
        assert!(
            format!("{}", env_var_error)
                .contains("Environment variable parse error"),
            "Unexpected error message for EnvVarParseError: {}",
            env_var_error
        );
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

    /// Comprehensive test for parsing various log levels, including invalid inputs.
    #[test]
    fn test_log_level_from_str_comprehensive() {
        let test_cases = [
            ("ALL", Ok(LogLevel::ALL)),
            ("DEBUG", Ok(LogLevel::DEBUG)),
            ("INFO", Ok(LogLevel::INFO)),
            ("WARN", Ok(LogLevel::WARN)),
            ("ERROR", Ok(LogLevel::ERROR)),
            ("FATAL", Ok(LogLevel::FATAL)),
            ("TRACE", Ok(LogLevel::TRACE)),
            ("VERBOSE", Ok(LogLevel::VERBOSE)),
            ("NONE", Ok(LogLevel::NONE)),
            ("DISABLED", Ok(LogLevel::DISABLED)),
            ("CRITICAL", Ok(LogLevel::CRITICAL)),
            ("invalid", Err(())),
        ];

        for (input, expected) in test_cases.iter() {
            let result = LogLevel::from_str(input);
            match (result, expected) {
                (Ok(level), Ok(expected_level)) => assert_eq!(
                    level, *expected_level,
                    "Failed for input: {}",
                    input
                ),
                (Err(_), Err(())) => {} // Test passed for invalid input
                _ => panic!("Unexpected result for input: {}", input),
            }
        }

        // Test case insensitivity
        assert!(matches!(
            LogLevel::from_str("info"),
            Ok(LogLevel::INFO)
        ));
        assert!(matches!(
            LogLevel::from_str("ErRoR"),
            Ok(LogLevel::ERROR)
        ));
    }

    /// Tests the Config::validate method with valid and invalid configurations.
    #[test]
    fn test_config_validate() {
        let temp_dir = env::temp_dir();
        let log_file_path = temp_dir.join("test_validate_RLG.log");

        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&log_file_path)
            .unwrap();

        let mut config = Config {
            log_file_path,
            ..Default::default()
        };

        assert!(
            config.validate().is_ok(),
            "Validation should pass with valid config"
        );

        config.log_file_path = PathBuf::new();
        assert!(
            config.validate().is_err(),
            "Validation should fail with empty log file path"
        );
    }

    /// Tests the Config::expand_env_vars method.
    #[test]
    fn test_config_expand_env_vars() {
        env::set_var("RLG_LOG_PATH", "/tmp/env_test_RLG.log");

        let mut config = Config::default();
        config.env_vars.insert(
            "RLG_LOG_PATH".to_string(),
            "${RLG_LOG_PATH}".to_string(),
        );

        let expanded_config = config.expand_env_vars();

        assert_eq!(
            expanded_config.env_vars.get("RLG_LOG_PATH").unwrap(),
            "/tmp/env_test_RLG.log"
        );

        env::remove_var("RLG_LOG_PATH");
    }

    /// Tests the Config::hot_reload_async method.
    #[tokio::test]
    async fn test_hot_reload_async() {
        use parking_lot::RwLock;
        use std::sync::Arc;

        let temp_dir = env::temp_dir();
        let config_file_path =
            temp_dir.join("test_hot_reload_RLG.toml");

        let config_content = r#"
    version = "1.0"
    profile = "default"
    "#;
        fs::write(&config_file_path, config_content).await.unwrap();

        let config = Arc::new(RwLock::new(Config::default()));

        let result = Config::hot_reload_async(
            config_file_path.to_str().unwrap(),
            config.clone(),
        )
        .await;
        assert!(result.is_ok(), "Hot reload setup should succeed");

        fs::remove_file(config_file_path)
            .await
            .expect("Failed to remove test config file");
    }

    /// Tests the Config::diff method.
    #[test]
    fn test_config_diff() {
        let config1 = Config::default();
        let config2 = Config {
            profile: "test_profile".to_string(),
            ..Default::default()
        };

        let differences = Config::diff(&config1, &config2);

        assert_eq!(
            differences.get("profile").unwrap(),
            "default -> test_profile"
        );
    }

    /// Tests the Config::merge method.
    #[test]
    fn test_config_merge() {
        let config1 = Config::default();
        let config2 = Config {
            profile: "test_profile".to_string(),
            log_format: "%level - %message".to_string(),
            ..Default::default()
        };

        let merged_config = config1.merge(&config2);

        assert_eq!(merged_config.profile, "test_profile");
        assert_eq!(merged_config.log_format, "%level - %message");
    }

    /// Tests the ConfigError enum variants thoroughly.
    #[test]
    fn test_config_error_enum() {
        #[allow(dead_code)]
        #[derive(Deserialize, Debug)]
        struct EnvConfig {
            required_field: String,
        }

        let env_var_error = ConfigError::EnvVarParseError(
            envy::from_env::<EnvConfig>().unwrap_err(),
        );

        let config_parse_error = ConfigError::ConfigParseError(
            config::ConfigError::Message("test error".to_string()),
        );

        let invalid_file_path_error =
            ConfigError::InvalidFilePath("invalid path".to_string());

        let env_var_error_message = format!("{}", env_var_error);
        println!("Env var error message: {}", env_var_error_message);

        assert!(
            env_var_error_message.contains("field") || env_var_error_message.contains("parse"),
            "Env var error should contain 'field' or 'parse' but was: {}",
            env_var_error_message
        );
        assert_eq!(
            format!("{}", config_parse_error),
            "Configuration parsing error: test error"
        );
        assert_eq!(
            format!("{}", invalid_file_path_error),
            "Invalid file path: invalid path"
        );
    }

    // Additional tests for Config methods

    /// Tests the Config::get method.
    #[test]
    fn test_config_get() {
        let config = Config {
            version: "1.0".to_string(),
            profile: "test".to_string(),
            log_file_path: PathBuf::from("test.log"),
            log_level: LogLevel::INFO,
            log_rotation: Some(LogRotation::Size(
                NonZeroU64::new(1024).unwrap(),
            )),
            log_format: "%level - %message".to_string(),
            logging_destinations: vec![LoggingDestination::File(
                PathBuf::from("test.log"),
            )],
            env_vars: HashMap::new(),
        };

        assert_eq!(
            config.get::<String>("version"),
            Some("1.0".to_string())
        );
        assert_eq!(
            config.get::<String>("profile"),
            Some("test".to_string())
        );
        assert_eq!(
            config.get::<LogLevel>("log_level"),
            Some(LogLevel::INFO)
        );
        assert_eq!(config.get::<String>("non_existent"), None);
    }

    /// Tests the Config::set method.
    #[test]
    fn test_config_set() {
        let mut config = Config::default();

        assert!(config.set("version", "2.0").is_ok());
        assert_eq!(config.version, "2.0");

        assert!(config.set("log_level", LogLevel::DEBUG).is_ok());
        assert_eq!(config.log_level, LogLevel::DEBUG);

        assert!(config.set("non_existent", "value").is_err());
    }

    /// Tests the Config::save_to_file method.
    #[test]
    fn test_config_save_to_file() {
        let temp_dir =
            tempdir().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("test_config.json");

        let config = Config::default();
        assert!(config.save_to_file(&config_path).is_ok());

        assert!(
            config_path.exists(),
            "Config file should have been created"
        );
    }
}
