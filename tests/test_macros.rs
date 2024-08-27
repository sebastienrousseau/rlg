// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(test)]

mod tests {
    use rlg::{log::Log, log_format::LogFormat, log_level::LogLevel};
    use rlg::{
        macro_debug_log, macro_error_log, macro_fatal_log,
        macro_info_log, macro_log, macro_log_if, macro_log_to_file,
        macro_log_with_metadata, macro_print_log,
        macro_set_log_format_clf, macro_trace_log, macro_warn_log,
    };
    use std::{fs::File, io::Read};

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
}
