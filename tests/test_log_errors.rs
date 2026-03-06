#![allow(missing_docs)]
#![allow(deprecated)]
#[cfg(test)]
mod tests {
    use rlg::log::Log;
    use rlg::log_format::LogFormat;
    use rlg::log_level::LogLevel;
    use std::sync::Mutex;

    // Use a mutex to prevent race conditions when manipulating the default "RLG.log" file
    static RLG_LOG_MUTEX: Mutex<()> = Mutex::new(());

    #[tokio::test]
    async fn test_log_file_open_error() {
        let _guard = RLG_LOG_MUTEX.lock().unwrap();

        // Remove RLG.log if it exists
        let _ = std::fs::remove_file("RLG.log");
        let _ = std::fs::remove_dir_all("RLG.log");

        // Create a directory named "RLG.log" so that it cannot be opened as a file for appending
        let _ = std::fs::create_dir("RLG.log");

        let log = Log::default();
        let res = log.log();

        // In the lock-free engine, ingestion succeeds even if flusher might fail later
        assert!(res.is_ok());

        // Cleanup
        let _ = std::fs::remove_dir("RLG.log");
    }

    #[tokio::test]
    async fn test_write_log_entry_open_error() {
        let _guard = RLG_LOG_MUTEX.lock().unwrap();

        // Remove RLG.log if it exists
        let _ = std::fs::remove_file("RLG.log");
        let _ = std::fs::remove_dir_all("RLG.log");

        // Create a directory named "RLG.log" so that it cannot be opened as a file
        let _ = std::fs::create_dir("RLG.log");

        let res = Log::write_log_entry(
            LogLevel::INFO,
            "proc",
            "msg",
            LogFormat::CLF,
        );

        // It should succeed ingest
        assert!(res.is_ok());

        // Cleanup
        let _ = std::fs::remove_dir("RLG.log");
    }
}
