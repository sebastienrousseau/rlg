// Integration test that owns the `INIT_GUARD` for this binary, so it
// can exercise the full `RlgBuilder::init()` body. Each integration
// test file runs in its own process — `OnceLock` resets between
// binaries even though it can't reset within one.
//
// Skipped under MIRI: `init()` spawns the flusher thread and registers
// the log/tracing facades, both of which MIRI cannot meaningfully run.

#![allow(missing_docs)]
#![cfg(not(miri))]

use rlg::init;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;

#[test]
fn first_init_consumes_guard_and_then_fails_on_second_call() {
    // SAFETY: process-local env vars; this binary contains a single
    // test so no other thread is racing us.
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
        std::env::set_var("RLG_FALLBACK_STDOUT", "1");
    }

    // Drive the *full* init path, including log + tracing facade
    // installation. We're the only test in this binary so the global
    // `log`/`tracing` slots are guaranteed unset.
    let guard = rlg::builder()
        .level(LogLevel::INFO)
        .format(LogFormat::JSON)
        .init();
    assert!(guard.is_ok(), "init failed: {guard:?}");

    // A second init must fail with AlreadyInitialized.
    let second = init();
    assert!(second.is_err(), "second init unexpectedly ok");

    #[allow(unsafe_code)]
    unsafe {
        std::env::remove_var("RUST_LOG");
    }
    // Drop guard explicitly — exercises FlushGuard::drop / ENGINE.shutdown.
    drop(guard);
}
