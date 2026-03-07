#![cfg(not(miri))]
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Tests for the macros functionality of RustLogs (RLG).

#[cfg(test)]
mod tests {
    use rlg::log::Log;
    use rlg::log_format::LogFormat;
    use rlg::log_level::LogLevel;

    #[test]
    fn test_fluent_api_info() {
        let log = Log::info("message")
            .component("app")
            .time("2022-01-01")
            .format(LogFormat::CLF);
        assert_eq!(log.level, LogLevel::INFO);
        assert_eq!(log.format, LogFormat::CLF);
        assert_eq!(log.time, "2022-01-01");
        assert_eq!(log.component, "app");
        assert_eq!(log.description, "message");
    }

    #[test]
    fn test_fluent_api_warn() {
        let log = Log::warn("warning message")
            .component("app")
            .time("2022-01-01");
        assert_eq!(log.level, LogLevel::WARN);
        assert_eq!(log.time, "2022-01-01");
        assert_eq!(log.component, "app");
        assert_eq!(log.description, "warning message");
    }

    #[test]
    fn test_fluent_api_error() {
        let log = Log::error("error message")
            .component("app")
            .time("2022-01-01");
        assert_eq!(log.level, LogLevel::ERROR);
        assert_eq!(log.component, "app");
        assert_eq!(log.description, "error message");
    }

    #[test]
    fn test_fluent_api_trace() {
        let log = Log::trace("trace message")
            .component("app")
            .time("2022-01-01");
        assert_eq!(log.level, LogLevel::TRACE);
        assert_eq!(log.component, "app");
        assert_eq!(log.description, "trace message");
    }

    #[test]
    fn test_fluent_api_fatal() {
        let log = Log::fatal("fatal message")
            .component("app")
            .time("2022-01-01");
        assert_eq!(log.level, LogLevel::FATAL);
        assert_eq!(log.component, "app");
        assert_eq!(log.description, "fatal message");
    }

    #[test]
    fn test_fluent_api_debug() {
        let log = Log::debug("debug message")
            .component("app")
            .time("2022-01-01");
        assert_eq!(log.level, LogLevel::DEBUG);
        assert_eq!(log.component, "app");
        assert_eq!(log.description, "debug message");
    }

    #[test]
    fn test_fluent_api_with_format() {
        let log = Log::info("message").format(LogFormat::JSON);
        assert_eq!(log.format, LogFormat::JSON);

        let log2 = Log::info("message").format(LogFormat::CLF);
        assert_eq!(log2.format, LogFormat::CLF);
    }

    #[test]
    fn test_fluent_api_with_special_characters() {
        let log = Log::info("message with \"quotes\" and \nnewlines")
            .component("app")
            .time("2022-01-01");
        assert_eq!(
            log.description,
            "message with \"quotes\" and \nnewlines"
        );
    }

    #[test]
    fn test_fluent_api_with_empty_fields() {
        let log = Log::build(LogLevel::INFO, "")
            .session_id(0)
            .time("")
            .component("");
        assert_eq!(log.session_id, 0);
        assert_eq!(log.time, "");
        assert_eq!(log.component, "");
        assert_eq!(log.description, "");
    }

    #[test]
    fn test_fluent_api_with_all_log_levels() {
        let levels = [
            (LogLevel::ALL, "ALL"),
            (LogLevel::DEBUG, "DEBUG"),
            (LogLevel::INFO, "INFO"),
            (LogLevel::WARN, "WARN"),
            (LogLevel::ERROR, "ERROR"),
            (LogLevel::FATAL, "FATAL"),
            (LogLevel::TRACE, "TRACE"),
            (LogLevel::VERBOSE, "VERBOSE"),
            (LogLevel::CRITICAL, "CRITICAL"),
        ];

        for (level, _name) in levels.iter() {
            let log = Log::build(*level, "test message")
                .component("app")
                .format(LogFormat::CLF);
            assert_eq!(log.level, *level);
        }
    }

    #[test]
    fn test_fluent_api_with_all_formats() {
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
            let log = Log::info("test message")
                .component("app")
                .format(*format);
            assert_eq!(log.format, *format);
        }
    }

    #[test]
    fn test_fluent_api_session_id_auto() {
        let log = Log::info("message");
        assert!(
            log.session_id > 0,
            "Session ID should be automatically generated"
        );
    }

    #[test]
    fn test_fluent_api_with_long_strings() {
        let long_string = "a".repeat(1000);
        let log = Log::build(LogLevel::INFO, &long_string)
            .session_id(999)
            .component("long_component");
        assert_eq!(log.session_id, 999);
        assert_eq!(log.component, "long_component");
        assert_eq!(log.description, long_string);
    }

    #[test]
    fn test_fluent_api_with_unicode() {
        let log = Log::info("Unicode: 你好, världen, 🌍")
            .component("unicode_app");
        assert_eq!(log.component, "unicode_app");
        assert_eq!(log.description, "Unicode: 你好, världen, 🌍");
    }

    #[test]
    fn test_fluent_api_with_newlines_in_message() {
        let message = "line1\nline2\nline3";
        let log =
            Log::info(message).component("app").time("2022-01-01");
        assert_eq!(log.description, message);
    }

    #[test]
    fn test_fluent_api_with_very_long_component() {
        let long_component = "a".repeat(10000);
        let log = Log::info("message")
            .component(&long_component)
            .time("2022-01-01");
        assert_eq!(log.component, long_component);
    }

    #[test]
    fn test_fluent_api_formatting_consistency() {
        let log1 = Log::info("message1")
            .component("app1")
            .time("2022-01-01")
            .format(LogFormat::CLF);
        let log2 = Log::info("message2")
            .component("app2")
            .time("2022-01-01")
            .format(LogFormat::CLF);
        let formatted1 = format!("{}", log1);
        let formatted2 = format!("{}", log2);
        assert_eq!(
            formatted1.split_whitespace().count(),
            formatted2.split_whitespace().count(),
            "Formatted logs should have the same number of fields"
        );
    }

    #[test]
    fn test_fluent_api_build_with_metadata() {
        let log = Log::build(LogLevel::INFO, "message")
            .session_id(1)
            .time("2022-01-01")
            .component("app")
            .format(LogFormat::JSON);
        let log_string = format!("{:?}", log);
        assert!(log_string.contains("id"));
        assert!(log_string.contains("2022-01-01"));
        assert!(log_string.contains("INFO"));
        assert!(log_string.contains("app"));
        assert!(log_string.contains("message"));
        assert!(log_string.contains("JSON"));
    }

    #[test]
    fn test_fluent_api_with_attributes() {
        let log =
            Log::info("test").with("key", "value").with("count", 42);
        assert_eq!(log.attributes.len(), 2);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_rlg_span_macro() {
        let res = rlg::rlg_span!("Compute Task", {
            let x = 10;
            let y = 20;
            x + y
        });
        assert_eq!(res, 30);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_rlg_time_it_macro() {
        let res = rlg::rlg_time_it!("Database Query", {
            let x = 100;
            x * 2
        });
        assert_eq!(res, 200);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_rlg_mcp_notify_macro() {
        // This is primarily for side effects (firing a log event),
        // we test that it compiles and runs without panicking.
        rlg::rlg_mcp_notify!("user_status", "logged_in");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_rlg_span_metric() {
        // We call the macro to ensure it compiles and executes.
        // We do not assert on the global span count because concurrent tests
        // may cause it to fluctuate.
        rlg::rlg_span!("Metric Test", {
            let _during = rlg::engine::ENGINE.active_spans();
        });
    }
}
