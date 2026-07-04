#![allow(missing_docs)]
#![cfg(not(miri))]
// test_lib.rs
// Integration tests for rlg

use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_new() {
        let log = Log::build(LogLevel::INFO, "desc")
            .session_id(123)
            .time("ts")
            .component("comp")
            .format(LogFormat::CLF);
        assert_eq!(log.session_id, 123);
    }

    #[test]
    fn test_log_json_format() {
        let log = Log::build(LogLevel::INFO, "desc")
            .session_id(123)
            .time("ts")
            .component("comp")
            .format(LogFormat::JSON);
        let output = format!("{}", log);
        assert!(output.contains("\"SessionID\":123"));
        assert!(output.contains("\"Attributes\":{}"));
    }

    #[test]
    fn test_log_mcp_format() {
        let log = Log::build(LogLevel::INFO, "desc")
            .session_id(1)
            .time("ts")
            .component("comp")
            .format(LogFormat::MCP);
        let output = format!("{}", log);
        assert!(output.contains("\"method\":\"notifications/log\""));
    }

    #[test]
    fn test_log_gelf_format() {
        let log = Log::build(LogLevel::INFO, "desc")
            .session_id(1)
            .time("ts")
            .component("comp")
            .format(LogFormat::GELF);
        let output = format!("{}", log);
        assert!(output.contains("\"version\":\"1.1\""));
        assert!(output.contains("\"_attributes\":{}"));
    }

    #[test]
    fn test_log_ndjson_format() {
        let log = Log::build(LogLevel::INFO, "desc")
            .session_id(1)
            .time("ts")
            .component("comp")
            .format(LogFormat::NDJSON);
        let output = format!("{}", log);
        assert!(output.contains("\"attributes\":{}"));
    }

    #[test]
    fn test_log_otlp_format() {
        let mut log = Log::build(LogLevel::INFO, "desc")
            .session_id(1)
            .time("ts")
            .component("comp")
            .format(LogFormat::OTLP);
        log.attributes
            .insert("trace_id".to_string(), serde_json::json!("t123"));
        let output = format!("{}", log);
        assert!(output.contains("\"traceId\":\"t123\""));
        assert!(output.contains("\"severityText\":\"INFO\""));
    }

    #[test]
    fn test_log_logfmt_format() {
        let mut log = Log::build(LogLevel::INFO, "desc")
            .session_id(1)
            .time("ts")
            .component("comp")
            .format(LogFormat::Logfmt);
        log.attributes
            .insert("user".to_string(), serde_json::json!("alice"));
        log.attributes.insert(
            "tags".to_string(),
            serde_json::json!(["tag1", "tag2"]),
        );
        log.attributes
            .insert("empty".to_string(), serde_json::json!(""));
        let output = format!("{}", log);
        assert!(output.contains("level=info"));
        assert!(output.contains("user=alice"));
        assert!(output.contains("empty=\"\""));
    }

    #[test]
    fn test_log_ecs_format() {
        let log = Log::build(LogLevel::INFO, "desc")
            .session_id(1)
            .time("ts")
            .component("comp")
            .format(LogFormat::ECS);
        let output = format!("{}", log);
        assert!(output.contains("\"@timestamp\":\"ts\""));
        assert!(output.contains("\"log.level\":\"info\""));
    }

    #[test]
    fn test_log_fire() {
        let log = Log::info("fire test");
        log.fire();
    }

    #[test]
    fn test_log_fire_all_levels() {
        Log::info("info").fire();
        Log::warn("warn").fire();
        Log::error("error").fire();
        Log::debug("debug").fire();
        Log::trace("trace").fire();
        Log::fatal("fatal").fire();
    }

    #[test]
    fn test_log_fire_all_formats() {
        Log::info("clf").format(LogFormat::CLF).fire();
        Log::info("json").format(LogFormat::JSON).fire();
        Log::info("cef").format(LogFormat::CEF).fire();
        Log::info("elf").format(LogFormat::ELF).fire();
        Log::info("w3c").format(LogFormat::W3C).fire();
        Log::info("gelf").format(LogFormat::GELF).fire();
        Log::info("mcp").format(LogFormat::MCP).fire();
        Log::info("otlp").format(LogFormat::OTLP).fire();
        Log::info("ecs").format(LogFormat::ECS).fire();
    }

    #[test]
    fn test_log_with_attributes_coverage() {
        let log = Log::info("attr test")
            .with("str", "val")
            .with("int", 42)
            .with("bool", true);
        assert_eq!(log.attributes.len(), 3);
    }

    #[test]
    fn test_log_methods_shortcuts() {
        assert_eq!(Log::info("desc").level, LogLevel::INFO);
        assert_eq!(Log::warn("desc").level, LogLevel::WARN);
        assert_eq!(Log::error("desc").level, LogLevel::ERROR);
        assert_eq!(Log::debug("desc").level, LogLevel::DEBUG);
        assert_eq!(Log::trace("desc").level, LogLevel::TRACE);
        assert_eq!(Log::fatal("desc").level, LogLevel::FATAL);
    }

    #[test]
    fn test_log_fire_via_build() {
        Log::build(LogLevel::INFO, "msg")
            .component("proc")
            .format(LogFormat::CLF)
            .fire();
    }

    #[tokio::test]
    async fn test_log_log_async_all_formats() {
        let formats = vec![
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
            LogFormat::MCP,
        ];
        for f in formats {
            let log = Log::build(LogLevel::INFO, "desc")
                .session_id(1)
                .time("ts")
                .component("comp")
                .format(f);
            log.log();
        }
    }
}
