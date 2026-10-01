// tests.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Unit tests for `engine.rs`.

use super::*;
use crate::LogLevel;
use crate::log::Log;

fn make_event(level: LogLevel) -> LogEvent {
    LogEvent {
        level,
        level_num: level.to_numeric(),
        log: Log::build(level, "test"),
        caller: None,
    }
}

/// The flusher renders the call site `fire()` captured as the
/// `caller` attribute, once, overriding a user-set `caller`.
#[test]
#[cfg(not(miri))]
fn the_flusher_attaches_the_caller_attribute() {
    let location = std::panic::Location::caller();
    let mut event = LogEvent {
        caller: Some(location),
        ..LogEvent::new(Log::info("msg").with("caller", "user"))
    };
    attach_caller(&mut event);
    let expected = format!("{}:{}", location.file(), location.line());
    assert_eq!(event.log.attributes["caller"], expected.as_str());
    assert!(event.caller.is_none(), "rendered once");

    let mut plain = LogEvent::new(Log::info("msg"));
    attach_caller(&mut plain);
    assert!(!plain.log.attributes.contains_key("caller"));
}

/// The flusher counts what it drains: after `shutdown` every ingested
/// event appears once in the total, level, error and format counters,
/// and events below the filter appear nowhere.
#[test]
#[cfg_attr(miri, ignore)] // spawns the flusher
fn the_flusher_counts_every_ingested_event() {
    use crate::LogFormat;
    let engine = LockFreeEngine::new(1_024);
    engine.set_filter(LogLevel::DEBUG.to_numeric());
    let event = |level: LogLevel, format: LogFormat| {
        let mut e = make_event(level);
        e.log.format = format;
        engine.ingest(e);
    };
    event(LogLevel::TRACE, LogFormat::JSON); // filtered out
    event(LogLevel::INFO, LogFormat::JSON);
    event(LogLevel::INFO, LogFormat::MCP);
    event(LogLevel::ERROR, LogFormat::JSON);
    event(LogLevel::FATAL, LogFormat::ECS);
    engine.shutdown();
    let m = &engine.metrics;
    let n =
        |c: &std::sync::atomic::AtomicUsize| c.load(Ordering::Relaxed);
    assert_eq!(n(&m.total_events), 4);
    assert_eq!(n(&m.error_count), 2);
    assert_eq!((n(&m.level_trace), n(&m.level_info)), (0, 2));
    assert_eq!((n(&m.level_error), n(&m.level_fatal)), (1, 1));
    assert_eq!(
        (n(&m.fmt_json), n(&m.fmt_mcp), n(&m.fmt_ecs)),
        (2, 1, 1)
    );
    assert_eq!(n(&m.dropped_events), 0);
}

/// Under contention on a full queue, every event pushed ends up
/// either in the queue or counted as dropped: nothing is lost
/// uncounted and nothing is counted twice.
#[test]
#[cfg_attr(miri, ignore)] // spawns threads
fn every_event_is_queued_or_counted_as_dropped() {
    const THREADS: usize = 8;
    const EACH: usize = 2_000;
    for round in 0..20 {
        let queue = Arc::new(ShardedQueue::new(8));
        let metrics = Arc::new(TuiMetrics::default());
        let producers: Vec<_> = (0..THREADS)
            .map(|_| {
                let (queue, metrics) =
                    (Arc::clone(&queue), Arc::clone(&metrics));
                thread::spawn(move || {
                    for _ in 0..EACH {
                        push_evicting(
                            &queue,
                            &metrics,
                            make_event(LogLevel::INFO),
                        );
                    }
                })
            })
            .collect();
        for producer in producers {
            producer.join().unwrap();
        }
        let mut queued = 0;
        while let Some(event) = queue.pop() {
            count_event(&metrics, &event);
            queued += 1;
        }
        let dropped = metrics.dropped_events.load(Ordering::Relaxed);
        assert_eq!(
            queued + dropped,
            THREADS * EACH,
            "round {round}: {queued} queued, {dropped} dropped"
        );
        assert_eq!(
            metrics.total_events.load(Ordering::Relaxed),
            THREADS * EACH,
            "round {round}: drained plus dropped events counted once"
        );
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn fast_serializer_round_trip() {
    let mut buf = Vec::new();
    FastSerializer::append_u64(&mut buf, 1234);
    FastSerializer::append_f64(&mut buf, 3.5);
    assert_eq!(buf, b"12343.5");
}

#[test]
#[cfg_attr(miri, ignore)]
fn ingest_overfills_small_queue_and_drops() {
    // Capacity 1 — every ingest after the first hits the retry loop
    // (covers the `Err(e) => to_push = e` continuation in the retry).
    let engine = LockFreeEngine::new(1);
    for _ in 0..32 {
        engine.ingest(make_event(LogLevel::INFO));
    }
    engine.shutdown();
}

#[test]
#[cfg_attr(miri, ignore)]
fn ingest_under_concurrent_overfill_hits_retry_err_branch() {
    // 8 producer threads hammer a capacity-1 queue concurrently.
    // Statistically guaranteed to hit the `Err(e) => to_push = e`
    // arm in the retry loop (line 204 in src/engine.rs) when a
    // peer thread refills the slot between `pop` and `push`.
    let engine = Arc::new(LockFreeEngine::new(1));
    let mut handles = Vec::new();
    for _ in 0..8 {
        let e = engine.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..2_000 {
                e.ingest(make_event(LogLevel::INFO));
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    engine.shutdown();
}

#[test]
#[cfg_attr(miri, ignore)]
fn ingest_below_filter_short_circuits() {
    let engine = LockFreeEngine::new(8);
    engine.set_filter(LogLevel::ERROR.to_numeric());
    // DEBUG is below ERROR — should be filtered out before push.
    engine.ingest(make_event(LogLevel::DEBUG));
    engine.shutdown();
}

#[test]
#[cfg_attr(miri, ignore)]
fn engine_records_errors_and_spans() {
    let engine = LockFreeEngine::new(8);
    engine.ingest(make_event(LogLevel::ERROR));
    engine.inc_format(crate::log_format::LogFormat::JSON);
    engine.inc_spans();
    engine.inc_spans();
    assert_eq!(engine.active_spans(), 2);
    engine.dec_spans();
    assert_eq!(engine.active_spans(), 1);
    engine.shutdown();
}

#[test]
#[cfg_attr(miri, ignore)]
fn shutdown_is_idempotent() {
    let engine = LockFreeEngine::new(4);
    engine.ingest(make_event(LogLevel::INFO));
    engine.shutdown();
    // Second shutdown: flusher_join has already been drained, so the
    // `Some(handle) = guard.take()` branch is None this time.
    engine.shutdown();
}

#[test]
fn apply_config_updates_filter() {
    let engine = LockFreeEngine::new(4);
    let cfg = crate::config::Config {
        log_level: LogLevel::WARN,
        ..crate::config::Config::default()
    };
    engine.apply_config(&cfg);
    assert_eq!(engine.filter_level(), LogLevel::WARN.to_numeric());
    engine.shutdown();
}
