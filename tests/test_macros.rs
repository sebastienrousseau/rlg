#[cfg(test)]

mod tests {
    use rlg::macro_error_log;
    use rlg::{macro_info_log, macro_log, macro_warn_log};
    use rlg::{LogFormat, LogLevel};

    #[test]
    fn test_macro_log() {
        let log = macro_log!(
            "id",
            "2022-01-01",
            &LogLevel::INFO,
            "app",
            "message",
            &LogFormat::JSON
        );
        assert_eq!(log.session_id, "id");
        assert_eq!(log.format, LogFormat::JSON);
    }

    #[test]
    fn test_macro_info_log() {
        let log = macro_info_log!("2022-01-01", "app", "message");
        assert_eq!(log.session_id.len(), 9);
        assert_eq!(log.format, LogFormat::CLF);
    }

    #[test]
    fn test_macro_warn_log() {
        let log = macro_warn_log!("2022-01-01", "app", "message");
        assert_eq!(log.level, LogLevel::WARNING);
    }

    #[test]
    fn test_macro_error_log() {
        let log = macro_error_log!("2022-01-01", "app", "message");
        assert_eq!(log.level, LogLevel::ERROR);
    }
}
