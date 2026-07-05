// Coverage-driving integration tests: walk every public surface that the
// rest of the suite leaves under 100 % so `cargo llvm-cov` reports a clean
// line/region total for `src/`.
//
// Each block is named after the source file it targets.
//
// Skipped under MIRI because the integration paths drive FFI
// (`hostname::get()` via `CACHED_HOSTNAME`, `syslog(3)` via the macOS
// sink, the `notify` watcher in config tests), which MIRI flags as
// foreign-memory leaks even though stable runs are clean.

#![cfg(not(miri))]
#![allow(missing_docs)]

use rlg::error::{RlgError, RlgResult};
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;

// ---------------------------------------------------------------------------
// log.rs — exercise every Display variant via the fluent API
// ---------------------------------------------------------------------------

fn sample_log(format: LogFormat) -> Log {
    Log::build(LogLevel::INFO, "hello \"world\"\n\r\tcontrol\u{0008}")
        .component("svc")
        .session_id(7)
        .time("2024-01-01T00:00:00Z")
        .with("k", "v")
        .with("trace_id", "abc")
        .with("span_id", "def")
        .with("num", 42_i64)
        .format(format)
}

#[test]
fn log_display_clf() {
    let s = format!("{}", sample_log(LogFormat::CLF));
    assert!(s.contains("SessionID=7"));
    assert!(s.contains("Level=INFO"));
}

#[test]
fn log_display_cef() {
    let s = format!("{}", sample_log(LogFormat::CEF));
    assert!(s.starts_with("CEF:0|7|"));
    assert!(s.ends_with("|CEF"));
}

#[test]
fn log_display_elf() {
    let s = format!("{}", sample_log(LogFormat::ELF));
    assert!(s.starts_with("ELF:0|"));
    assert!(s.ends_with("|ELF"));
}

#[test]
fn log_display_w3c() {
    let s = format!("{}", sample_log(LogFormat::W3C));
    assert!(s.starts_with("W3C:0|"));
}

#[test]
fn log_display_apache() {
    let s = format!("{}", sample_log(LogFormat::ApacheAccessLog));
    // Apache common log has `- - [time]` form
    assert!(s.contains(" - - ["));
}

#[test]
fn log_display_log4j_xml() {
    let s = format!("{}", sample_log(LogFormat::Log4jXML));
    assert!(s.starts_with("<log4j:event"));
    assert!(s.contains("<log4j:message>"));
}

#[test]
fn log_display_json() {
    let s = format!("{}", sample_log(LogFormat::JSON));
    assert!(s.contains("\"Format\":\"JSON\""));
    assert!(s.contains("\"Level\":\"INFO\""));
    // String escaping should produce \n, \r, \t, \" sequences
    assert!(s.contains("\\\""));
    assert!(s.contains("\\n"));
    assert!(s.contains("\\r"));
    assert!(s.contains("\\t"));
    // Control char \u{0008} → 
    assert!(s.contains("\\u0008"));
}

#[test]
fn log_display_gelf() {
    let s = format!("{}", sample_log(LogFormat::GELF));
    assert!(s.contains("\"version\":\"1.1\""));
    assert!(s.contains("\"short_message\""));
}

#[test]
fn log_display_logstash() {
    let s = format!("{}", sample_log(LogFormat::Logstash));
    assert!(s.contains("\"@timestamp\""));
    assert!(s.contains("\"message\""));
}

#[test]
fn log_display_ndjson() {
    let s = format!("{}", sample_log(LogFormat::NDJSON));
    assert!(s.contains("\"timestamp\""));
    assert!(s.contains("\"level\":\"INFO\""));
}

#[test]
fn log_display_mcp() {
    let s = format!("{}", sample_log(LogFormat::MCP));
    assert!(s.contains("\"jsonrpc\":\"2.0\""));
    assert!(s.contains("notifications/log"));
}

#[test]
fn log_display_otlp() {
    let s = format!("{}", sample_log(LogFormat::OTLP));
    assert!(s.contains("\"severityNumber\""));
    assert!(s.contains("\"spanId\":\"def\""));
    assert!(s.contains("\"traceId\":\"abc\""));
}

#[test]
fn log_display_logfmt() {
    let s = format!("{}", sample_log(LogFormat::Logfmt));
    assert!(s.contains("level=info"));
    assert!(s.contains("session_id=7"));
}

#[test]
fn log_display_ecs() {
    let s = format!("{}", sample_log(LogFormat::ECS));
    assert!(s.contains("\"@timestamp\""));
    assert!(s.contains("\"log.level\":\"info\""));
}

