#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    use rlg::log_level::LogLevel;
    use rlg::log_format::LogFormat;
    use rlg::utils::{generate_timestamp, is_file_writable, is_directory_writable};
    use tempfile::tempdir;
    use std::fs;
    use std::path::Path;

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
    async fn test_utils_is_file_writable_cannot_create() {
        // A file in a non-existent directory cannot be created
        let path = Path::new("/non_existent_dir_12345/some_file.log");
        let result = is_file_writable(path).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
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
    fn test_log_semantic_context_tagging() {
        use rlg::log::Log;
        let mut log = Log::new("sid", "ts", &LogLevel::INFO, "comp", "desc", &LogFormat::JSON);
        log.attributes.insert("user_id".to_string(), serde_json::json!(123));
        log.attributes.insert("action".to_string(), serde_json::json!("login"));
        
        let output = format!("{}", log);
        assert!(output.contains("\"user_id\":123"));
        assert!(output.contains("\"action\":\"login\""));
    }
}
