#![allow(missing_docs)]
#![cfg(not(miri))]
use rlg::engine::{ENGINE, FastSerializer, LockFreeEngine, LogEvent};

#[test]
fn test_fast_serializer() {
    let mut buf = Vec::new();
    FastSerializer::append_u64(&mut buf, 12345);
    assert_eq!(std::str::from_utf8(&buf).unwrap(), "12345");

    buf.clear();
    FastSerializer::append_f64(&mut buf, 12.34);
    assert_eq!(std::str::from_utf8(&buf).unwrap(), "12.34");
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_engine_shutdown() {
    // This just ensures shutdown() doesn't panic
    ENGINE.shutdown();
}

#[test]
fn test_engine_span_metrics() {
    let engine = LockFreeEngine::new(10);
    assert_eq!(engine.active_spans(), 0);
    engine.inc_spans();
    assert_eq!(engine.active_spans(), 1);
    engine.dec_spans();
    assert_eq!(engine.active_spans(), 0);
    engine.shutdown();
}

#[test]
fn test_engine_set_filter() {
    let engine = LockFreeEngine::new(10);
    engine.set_filter(3);
    assert_eq!(engine.filter_level(), 3);
    engine.shutdown();
}

#[test]
fn test_engine_apply_config() {
    let engine = LockFreeEngine::new(10);
    let config = rlg::config::Config {
        log_level: rlg::LogLevel::WARN,
        ..rlg::config::Config::default()
    };
    engine.apply_config(&config);
    assert_eq!(engine.filter_level(), rlg::LogLevel::WARN.to_numeric());
    engine.shutdown();
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_engine_queue_full_and_errors() {
    // Test the specific branch where an error increments metrics
    let event_err = LogEvent {
        level: rlg::LogLevel::ERROR,
        level_num: 8,
        log: rlg::log::Log::error("error"),
    };
    ENGINE.ingest(event_err.clone());

    // Test the queue full fallback by filling a very small queue
    let small_engine = LockFreeEngine::new(2);
    small_engine.ingest(event_err.clone());
    small_engine.ingest(event_err.clone());
    small_engine.ingest(event_err); // This triggers the while let Err(...) loop
}

#[test]
#[cfg_attr(miri, ignore)]
#[allow(unsafe_code)]
fn test_engine_tui_flag() {
    // Temporarily set the flag and spawn an engine to cover the TUI spawn branch
    // SAFETY: Test-only; no other threads depend on this env var.
    unsafe { std::env::set_var("RLG_TUI", "1") };
    let tui_engine = LockFreeEngine::new(10);
    tui_engine.shutdown();
    // SAFETY: Test-only cleanup.
    unsafe { std::env::remove_var("RLG_TUI") };
}
