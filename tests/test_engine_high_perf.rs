#![cfg(not(miri))]
#![allow(missing_docs)]
use rlg::engine::{FastSerializer, LockFreeEngine, LogEvent, ENGINE};

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
fn test_engine_set_filter() {
    ENGINE.set_filter(3);
    assert_eq!(ENGINE.filter_level(), 3);
    ENGINE.set_filter(0);
}

#[test]
fn test_engine_apply_config() {
    let config = rlg::config::Config {
        log_level: rlg::LogLevel::WARN,
        ..rlg::config::Config::default()
    };
    ENGINE.apply_config(&config);
    assert_eq!(ENGINE.filter_level(), rlg::LogLevel::WARN.to_numeric());
    ENGINE.set_filter(0);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_engine_queue_full_and_errors() {
    // Test the specific branch where an error increments metrics
    let event_err = LogEvent {
        level: "ERROR".to_string(),
        level_num: 8,
        payload: b"error".to_vec(),
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
fn test_engine_tui_flag() {
    // Temporarily set the flag and spawn an engine to cover the TUI spawn branch
    std::env::set_var("RLG_TUI", "1");
    let tui_engine = LockFreeEngine::new(10);
    tui_engine.shutdown();
    std::env::remove_var("RLG_TUI");
}
