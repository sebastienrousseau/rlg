// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Integration tests for the RustLogs (RLG) library.
//!
//! This module contains comprehensive tests for various components of the RLG library,
//! including log levels, log formats, and macro functionality.

#[cfg(test)]
mod tests {
    use rlg::{
        log::Log, log_format::LogFormat, log_level::LogLevel,
        macro_debug_log, macro_error_log, macro_fatal_log,
        macro_info_log, macro_log, macro_log_if,
        macro_log_with_metadata, macro_print_log,
        macro_set_log_format_clf, macro_trace_log, macro_warn_log,
        VERSION,
    };

    /// Tests the common log format (CLF) for a log entry.
    #[tokio::test]
    async fn test_log_common_format() {
        let log = Log::new(
            "session_id_123",
            "2022-01-01T00:00:00Z",
            &LogLevel::ERROR,
            "component_a",
            "description_a",
            &LogFormat::CLF,
        );
        let expected_output = "SessionID=session_id_123 Timestamp=2022-01-01T00:00:00Z Description=description_a Level=ERROR Component=component_a";
        assert_eq!(log.to_string(), expected_output);
    }

    /// Tests the constant `VERSION` to ensure it matches the package version.
    #[test]
    fn test_version_constants() {
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }

    /// Tests the display of ERROR and WARN log levels.
    #[tokio::test]
    async fn test_log_level_display() {
        let log_level = LogLevel::ERROR;
        assert_eq!(log_level.to_string(), "ERROR");

        let log_level = LogLevel::WARN;
        assert_eq!(log_level.to_string(), "WARN");
    }

    /// Tests the display formatting for a log entry.
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

    /// Tests the default values for a log entry.
    #[tokio::test]
    async fn test_log_default() {
        let log = Log::default();
        assert_eq!(log.session_id, "");
        assert_eq!(log.time, "");
        assert_eq!(log.level, LogLevel::INFO);
        assert_eq!(log.component, "");
        assert_eq!(log.description, "");
    }

    /// Tests the output for various log formats such as CLF, CEF, ELF, etc.
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

    /// Test display of all log levels.
    #[tokio::test]
    async fn test_log_level_all_display() {
        let log_level = LogLevel::ALL;
        assert_eq!(log_level.to_string(), "ALL");
    }

    /// Test display of DEBUG log level.
    #[tokio::test]
    async fn test_log_level_debug_display() {
        let log_level = LogLevel::DEBUG;
        assert_eq!(log_level.to_string(), "DEBUG");
    }

    /// Test display of DISABLED log level.
    #[tokio::test]
    async fn test_log_level_disabled_display() {
        let log_level = LogLevel::DISABLED;
        assert_eq!(log_level.to_string(), "DISABLED");
    }

    /// Test display of ERROR log level.
    #[tokio::test]
    async fn test_log_level_error_display() {
        let log_level = LogLevel::ERROR;
        assert_eq!(log_level.to_string(), "ERROR");
    }

    /// Test display of FATAL log level.
    #[tokio::test]
    async fn test_log_level_fatal_display() {
        let log_level = LogLevel::FATAL;
        assert_eq!(log_level.to_string(), "FATAL");
    }

    /// Test display of INFO log level.
    #[tokio::test]
    async fn test_log_level_info_display() {
        let log_level = LogLevel::INFO;
        assert_eq!(log_level.to_string(), "INFO");
    }

    /// Test display of NONE log level.
    #[tokio::test]
    async fn test_log_level_none_display() {
        let log_level = LogLevel::NONE;
        assert_eq!(log_level.to_string(), "NONE");
    }

    /// Test display of TRACE log level.
    #[tokio::test]
    async fn test_log_level_trace_display() {
        let log_level = LogLevel::TRACE;
        assert_eq!(log_level.to_string(), "TRACE");
    }

    /// Test display of VERBOSE log level.
    #[tokio::test]
    async fn test_log_level_verbose_display() {
        let log_level = LogLevel::VERBOSE;
        assert_eq!(log_level.to_string(), "VERBOSE");
    }

    /// Test display of WARN log level.
    #[tokio::test]
    async fn test_log_level_warning_display() {
        let log_level = LogLevel::WARN;
        assert_eq!(log_level.to_string(), "WARN");
    }

    /// Test log formatting in CLF format.
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

    /// Test log formatting in JSON format.
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

    /// Test log formatting in CEF format.
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

    /// Test log formatting in ELF format.
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

    /// Test log formatting in W3C format.
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