#[test]
fn log_otlp_without_trace_or_span_attributes_uses_empty_default() {
    // Hits the `unwrap_or(&empty)` branch in fmt_otlp.
    let log = Log::build(LogLevel::INFO, "no trace")
        .session_id(1)
        .time("2024-01-01T00:00:00Z")
        .format(LogFormat::OTLP);
    let s = format!("{log}");
    assert!(s.contains("\"spanId\":\"\""));
    assert!(s.contains("\"traceId\":\"\""));
}

#[test]
fn log_logfmt_handles_string_with_quote() {
    // Drive the `contains('"')` branch in write_logfmt.
    let mut log = Log::build(LogLevel::INFO, "x")
        .session_id(1)
        .time("t")
        .component("c")
        .format(LogFormat::Logfmt);
    log.attributes.insert(
        "msg".to_string(),
        serde_json::Value::String(r#"has "quote""#.to_string()),
    );
    let s = format!("{log}");
    assert!(s.contains(r#"msg="has \"quote\"""#), "got: {s}");
}

#[test]
fn log_builds_each_level_shortcut() {
    // Cover info/warn/error/debug/trace/verbose/fatal/critical builders.
    assert_eq!(Log::info("x").level, LogLevel::INFO);
    assert_eq!(Log::warn("x").level, LogLevel::WARN);
    assert_eq!(Log::error("x").level, LogLevel::ERROR);
    assert_eq!(Log::debug("x").level, LogLevel::DEBUG);
    assert_eq!(Log::trace("x").level, LogLevel::TRACE);
    assert_eq!(Log::verbose("x").level, LogLevel::VERBOSE);
    assert_eq!(Log::fatal("x").level, LogLevel::FATAL);
    assert_eq!(Log::critical("x").level, LogLevel::CRITICAL);
}

#[test]
fn log_with_unserializable_value_silently_drops() {
    // serde_json::to_value cannot fail for &str/i64/etc., but verify the
    // `if let Ok(...)` shape still produces a sane Log for an array.
    let log = Log::info("desc").with("arr", vec![1_i32, 2, 3]);
    assert!(log.attributes.contains_key("arr"));
}

// ---------------------------------------------------------------------------
// utils.rs — public re-exports
// ---------------------------------------------------------------------------

#[test]
fn utils_format_file_size_scales_through_units() {
    assert_eq!(rlg::utils::format_file_size(0), "0.00 B");
    assert_eq!(rlg::utils::format_file_size(1023), "1023.00 B");
    assert_eq!(rlg::utils::format_file_size(1024), "1.00 KB");
    assert_eq!(rlg::utils::format_file_size(1_048_576), "1.00 MB");
    // Force progression through every unit including TB and PB.
    let pb = 1024_u64.pow(5);
    assert!(rlg::utils::format_file_size(pb).ends_with(" PB"));
}

#[test]
fn utils_parse_datetime_round_trip() {
    let ok: RlgResult<String> =
        rlg::utils::parse_datetime("2024-08-29T12:00:00Z");
    assert_eq!(ok.unwrap(), "2024-08-29T12:00:00Z");
    assert!(rlg::utils::parse_datetime("garbage").is_err());
}

#[test]
fn utils_generate_timestamp_is_iso8601() {
    let ts = rlg::utils::generate_timestamp();
    // 30 chars: YYYY-MM-DDTHH:MM:SS.fffffffffZ
    assert_eq!(ts.len(), 30);
    assert!(ts.ends_with('Z'));
}

#[test]
fn utils_span_and_trace_id_lengths() {
    assert_eq!(rlg::utils::generate_span_id().len(), 16);
    assert_eq!(rlg::utils::generate_trace_id().len(), 32);
}

#[test]
fn utils_sanitize_log_message_strips_controls() {
    let raw = "hello\nworld\r\ttab";
    let sanitized = rlg::utils::sanitize_log_message(raw);
    assert!(!sanitized.contains('\n'));
    assert!(!sanitized.contains('\r'));
    assert!(!sanitized.contains('\t'));
}

// ---------------------------------------------------------------------------
// error.rs — From conversion via parse_datetime error
// ---------------------------------------------------------------------------

#[test]
fn error_datetime_parse_error_round_trip() {
    let err = rlg::utils::parse_datetime("not a date").unwrap_err();
    assert!(matches!(err, RlgError::DateTimeParseError(_)));
    assert!(err.to_string().contains("DateTime parse error"));
}

// ---------------------------------------------------------------------------
// sink.rs — exercise PlatformSink::from_config across every destination
// ---------------------------------------------------------------------------

#[test]
fn sink_from_config_file_destination() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("from_config.log");
    let cfg = rlg::config::Config {
        logging_destinations: vec![
            rlg::config::LoggingDestination::File(path.clone()),
        ],
        ..rlg::config::Config::default()
    };
    let sink = rlg::sink::PlatformSink::from_config(&cfg);
    assert!(matches!(sink, rlg::sink::PlatformSink::File(_)));
    assert!(path.exists());
}

#[test]
fn sink_from_config_stdout_destination() {
    let cfg = rlg::config::Config {
        logging_destinations: vec![
            rlg::config::LoggingDestination::Stdout,
        ],
        ..rlg::config::Config::default()
    };
    let sink = rlg::sink::PlatformSink::from_config(&cfg);
    assert!(matches!(sink, rlg::sink::PlatformSink::Stdout));
}

#[test]
fn sink_from_config_network_falls_through_to_native() {
    // Network variant is "not yet implemented" — config should fall
    // through to PlatformSink::native(). With no remaining destinations
    // we expect whatever native() returns on this host.
    let cfg = rlg::config::Config {
        logging_destinations: vec![
            rlg::config::LoggingDestination::Network(
                "remote:9200".into(),
            ),
        ],
        ..rlg::config::Config::default()
    };
    let _ = rlg::sink::PlatformSink::from_config(&cfg);
}

// ---------------------------------------------------------------------------
// config.rs — load/save round-trip with a real TOML file
// ---------------------------------------------------------------------------

#[test]
fn config_save_then_load_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rlg.toml");
    let original = rlg::config::Config::default();
    // Exercises `save_to_file`.
    original.save_to_file(&path).unwrap();
    assert!(path.exists());

    // Exercises the `Some(path)` arm of `load`, the version check,
    // try_deserialize, validate, ensure_paths.
    let loaded = rlg::config::Config::load(Some(&path)).unwrap();
    let guard = loaded.read();
    assert_eq!(guard.version, original.version);
}

#[test]
fn config_load_none_path_uses_default() {
    // Exercises the `else { Self::default() }` branch in `load`.
    let cfg =
        rlg::config::Config::load(None::<&std::path::Path>).unwrap();
    let guard = cfg.read();
    assert!(!guard.version.is_empty());
}

#[test]
fn config_load_rejects_wrong_version() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.toml");
    std::fs::write(
        &path,
        b"version = \"0.0.0-bogus\"\nlog_level = \"INFO\"\n",
    )
    .unwrap();
    let res = rlg::config::Config::load(Some(&path));
    assert!(res.is_err(), "expected version error, got {res:?}");
}

