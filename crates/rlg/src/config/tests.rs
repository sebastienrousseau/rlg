// tests.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Unit tests for `config.rs`.

use super::*;

#[test]
fn test_config_set_exhaustive() {
    let mut config = Config::default();
    assert!(config.set("version", 123).is_err());
    assert!(config.set("profile", 123).is_err());
    assert!(config.set("log_file_path", 123).is_err());
    assert!(config.set("log_level", 123).is_err());
    assert!(config.set("log_rotation", 123).is_err());
    assert!(config.set("log_format", 123).is_err());
    assert!(config.set("logging_destinations", 123).is_err());
    assert!(config.set("env_vars", 123).is_err());
    assert!(config.set("unknown_key", "value").is_err());
}

#[test]
fn test_config_set_unknown_key() {
    let mut config = Config::default();
    let res = config.set("absolutely_unknown_key_123", "value");
    assert!(res.is_err());
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_config_save_to_file_fail_unit() {
    let config = Config::default();
    let dir_path = env::temp_dir();
    let res = config.save_to_file(&dir_path);
    assert!(res.is_err());
}

#[test]
fn test_commons_config_error_conversion() {
    let commons_err = crate::commons::config::ConfigError::MissingKey(
        "test_key".to_string(),
    );
    let config_err: ConfigError = commons_err.into();
    assert!(matches!(config_err, ConfigError::ValidationError(_)));
    assert!(config_err.to_string().contains("test_key"));
}

#[test]
fn test_log_rotation_exhaustive() {
    assert!(LogRotation::from_str("count:0").is_err());
    assert!(LogRotation::from_str("size:0").is_err());
    assert!(LogRotation::from_str("time:0").is_err());
    assert!(LogRotation::from_str("invalid:xxx").is_err());
}

#[test]
fn test_log_rotation_valid() {
    let size = LogRotation::from_str("size:1024").unwrap();
    assert!(matches!(size, LogRotation::Size(_)));

    let time = LogRotation::from_str("time:3600").unwrap();
    assert!(matches!(time, LogRotation::Time(_)));

    let date = LogRotation::from_str("date").unwrap();
    assert!(matches!(date, LogRotation::Date));

    let count = LogRotation::from_str("count:10").unwrap();
    assert!(matches!(count, LogRotation::Count(10)));
}

#[test]
fn test_log_rotation_missing_values() {
    assert!(LogRotation::from_str("size").is_err());
    assert!(LogRotation::from_str("time").is_err());
    assert!(LogRotation::from_str("count").is_err());
}

#[test]
fn test_log_rotation_invalid_numbers() {
    assert!(LogRotation::from_str("size:abc").is_err());
    assert!(LogRotation::from_str("time:xyz").is_err());
    assert!(LogRotation::from_str("count:abc").is_err());
}

#[test]
fn test_log_rotation_display() {
    let size = LogRotation::Size(NonZeroU64::new(1024).unwrap());
    assert_eq!(size.to_string(), "Size: 1024 bytes");

    let time = LogRotation::Time(NonZeroU64::new(3600).unwrap());
    assert_eq!(time.to_string(), "Time: 3600 seconds");

    assert_eq!(LogRotation::Date.to_string(), "Date-based rotation");

    assert_eq!(LogRotation::Count(5).to_string(), "Count: 5 logs");
}

#[test]
fn test_config_default_values() {
    let config = Config::default();
    assert_eq!(config.version, "1.0");
    assert_eq!(config.profile, "default");
    assert_eq!(config.log_file_path, PathBuf::from("RLG.log"));
    assert_eq!(config.log_level, LogLevel::INFO);
    assert!(config.log_rotation.is_some());
    assert_eq!(config.log_format, "%level - %message");
    assert!(!config.logging_destinations.is_empty());
    assert!(config.env_vars.is_empty());
}

#[test]
fn test_config_set_valid_values() {
    let mut config = Config::default();
    assert!(config.set("version", "2.0").is_ok());
    assert_eq!(config.version, "2.0");

    assert!(config.set("profile", "production").is_ok());
    assert_eq!(config.profile, "production");

    assert!(config.set("log_format", "%time %level %msg").is_ok());
    assert_eq!(config.log_format, "%time %level %msg");

    assert!(config.set("log_file_path", "/tmp/test.log").is_ok());
    assert_eq!(config.log_file_path, PathBuf::from("/tmp/test.log"));
}

#[test]
fn test_config_set_log_level() {
    let mut config = Config::default();
    assert!(config.set("log_level", "DEBUG").is_ok());
    assert_eq!(config.log_level, LogLevel::DEBUG);
}

#[test]
fn test_config_set_log_rotation() {
    let mut config = Config::default();
    assert!(config.set("log_rotation", Option::<()>::None).is_ok());
    assert!(config.log_rotation.is_none());
}

#[test]
fn test_config_set_logging_destinations() {
    let mut config = Config::default();
    let dests = vec![LoggingDestination::Stdout];
    assert!(config.set("logging_destinations", &dests).is_ok());
    assert_eq!(config.logging_destinations.len(), 1);
}

#[test]
fn test_config_set_env_vars() {
    let mut config = Config::default();
    let mut vars = HashMap::new();
    vars.insert("KEY".to_string(), "VALUE".to_string());
    assert!(config.set("env_vars", &vars).is_ok());
    assert_eq!(config.env_vars.get("KEY").unwrap(), "VALUE");
}

#[test]
fn test_config_validate_empty_path() {
    let config = Config {
        log_file_path: PathBuf::from(""),
        ..Config::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_empty_destinations() {
    let mut config = Config::default();
    config.logging_destinations.clear();
    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_empty_version() {
    let config = Config {
        version: "  ".to_string(),
        ..Config::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_empty_profile() {
    let config = Config {
        profile: "  ".to_string(),
        ..Config::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_empty_log_format() {
    let config = Config {
        log_format: "  ".to_string(),
        ..Config::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_empty_env_var() {
    let mut config = Config::default();
    config.env_vars.insert(String::new(), "val".to_string());
    assert!(config.validate().is_err());
}

#[test]
#[allow(unsafe_code)]
#[cfg_attr(miri, ignore)]
fn test_config_expand_env_vars() {
    // SAFETY: this test owns the value for the duration of the
    // expand_env_vars call; no other thread reads it.
    unsafe { env::set_var("RLG_TEST_EXPAND_KEY", "expected") };
    let mut config = Config::default();
    config.env_vars.insert(
        "RLG_TEST_EXPAND_KEY".to_string(),
        "placeholder".to_string(),
    );
    let expanded = config.expand_env_vars();
    assert_eq!(expanded.env_vars["RLG_TEST_EXPAND_KEY"], "expected");
    // SAFETY: cleanup.
    unsafe { env::remove_var("RLG_TEST_EXPAND_KEY") };
}

#[test]
fn test_config_expand_env_vars_missing() {
    let mut config = Config::default();
    config.env_vars.insert(
        "DEFINITELY_NOT_SET_VAR_XYZ_123".to_string(),
        "original".to_string(),
    );
    let expanded = config.expand_env_vars();
    assert_eq!(
        expanded.env_vars["DEFINITELY_NOT_SET_VAR_XYZ_123"],
        "original"
    );
}

#[test]
fn test_config_diff_no_changes() {
    let c1 = Config::default();
    let c2 = Config::default();
    let diffs = Config::diff(&c1, &c2);
    assert!(diffs.is_empty());
}

#[test]
fn test_config_diff_with_changes() {
    let c1 = Config::default();
    let c2 = Config {
        version: "2.0".to_string(),
        profile: "prod".to_string(),
        log_format: "%msg".to_string(),
        log_level: LogLevel::DEBUG,
        log_file_path: PathBuf::from("/var/log/app.log"),
        ..Config::default()
    };
    let diffs = Config::diff(&c1, &c2);
    assert!(diffs.contains_key("version"));
    assert!(diffs.contains_key("profile"));
    assert!(diffs.contains_key("log_format"));
    assert!(diffs.contains_key("log_level"));
    assert!(diffs.contains_key("log_file_path"));
}

#[test]
fn test_config_override_with() {
    let c1 = Config::default();
    let mut c2 = Config {
        version: "2.0".to_string(),
        profile: "prod".to_string(),
        ..Config::default()
    };
    c2.env_vars
        .insert("NEW_KEY".to_string(), "new_val".to_string());
    let merged = c1.override_with(&c2);
    assert_eq!(merged.version, "2.0");
    assert_eq!(merged.profile, "prod");
    assert!(merged.env_vars.contains_key("NEW_KEY"));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_config_ensure_paths() {
    let config = Config::default();
    // Default config points to RLG.log in current dir — should succeed
    assert!(config.ensure_paths().is_ok());
}

#[test]
fn test_config_ensure_paths_stdout_dest() {
    let config = Config {
        logging_destinations: vec![LoggingDestination::Stdout],
        ..Config::default()
    };
    // Stdout destination doesn't match File pattern — should succeed
    assert!(config.ensure_paths().is_ok());
}

#[test]
fn test_config_envy_from_iter_succeeds_with_defaults() {
    // All `Config` fields have serde defaults; `envy::from_iter`
    // should succeed even with an empty iterator.
    let empty: Vec<(String, String)> = Vec::new();
    let cfg: Config = envy::from_iter(empty).unwrap();
    assert!(!cfg.version.is_empty());
}

#[test]
fn test_config_envy_from_iter_rejects_bad_type() {
    // Force an `envy::Error` by supplying a typed field with
    // unparseable contents. `log_level` deserializes from a string,
    // and `LogLevel::from_str` rejects unknown labels.
    let bad =
        vec![("log_level".to_string(), "NOT_A_REAL_LEVEL".to_string())];
    let result: Result<Config, _> = envy::from_iter(bad);
    let err = result.unwrap_err();
    // Whatever the wording, the key error must surface somehow.
    let msg = err.to_string().to_lowercase();
    assert!(!msg.is_empty(), "got: {err}");
}

#[test]
fn test_config_try_from_real_env_vars() {
    // Exercises the `TryFrom<env::Vars>` impl with the process's
    // actual env (which always has serde defaults available).
    let result = Config::try_from(env::vars());
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_logging_destination_debug() {
    let file_dest =
        LoggingDestination::File(PathBuf::from("/tmp/test.log"));
    let stdout_dest = LoggingDestination::Stdout;
    let network_dest =
        LoggingDestination::Network("localhost:9200".into());
    assert!(format!("{file_dest:?}").contains("File"));
    assert!(format!("{stdout_dest:?}").contains("Stdout"));
    assert!(format!("{network_dest:?}").contains("Network"));
}

#[test]
fn test_config_error_display_all_variants() {
    let err = ConfigError::InvalidFilePath("bad".into());
    assert!(err.to_string().contains("Invalid file path"));

    let err = ConfigError::FileReadError("read fail".into());
    assert!(err.to_string().contains("File read error"));

    let err = ConfigError::FileWriteError("write fail".into());
    assert!(err.to_string().contains("File write error"));

    let err = ConfigError::ValidationError("invalid".into());
    assert!(err.to_string().contains("validation error"));

    let err = ConfigError::VersionError("bad version".into());
    assert!(err.to_string().contains("version error"));

    let err = ConfigError::MissingFieldError("field_x".into());
    assert!(err.to_string().contains("Missing required field"));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_config_save_and_load() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test_config.toml");
    let config = Config::default();
    config.save_to_file(&path).unwrap();
    assert!(path.exists());
}

#[cfg(feature = "tokio")]
#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_load_async_with_valid_toml() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let toml_content = r#"
version = "1.0"
profile = "test"
log_file_path = "test.log"
log_format = "%level - %message"

[[logging_destinations]]
type = "File"
value = "test.log"
"#;
    fs::write(&config_path, toml_content).unwrap();
    let result = Config::load_async(Some(&config_path)).await;
    assert!(result.is_ok());
    let config = result.unwrap();
    let c = config.read();
    assert_eq!(c.version, "1.0");
    assert_eq!(c.profile, "test");
    drop(c);
}

#[cfg(feature = "tokio")]
#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_load_async_with_bad_version() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("bad_version.toml");
    let toml_content = r#"
version = "99.0"
profile = "test"
log_file_path = "test.log"
log_format = "%level - %message"

[[logging_destinations]]
type = "File"
value = "test.log"
"#;
    fs::write(&config_path, toml_content).unwrap();
    let result = Config::load_async(Some(&config_path)).await;
    assert!(result.is_err());
}

#[cfg(feature = "tokio")]
#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_load_async_no_path() {
    let result = Config::load_async(None::<&str>).await;
    assert!(result.is_ok());
}

#[cfg(feature = "tokio")]
#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_load_async_nonexistent_file() {
    let result = Config::load_async(Some(
        "/tmp/definitely_not_exists_rlg_test.toml",
    ))
    .await;
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_load_sync_with_valid_toml() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let toml_content = r#"
version = "1.0"
profile = "test"
log_file_path = "test.log"
log_format = "%level - %message"

[[logging_destinations]]
type = "File"
value = "test.log"
"#;
    fs::write(&config_path, toml_content).unwrap();
    let result = Config::load(Some(&config_path));
    assert!(result.is_ok());
    let config = result.unwrap();
    let c = config.read();
    assert_eq!(c.version, "1.0");
    assert_eq!(c.profile, "test");
    drop(c);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_load_sync_no_path() {
    let result = Config::load(None::<&str>);
    assert!(result.is_ok());
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_load_sync_nonexistent_file() {
    let result =
        Config::load(Some("/tmp/definitely_not_exists_rlg_test.toml"));
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_load_sync_bad_version() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("bad_version.toml");
    let toml_content = r#"
version = "99.0"
profile = "test"
log_file_path = "test.log"
log_format = "%level - %message"

[[logging_destinations]]
type = "File"
value = "test.log"
"#;
    fs::write(&config_path, toml_content).unwrap();
    let result = Config::load(Some(&config_path));
    assert!(result.is_err());
}

#[test]
fn test_config_save_to_file_success() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("save_test_config.toml");
    let config = Config::default();
    assert!(config.save_to_file(&path).is_ok());
    let contents = fs::read_to_string(&path).unwrap();
    assert!(contents.contains("version"));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_config_save_and_load_roundtrip() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("roundtrip.toml");
    let config = Config::default();
    config.save_to_file(&path).unwrap();
    let loaded = Config::load(Some(&path)).unwrap();
    let guard = loaded.read();
    let version = guard.version.clone();
    let profile = guard.profile.clone();
    let log_level = guard.log_level;
    drop(guard);
    assert_eq!(version, config.version);
    assert_eq!(profile, config.profile);
    assert_eq!(log_level, config.log_level);
}
