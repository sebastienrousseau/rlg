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
    use std::collections::HashMap;
    // use std::fs::File;
    // use std::io::Write;
    use std::num::NonZeroU64;
    use std::{path::PathBuf, str::FromStr};
    // use tempfile::tempdir;
    // use tokio::fs;

    /// Tests for parsing different variants of the LogLevel enum from strings.
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
        assert!(config.log_file_path.is_file() || !config.log_file_path.exists(),
        "The path should either point to a valid file or be non-existent yet valid.");

        // Check the log file path using a different format
        assert_eq!(
            config.log_file_path.to_str().unwrap(),
            "RLG.log",
            "The string representation of the path should be 'RLG.log'"
        );

        // If needed, simulate the file being created and validate path again
        // Here we just validate that the path is correctly identified as a path
        assert!(
            config.log_file_path.is_relative(),
            "The log file path should be a relative path"
        );
    }

    // /// Tests loading the configuration with invalid environment variable values for LOG_LEVEL and LOG_ROTATION.
    // #[tokio::test]
    // async fn test_config_load_with_invalid_values() {
    //     // Create a temporary directory to store the log file
    //     let temp_dir =
    //         tempdir().expect("Failed to create temp directory");
    //     let log_file_path = temp_dir.path().join("RLG.log");

    //     // Create the log file so that it exists and is writable
    //     let mut log_file = File::create(&log_file_path)
    //         .expect("Failed to create log file");
    //     writeln!(log_file, "This is a test log file")
    //         .expect("Failed to write to log file");

    //     // Set valid values for all required fields except LOG_LEVEL and LOG_ROTATION
    //     env::set_var("LOG_FILE_PATH", log_file_path.to_str().unwrap());
    //     env::set_var("LOG_FORMAT", "%level - %message");
    //     env::set_var("LOG_LEVEL", "INVALID_LOG_LEVEL"); // Invalid log level
    //     env::set_var("LOG_ROTATION", "INVALID_LOG_ROTATION"); // Invalid log rotation

    //     // Attempt to load the configuration, which should fail due to invalid log level and rotation
    //     let result = Config::load_async(None::<&str>).await;

    //     // Check the specific error type and message
    //     if let Err(e) = result {
    //         match e {
    //             ConfigError::ConfigParseError(msg) => {
    //                 assert!(
    //                 msg.to_string().contains("Invalid log level") || msg.to_string().contains("Invalid log rotation"),
    //                 "Expected error message to mention invalid log level or rotation, got: {}",
    //                 msg
    //             );
    //             }
    //             _ => {
    //                 panic!("Expected a ParseError due to invalid log level or rotation, but got a different error: {:?}", e);
    //             }
    //         }
    //     }

    //     // Clean up environment variables after the test
    //     env::remove_var("LOG_FILE_PATH");
    //     env::remove_var("LOG_FORMAT");
    //     env::remove_var("LOG_LEVEL");
    //     env::remove_var("LOG_ROTATION");

    //     // Clean up the temporary directory and log file
    //     fs::remove_file(log_file_path)
    //         .await
    //         .expect("Failed to remove log file");
    // }

    /// Tests the cloning and copying capabilities of the LogRotation enum.
    #[test]
    fn test_log_rotation_clone_and_copy() {
        // Create a NonZeroSize, instance
        let size = NonZeroU64::new(1024 * 1024)
            .expect("Failed to create NonZeroSize,");

        // Use the NonZeroSize, instance to create LogRotation
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
}
