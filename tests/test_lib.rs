// test_lib.rs
// Integration tests for rlg
#![allow(missing_docs)]
#![allow(deprecated)]

use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_new() {
        let log = Log::new("123", "ts", &LogLevel::INFO, "comp", "desc", &LogFormat::CLF);
        assert_eq!(log.session_id, "123");
    }

    #[test]
    fn test_log_json_format() {
        let log = Log::new("123", "ts", &LogLevel::INFO, "comp", "desc", &LogFormat::JSON);
        let output = format!("{}", log);
        assert!(output.contains("\"SessionID\":\"123\""));
        assert!(output.contains("\"Attributes\":{}"));
    }

    #[test]
    fn test_log_mcp_format() {
        let log = Log::new("sid", "ts", &LogLevel::INFO, "comp", "desc", &LogFormat::MCP);
        let output = format!("{}", log);
        assert!(output.contains("\"method\":\"notifications/log\""));
    }

    #[test]
    fn test_log_gelf_format() {
        let log = Log::new("sid", "ts", &LogLevel::INFO, "comp", "desc", &LogFormat::GELF);
        let output = format!("{}", log);
        assert!(output.contains("\"version\":\"1.1\""));
        assert!(output.contains("\"_attributes\":{}"));
    }

    #[test]
    fn test_log_ndjson_format() {
        let log = Log::new("sid", "ts", &LogLevel::INFO, "comp", "desc", &LogFormat::NDJSON);
        let output = format!("{}", log);
        assert!(output.contains("\"attributes\":{}"));
    }

    #[test]
    fn test_log_otlp_format() {
        let mut log = Log::new("sid", "ts", &LogLevel::INFO, "comp", "desc", &LogFormat::OTLP);
        log.attributes.insert("trace_id".to_string(), serde_json::json!("t123"));
        let output = format!("{}", log);
        assert!(output.contains("\"traceId\":\"t123\""));
        assert!(output.contains("\"severityText\":\"INFO\""));
    }

    #[test]
    fn test_log_logfmt_format() {
        let mut log = Log::new("sid", "ts", &LogLevel::INFO, "comp", "desc", &LogFormat::Logfmt);
        log.attributes.insert("user".to_string(), serde_json::json!("alice"));
        log.attributes.insert("tags".to_string(), serde_json::json!(["tag1", "tag2"]));
        log.attributes.insert("empty".to_string(), serde_json::json!(""));
        let output = format!("{}", log);
        assert!(output.contains("level=info"));
        assert!(output.contains("user=alice"));
        assert!(output.contains("empty=\"\""));
    }

    #[test]
    fn test_log_ecs_format() {
        let log = Log::new("sid", "ts", &LogLevel::INFO, "comp", "desc", &LogFormat::ECS);
        let output = format!("{}", log);
        assert!(output.contains("\"@timestamp\":\"ts\""));
        assert!(output.contains("\"log.level\":\"info\""));
    }

    #[tokio::test]
    async fn test_log_log_async_all_formats() {
        let formats = vec![
            LogFormat::CLF, LogFormat::JSON, LogFormat::CEF,
            LogFormat::ELF, LogFormat::W3C, LogFormat::GELF,
            LogFormat::ApacheAccessLog, LogFormat::Logstash,
            LogFormat::Log4jXML, LogFormat::NDJSON, LogFormat::MCP,
        ];
        for f in formats {
            let log = Log::new("sid", "ts", &LogLevel::INFO, "comp", "desc", &f);
            assert!(log.log().is_ok());
        }
    }
}
