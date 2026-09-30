// tests.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Unit tests for `init.rs`.

use super::*;
use serial_test::serial;

/// SAFETY shim: tests in this module manipulate process-wide env vars
/// — they must run serially.
#[allow(unsafe_code)]
fn set_env(key: &str, val: &str) {
    // SAFETY: tests using this helper are `#[serial]`, so no other
    // thread reads env vars concurrently.
    unsafe { std::env::set_var(key, val) };
}

#[allow(unsafe_code)]
fn unset_env(key: &str) {
    // SAFETY: see `set_env`.
    unsafe { std::env::remove_var(key) };
}

#[test]
#[serial]
#[cfg_attr(miri, ignore)]
fn parse_rust_log_returns_most_permissive() {
    set_env("RUST_LOG", "warn,my_crate=debug,other=info");
    let level = parse_rust_log();
    assert_eq!(level, Some(LogLevel::DEBUG));
    unset_env("RUST_LOG");
}

#[test]
#[serial]
#[cfg_attr(miri, ignore)]
fn parse_rust_log_simple_level() {
    set_env("RUST_LOG", "trace");
    assert_eq!(parse_rust_log(), Some(LogLevel::TRACE));
    unset_env("RUST_LOG");
}

#[test]
#[serial]
#[cfg_attr(miri, ignore)]
fn parse_rust_log_ignores_unparseable_directives() {
    set_env("RUST_LOG", "garbage_xyz,info");
    assert_eq!(parse_rust_log(), Some(LogLevel::INFO));
    unset_env("RUST_LOG");
}

#[test]
#[serial]
#[cfg_attr(miri, ignore)]
fn parse_rust_log_returns_none_when_unset() {
    unset_env("RUST_LOG");
    assert!(parse_rust_log().is_none());
}

#[test]
#[serial]
#[cfg_attr(miri, ignore)]
fn detect_default_format_production_env_forces_json() {
    set_env("RLG_ENV", "production");
    assert_eq!(detect_default_format(), LogFormat::JSON);
    unset_env("RLG_ENV");
}

#[test]
#[serial]
#[cfg_attr(miri, ignore)]
fn detect_default_format_outside_production_picks_via_tty() {
    unset_env("RLG_ENV");
    // Outside production the answer depends on stdout being a TTY.
    // Either branch is acceptable — what matters is that the function
    // runs without panicking and returns one of the two valid choices.
    let fmt = detect_default_format();
    assert!(fmt == LogFormat::JSON || fmt == LogFormat::Logfmt);
}

#[test]
fn detect_default_format_for_covers_every_branch() {
    // production → JSON regardless of TTY.
    assert_eq!(
        detect_default_format_for(Some("production"), true),
        LogFormat::JSON
    );
    assert_eq!(
        detect_default_format_for(Some("production"), false),
        LogFormat::JSON
    );
    // Non-production + TTY → Logfmt (the branch real-life tests
    // cannot reach because stdout is piped in `cargo test`).
    assert_eq!(
        detect_default_format_for(None, true),
        LogFormat::Logfmt
    );
    assert_eq!(
        detect_default_format_for(Some("staging"), true),
        LogFormat::Logfmt
    );
    // Non-production + pipe → JSON.
    assert_eq!(detect_default_format_for(None, false), LogFormat::JSON);
}

#[test]
fn atty_stdout_runs() {
    // Only verifying the call is reachable.
    let _ = atty_stdout();
}

#[test]
fn test_init_error_display_logger_already_set() {
    let err = InitError::LoggerAlreadySet;
    assert_eq!(err.to_string(), "a log crate logger was already set");
}

#[test]
fn test_init_error_display_subscriber_already_set() {
    let err = InitError::SubscriberAlreadySet;
    assert_eq!(err.to_string(), "a tracing subscriber was already set");
}

#[test]
fn test_init_error_display_already_initialized() {
    let err = InitError::AlreadyInitialized;
    assert_eq!(err.to_string(), "rlg was already initialized");
}

#[test]
fn test_init_error_debug() {
    let err = InitError::LoggerAlreadySet;
    assert_eq!(format!("{err:?}"), "LoggerAlreadySet");
}

#[test]
fn test_init_error_clone_copy() {
    let err = InitError::AlreadyInitialized;
    let cloned = err;
    assert_eq!(format!("{err:?}"), format!("{cloned:?}"));
}

#[test]
fn test_init_error_is_error() {
    let err = InitError::LoggerAlreadySet;
    // Verify it implements std::error::Error
    let _: &dyn std::error::Error = &err;
}

