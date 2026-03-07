#![cfg(not(miri))]
#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    use rlg::log::Log;
    use rlg::log_format::LogFormat;
    use rlg::log_level::LogLevel;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    #[allow(unsafe_code)]
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
        // SAFETY: Test-only; no other threads depend on this env var.
        unsafe { std::env::set_var("RLG_LOG_FILE", &log_file) };

        let log = Log::default();
        log.log();

        let log2 = Log::build(LogLevel::INFO, "msg")
            .component("proc")
            .format(LogFormat::CLF);
        log2.log();

        // SAFETY: Test-only cleanup.
        unsafe { std::env::remove_var("RLG_LOG_FILE") };
    }
}
