// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Tests for the macros functionality of RustLogs (RLG).

#[cfg(test)]
mod tests {
    use dtt::datetime::DateTime;
    use rlg::{log_format::LogFormat, log_level::LogLevel};
    #[allow(unused_imports)]
    use rlg::{macro_debug_log, macro_error_log, macro_fatal_log};
    use rlg::{
        macro_info_log, macro_log, macro_log_if,
        macro_log_with_metadata, macro_print_log,
        macro_set_log_format_clf, macro_trace_log, macro_warn_log,
    };

    #[allow(unused_imports)]
    use std::io::{self, Write};

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
        assert_eq!(log.time, "2022-01-01");
        assert_eq!(log.level, LogLevel::INFO);
        assert_eq!(log.component, "app");
        assert_eq!(log.description, "message");
        assert_eq!(log.format, LogFormat::JSON);
    }

    #[test]
    fn test_macro_info_log() {
        let log = macro_info_log!("2022-01-01", "app", "message");
        assert_eq!(log.level, LogLevel::INFO);
        assert_eq!(log.format, LogFormat::CLF);
        assert_eq!(log.time, "2022-01-01");
        assert_eq!(log.component, "app");
        assert_eq!(log.description, "message");
    }

    #[test]
    fn test_macro_warn_log() {
        let log =
            macro_warn_log!("2022-01-01", "app", "warning message");
        assert_eq!(log.level, LogLevel::WARN);
        assert_eq!(log.format, LogFormat::CLF);
        assert_eq!(log.time, "2022-01-01");
        assert_eq!(log.component, "app");
        assert_eq!(log.description, "warning message");
    }

    #[test]
    fn test_macro_error_log() {
        let log =
            macro_error_log!("2022-01-01", "app", "error message");
        assert_eq!(log.level, LogLevel::ERROR);
        assert_eq!(log.format, LogFormat::CLF);
        assert_eq!(log.time, "2022-01-01");
        assert_eq!(log.component, "app");
        assert_eq!(log.description, "error message");
    }

    #[test]
    fn test_macro_trace_log() {
        let log =
            macro_trace_log!("2022-01-01", "app", "trace message");
        assert_eq!(log.level, LogLevel::TRACE);
        assert_eq!(log.format, LogFormat::CLF);
        assert_eq!(log.time, "2022-01-01");
        assert_eq!(log.component, "app");
        assert_eq!(log.description, "trace message");
    }

    #[test]
    fn test_macro_fatal_log() {
        let log =
            macro_fatal_log!("2022-01-01", "app", "fatal message");
        assert_eq!(log.level, LogLevel::FATAL);
        assert_eq!(log.format, LogFormat::CLF);
        assert_eq!(log.time, "2022-01-01");
        assert_eq!(log.component, "app");
        assert_eq!(log.description, "fatal message");
    }

    #[test]
    fn test_macro_set_log_format_clf() {
        let mut log = macro_info_log!("2022-01-01", "app", "message");
        log.format = LogFormat::JSON;
        macro_set_log_format_clf!(log);
        assert_eq!(log.format, LogFormat::CLF);
    }

    #[test]
    fn test_macro_log_if_false() {
        let log =
            macro_info_log!("2022-01-01", "app", "should not appear");
        let mut output = Vec::new();
        {
            macro_log_if!(false, log);
            output.flush().unwrap();
        }
        let printed = String::from_utf8(output).unwrap();
        assert!(printed.is_empty());
    }

    #[test]
    fn test_macro_log_with_metadata() {
        let log_message = macro_log_with_metadata!(
            "id",
            "2022-01-01",
            &LogLevel::INFO,
            "app",
            "message with metadata",
            &LogFormat::JSON
        );

        assert!(log_message.contains("\"SessionID\":\"id\""));
        assert!(log_message.contains("\"Timestamp\":\"2022-01-01\""));
        assert!(log_message.contains("\"Level\":\"INFO\""));
        assert!(log_message.contains("\"Component\":\"app\""));
        assert!(log_message
            .contains("\"Description\":\"message with metadata\""));
        assert!(log_message.contains("\"Format\":\"JSON\""));
    }

    #[test]
    fn test_macro_info_log_with_special_characters() {
        let log = macro_info_log!(
            "2022-01-01",
            "app",
            "message with \"quotes\" and \nnewlines"
        );
        assert_eq!(
            log.description,
            "message with \"quotes\" and \nnewlines"
        );
    }

    #[test]
    fn test_macro_log_with_empty_fields() {
        let log = macro_log!(
            "",
            "",
            &LogLevel::INFO,
            "",
            "",
            &LogFormat::CLF
        );
        assert_eq!(log.session_id, "");
        assert_eq!(log.time, "");
        assert_eq!(log.component, "");
        assert_eq!(log.description, "");
    }

    #[test]
    #[cfg(not(feature = "debug_enabled"))]
    fn test_macro_debug_log_disabled() {
        let mut output = Vec::new();
        {
            macro_debug_log!(log);
            output.flush().unwrap();
        }
        let printed = String::from_utf8(output).unwrap();
        assert!(printed.is_empty());
    }

    #[test]
    fn test_macro_log_with_long_strings() {
        let long_string = "a".repeat(1000);
        let log = macro_log!(
            "long_id",
            "2022-01-01",
            &LogLevel::INFO,
            "long_component",
            &long_string,
            &LogFormat::CLF
        );
        assert_eq!(log.session_id, "long_id");
        assert_eq!(log.component, "long_component");
        assert_eq!(log.description, long_string);
    }

    #[test]
    fn test_macro_log_with_unicode() {
        let log = macro_info_log!(
            "2022-01-01",
            "unicode_app",
            "Unicode: 你好, världen, 🌍"
        );
        assert_eq!(log.component, "unicode_app");
        assert_eq!(log.description, "Unicode: 你好, världen, 🌍");
    }

    #[test]
    fn test_macro_log_with_all_log_levels() {
        let levels = [
            LogLevel::ALL,
            LogLevel::DEBUG,
            LogLevel::INFO,
            LogLevel::WARN,
            LogLevel::ERROR,
            LogLevel::FATAL,
            LogLevel::TRACE,
            LogLevel::VERBOSE,
            LogLevel::CRITICAL,
        ];

        for level in levels.iter() {
            let log = macro_log!(
                "id",
                "2022-01-01",
                level,
                "app",
                "test message",
                &LogFormat::CLF
            );
            assert_eq!(&log.level, level);
        }
    }

    #[test]
    fn test_macro_log_with_all_formats() {
        let formats = [
            LogFormat::CLF,
            LogFormat::JSON,
            LogFormat::CEF,
            LogFormat::ELF,
            LogFormat::W3C,
            LogFormat::GELF,
            LogFormat::ApacheAccessLog,
            LogFormat::Logstash,
            LogFormat::Log4jXML,
            LogFormat::NDJSON,
        ];

        for format in formats.iter() {
            let log = macro_log!(
                "id",
                "2022-01-01",
                &LogLevel::INFO,
                "app",
                "test message",
                format
            );
            assert_eq!(&log.format, format);
        }
    }

    #[test]
    fn test_macro_log_default_session_id() {
        let log = macro_info_log!("2022-01-01", "app", "message");
        assert!(
            !log.session_id.is_empty(),
            "Session ID should be automatically generated"
        );
    }

    #[test]
    fn test_macro_log_current_timestamp() {
        let now = DateTime::new();
        let current_time = now.to_string();

        let log = macro_info_log!(&current_time, "app", "message");

        // Parse the log time
        let log_time = DateTime::parse(&log.time)
            .expect("Failed to parse log time");

        // Check if the log time is within 1 second of the current time
        let time_diff =
            now.unix_timestamp() - log_time.unix_timestamp();
        assert!(time_diff.abs() <= 1,
        "Log timestamp should be close to current time. Log time: {}, Current time: {}",
        log.time, current_time);

        // Additional check to ensure the date part matches
        assert_eq!(log_time.format("%Y-%m-%d").unwrap(), now.format("%Y-%m-%d").unwrap(),
        "Log date should match current date. Log date: {}, Current date: {}",
        log_time.format("%Y-%m-%d").unwrap(), now.format("%Y-%m-%d").unwrap());

        // Ensure the log time matches the provided time
        assert_eq!(log.time, current_time,
        "Log time should match the provided time. Log time: {}, Provided time: {}",
        log.time, current_time);
    }

    #[test]
    fn test_macro_log_with_very_long_component() {
        let long_component = "a".repeat(10000);
        let log =
            macro_info_log!("2022-01-01", &long_component, "message");
        assert_eq!(log.component, long_component);
    }

    #[test]
    fn test_macro_log_with_newlines_in_message() {
        let message = "line1\nline2\nline3";
        let log = macro_info_log!("2022-01-01", "app", message);
        assert_eq!(log.description, message);
    }

    #[test]
    fn test_macro_set_log_format_clf_idempotent() {
        let mut log = macro_info_log!("2022-01-01", "app", "message");
        macro_set_log_format_clf!(log);
        let original_format = log.format;
        macro_set_log_format_clf!(log);
        assert_eq!(log.format, original_format, "Calling macro_set_log_format_clf twice should not change the format");
    }

    #[test]
    fn test_macro_log_with_custom_metadata() {
        let log = macro_log!(
            "id",
            "2022-01-01",
            &LogLevel::INFO,
            "app",
            "message",
            &LogFormat::JSON
        );
        let log_string = format!("{:?}", log);
        assert!(log_string.contains("id"));
        assert!(log_string.contains("2022-01-01"));
        assert!(log_string.contains("INFO"));
        assert!(log_string.contains("app"));
        assert!(log_string.contains("message"));
        assert!(log_string.contains("JSON"));
    }

    #[test]
    fn test_macro_log_with_all_log_components() {
        let log = macro_log!(
            "session123",
            "2022-01-01T12:00:00Z",
            &LogLevel::INFO,
            "TestComponent",
            "Test message",
            &LogFormat::JSON
        );
        let log_string = format!("{:?}", log);
        assert!(log_string.contains("session123"));
        assert!(log_string.contains("2022-01-01T12:00:00Z"));
        assert!(log_string.contains("INFO"));
        assert!(log_string.contains("TestComponent"));
        assert!(log_string.contains("Test message"));
        assert!(log_string.contains("JSON"));
    }

    #[test]
    fn test_macro_log_formatting_consistency() {
        let log1 = macro_info_log!("2022-01-01", "app1", "message1");
        let log2 = macro_info_log!("2022-01-01", "app2", "message2");
        let formatted1 = format!("{}", log1);
        let formatted2 = format!("{}", log2);
        assert_eq!(
            formatted1.split_whitespace().count(),
            formatted2.split_whitespace().count(),
            "Formatted logs should have the same number of fields"
        );
    }

    #[test]
    fn test_macro_log_with_dtt_timestamp() {
        let now = DateTime::new();
        let formatted_now = now.to_string();
        let log = macro_info_log!(&formatted_now, "app", "message");
        assert_eq!(log.time, formatted_now);
    }

    #[test]
    fn test_macro_log_with_dtt_custom_format() {
        let now = DateTime::new();
        let custom_format = "%Y-%m-%d %H:%M:%S";
        let formatted_now = now.format(custom_format).unwrap();
        let log = macro_info_log!(&formatted_now, "app", "message");
        assert_eq!(log.time, formatted_now);
    }
}