#[test]
fn test_builder_defaults() {
    let b = RlgBuilder::default();
    assert_eq!(b.level, LogLevel::INFO);
    assert!(b.install_log);
    assert!(b.install_tracing);
    // Format is auto-detected (Logfmt for TTY, JSON for pipe/CI)
    assert!(
        b.format == LogFormat::JSON || b.format == LogFormat::Logfmt
    );
}

#[test]
fn test_builder_level() {
    let b = builder().level(LogLevel::DEBUG);
    assert_eq!(b.level, LogLevel::DEBUG);
}

#[test]
fn test_builder_format() {
    let b = builder().format(LogFormat::JSON);
    assert_eq!(b.format, LogFormat::JSON);
}

#[test]
fn test_builder_without_log() {
    let b = builder().without_log();
    assert!(!b.install_log);
    assert!(b.install_tracing);
}

#[test]
fn test_builder_without_tracing() {
    let b = builder().without_tracing();
    assert!(b.install_log);
    assert!(!b.install_tracing);
}

#[test]
fn test_builder_chaining() {
    let b = builder()
        .level(LogLevel::TRACE)
        .format(LogFormat::ECS)
        .without_log()
        .without_tracing();
    assert_eq!(b.level, LogLevel::TRACE);
    assert_eq!(b.format, LogFormat::ECS);
    assert!(!b.install_log);
    assert!(!b.install_tracing);
}

#[test]
fn test_builder_clone_copy() {
    let b = builder().level(LogLevel::WARN);
    let b2 = b;
    // Both usable since RlgBuilder is Copy
    assert_eq!(b.level, b2.level);
    assert_eq!(b.format, b2.format);
}

#[test]
fn test_builder_without_facades_configuration() {
    let b = builder().without_log().without_tracing();
    assert!(!b.install_log);
    assert!(!b.install_tracing);
}

#[test]
fn test_builder_fn() {
    let b = builder();
    assert_eq!(b.level, LogLevel::INFO);
    // Format is auto-detected based on output context
    assert!(
        b.format == LogFormat::JSON || b.format == LogFormat::Logfmt
    );
    assert!(b.install_log);
    assert!(b.install_tracing);
}

#[test]
fn test_init_error_source() {
    let err = InitError::LoggerAlreadySet;
    // std::error::Error::source should return None
    assert!(std::error::Error::source(&err).is_none());
}

#[test]
fn test_builder_default_impl() {
    let b1 = RlgBuilder::default();
    let b2 = builder();
    assert_eq!(b1.level, b2.level);
    assert_eq!(b1.format, b2.format);
    assert_eq!(b1.install_log, b2.install_log);
    assert_eq!(b1.install_tracing, b2.install_tracing);
}

#[test]
fn test_init_error_all_display_variants() {
    // Exercise all three Display paths
    let msgs: Vec<String> = vec![
        InitError::LoggerAlreadySet,
        InitError::SubscriberAlreadySet,
        InitError::AlreadyInitialized,
    ]
    .into_iter()
    .map(|e| e.to_string())
    .collect();
    assert_eq!(msgs.len(), 3);
    assert!(msgs[0].contains("log"));
    assert!(msgs[1].contains("tracing"));
    assert!(msgs[2].contains("already initialized"));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_init_guard_static() {
    // Exercise the OnceLock guard
    // First attempt may succeed or fail depending on test ordering
    let _ = INIT_GUARD.set(());
    // Second attempt should always fail
    assert!(INIT_GUARD.set(()).is_err());
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_logger_static() {
    // Exercise the LOGGER OnceLock
    let logger = LOGGER.get_or_init(|| RlgLogger::new(LogFormat::JSON));
    assert!(format!("{logger:?}").contains("RlgLogger"));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_install_log_facade() {
    // First call may succeed or fail (test ordering is non-deterministic)
    let r1 =
        RlgBuilder::install_log_facade(LogFormat::JSON, LogLevel::INFO);
    assert!(
        r1.is_ok() || matches!(r1, Err(InitError::LoggerAlreadySet))
    );
    // Second call should definitely fail
    let r2 =
        RlgBuilder::install_log_facade(LogFormat::MCP, LogLevel::DEBUG);
    assert!(matches!(r2, Err(InitError::LoggerAlreadySet)));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_install_tracing_subscriber() {
    // First call may succeed or fail (test ordering is non-deterministic)
    let r1 = RlgBuilder::install_tracing_subscriber();
    assert!(
        r1.is_ok()
            || matches!(r1, Err(InitError::SubscriberAlreadySet))
    );
    // Second call should definitely fail
    let r2 = RlgBuilder::install_tracing_subscriber();
    assert!(matches!(r2, Err(InitError::SubscriberAlreadySet)));
}