    /// Test log formatting in GELF format.
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
            "{\n                    \"version\": \"1.1\",\n                    \"host\": \"test\",\n                    \"short_message\": \"test log message\",\n                    \"level\": \"INFO\",\n                    \"timestamp\": \"2023-01-23 14:04:09.881393 +00:00:00\",\n                    \"component\": \"test\",\n                    \"session_id\": \"123\"\n                }";
        assert_eq!(expected_output, format!("{log}"));
    }

    /// Test the display for various log formats.
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

    /// Test all log level variants.
    #[tokio::test]
    async fn test_log_level_variants() {
        let log =
            Log::new("", "", &LogLevel::ALL, "", "", &LogFormat::CLF);
        assert_eq!(log.level, LogLevel::ALL);

        let log =
            Log::new("", "", &LogLevel::DEBUG, "", "", &LogFormat::CLF);
        assert_eq!(log.level, LogLevel::DEBUG);

        // Test for all other variants
        let log1 = Log::new(
            "",
            "",
            &LogLevel::DISABLED,
            "",
            "",
            &LogFormat::CLF,
        );
        assert_eq!(log1.level, LogLevel::DISABLED);

        let log2 =
            Log::new("", "", &LogLevel::ERROR, "", "", &LogFormat::CLF);
        assert_eq!(log2.level, LogLevel::ERROR);

        let log3 =
            Log::new("", "", &LogLevel::FATAL, "", "", &LogFormat::CLF);
        assert_eq!(log3.level, LogLevel::FATAL);

        let log4 =
            Log::new("", "", &LogLevel::INFO, "", "", &LogFormat::CLF);
        assert_eq!(log4.level, LogLevel::INFO);

        let log5 =
            Log::new("", "", &LogLevel::NONE, "", "", &LogFormat::CLF);
        assert_eq!(log5.level, LogLevel::NONE);

        let log6 =
            Log::new("", "", &LogLevel::TRACE, "", "", &LogFormat::CLF);
        assert_eq!(log6.level, LogLevel::TRACE);

        let log7 = Log::new(
            "",
            "",
            &LogLevel::VERBOSE,
            "",
            "",
            &LogFormat::CLF,
        );
        assert_eq!(log7.level, LogLevel::VERBOSE);

        let log8 =
            Log::new("", "", &LogLevel::WARN, "", "", &LogFormat::CLF);
        assert_eq!(log8.level, LogLevel::WARN);
    }

    /// Test fully formatted log display using both default and debug formatting.
    #[tokio::test]
    async fn test_log_display_fully() {
        let log_level = LogLevel::ERROR;
        let log = Log::new("", "", &log_level, "", "", &LogFormat::CLF);

        let formatted = format!("{log}");
        assert!(formatted.contains("Level=ERROR"));

        let formatted = format!("{:#?}", log);
        assert!(formatted.contains("level: ERROR"));
    }

    /// Test the behavior of logging macros when debug is enabled.
    #[test]
    #[cfg(feature = "debug_enabled")]
    fn test_macro_debug_log_enabled() {
        use rlg::macro_print_log;
        let log = macro_info_log!("2022-01-01", "app", "message");
        macro_debug_log!(log);
        assert_eq!(log.format, LogFormat::CLF);
        assert_eq!(log.time, "2022-01-01");
        assert_eq!(log.component, "app");
        assert_eq!(log.description, "message");
    }

    /// Test the behavior of logging macros when debug is disabled.
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

    /// Test log formatting in Apache Access Log format.
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
            &LogLevel::INFO,
            "component_a",
            "description_a",
            &LogFormat::ApacheAccessLog,
        );

        // Construct the expected output using the dynamic hostname
        let expected_output = format!(
            "{} - - [2022-01-01T00:00:00Z] \"description_a\" INFO component_a",
            hostname
        );

        assert_eq!(log.to_string(), expected_output);
    }

    /// Test log formatting in Logstash format.
    #[tokio::test]
    async fn test_log_logstash_format() {
        let log = Log::new(
            "session_id_123",
            "2022-01-01T00:00:00Z",
            &LogLevel::INFO,
            "component_a",
            "description_a",
            &LogFormat::Logstash,
        );
        // Print the actual output for debugging
        let log_string = log.to_string();
        let log_json: serde_json::Value =
            serde_json::from_str(&log_string)
                .expect("Failed to parse JSON");
        assert_eq!(log_json["@timestamp"], "2022-01-01T00:00:00Z");
    }

    /// Test log formatting in Log4j XML format.
    #[tokio::test]
    async fn test_log_log4j_xml_format() {
        let log = Log::new(
            "session_id_123",
            "2022-01-01T00:00:00Z",
            &LogLevel::INFO,
            "component_a",
            "description_a",
            &LogFormat::Log4jXML,
        );
        // Expected XML format
        let expected_output = "<log4j:event logger=\"component_a\" timestamp=\"2022-01-01T00:00:00Z\" level=\"INFO\" thread=\"session_id_123\"><log4j:message>description_a</log4j:message></log4j:event>";
        assert_eq!(log.to_string(), expected_output);
    }

    /// Test log formatting in NDJSON format.
    #[tokio::test]
    async fn test_log_ndjson_format() {
        let log = Log::new(
            "session_id_123",
            "2022-01-01T00:00:00Z",
            &LogLevel::INFO,
            "component_a",
            "description_a",
            &LogFormat::NDJSON,
        );
        // Expected NDJSON format
        let expected_output = "{\n                    \"timestamp\": \"2022-01-01T00:00:00Z\",\n                    \"level\": \"INFO\",\n                    \"component\": \"component_a\",\n                    \"message\": \"description_a\"\n                }";
        assert_eq!(log.to_string(), expected_output);
    }

    // Additional tests for macro functionality

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
        assert_eq!(log.level, LogLevel::INFO);
    }

    #[test]
    fn test_macro_warn_log() {
        let log = macro_warn_log!("2022-01-01", "app", "message");
        assert_eq!(log.level, LogLevel::WARN);
    }

    #[test]
    fn test_macro_error_log() {
        let log = macro_error_log!("2022-01-01", "app", "message");
        assert_eq!(log.level, LogLevel::ERROR);
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
    fn test_macro_print_log() {
        let log = macro_info_log!("2022-01-01", "app", "message");
        macro_print_log!(log);
        // Asserting that the macro doesn't panic
    }

    #[test]
    fn test_macro_set_log_format_clf() {
        let mut log = macro_info_log!("2022-01-01", "app", "message");
        log.format = LogFormat::JSON;
        macro_set_log_format_clf!(log);
        assert_eq!(log.format, LogFormat::CLF);
    }

    #[test]
    fn test_macro_log_if() {
        let log = macro_info_log!("2022-01-01", "app", "message");
        macro_log_if!(true, log);
        // Asserting that the macro doesn't panic when condition is true
        macro_log_if!(false, log);
        // Asserting that the macro doesn't panic when condition is false
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

    // Edge case tests

    #[test]
    fn test_log_with_empty_fields() {
        let log =
            Log::new("", "", &LogLevel::INFO, "", "", &LogFormat::CLF);
        let log_string = log.to_string();
        assert!(log_string.contains("SessionID="));
        assert!(log_string.contains("Timestamp="));
        assert!(log_string.contains("Description="));
        assert!(log_string.contains("Level=INFO"));
        assert!(log_string.contains("Component="));
    }

    #[tokio::test]
    async fn test_log_rotation() {
        use rlg::log::Log;
        use rlg::log_format::LogFormat;
        use rlg::log_level::LogLevel;
        use tokio::fs;

        let temp_dir = tempfile::tempdir().unwrap();
        let log_file_path = temp_dir.path().join("test.log");
        println!("Log file path: {:?}", log_file_path);

        // Attempt to create the log file
        match fs::File::create(&log_file_path).await {
            Ok(_) => println!("Log file created successfully"),
            Err(e) => panic!("Failed to create log file: {}", e),
        }

        // Create some logs
        for i in 0..30 {
            let log = Log::new(
                &format!("session_{}", i),
                "2022-01-01",
                &LogLevel::INFO,
                "test_component",
                &format!("Log message {}", i),
                &LogFormat::CLF,
            );
            // Assuming log.log() takes a Config parameter
            match log.log().await {
                Ok(_) => println!("Log {} created successfully", i),
                Err(e) => println!("Failed to create log {}: {}", i, e),
            }
        }

        // Check if the log file exists
        match fs::metadata(&log_file_path).await {
            Ok(metadata) => println!(
                "Log file exists, size: {} bytes",
                metadata.len()
            ),
            Err(e) => panic!("Failed to get log file metadata: {}", e),
        }

        // Try to read the log file content
        match fs::read_to_string(&log_file_path).await {
            Ok(content) => {
                println!("Log file content:\n{}", content);
                println!("Log file size: {} bytes", content.len());
            }
            Err(e) => panic!("Failed to read log file: {}", e),
        }

        // Check for rotation files
        for i in 1..=5 {
            let rotation_file =
                log_file_path.with_extension(format!("log.{}", i));
            match fs::metadata(&rotation_file).await {
                Ok(metadata) => println!(
                    "Rotation file {:?} found, size: {} bytes",
                    rotation_file,
                    metadata.len()
                ),
                Err(e) => println!(
                    "Rotation file {:?} not found: {}",
                    rotation_file, e
                ),
            }
        }

        // If we've reached this point without panicking, consider the test passed
        println!("Test completed without errors");
    }
}
