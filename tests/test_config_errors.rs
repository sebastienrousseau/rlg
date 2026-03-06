#![allow(missing_docs)]
#![allow(deprecated)]
#[cfg(test)]
mod tests {
    use rlg::config::{Config, ConfigError};
    use std::fs;

    #[test]
    fn test_config_set_invalid_types() {
        let mut config = Config::default();

        // 1. Invalid version format (expects a string, passing a number)
        let res = config.set("version", 123);
        assert!(matches!(res, Err(ConfigError::ValidationError(_))));

        // 2. Invalid profile format (expects a string, passing a boolean)
        let res = config.set("profile", true);
        assert!(matches!(res, Err(ConfigError::ValidationError(_))));

        // 3. Invalid log format (expects a string, passing a number)
        let res = config.set("log_format", 456);
        assert!(matches!(res, Err(ConfigError::ValidationError(_))));

        // 4. Invalid log_file_path (expects a string or path, passing something that fails to deserialize into PathBuf)
        let res = config.set("log_file_path", vec![1, 2, 3]);
        assert!(matches!(res, Err(ConfigError::ConfigParseError(_))));

        // 5. Invalid log_level
        let res = config.set("log_level", "INVALID_LEVEL");
        assert!(matches!(res, Err(ConfigError::ConfigParseError(_))));

        // 6. Invalid log_rotation
        let res = config.set("log_rotation", "INVALID_ROTATION");
        assert!(matches!(res, Err(ConfigError::ConfigParseError(_))));

        // 7. Invalid logging_destinations
        let res = config.set("logging_destinations", "NOT_A_LIST");
        assert!(matches!(res, Err(ConfigError::ConfigParseError(_))));

        // 8. Invalid env_vars
        let res = config.set("env_vars", "NOT_A_MAP");
        assert!(matches!(res, Err(ConfigError::ConfigParseError(_))));
    }

    #[test]
    fn test_save_to_file_errors() {
        let config = Config::default();

        // Use a path that is a directory to trigger a write error
        let dir_path = std::env::temp_dir();
        let res = config.save_to_file(&dir_path);
        assert!(matches!(res, Err(ConfigError::FileWriteError(_))));
    }

    #[tokio::test]
    async fn test_hot_reload_async_coverage_events() {
        use parking_lot::RwLock;
        use std::sync::Arc;
        use tokio::time::{sleep, Duration};

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let config = Config::default();
        config.save_to_file(&config_path).unwrap();

        let shared_config = Arc::new(RwLock::new(Config::default()));
        let stop_tx = Config::hot_reload_async(
            config_path.to_str().unwrap(),
            &shared_config,
        )
        .unwrap();

        // Trigger Modify
        config.save_to_file(&config_path).unwrap();
        sleep(Duration::from_millis(100)).await;

        // Trigger Remove
        let _ = fs::remove_file(&config_path);
        sleep(Duration::from_millis(100)).await;

        // Trigger Create
        config.save_to_file(&config_path).unwrap();
        sleep(Duration::from_millis(100)).await;

        let _ = stop_tx.send(()).await;
    }

    #[tokio::test]
    async fn test_hot_reload_async_invalid_toml() {
        use parking_lot::RwLock;
        use std::sync::Arc;
        use tokio::time::{sleep, Duration};

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let config = Config::default();
        config.save_to_file(&config_path).unwrap();

        let shared_config = Arc::new(RwLock::new(Config::default()));
        let stop_tx = Config::hot_reload_async(
            config_path.to_str().unwrap(),
            &shared_config,
        )
        .unwrap();

        // Write invalid TOML to trigger reload error branch
        fs::write(&config_path, "invalid = [toml").unwrap();
        sleep(Duration::from_millis(200)).await;

        let _ = stop_tx.send(()).await;
    }

    #[tokio::test]
    async fn test_load_async_missing_version() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("missing_version.json");
        fs::write(&config_path, r#"{"profile": "test"}"#).unwrap();
        let result =
            Config::load_async(Some(config_path.to_str().unwrap()))
                .await;
        assert!(result.is_err());
    }
}
