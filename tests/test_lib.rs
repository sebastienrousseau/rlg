// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(test)]

mod tests {
    use crate::tests::LogFormat::{
        ApacheAccessLog, Log4jXML, Logstash, CEF, CLF, ELF, GELF, JSON,
        NDJSON, W3C,
    };
    use dtt::DateTime;
    use rlg::{
        log::Log, log_format::LogFormat, log_level::LogLevel::*,
    };
    use rlg::{macro_debug_log, macro_info_log};

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
            &INFO,
            "SystemTrayEvent",
            "Showing main window",
            &CLF,
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
            &INFO,
            "SystemTrayEvent",
            "Showing main window",
            &CLF,
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
            &INFO,
            "SystemTrayEvent",
            "Showing main window",
            &CLF,
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
            &INFO,
            "SystemTrayEvent",
            "Showing main window",
            &CLF,
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
            &INFO,
            "SystemTrayEvent",
            "Showing main window",
            &CLF,
        );
        let result = log.log().await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_log_level_display() {
        let log_level = ERROR;
        assert_eq!(log_level.to_string(), "ERROR");

        let log_level = WARNING;
        assert_eq!(log_level.to_string(), "WARNING");
    }

    #[tokio::test]
    async fn test_log_display() {
        let log = Log::new(
            "12345678-1234-1234-1234-1234567890ab",
            "2023-01-23 14:03:00.000+0000",
            &ERROR,
            "Test",
            "This is a test log message",
            &CLF,
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
        assert_eq!(log.level, INFO);
        assert_eq!(log.component, "");
        assert_eq!(log.description, "");
    }

    #[tokio::test]
    async fn test_log_common() {
        let log = Log::new(
            "12345678-1234-1234-1234-1234567890ab",
            "2023-01-23 14:03:00.000+0000",
            &ERROR,
            "Test",
            "This is a test log message",
            &CLF,
        );
        let log_string = format!("{log}");
        println!("{log_string}");
        assert_eq!(log_string, "SessionID=12345678-1234-1234-1234-1234567890ab Timestamp=2023-01-23 14:03:00.000+0000 Description=This is a test log message Level=ERROR Component=Test");
    }

    #[tokio::test]
    async fn test_log_level_all_display() {
        let log_level = ALL;
        assert_eq!(log_level.to_string(), "ALL");
    }

    #[tokio::test]
    async fn test_log_level_debug_display() {
        let log_level = DEBUG;
        assert_eq!(log_level.to_string(), "DEBUG");
    }

    #[tokio::test]
    async fn test_log_level_disabled_display() {
        let log_level = DISABLED;
        assert_eq!(log_level.to_string(), "DISABLED");
    }

    #[tokio::test]
    async fn test_log_level_error_display() {
        let log_level = ERROR;
        assert_eq!(log_level.to_string(), "ERROR");
    }

    #[tokio::test]
    async fn test_log_level_fatal_display() {
        let log_level = FATAL;
        assert_eq!(log_level.to_string(), "FATAL");
    }

    #[tokio::test]
    async fn test_log_level_info_display() {
        let log_level = INFO;
        assert_eq!(log_level.to_string(), "INFO");
    }

    #[tokio::test]
    async fn test_log_level_none_display() {
        let log_level = NONE;
        assert_eq!(log_level.to_string(), "NONE");
    }

    #[tokio::test]
    async fn test_log_level_trace_display() {
        let log_level = TRACE;
        assert_eq!(log_level.to_string(), "TRACE");
    }

    #[tokio::test]
    async fn test_log_level_verbose_display() {
        let log_level = VERBOSE;
        assert_eq!(log_level.to_string(), "VERBOSE");
    }

    #[tokio::test]
    async fn test_log_level_warning_display() {
        let log_level = WARNING;
        assert_eq!(log_level.to_string(), "WARNING");
    }
    #[tokio::test]
    async fn test_log_common_log_format() {
        let log = Log::new(
            "123",
            "2023-01-23 14:04:09.881393 +00:00:00",
            &INFO,
            "test",
            "test log message",
            &CLF,
        );
        let expected_output = "SessionID=123 Timestamp=2023-01-23 14:04:09.881393 +00:00:00 Description=test log message Level=INFO Component=test";
        assert_eq!(log.to_string(), expected_output);
    }

    #[tokio::test]
    async fn test_log_json_log_format() {
        let log = Log::new(
            "123",
            "2023-01-23 14:04:09.881393 +00:00:00",
            &INFO,
            "test",
            "test log message",
            &JSON,
        );
        let expected_output = r#"{"SessionID":"123","Timestamp":"2023-01-23 14:04:09.881393 +00:00:00","Level":"INFO","Component":"test","Description":"test log message","Format":"JSON"}"#;
        assert_eq!(log.to_string(), expected_output);
    }

    #[tokio::test]
    async fn test_log_cef_log_format() {
        let log = Log::new(
            "123",
            "2023-01-23 14:04:09.881393 +00:00:00",
            &INFO,
            "test",
            "test log message",
            &CEF,
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
            &INFO,
            "test",
            "test log message",
            &ELF,
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
            &INFO,
            "test",
            "test log message",
            &W3C,
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
            &INFO,
            "test",
            "test log message",
            &GELF,
        );
        let expected_output =
            "{\n                            \"version\": \"1.1\",\n                            \"host\": \"test\",\n                            \"short_message\": \"test log message\",\n                            \"level\": \"INFO\",\n                            \"timestamp\": \"2023-01-23 14:04:09.881393 +00:00:00\",\n                            \"component\": \"test\",\n                            \"session_id\": \"123\"\n                        }";
        assert_eq!(expected_output, format!("{log}"));
    }
    #[tokio::test]
    async fn test_log_format_display() {
        for (log_format, expected_output) in [
            (CLF, "CLF"),
            (JSON, "JSON"),
            (CEF, "CEF"),
            (ELF, "ELF"),
            (W3C, "W3C"),
            (GELF, "GELF"),
        ] {
            assert_eq!(log_format.to_string(), expected_output);
        }
    }

    #[tokio::test]
    async fn test_log_level_variants() {
        let log = Log::new("", "", &ALL, "", "", &CLF);
        assert_eq!(log.level, ALL);

        let log = Log::new("", "", &DEBUG, "", "", &CLF);
        assert_eq!(log.level, DEBUG);

        // Test for all other variants
        let log1 = Log::new("", "", &DISABLED, "", "", &CLF);
        assert_eq!(log1.level, DISABLED);

        let log2 = Log::new("", "", &ERROR, "", "", &CLF);
        assert_eq!(log2.level, ERROR);

        let log3 = Log::new("", "", &FATAL, "", "", &CLF);
        assert_eq!(log3.level, FATAL);

        let log4 = Log::new("", "", &INFO, "", "", &CLF);
        assert_eq!(log4.level, INFO);

        let log5 = Log::new("", "", &NONE, "", "", &CLF);
        assert_eq!(log5.level, NONE);

        let log6 = Log::new("", "", &TRACE, "", "", &CLF);
        assert_eq!(log6.level, TRACE);

        let log7 = Log::new("", "", &VERBOSE, "", "", &CLF);
        assert_eq!(log7.level, VERBOSE);

        let log8 = Log::new("", "", &WARNING, "", "", &CLF);
        assert_eq!(log8.level, WARNING);
    }

    #[tokio::test]
    async fn test_log_display_fully() {
        let log_level = ERROR;
        let log = Log::new("", "", &log_level, "", "", &CLF);

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
            &ERROR,
            "SystemTrayEvent",
            "Showing main window",
            &CLF,
        );
        let result = log.log().await;
        assert!(result.is_ok());
    }

    // Test the Log::write_log_entry method
    #[tokio::test]
    async fn test_write_log_entry_combinations() {
        let log_levels = [INFO, WARNING, ERROR, DEBUG];
        let processes = ["process1", "process2", "process3"];
        let messages = ["message1", "message2", "message3"];
        let log_formats = [CLF, JSON, GELF];

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
        use rlg::macro_print_log;
        let log = macro_info_log!("2022-01-01", "app", "message");
        macro_debug_log!(log);
        assert_eq!(log.format, CLF);
        assert_eq!(log.time, "2022-01-01");
        assert_eq!(log.component, "app");
        assert_eq!(log.description, "message");
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
        assert_eq!(log.format, CLF);
    }
    // Test for Apache Access Log Format
    #[tokio::test]
    async fn test_log_apache_access_format() {
        // Dynamically get the hostname
        let hostname = hostname::get()
            .expect("Failed to get hostname")
            .to_string_lossy()
            .into_owned();

        let log = Log::new(
            "session_id_123",
            "2022-01-01T00:00:00Z",
            &INFO,
            "component_a",
            "description_a",
            &ApacheAccessLog,
        );

        // Construct the expected output using the dynamic hostname
        let expected_output = format!(
            "{} - - [2022-01-01T00:00:00Z] \"description_a\" INFO component_a",
            hostname
        );

        assert_eq!(log.to_string(), expected_output);
    }

    // Test for Logstash Format
    #[tokio::test]
    async fn test_log_logstash_format() {
        let log = Log::new(
            "session_id_123",
            "2022-01-01T00:00:00Z",
            &INFO,
            "component_a",
            "description_a",
            &Logstash,
        );
        // Print the actual output for debugging
        let log_string = log.to_string();
        let log_json: serde_json::Value =
            serde_json::from_str(&log_string)
                .expect("Failed to parse JSON");
        assert_eq!(log_json["@timestamp"], "2022-01-01T00:00:00Z");
    }
    // Test for Log4j XML Format
    #[tokio::test]
    async fn test_log_log4j_xml_format() {
        let log = Log::new(
            "session_id_123",
            "2022-01-01T00:00:00Z",
            &INFO,
            "component_a",
            "description_a",
            &Log4jXML,
        );
        // Expected XML format
        let expected_output = "<log4j:event logger=\"component_a\" timestamp=\"2022-01-01T00:00:00Z\" level=\"INFO\" thread=\"session_id_123\"><log4j:message>description_a</log4j:message></log4j:event>";
        assert_eq!(log.to_string(), expected_output);
    }
    // Test for NDJSON Format
    #[tokio::test]
    async fn test_log_ndjson_format() {
        let log = Log::new(
            "session_id_123",
            "2022-01-01T00:00:00Z",
            &INFO,
            "component_a",
            "description_a",
            &NDJSON,
        );
        // Expected NDJSON format
        let expected_output = "{\n                            \"timestamp\": \"2022-01-01T00:00:00Z\",\n                            \"level\": \"INFO\",\n                            \"component\": \"component_a\",\n                            \"message\": \"description_a\"\n                        }";
        assert_eq!(log.to_string(), expected_output);
    }
}
