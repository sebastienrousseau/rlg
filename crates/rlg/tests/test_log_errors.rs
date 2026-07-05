#![allow(missing_docs)]
#![cfg(not(miri))]
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
        log.log();

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

        // Use fluent API instead of deprecated write_log_entry
        let log = Log::build(LogLevel::INFO, "msg")
            .component("proc")
            .format(LogFormat::CLF);
        log.log();

        // Cleanup
        let _ = std::fs::remove_dir("RLG.log");
    }
}
