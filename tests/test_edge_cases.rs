#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    use rlg::log_format::LogFormat;
    use rlg::log_level::LogLevel;
    #[allow(unused_imports)]
    use rlg::utils::{
        generate_timestamp, is_directory_writable, is_file_writable,
    };
    use std::fs;
    #[allow(unused_imports)]
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_log_level_numeric_all_variants() {
        assert_eq!(LogLevel::ALL.to_numeric(), 0);
        assert_eq!(LogLevel::NONE.to_numeric(), 1);
        assert_eq!(LogLevel::DISABLED.to_numeric(), 2);
        assert_eq!(LogLevel::DEBUG.to_numeric(), 3);
        assert_eq!(LogLevel::TRACE.to_numeric(), 4);
        assert_eq!(LogLevel::VERBOSE.to_numeric(), 5);
        assert_eq!(LogLevel::INFO.to_numeric(), 6);
        assert_eq!(LogLevel::WARN.to_numeric(), 7);
        assert_eq!(LogLevel::ERROR.to_numeric(), 8);
        assert_eq!(LogLevel::FATAL.to_numeric(), 9);
        assert_eq!(LogLevel::CRITICAL.to_numeric(), 10);
    }

    #[test]
    fn test_log_format_json_formatting_error() {
        let format = LogFormat::JSON;
        // Truly invalid JSON to trigger error
        let result = format.format_log("{");
        assert!(result.is_err());
    }

    #[test]
    fn test_utils_generate_timestamp_coverage() {
        let ts = generate_timestamp();
        assert!(!ts.is_empty());
    }

    #[tokio::test]
    #[cfg(not(windows))]
    async fn test_utils_is_file_writable_cannot_create() {
        // A file in a non-existent directory cannot be created
        let path = Path::new("/non_existent_dir_12345/some_file.log");
        let result = is_file_writable(path).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    #[cfg(not(windows))]
    async fn test_utils_is_directory_writable_read_only() {
        // Create a temp directory and make it read-only
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path();

        let mut perms = fs::metadata(dir_path).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(dir_path, perms).unwrap();

        let result = is_directory_writable(dir_path).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    #[allow(deprecated)]
    fn test_log_semantic_context_tagging() {
        use rlg::log::Log;
        let mut log = Log::new(
            "sid",
            "ts",
            &LogLevel::INFO,
            "comp",
            "desc",
            &LogFormat::JSON,
        );
        log.attributes
            .insert("user_id".to_string(), serde_json::json!(123));
        log.attributes
            .insert("action".to_string(), serde_json::json!("login"));

        let output = format!("{}", log);
        assert!(output.contains("\"user_id\":123"));
        assert!(output.contains("\"action\":\"login\""));
    }

    #[test]
    fn test_engine_fast_serializer() {
        use rlg::engine::FastSerializer;
        let mut buf = Vec::new();
        FastSerializer::append_u64(&mut buf, 12345);
        assert_eq!(String::from_utf8(buf).unwrap(), "12345");

        let mut buf = Vec::new();
        FastSerializer::append_f64(&mut buf, 123.45);
        assert_eq!(String::from_utf8(buf).unwrap(), "123.45");
    }

    #[test]
    fn test_engine_filter_level_getter() {
        use rlg::engine::ENGINE;
        ENGINE.set_filter(5);
        assert_eq!(ENGINE.filter_level(), 5);
        ENGINE.set_filter(0);
    }

    #[tokio::test]
    async fn test_utils_truncate_file_larger() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.log");
        fs::write(&file_path, "12345").unwrap();

        // Truncate to larger size
        rlg::utils::truncate_file(&file_path, 10).await.unwrap();
        let metadata = fs::metadata(&file_path).unwrap();
        assert_eq!(metadata.len(), 10);
    }

    #[test]
    fn test_utils_sanitize_control_chars() {
        let sanitized =
            rlg::utils::sanitize_log_message("hello\x01world");
        assert_eq!(sanitized, "hello world");
    }

    #[test]
    fn test_log_format_unknown() {
        use rlg::log_format::LogFormat;
        use std::str::FromStr;
        let res = LogFormat::from_str("UNKNOWN_FORMAT_123");
        assert!(res.is_err());
    }

    #[test]
    fn test_tui_metrics_dec_spans() {
        use rlg::tui::TuiMetrics;
        use std::sync::atomic::Ordering;
        let metrics = TuiMetrics::default();
        metrics.inc_spans();
        assert_eq!(metrics.active_spans.load(Ordering::Relaxed), 1);
        metrics.dec_spans();
        assert_eq!(metrics.active_spans.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_log_json_with_attributes() {
        use rlg::log::Log;
        let mut log = Log::info("json test").format(LogFormat::JSON);
        log.attributes
            .insert("key".to_string(), serde_json::json!("val"));
        let output = format!("{}", log);
        assert!(output.contains("\"key\":\"val\""));
    }
}
