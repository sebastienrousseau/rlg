// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

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
        collections::HashMap, env, num::NonZeroU64, path::PathBuf,
        str::FromStr,
    };
    use tokio::fs;

    /// Tests for parsing different variants of the LogLevel enum from strings.
    #[test]
    fn test_log_level_from_str_basic() {
        // Test valid log levels
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

        // Test an invalid log level
        assert!(LogLevel::from_str("INVALID").is_err());
    }

    /// Tests displaying the log file path from the Config struct.
    #[test]
    fn test_config_log_file_path_display() {
        // Set up the configuration instance with a known log file path.
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

        // Check that the log file path is correctly set.
        assert_eq!(
            config.log_file_path.display().to_string(),
            "RLG.log",
            "Log file path should be 'RLG.log'"
        );

        // Check that the log file path is a valid path and correctly points to the intended file.
        assert!(
            config.log_file_path.is_relative(),
            "The log file path should be a relative path"
        );

        // Check the string representation of the log file path.
        assert_eq!(
            config.log_file_path.to_str().unwrap(),
            "RLG.log",
            "The string representation of the path should be 'RLG.log'"
        );
    }

    #[tokio::test]
    async fn test_config_load_with_invalid_values() {
        use std::io::Write;
        use tempfile::tempdir;
        use tokio::fs;

        // Create a temporary directory to store the log file
        let temp_dir =
            tempdir().expect("Failed to create temp directory");
        let log_file_path = temp_dir.path().join("RLG.log");

        // Create the log file so that it exists and is writable
        let mut log_file = std::fs::File::create(&log_file_path)
            .expect("Failed to create log file");
        writeln!(log_file, "This is a test log file")
            .expect("Failed to write to log file");

        // Simulate loading a TOML configuration file with some default values
        let config_content = r#"
        version = "1.0"
        log_file_path = "RLG.log"
        log_format = "%level - %message"
    "#;

        let config_file_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_file_path, config_content)
            .expect("Failed to write config file");

        // Set environment variables, using invalid values for LOG_LEVEL and LOG_ROTATION
        env::set_var("LOG_LEVEL", "INVALID_LOG_LEVEL"); // Invalid log level
        env::set_var("LOG_ROTATION", "INVALID_LOG_ROTATION"); // Invalid log rotation

        // Load the configuration, which should still load successfully
        let config_result =
            Config::load_async(Some(config_file_path)).await;
        let _config = config_result.unwrap(); // Load should not fail yet

        // Now manually handle the log level and rotation parsing, mimicking the behavior of validation
        let log_level_env =
            env::var("LOG_LEVEL").expect("LOG_LEVEL not set");
        let log_rotation_env =
            env::var("LOG_ROTATION").expect("LOG_ROTATION not set");

        // Expect LogLevel parsing to fail
        assert!(
            log_level_env.parse::<LogLevel>().is_err(),
            "Expected LOG_LEVEL to be invalid"
        );

        // Expect LogRotation parsing to fail
        assert!(
            log_rotation_env.parse::<LogRotation>().is_err(),
            "Expected LOG_ROTATION to be invalid"
        );

        // Clean up environment variables after the test
        env::remove_var("LOG_LEVEL");
        env::remove_var("LOG_ROTATION");

        // Clean up the temporary directory and log file
        fs::remove_file(log_file_path)
            .await
            .expect("Failed to remove log file");
    }

    /// Tests the cloning and copying capabilities of the LogRotation enum.
    #[test]
    fn test_log_rotation_clone_and_copy() {
        // Create a NonZeroU64 instance
        let size = NonZeroU64::new(1024 * 1024)
            .expect("Failed to create NonZeroU64 instance");

        // Use the NonZeroU64 instance to create LogRotation
        let rotation1 = LogRotation::Size(size);
        let rotation2 = rotation1;

        // Check that the copied instance is equal to the original
        assert_eq!(rotation1, rotation2);
    }

    /// Tests the ConfigError enum variants.
    #[test]
    fn test_config_error() {
        let env_var_error = ConfigError::EnvVarParseError(
            envy::Error::MissingValue("Test error"),
        );

        // Check the error message for EnvVarParseError
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
        // Create different logging destinations
        let file_dest =
            LoggingDestination::File(PathBuf::from("test.log"));
        let stdout_dest = LoggingDestination::Stdout;
        let network_dest =
            LoggingDestination::Network("127.0.0.1:514".to_string());

        // Check if the destinations match the expected variant
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

    /// Tests the Config::validate method with a valid configuration.
    #[test]
    fn test_config_validate() {
        use std::env;
        use std::fs::OpenOptions;

        let temp_dir = env::temp_dir();
        let log_file_path = temp_dir.join("test_validate_RLG.log");

        // Ensure the log file exists
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&log_file_path)
            .unwrap();

        // Set up a valid configuration
        let mut config = Config {
            log_file_path,
            ..Default::default()
        };

        // Validate the configuration
        assert!(
            config.validate().is_ok(),
            "Validation should pass with valid config"
        );

        // Set an invalid log file path and validate
        config.log_file_path = "".into();
        assert!(
            config.validate().is_err(),
            "Validation should fail with empty log file path"
        );
    }

    #[test]
    fn test_config_expand_env_vars() {
        use std::env;

        // Set an environment variable
        env::set_var("RLG_LOG_PATH", "/tmp/env_test_RLG.log");

        // Create config with env var reference
        let mut config = Config::default();
        config.env_vars.insert(
            "RLG_LOG_PATH".to_string(),
            "${RLG_LOG_PATH}".to_string(),
        );

        // Expand environment variables
        let expanded_config = config.expand_env_vars();

        // Check that the variable has been expanded
        assert_eq!(
            expanded_config.env_vars.get("RLG_LOG_PATH").unwrap(),
            "/tmp/env_test_RLG.log"
        );
    }

    #[tokio::test]
    async fn test_hot_reload_async() {
        use parking_lot::RwLock;
        use std::sync::Arc;

        let temp_dir = env::temp_dir();
        let config_file_path =
            temp_dir.join("test_hot_reload_RLG.toml");

        // Create a simple config file
        let config_content = r#"
    version = "1.0"
    profile = "default"
    "#;
        fs::write(&config_file_path, config_content).await.unwrap();

        // Load default configuration and start watching the config file
        let config = Arc::new(RwLock::new(Config::default()));

        // Start hot reload and check for no errors
        let result = Config::hot_reload_async(
            config_file_path.to_str().unwrap(),
            config.clone(),
        )
        .await;
        assert!(result.is_ok(), "Hot reload setup should succeed");
    }

    #[test]
    fn test_config_diff() {
        // Create two different configurations
        let config1 = Config::default();

        let config2 = Config {
            profile: "test_profile".to_string(),
            ..Default::default()
        };

        // Get the differences
        let differences = Config::diff(&config1, &config2);

        // Check the differences are as expected
        assert_eq!(
            differences.get("profile").unwrap(),
            "default -> test_profile"
        );
    }
    #[test]
    fn test_config_merge() {
        // Create two configurations
        let config1 = Config::default();
        let config2 = Config {
            profile: "test_profile".to_string(),
            log_format: "%level - %message".to_string(),
            ..Default::default()
        };

        // Merge the configurations
        let merged_config = config1.merge(&config2);

        // Check the merged configuration
        assert_eq!(merged_config.profile, "test_profile");
        assert_eq!(merged_config.log_format, "%level - %message");
    }

    #[test]
    fn test_config_error_enum() {
        // Define a struct that expects a specific environment variable
        #[allow(dead_code)] // Ignore unused field warning
        #[derive(Deserialize, Debug)]
        struct EnvConfig {
            required_field: String,
        }

        // Simulate an envy error by trying to deserialize missing environment variables
        let env_var_error = ConfigError::EnvVarParseError(
            envy::from_env::<EnvConfig>().unwrap_err(),
        );

        // Create a custom config parse error
        let config_parse_error = ConfigError::ConfigParseError(
            config::ConfigError::Message("test error".to_string()),
        );

        // Create an invalid file path error
        let invalid_file_path_error =
            ConfigError::InvalidFilePath("invalid path".to_string());

        // Output the actual error message for debugging
        let env_var_error_message = format!("{}", env_var_error);
        println!("Env var error message: {}", env_var_error_message);

        // Assertions for expected error messages
        assert!(env_var_error_message.contains("field") || env_var_error_message.contains("parse"),
            "Env var error should contain 'field' or 'parse' but was: {}", env_var_error_message);
        assert_eq!(
            format!("{}", config_parse_error),
            "Configuration parsing error: test error"
        );
        assert_eq!(
            format!("{}", invalid_file_path_error),
            "Invalid file path: invalid path"
        );
    }
}
