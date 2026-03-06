#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    use rlg::config::{Config, ConfigError, LoggingDestination};
    use std::fs;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_load_async_file_read_error() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = Config::load_async(Some(temp_dir.path())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_async_unsupported_version() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("invalid_version.json");
        fs::write(&config_path, r#"{"version": "0.0"}"#).unwrap();
        let result =
            Config::load_async(Some(config_path.to_str().unwrap()))
                .await;
        assert!(result.is_err());
    }

    #[test]
    fn test_log_rotation_from_str_invalid_values() {
        use rlg::config::LogRotation;
        use std::str::FromStr;

        assert!(LogRotation::from_str("invalid").is_err());
        assert!(LogRotation::from_str("count:0").is_err());
        assert!(LogRotation::from_str("size:0").is_err());
        assert!(LogRotation::from_str("time:0").is_err());
    }

    #[test]
    fn test_validate_network_address_is_ignored() {
        let config = Config {
            logging_destinations: vec![LoggingDestination::Network(
                "nonexistent.invalid:8080".to_string(),
            )],
            ..Config::default()
        };
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_async_missing_version() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("missing_version.toml");
        fs::write(&config_path, r#"profile = "test""#).unwrap();
        let result =
            Config::load_async(Some(config_path.to_str().unwrap()))
                .await;
        assert!(result.is_err());
    }

    #[test]
    fn test_config_error_all_variants() {
        let err = ConfigError::ValidationError("val".to_string());
        assert_eq!(
            format!("{}", err),
            "Configuration validation error: val"
        );

        let err = ConfigError::FileWriteError("io".to_string());
        assert_eq!(format!("{}", err), "File write error: io");

        let err = ConfigError::FileReadError("read".to_string());
        assert_eq!(format!("{}", err), "File read error: read");

        let err = ConfigError::InvalidFilePath("path".to_string());
        assert_eq!(format!("{}", err), "Invalid file path: path");

        let err = ConfigError::VersionError("v".to_string());
        assert_eq!(
            format!("{}", err),
            "Configuration version error: v"
        );

        let err = ConfigError::MissingFieldError("f".to_string());
        assert_eq!(format!("{}", err), "Missing required field: f");

        let err = ConfigError::EnvVarParseError(envy::Error::Custom(
            "env".to_string(),
        ));
        assert_eq!(
            format!("{}", err),
            "Environment variable parse error: env"
        );

        // SourceConfigError is an alias for config::ConfigError
        // We can create one by trying to build an empty config source or similar
        let config_source = config::Config::builder()
            .add_source(config::File::with_name("non_existent"))
            .build();
        let source_err = config_source.unwrap_err();
        let err = ConfigError::ConfigParseError(source_err);
        assert!(
            format!("{}", err).contains("Configuration parsing error:")
        );
    }

    #[test]
    fn test_logging_destination_display() {
        let dest =
            LoggingDestination::Network("127.0.0.1:8080".to_string());
        assert_eq!(
            format!("{:?}", dest),
            "Network(\"127.0.0.1:8080\")"
        );

        let dest = LoggingDestination::Stdout;
        assert_eq!(format!("{:?}", dest), "Stdout");

        let dest = LoggingDestination::File(PathBuf::from("test.log"));
        assert_eq!(format!("{:?}", dest), "File(\"test.log\")");
    }

    #[tokio::test]
    async fn test_config_validate_create_parent_dir_fail() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("a_file");
        fs::write(&file_path, "not a dir").unwrap();

        let log_file = file_path.join("uncreatable/test.log");
        let config = Config {
            logging_destinations: vec![LoggingDestination::File(
                log_file,
            )],
            ..Config::default()
        };

        let result = config.validate();
        assert!(result.is_err());
    }
}
