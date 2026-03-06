#![cfg(not(miri))]
#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    use rlg::utils::*;
    use std::path::Path;
    use tokio::fs::{self, File};

    use tempfile::tempdir;
    use tokio::io::AsyncWriteExt;

    #[test]
    fn test_sanitize_log_message() {
        let input = "Hello\nWorld\r\u{0007}";
        let expected = "Hello World  ";
        assert_eq!(sanitize_log_message(input), expected);
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(1023), "1023.00 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[tokio::test]
    async fn test_is_file_writable() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.log");

        // Test non-existent file
        assert!(is_file_writable(&file_path).await.unwrap());

        // Test writable file
        File::create(&file_path).await.unwrap();
        assert!(is_file_writable(&file_path).await.unwrap());

        // Test read-only file
        let mut perms =
            fs::metadata(&file_path).await.unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&file_path, perms).await.unwrap();
        assert!(!is_file_writable(&file_path).await.unwrap());
    }

    #[tokio::test]
    async fn test_truncate_file() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.log");

        // Test truncation
        {
            let mut file = File::create(&file_path).await.unwrap();
            file.write_all(b"Hello, World!").await.unwrap();
            truncate_file(&file_path, 5).await.unwrap();
            let content = fs::read_to_string(&file_path).await.unwrap();
            assert_eq!(content, "Hello");
        }

        // Test extension
        {
            truncate_file(&file_path, 10).await.unwrap();
            let content = fs::read_to_string(&file_path).await.unwrap();
            assert_eq!(content, "Hello\0\0\0\0\0");
        }
    }

    #[test]
    fn test_parse_datetime() {
        let test_case = "2023-05-17T15:30:45Z";
        assert!(parse_datetime(test_case).is_ok());

        assert!(parse_datetime("invalid datetime").is_err());
    }

    #[test]
    fn test_generate_timestamp_coverage() {
        let ts = generate_timestamp();
        assert!(!ts.is_empty());
    }

    #[tokio::test]
    async fn test_is_file_writable_not_file() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().to_path_buf();
        // A directory is not a file
        assert!(!is_file_writable(&dir_path).await.unwrap());
    }

    #[tokio::test]
    async fn test_truncate_file_no_op() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_no_op.log");
        fs::write(&file_path, "12345").await.unwrap();
        // Truncate to same size should be essentially no-op or just set_len
        truncate_file(&file_path, 5).await.unwrap();
        assert_eq!(
            fs::read_to_string(&file_path).await.unwrap(),
            "12345"
        );
    }

    #[tokio::test]
    async fn test_is_file_writable_invalid_path() {
        let invalid_path = Path::new("/root/no_access_123.log");
        // This might return false or Ok(false) depending on OS, but should be handled
        let _ = is_file_writable(invalid_path).await;
        
        let empty_path = Path::new("");
        assert!(is_file_writable(empty_path).await.is_ok()); // exists() is false
    }

    #[tokio::test]
    async fn test_truncate_file_not_found() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("does_not_exist.log");
        // truncate_file uses OpenOptions with create(true) so it should actually create it
        truncate_file(&file_path, 1024).await.unwrap();
        assert!(file_path.exists());

        // Test with a path that definitely fails
        let invalid_path = Path::new("/root/no_access_truncate.log");
        assert!(truncate_file(invalid_path, 1024).await.is_err());
    }

    #[tokio::test]
    async fn test_is_directory_writable() {
        let temp_dir = tempdir().unwrap();
        assert!(is_directory_writable(temp_dir.path()).await.unwrap());

        let non_existent_dir = temp_dir.path().join("non_existent");
        assert!(!is_directory_writable(&non_existent_dir)
            .await
            .unwrap());
    }
}
