#![cfg(not(miri))]
#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    use rlg::config::{Config, ConfigError, LoggingDestination};
    use std::collections::HashMap;
    use std::env;
    use std::path::PathBuf;

    #[test]
    fn test_config_validate_empty_version() {
        let config = Config {
            version: "".to_string(),
            ..Config::default()
        };
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError(msg) if msg.contains("Version cannot be empty"))
        );
    }

    #[test]
    fn test_config_validate_empty_profile() {
        let config = Config {
            profile: "".to_string(),
            ..Config::default()
        };
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError(msg) if msg.contains("Profile cannot be empty"))
        );
    }

    #[test]
    fn test_config_validate_empty_log_file_path() {
        let config = Config {
            log_file_path: PathBuf::from(""),
            ..Config::default()
        };
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError(msg) if msg.contains("Log file path cannot be empty"))
        );
    }

    #[test]
    fn test_config_validate_empty_log_format() {
        let config = Config {
            log_format: "".to_string(),
            ..Config::default()
        };
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError(msg) if msg.contains("Log format cannot be empty"))
        );
    }

    #[test]
    fn test_config_validate_empty_logging_destinations() {
        let config = Config {
            logging_destinations: vec![],
            ..Config::default()
        };
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError(msg) if msg.contains("At least one logging destination must be specified"))
        );
    }

    #[test]
    fn test_config_validate_empty_network_address() {
        let config = Config {
            logging_destinations: vec![LoggingDestination::Network(
                "".to_string(),
            )],
            ..Config::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_invalid_network_address() {
        let config = Config {
            logging_destinations: vec![LoggingDestination::Network(
                "invalid_address".to_string(),
            )],
            ..Config::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_empty_env_var_key() {
        let mut env_vars = HashMap::new();
        env_vars.insert("".to_string(), "value".to_string());
        let config = Config {
            env_vars,
            ..Config::default()
        };
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError(msg) if msg.contains("Environment variable key cannot be empty"))
        );
    }

    #[test]
    fn test_config_validate_empty_env_var_value() {
        let mut env_vars = HashMap::new();
        env_vars.insert("KEY".to_string(), "".to_string());
        let config = Config {
            env_vars,
            ..Config::default()
        };
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError(msg) if msg.contains("Value for environment variable 'KEY' cannot be empty"))
        );
    }

    #[test]
    fn test_config_try_from_env_vars() {
        env::set_var("LOG_LEVEL", "NOT_A_LEVEL");
        let env_vars = env::vars();
        let res = Config::try_from(env_vars);
        assert!(res.is_err());
        env::remove_var("LOG_LEVEL");
    }

    #[test]
    fn test_config_set_complex_fields() {
        let mut config = Config::default();

        let dests = vec![LoggingDestination::Stdout];
        config.set("logging_destinations", &dests).unwrap();
        assert_eq!(config.logging_destinations, dests);

        let mut envs = HashMap::new();
        envs.insert("K".to_string(), "V".to_string());
        config.set("env_vars", &envs).unwrap();
        assert_eq!(config.env_vars, envs);
    }

    #[test]
    fn test_config_validate_network_address_success() {
        let config = Config {
            logging_destinations: vec![LoggingDestination::Network(
                "127.0.0.1:8080".to_string(),
            )],
            ..Config::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_save_to_file_fail() {
        let config = Config::default();
        let res = config.save_to_file(env::temp_dir());
        assert!(res.is_err());
    }

    #[test]
    fn test_config_set_unserializeable() {
        let mut config = Config::default();
        let res = config.set("unknown_key", "value");
        assert!(res.is_err());
    }

    #[test]
    fn test_config_expand_env_vars_coverage() {
        let mut config = Config::default();
        config.env_vars.insert(
            "LOG_LEVEL_ENV".to_string(),
            "original".to_string(),
        );
        env::set_var("LOG_LEVEL_ENV", "DEBUG_ENV_VAR");
        let expanded = config.expand_env_vars();
        assert_eq!(
            expanded.env_vars.get("LOG_LEVEL_ENV").unwrap(),
            "DEBUG_ENV_VAR"
        );
    }

    #[tokio::test]
    async fn test_load_async_no_path() {
        let res = Config::load_async(None::<&str>).await;
        assert!(res.is_ok());
    }

    #[test]
    fn test_config_validate_unwritable_file() {
        let config = Config {
            logging_destinations: vec![LoggingDestination::File(
                env::temp_dir(),
            )],
            ..Config::default()
        };
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError(msg) if msg.contains("Log file is not writable"))
        );
    }

    #[test]
    fn test_config_set_error_branches() {
        let mut config = Config::default();
        assert!(config.set("profile", 123).is_err());
        assert!(config.set("log_format", 123).is_err());
        assert!(config.set("version", 123).is_err());
        assert!(config.set("log_file_path", 123).is_err());
        assert!(config.set("log_level", 123).is_err());
        assert!(config.set("log_rotation", 123).is_err());
        assert!(config.set("logging_destinations", 123).is_err());
        assert!(config.set("env_vars", 123).is_err());
        assert!(config.set("unknown", "value").is_err());
    }

    #[test]
    fn test_log_rotation_from_str_errors() {
        use rlg::config::LogRotation;
        use std::str::FromStr;
        assert!(LogRotation::from_str("size").is_err());
        assert!(LogRotation::from_str("size:invalid").is_err());
        assert!(LogRotation::from_str("time").is_err());
        assert!(LogRotation::from_str("time:invalid").is_err());
        assert!(LogRotation::from_str("count").is_err());
        assert!(LogRotation::from_str("count:invalid").is_err());
        assert!(LogRotation::from_str("invalid:val").is_err());
    }

    #[test]
    fn test_config_error_display() {
        use rlg::config::ConfigError;
        let err = ConfigError::ValidationError("test".to_string());
        assert_eq!(
            format!("{}", err),
            "Configuration validation error: test"
        );
    }

    #[test]
    fn test_config_diff() {
        use rlg::LogLevel;
        let config1 = Config::default();
        let config2 = Config {
            version: "2.0".to_string(),
            profile: "prod".to_string(),
            log_file_path: PathBuf::from("prod.log"),
            log_level: LogLevel::ERROR,
            log_rotation: None,
            log_format: "prod_format".to_string(),
            logging_destinations: vec![LoggingDestination::Stdout],
            env_vars: {
                let mut map = HashMap::new();
                map.insert("K".to_string(), "V".to_string());
                map
            },
        };

        let diffs = Config::diff(&config1, &config2);
        assert!(diffs.contains_key("version"));
        assert!(diffs.contains_key("profile"));
        assert!(diffs.contains_key("log_file_path"));
        assert!(diffs.contains_key("log_level"));
        assert!(diffs.contains_key("log_rotation"));
        assert!(diffs.contains_key("log_format"));
        assert!(diffs.contains_key("logging_destinations"));
        assert!(diffs.contains_key("env_vars"));
    }

    #[test]
    fn test_config_merge() {
        let config1 = Config::default();
        let mut config2 = Config::default();
        config2.env_vars.insert("K".to_string(), "V".to_string());
        let merged = config1.merge(&config2);
        assert_eq!(merged.env_vars.get("K").unwrap(), "V");
    }

    #[test]
    fn test_log_rotation_display() {
        use rlg::config::LogRotation;
        use std::num::NonZeroU64;
        let s = format!(
            "{}",
            LogRotation::Size(NonZeroU64::new(10).unwrap())
        );
        assert!(s.contains("Size: 10 bytes"));
        let s = format!(
            "{}",
            LogRotation::Time(NonZeroU64::new(60).unwrap())
        );
        assert!(s.contains("Time: 60 seconds"));
        let s = format!("{}", LogRotation::Date);
        assert!(s.contains("Date-based rotation"));
        let s = format!("{}", LogRotation::Count(5));
        assert!(s.contains("Count: 5 logs"));
    }
}