#[test]
fn config_load_rejects_missing_file() {
    let res = rlg::config::Config::load(Some(std::path::Path::new(
        "/nonexistent/path/rlg.toml",
    )));
    assert!(res.is_err());
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn config_load_async_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("async.toml");
    let original = rlg::config::Config::default();
    original.save_to_file(&path).unwrap();
    let loaded =
        rlg::config::Config::load_async(Some(&path)).await.unwrap();
    let guard = loaded.read();
    assert_eq!(guard.version, original.version);
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn config_load_async_rejects_wrong_version() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("async_bad.toml");
    std::fs::write(
        &path,
        b"version = \"42.0.0\"\nlog_level = \"INFO\"\n",
    )
    .unwrap();
    let res = rlg::config::Config::load_async(Some(&path)).await;
    assert!(res.is_err());
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn config_load_async_none_path_uses_default() {
    let cfg = rlg::config::Config::load_async(None::<&std::path::Path>)
        .await
        .unwrap();
    let guard = cfg.read();
    assert!(!guard.version.is_empty());
}

#[test]
fn sink_from_config_file_with_unwritable_path_falls_through() {
    // A path that cannot be opened (e.g. a directory we cannot create)
    // should be skipped, then Stdout consumed as the next destination.
    let cfg = rlg::config::Config {
        logging_destinations: vec![
            rlg::config::LoggingDestination::File(
                "/this/path/should/never/exist/abc.log".into(),
            ),
            rlg::config::LoggingDestination::Stdout,
        ],
        ..rlg::config::Config::default()
    };
    let sink = rlg::sink::PlatformSink::from_config(&cfg);
    assert!(matches!(sink, rlg::sink::PlatformSink::Stdout));
}
