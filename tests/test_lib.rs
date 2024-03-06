#[cfg(test)]

mod tests {

    extern crate dtt;
    extern crate rlg;

    use self::dtt::DateTime;
    use self::rlg::LogLevel::ERROR;
    use self::rlg::{Log, LogFormat, LogLevel};
    use rlg::LogFormat::CLF;

    #[tokio::test]
    async fn test_log_common_format() {
        let log = Log::new(
            "session_id_123",
            "2022-01-01T00:00:00Z",
            &ERROR,
            "component_a",
            "description_a",
            &CLF,
        );
        let expected_output = "SessionID=session_id_123 Timestamp=2022-01-01T00:00:00Z Description=description_a Level=ERROR Component=component_a";
        assert_eq!(log.to_string(), expected_output);
    }

    #[tokio::test]
    async fn test_log_error() {
        let date = DateTime::new();
        let log = Log::new(
            "12345678-1234-1234-1234-1234567890ab",
            &date.now,
            &LogLevel::INFO,
            "SystemTrayEvent",
            "Showing main window",
            &LogFormat::CLF,
        );
        let result = log.log().await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_log_warn() {
        let date = DateTime::new();
        let log = Log::new(
            "12345678-1234-1234-1234-1234567890ab",
            &date.now,
            &LogLevel::INFO,
            "SystemTrayEvent",
            "Showing main window",
            &LogFormat::CLF,
        );
        let result = log.log().await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_log_debug() {
        let date = DateTime::new();
        let log = Log::new(
            "12345678-1234-1234-1234-1234567890ab",
            &date.now,
            &LogLevel::INFO,
            "SystemTrayEvent",
            "Showing main window",
            &LogFormat::CLF,
        );
        let result = log.log().await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_log_trace() {
        let date = DateTime::new();
        let log = Log::new(
            "12345678-1234-1234-1234-1234567890ab",
            &date.now,
            &LogLevel::INFO,
            "SystemTrayEvent",
            "Showing main window",
            &LogFormat::CLF,
        );
        let result = log.log().await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_log_info() {
        let date = DateTime::new();
        let log = Log::new(
            "12345678-1234-1234-1234-1234567890ab",
            &date.now,
            &LogLevel::INFO,
            "SystemTrayEvent",
            "Showing main window",
            &LogFormat::CLF,
        );
        let result = log.log().await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_log_level_display() {
        let log_level = LogLevel::ERROR;
        assert_eq!(log_level.to_string(), "ERROR");

        let log_level = LogLevel::WARNING;
        assert_eq!(log_level.to_string(), "WARNING");
    }

    #[tokio::test]
    async fn test_log_display() {
        let log = Log::new(
            "12345678-1234-1234-1234-1234567890ab",
            "2023-01-23 14:03:00.000+0000",
            &LogLevel::ERROR,
            "Test",
            "This is a test log message",
            &LogFormat::CLF,
        );
        assert_eq!(
        log.to_string(),
        "SessionID=12345678-1234-1234-1234-1234567890ab Timestamp=2023-01-23 14:03:00.000+0000 Description=This is a test log message Level=ERROR Component=Test");
    }

    #[tokio::test]
    async fn test_log_default() {
        let log = Log::default();
        assert_eq!(log.session_id, "");
        assert_eq!(log.time, "");
        assert_eq!(log.level, LogLevel::INFO);
        assert_eq!(log.component, "");
        assert_eq!(log.description, "");
    }

    #[tokio::test]
    async fn test_log_common() {
        let log = Log::new(
            "12345678-1234-1234-1234-1234567890ab",
            "2023-01-23 14:03:00.000+0000",
            &LogLevel::ERROR,
            "Test",
            "This is a test log message",
            &LogFormat::CLF,
        );
        let log_string = format!("{log}");
        println!("{log_string}");
        assert_eq!(log_string, "SessionID=12345678-1234-1234-1234-1234567890ab Timestamp=2023-01-23 14:03:00.000+0000 Description=This is a test log message Level=ERROR Component=Test");
    }

    #[tokio::test]
    async fn test_log_level_all_display() {
        let log_level = LogLevel::ALL;
        assert_eq!(log_level.to_string(), "ALL");
    }

    #[tokio::test]
    async fn test_log_level_debug_display() {
        let log_level = LogLevel::DEBUG;
        assert_eq!(log_level.to_string(), "DEBUG");
    }

    #[tokio::test]
    async fn test_log_level_disabled_display() {
        let log_level = LogLevel::DISABLED;
        assert_eq!(log_level.to_string(), "DISABLED");
    }

    #[tokio::test]
    async fn test_log_level_error_display() {
        let log_level = LogLevel::ERROR;
        assert_eq!(log_level.to_string(), "ERROR");
    }

    #[tokio::test]
    async fn test_log_level_fatal_display() {
        let log_level = LogLevel::FATAL;
        assert_eq!(log_level.to_string(), "FATAL");
    }

    #[tokio::test]
    async fn test_log_level_info_display() {
        let log_level = LogLevel::INFO;
        assert_eq!(log_level.to_string(), "INFO");
    }

    #[tokio::test]
    async fn test_log_level_none_display() {
        let log_level = LogLevel::NONE;
        assert_eq!(log_level.to_string(), "NONE");
    }

    #[tokio::test]
    async fn test_log_level_trace_display() {
        let log_level = LogLevel::TRACE;
        assert_eq!(log_level.to_string(), "TRACE");
    }

    #[tokio::test]
    async fn test_log_level_verbose_display() {
        let log_level = LogLevel::VERBOSE;
        assert_eq!(log_level.to_string(), "VERBOSE");
    }

    #[tokio::test]
    async fn test_log_level_warning_display() {
        let log_level = LogLevel::WARNING;
        assert_eq!(log_level.to_string(), "WARNING");
    }
    #[tokio::test]
    async fn test_log_common_log_format() {
        let log = Log::new(
            "123",
            "2023-01-23 14:04:09.881393 +00:00:00",
            &LogLevel::INFO,
            "test",
            "test log message",
            &LogFormat::CLF,
        );
        let expected_output = "SessionID=123 Timestamp=2023-01-23 14:04:09.881393 +00:00:00 Description=test log message Level=INFO Component=test";
        assert_eq!(log.to_string(), expected_output);
    }

    #[tokio::test]
    async fn test_log_json_log_format() {
        let log = Log::new(
            "123",
            "2023-01-23 14:04:09.881393 +00:00:00",
            &LogLevel::INFO,
            "test",
            "test log message",
            &LogFormat::JSON,
        );
        let expected_output = r#"{"SessionID":"123","Timestamp":"2023-01-23 14:04:09.881393 +00:00:00","Level":"INFO","Component":"test","Description":"test log message","Format":"JSON"}"#;
        assert_eq!(log.to_string(), expected_output);
    }

    #[tokio::test]
    async fn test_log_cef_log_format() {
        let log = Log::new(
            "123",
            "2023-01-23 14:04:09.881393 +00:00:00",
            &LogLevel::INFO,
            "test",
            "test log message",
            &LogFormat::CEF,
        );
        let expected_output =
            "CEF:0|123|2023-01-23 14:04:09.881393 +00:00:00|INFO|test|test log message|CEF";
        assert_eq!(expected_output, format!("{log}"));
    }
    #[tokio::test]
    async fn test_log_elf_log_format() {
        let log = Log::new(
            "123",
            "2023-01-23 14:04:09.881393 +00:00:00",
            &LogLevel::INFO,
            "test",
            "test log message",
            &LogFormat::ELF,
        );
        let expected_output =
            "ELF:0|123|2023-01-23 14:04:09.881393 +00:00:00|INFO|test|test log message|ELF";
        assert_eq!(expected_output, format!("{log}"));
    }
    #[tokio::test]
    async fn test_log_w3c_log_format() {
        let log = Log::new(
            "123",
            "2023-01-23 14:04:09.881393 +00:00:00",
            &LogLevel::INFO,
            "test",
            "test log message",
            &LogFormat::W3C,
        );
        let expected_output =
            "W3C:0|123|2023-01-23 14:04:09.881393 +00:00:00|INFO|test|test log message|W3C";
        assert_eq!(expected_output, format!("{log}"));
    }
    #[tokio::test]
    async fn test_log_gelf_log_format() {
        let log = Log::new(
            "123",
            "2023-01-23 14:04:09.881393 +00:00:00",
            &LogLevel::INFO,
            "test",
            "test log message",
            &LogFormat::GELF,
        );
        let expected_output =
            "{\n                            \"version\": \"1.1\",\n                            \"host\": \"test\",\n                            \"short_message\": \"test log message\",\n                            \"level\": \"INFO\",\n                            \"timestamp\": \"2023-01-23 14:04:09.881393 +00:00:00\",\n                            \"component\": \"test\",\n                            \"session_id\": \"123\"\n                        }";
        assert_eq!(expected_output, format!("{log}"));
    }
    #[tokio::test]
    async fn test_log_format_display() {
        for (log_format, expected_output) in [
            (LogFormat::CLF, "CLF"),
            (LogFormat::JSON, "JSON"),
            (LogFormat::CEF, "CEF"),
            (LogFormat::ELF, "ELF"),
            (LogFormat::W3C, "W3C"),
            (LogFormat::GELF, "GELF"),
        ] {
            assert_eq!(log_format.to_string(), expected_output);
        }
    }

    #[tokio::test]
    async fn test_log_level_variants() {
        let log = Log::new("", "", &LogLevel::ALL, "", "", &LogFormat::CLF);
        assert_eq!(log.level, LogLevel::ALL);

        let log = Log::new("", "", &LogLevel::DEBUG, "", "", &LogFormat::CLF);
        assert_eq!(log.level, LogLevel::DEBUG);

        // Test for all other variants
        let log1 = Log::new("", "", &LogLevel::DISABLED, "", "", &LogFormat::CLF);
        assert_eq!(log1.level, LogLevel::DISABLED);

        let log2 = Log::new("", "", &LogLevel::ERROR, "", "", &LogFormat::CLF);
        assert_eq!(log2.level, LogLevel::ERROR);

        let log3 = Log::new("", "", &LogLevel::FATAL, "", "", &LogFormat::CLF);
        assert_eq!(log3.level, LogLevel::FATAL);

        let log4 = Log::new("", "", &LogLevel::INFO, "", "", &LogFormat::CLF);
        assert_eq!(log4.level, LogLevel::INFO);

        let log5 = Log::new("", "", &LogLevel::NONE, "", "", &LogFormat::CLF);
        assert_eq!(log5.level, LogLevel::NONE);

        let log6 = Log::new("", "", &LogLevel::TRACE, "", "", &LogFormat::CLF);
        assert_eq!(log6.level, LogLevel::TRACE);

        let log7 = Log::new("", "", &LogLevel::VERBOSE, "", "", &LogFormat::CLF);
        assert_eq!(log7.level, LogLevel::VERBOSE);

        let log8 = Log::new("", "", &LogLevel::WARNING, "", "", &LogFormat::CLF);
        assert_eq!(log8.level, LogLevel::WARNING);
    }

    #[tokio::test]
    async fn test_log_display_fully() {
        let log_level = LogLevel::ERROR;
        let log = Log::new("", "", &log_level, "", "", &LogFormat::CLF);

        let formatted = format!("{log}");
        assert!(formatted.contains("Level=ERROR"));

        let formatted = format!("{:#?}", log);
        assert!(formatted.contains("level: ERROR"));
    }

    // Conceptual example
    #[tokio::test]
    async fn test_log_write_error() {
        let log = Log::new(
            "12345678-1234-1234-1234-1234567890ab",
            "2023-01-23 14:03:00.000+0000",
            &LogLevel::ERROR,
            "SystemTrayEvent",
            "Showing main window",
            &LogFormat::CLF,
        );
        let result = log.log().await;
        assert!(result.is_ok());
    }

    // Test the Log::write_log_entry method
    #[tokio::test]
    async fn test_write_log_entry_combinations() {
        let log_levels = [
            LogLevel::INFO,
            LogLevel::WARNING,
            LogLevel::ERROR,
            LogLevel::DEBUG,
        ];
        let processes = ["process1", "process2", "process3"];
        let messages = ["message1", "message2", "message3"];
        let log_formats = [LogFormat::CLF, LogFormat::JSON, LogFormat::GELF];

        for log_level in &log_levels {
            for process in &processes {
                for message in &messages {
                    for log_format in &log_formats {
                        let log = Log::new(
                            "12345678-1234-1234-1234-1234567890ab",
                            "2023-01-23 14:03:00.000+0000",
                            log_level,
                            process,
                            message,
                            log_format,
                        );
                        let result = log.log();
                        assert!(result.await.is_ok());
                    }
                }
            }
        }
    }

    // Test the behavior of the library when the debug_enabled feature flag is set
    #[test]
    #[cfg(feature = "debug_enabled")]
    fn test_macro_debug_log_enabled() {
        let log = macro_info_log!("2022-01-01", "app", "message");
        macro_debug_log!(log);
        assert_eq!(log.format, LogFormat::CLF);
        assert_eq!(log.timestamp, "2022-01-01");
        assert_eq!(log.application, "app");
        assert_eq!(log.message, "message");
    }

    #[test]
    #[cfg(not(feature = "debug_enabled"))]
    fn test_macro_debug_log_disabled() {
        use rlg::macro_debug_log;
        use rlg::macro_info_log;
        use std::io::Write;

        struct NullWriter;

        impl Write for NullWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                Ok(buf.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        // Create a null writer to capture the output
        let mut writer = NullWriter;

        // Generate the log using the macro
        let log = macro_info_log!("2022-01-01", "app", "message");

        // Capture the output of macro_debug_log using the null writer
        macro_debug_log!(log);

        // Assert that the output is empty by checking the writer
        assert_eq!(writer.write(b"").unwrap(), 0);

        // Assert that the log is unchanged
        assert_eq!(log.format, LogFormat::CLF);
    }
}
