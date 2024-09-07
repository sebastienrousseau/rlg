#[cfg(test)]
mod tests {
    use rlg::utils::*;
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

        let mut file = File::create(&file_path).await.unwrap();
        file.write_all(b"Hello, World!").await.unwrap();

        truncate_file(&file_path, 5).await.unwrap();

        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "Hello");
    }

    #[test]
    fn test_parse_datetime() {
        let test_case = "2023-05-17T15:30:45Z";
        assert!(parse_datetime(test_case).is_ok());

        assert!(parse_datetime("invalid datetime").is_err());
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
