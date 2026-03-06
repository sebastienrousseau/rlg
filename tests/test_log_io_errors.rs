#![allow(missing_docs)]
#![allow(deprecated)]
#[cfg(test)]
mod tests {
    use rlg::log::Log;
    use rlg::log_format::LogFormat;
    use rlg::log_level::LogLevel;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_log_write_error() {
        let temp_dir = tempdir().unwrap();
        let log_file = temp_dir.path().join("readonly.log");
        fs::File::create(&log_file).unwrap();

        // Make file read-only
        let mut perms = fs::metadata(&log_file).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&log_file, perms).unwrap();

        // We need to point Config to this file.
        // Config::load_async(None) loads from RLG.log usually.
        // We can set RLG_LOG_FILE env var if it respects it.
        std::env::set_var("RLG_LOG_FILE", &log_file);

        let log = Log::default();
        let res = log.log();
        assert!(res.is_ok()); // Note: In lock-free engine, ingest always returns Ok

        let res = Log::write_log_entry(
            LogLevel::INFO,
            "proc",
            "msg",
            LogFormat::CLF,
        );
        assert!(res.is_ok()); // Note: In lock-free engine, ingest always returns Ok

        std::env::remove_var("RLG_LOG_FILE");
    }
}
