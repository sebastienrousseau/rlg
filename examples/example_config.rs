// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # RustLogs (RLG) Configuration Examples
//!
//! This example demonstrates the usage of the `Config` struct and related functionality in
//! the RustLogs (RLG) library. It covers configuration parsing, validation, environment
//! variable expansion, log level handling, configuration merging, and error management.

#![allow(missing_docs)]

use rlg::config::{Config, ConfigError};
use rlg::log_level::LogLevel;
use std::{env, path::PathBuf, str::FromStr};
use tempfile::tempdir;
use tokio::fs;

/// Entry point for the RustLogs configuration examples.
///
/// This function runs various examples demonstrating configuration handling,
/// including log level parsing, configuration loading, environment variable expansion,
/// and configuration validation.
///
/// # Errors
///
/// Returns an error if any of the example functions fail.
#[tokio::main]
pub(crate) async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀  **RustLogs Configuration Examples**  🦀\n");

    log_level_parsing_example();
    config_loading_example().await?;
    config_env_var_expansion_example()?;
    config_validation_example()?;
    config_merging_example()?;
    config_error_handling_example();

    println!("\n🎉  **All examples completed successfully!**");

    Ok(())
}

/// Demonstrates parsing of log levels from string representations.
///
/// This function parses various log levels (e.g., `INFO`, `DEBUG`, `ERROR`) and handles invalid inputs.
fn log_level_parsing_example() {
    println!("🦀  **Log Level Parsing Example**");
    println!("---------------------------------------------");

    let valid_levels = ["INFO", "DEBUG", "WARN", "ERROR", "NONE"];
    let invalid_level = "INVALID";

    for &level in &valid_levels {
        let parsed_level = LogLevel::from_str(level).unwrap();
        println!(
            "    🟢  Parsed log level: {}  ->  {:?}",
            level, parsed_level
        );
    }

    match LogLevel::from_str(invalid_level) {
        Ok(_) => println!(
            "    ❌  Unexpected success for invalid log level: {}",
            invalid_level
        ),
        Err(_) => println!(
            "    ✅  Correctly failed to parse invalid log level: {}",
            invalid_level
        ),
    }
}

/// Demonstrates loading configuration from a file.
///
/// This function creates a temporary configuration file and loads it into a `Config` struct.
///
/// # Errors
///
/// Returns an error if the configuration loading fails.
async fn config_loading_example(
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🦀  **Config Loading Example**");
    println!("---------------------------------------------");

    let temp_dir = tempdir()?;
    let config_content = r#"
        version = "1.0"
        log_file_path = "RLG.log"
        log_format = "%level - %message"
    "#;

    let config_file_path = temp_dir.path().join("config.toml");
    fs::write(&config_file_path, config_content).await?;

    let config = Config::load_async(Some(&config_file_path)).await?;
    println!("    ✅  Loaded config:\n    {:#?}", config);

    Ok(())
}

/// Demonstrates expanding environment variables in a configuration.
///
/// This function sets environment variables and expands them in the configuration.
///
/// # Errors
///
/// Returns an error if the environment variable handling fails.
fn config_env_var_expansion_example(
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🦀  **Config Environment Variable Expansion Example**");
    println!("---------------------------------------------");

    env::set_var("RLG_LOG_PATH", "/tmp/env_test_RLG.log");

    let mut config = Config::default();
    config.env_vars.insert(
        "RLG_LOG_PATH".to_string(),
        "${RLG_LOG_PATH}".to_string(),
    );

    let expanded_config = config.expand_env_vars();
    println!(
        "    ✅  Expanded config with env vars:\n    {:#?}",
        expanded_config
    );

    env::remove_var("RLG_LOG_PATH");

    Ok(())
}

/// Demonstrates the validation of a configuration.
///
/// This function validates the configuration and handles both valid and invalid cases.
///
/// # Errors
///
/// Returns an error if configuration validation fails.
fn config_validation_example() -> Result<(), Box<dyn std::error::Error>>
{
    println!("\n🦀  **Config Validation Example**");
    println!("---------------------------------------------");

    let temp_dir = env::temp_dir();
    let log_file_path = temp_dir.join("test_validate_RLG.log");

    let mut config = Config {
        log_file_path: log_file_path.clone(),
        ..Default::default()
    };

    // Valid configuration
    match config.validate() {
        Ok(_) => println!("    ✅  Validation passed with valid config: {:#?}", config),
        Err(e) => println!("    ❌  Validation failed with valid config: {:#?}\n    Error: {}", config, e),
    }

    // Invalid configuration (empty log file path)
    config.log_file_path = PathBuf::new();
    match config.validate() {
        Ok(_) => println!("    ❌  Validation unexpectedly passed with invalid config: {:#?}", config),
        Err(e) => println!("    ✅  Validation failed as expected with invalid config: {:#?}\n    Error: {}", config, e),
    }

    Ok(())
}

/// Demonstrates merging two configurations.
///
/// This function creates two `Config` objects and merges them, demonstrating how new
/// values override existing values.
fn config_merging_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🦀  **Config Merging Example**");
    println!("---------------------------------------------");

    let config1 = Config::default();
    let config2 = Config {
        profile: "test_profile".to_string(),
        log_format: "%level - %message".to_string(),
        ..Default::default()
    };

    let merged_config = config1.merge(&config2);
    println!("    ✅  Merged config:\n    {:#?}", merged_config);

    Ok(())
}

/// Demonstrates handling various configuration-related errors.
///
/// This function covers different error cases for environment variable parsing, configuration
/// parsing, and invalid file paths.
fn config_error_handling_example() {
    println!("\n🦀  **Config Error Handling Example**");
    println!("---------------------------------------------");

    let env_var_error = ConfigError::EnvVarParseError(
        envy::Error::MissingValue("Test error"),
    );
    let config_parse_error = ConfigError::ConfigParseError(
        config::ConfigError::Message("test error".to_string()),
    );
    let invalid_file_path_error =
        ConfigError::InvalidFilePath("invalid/path".to_string());

    println!("    ✅  EnvVarParseError: {}", env_var_error);
    println!("    ✅  ConfigParseError: {}", config_parse_error);
    println!(
        "    ✅  InvalidFilePathError: {}",
        invalid_file_path_error
    );
}
