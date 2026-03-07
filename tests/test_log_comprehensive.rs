#![cfg(not(miri))]
#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    use rlg::log::Log;
    use rlg::log_format::LogFormat;
    use rlg::log_level::LogLevel;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn log_default_values_are_correct() {
        let log = Log::default();
        assert_eq!(log.session_id, 0);
        assert_eq!(log.time, "");
        assert_eq!(log.level, LogLevel::INFO);
        assert_eq!(log.component, "");
        assert_eq!(log.description, "");
        assert_eq!(log.format, LogFormat::CLF);
    }

    #[tokio::test]
    async fn log_new_creates_correct_instance() {
        let log = Log::build(LogLevel::DEBUG, "descriptionB")
            .session_id(123)
            .time("2023-10-27T10:00:00Z")
            .component("componentA")
            .format(LogFormat::JSON);
        assert_eq!(log.session_id, 123);
        assert_eq!(log.time, "2023-10-27T10:00:00Z");
        assert_eq!(log.level, LogLevel::DEBUG);
        assert_eq!(log.component, "componentA");
        assert_eq!(log.description, "descriptionB");
        assert_eq!(log.format, LogFormat::JSON);
    }

    #[tokio::test]
    async fn log_to_file_works_for_all_formats() {
        let _temp_dir = tempdir().unwrap();

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
            LogFormat::OTLP,
            LogFormat::Logfmt,
            LogFormat::ECS,
        ];

        for format in formats {
            let log = Log::build(LogLevel::INFO, "desc")
                .session_id(1)
                .time("time")
                .component("comp")
                .format(format);
            log.log();
        }

        let _ = fs::remove_file("RLG.log");
    }

    #[test]
    fn test_log_display_all_variants() {
        let log = Log::build(LogLevel::INFO, "desc")
            .session_id(1)
            .time("ts")
            .component("comp")
            .format(LogFormat::CLF);

        let variants = vec![
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
            LogFormat::OTLP,
            LogFormat::Logfmt,
            LogFormat::ECS,
        ];

        for v in variants {
            let mut l = log.clone();
            l.format = v;
            let s = format!("{}", l);
            assert!(!s.is_empty(), "Display failed for {:?}", v);
        }
    }

    #[test]
    fn write_log_entry_success() {
        let log = Log::build(LogLevel::WARN, "message")
            .component("process")
            .format(LogFormat::JSON);
        log.fire();
        let _ = fs::remove_file("RLG.log");
    }

    #[test]
    fn log_display_gelf_format() {
        let log = Log::build(LogLevel::INFO, "desc")
            .session_id(1)
            .time("ts")
            .component("comp")
            .format(LogFormat::GELF);
        let output = format!("{}", log);
        assert!(output.contains("\"version\":\"1.1\""));
        assert!(output.contains("\"host\":\"comp\""));
    }

    #[test]
    fn log_display_logstash_format() {
        let log = Log::build(LogLevel::INFO, "desc")
            .session_id(1)
            .time("ts")
            .component("comp")
            .format(LogFormat::Logstash);
        let output = format!("{}", log);
        assert!(output.contains("\"@timestamp\":\"ts\""));
        assert!(output.contains("\"message\":\"desc\""));
    }

    #[test]
    fn log_display_log4jxml_format() {
        let log = Log::build(LogLevel::INFO, "desc")
            .session_id(1)
            .time("ts")
            .component("comp")
            .format(LogFormat::Log4jXML);
        let output = format!("{}", log);
        assert!(output.contains("<log4j:event"));
        assert!(output.contains("logger=\"comp\""));
    }

    #[test]
    fn log_display_apache_access_log_format() {
        let log = Log::build(LogLevel::INFO, "desc")
            .session_id(1)
            .time("ts")
            .component("comp")
            .format(LogFormat::ApacheAccessLog);
        let output = format!("{}", log);
        // Note: ApacheAccessLog uses hostname::get() which might be different on different machines
        assert!(output.contains("- - [ts] \"desc\" INFO comp"));
    }

    #[test]
    fn test_log_with_diverse_attributes() {
        let log = Log::info("test")
            .with("int", 123)
            .with("bool", true)
            .with("float", 1.23)
            .format(LogFormat::Logfmt);
        let output = format!("{}", log);
        assert!(output.contains("int=123"));
        assert!(output.contains("bool=true"));
        assert!(output.contains("float=1.23"));
    }
}
