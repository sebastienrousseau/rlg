#[cfg(test)]

mod tests {
    use rlg::{
        macro_debug_log, macro_error_log, macro_fatal_log, macro_info_log, macro_log, macro_log_if,
        macro_log_to_file, macro_log_with_metadata, macro_print_log, macro_set_log_format_clf,
        macro_trace_log, macro_warn_log, Log, LogFormat, LogLevel,
    };
    use std::fs::File;
    use std::io::Read;

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

    #[test]
    fn test_macro_print_log() {
        let log = macro_info_log!("2022-01-01", "app", "message");
        macro_print_log!(log);
        // Asserting that the macro doesn't panic
    }

    #[tokio::test]
    async fn test_macro_log_to_file() {
        let log = macro_info_log!("2022-01-01", "app", "message");
        let result = macro_log_to_file!(log);
        // Asserting that the macro doesn't panic
        assert!(result.is_ok());
    }

    #[test]
    fn test_macro_set_log_format_clf() {
        let mut log = macro_info_log!("2022-01-01", "app", "message");
        log.format = LogFormat::JSON;
        macro_set_log_format_clf!(log);
        assert_eq!(log.format, LogFormat::CLF);
    }

    #[test]
    fn test_macro_debug_log() {
        let log = macro_info_log!("2022-01-01", "app", "message");
        macro_debug_log!(log);
        // Asserting that the macro doesn't panic
        assert_eq!(log.level, LogLevel::INFO);
    }

    #[test]
    fn test_macro_trace_log() {
        let log = macro_trace_log!("2022-01-01", "app", "message");
        assert_eq!(log.level, LogLevel::TRACE);
    }

    #[test]
    fn test_macro_fatal_log() {
        let log = macro_fatal_log!("2022-01-01", "app", "message");
        assert_eq!(log.level, LogLevel::FATAL);
    }

    #[test]
    fn test_macro_log_if_true() {
        let log = macro_info_log!("2022-01-01", "app", "message");
        macro_log_if!(true, log);
        // Asserting that the macro doesn't panic
    }

    #[test]
    fn test_macro_log_if_false() {
        let log = macro_info_log!("2022-01-01", "app", "message");
        macro_log_if!(false, log);
        // Asserting that the macro doesn't panic
    }

    #[test]
    fn test_macro_log_with_metadata() {
        let log_message = macro_log_with_metadata!(
            "id",
            "2022-01-01",
            &LogLevel::INFO,
            "app",
            "message",
            &LogFormat::JSON
        );

        // Assert that the log message contains the expected keys and values
        assert!(log_message.contains("\"SessionID\":\"id\""));
        assert!(log_message.contains("\"Timestamp\":\"2022-01-01\""));
        assert!(log_message.contains("\"Level\":\"INFO\""));
        assert!(log_message.contains("\"Component\":\"app\""));
        assert!(log_message.contains("\"Description\":\"message\""));
    }

    #[test]
    fn test_write_log_entry_multiple_entries() {
        // Arrange
        let log_level = LogLevel::INFO;
        let process = "test_process";
        let message = "This is a test log message";
        let log_format = LogFormat::CLF;

        // Act
        let result = Log::write_log_entry(log_level.clone(), process, message, log_format.clone());

        // Assert
        assert!(result.is_ok());

        // Check that the log file was created and contains the expected log entry
        let mut file = File::open("RLG.log").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert!(contents.contains(process), "The log file does not contain the process.");
        assert!(contents.contains(message), "The log file does not contain the message.");
        assert!(contents.contains(&log_level.to_string()), "The log file does not contain the log level.");

        // Act
        let result = Log::write_log_entry(log_level.clone(), process, message, log_format.clone());

        // Assert
        assert!(result.is_ok());

        // Check that the log file was created and contains the expected log entry
        let mut file = File::open("RLG.log").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert!(contents.contains(process), "The log file does not contain the process.");
        assert!(contents.contains(message), "The log file does not contain the message.");
        assert!(contents.contains(&log_level.to_string()), "The log file does not contain the log level.");
    }
}
