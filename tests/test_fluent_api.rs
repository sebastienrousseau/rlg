#![cfg(not(miri))]
#![allow(missing_docs)]
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;

#[test]
fn test_fluent_api_levels() {
    let log = Log::info("info")
        .component("comp")
        .format(LogFormat::JSON)
        .with("key", "val");
    assert_eq!(log.level, LogLevel::INFO);
    assert_eq!(log.description, "info");
    assert_eq!(log.component, "comp");
    assert_eq!(log.format, LogFormat::JSON);
    assert!(log.attributes.contains_key("key"));

    assert_eq!(Log::warn("warn").level, LogLevel::WARN);
    assert_eq!(Log::error("error").level, LogLevel::ERROR);
    assert_eq!(Log::debug("debug").level, LogLevel::DEBUG);
    assert_eq!(Log::trace("trace").level, LogLevel::TRACE);
    assert_eq!(Log::fatal("fatal").level, LogLevel::FATAL);
}

#[test]
fn test_fluent_api_fire() {
    // This just ensures fire() doesn't panic and reaches the engine
    Log::info("test fire").fire();
}

#[test]
fn test_fluent_api_session_and_time() {
    let log = Log::info("test")
        .session_id("sid")
        .time("now")
        .with("int", 1)
        .with("float", 1.5)
        .with("bool", true)
        .with("str", "val");
    assert_eq!(log.session_id, "sid");
    assert_eq!(log.time, "now");
    assert_eq!(
        log.attributes.get("int").unwrap(),
        &serde_json::json!(1)
    );
    assert_eq!(
        log.attributes.get("float").unwrap(),
        &serde_json::json!(1.5)
    );
    assert_eq!(
        log.attributes.get("bool").unwrap(),
        &serde_json::json!(true)
    );
    assert_eq!(
        log.attributes.get("str").unwrap(),
        &serde_json::json!("val")
    );
}

#[test]
fn test_fluent_api_formats() {
    let log_otlp = Log::info("otlp").format(LogFormat::OTLP);
    assert_eq!(log_otlp.format, LogFormat::OTLP);

    let log_ecs = Log::info("ecs").format(LogFormat::ECS);
    assert_eq!(log_ecs.format, LogFormat::ECS);

    let log_logfmt = Log::info("logfmt").format(LogFormat::Logfmt);
    assert_eq!(log_logfmt.format, LogFormat::Logfmt);
}
