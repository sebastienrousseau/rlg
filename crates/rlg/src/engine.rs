// engine.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Near-lock-free ingestion engine backed by a bounded ring buffer.
//!
//! The global [`ENGINE` static](crate::engine::ENGINE) accepts
//! [`LogEvent` values](crate::engine::LogEvent) via
//! [`LockFreeEngine::ingest()`][crate::engine::LockFreeEngine::ingest]
//! using only atomic operations. A dedicated background thread drains events
//! in batches of 64 and writes them through [`crate::sink::PlatformSink`].
//!
//! **The Mutex is never locked on the hot path.** It exists solely for
//! `shutdown()` to join the flusher thread.

use crate::log_level::LogLevel;
use crate::sharded_queue::ShardedQueue;
#[cfg(not(miri))]
use crate::sink::PlatformSink;
use crate::tui::TuiMetrics;
#[cfg(not(miri))]
use crate::tui::spawn_tui_thread;
use std::fmt;
#[cfg(not(miri))]
use std::sync::atomic::fence;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::thread;
#[cfg(not(miri))]
use std::time::Duration;

/// Capacity of the lock-free ring buffer (number of log events).
const RING_BUFFER_CAPACITY: usize = 65_536;

/// Maximum number of events drained per flusher wake-up cycle.
#[cfg(not(miri))]
const MAX_DRAIN_BATCH_SIZE: usize = 64;

/// A structured log event passed through the ring buffer.
///
/// The caller pays only for a `Log` move (~128-byte memcpy).
/// Serialization happens on the flusher thread.
#[derive(Debug, Clone)]
pub struct LogEvent {
    /// Severity level of this event.
    pub level: LogLevel,
    /// Numeric severity for fast level-gating comparisons.
    pub level_num: u8,
    /// Structured log data. Formatted on the flusher thread, not here.
    pub log: crate::log::Log,
}

/// The near-lock-free ingestion engine.
///
/// Owns the ring buffer, flusher thread, and TUI metrics counters.
/// Access the global instance via [`ENGINE`].
pub struct LockFreeEngine {
    /// Bounded, sharded ring buffer.
    ///
    /// - Default build: one shard — semantically identical to the
    ///   direct `ArrayQueue` use in prior releases.
    /// - `fast-queue` feature: eight shards — reduces producer-side
    ///   cache-line contention on the underlying atomic tag when
    ///   many threads ingest concurrently.
    ///
    /// See `docs/adr/0009-sharded-producer-queue.md`.
    queue: Arc<ShardedQueue>,
    /// Signals the flusher thread to drain and exit.
    shutdown_flag: Arc<AtomicBool>,
    /// Atomic counters consumed by the opt-in TUI dashboard.
    metrics: Arc<TuiMetrics>,
    /// Minimum severity level. Events below this are dropped at `ingest()`.
    filter_level: AtomicU8,
    /// Flusher thread handle for lock-free `unpark()`. No Mutex involved.
    flusher_thread_handle: Option<thread::Thread>,
    /// Set by the flusher just before it parks. Producers only read it,
    /// so a busy flusher costs them no shared write; the first producer
    /// to see it set clears it and wakes the flusher.
    flusher_idle: Arc<AtomicBool>,
    /// `JoinHandle` for `shutdown()` only. **Never locked on the hot path.**
    flusher_join: Mutex<Option<thread::JoinHandle<()>>>,
}

impl fmt::Debug for LockFreeEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LockFreeEngine")
            .field("queue", &self.queue)
            .field("shutdown_flag", &self.shutdown_flag)
            .field("metrics", &self.metrics)
            .field("filter_level", &self.filter_level)
            .field(
                "flusher_thread_handle",
                &self
                    .flusher_thread_handle
                    .as_ref()
                    .map(thread::Thread::id),
            )
            .finish_non_exhaustive()
    }
}

/// Global engine instance, lazily initialized on first access.
pub static ENGINE: LazyLock<LockFreeEngine> =
    LazyLock::new(|| LockFreeEngine::new(RING_BUFFER_CAPACITY));

/// Push `event`; if its shard is full, evict and retry with bounded
/// retries.
///
/// `pop_local` targets the same shard as `push` so the eviction makes
/// room for the retry on the shard the producer is actually contending
/// on. Under the default (1-shard) build this is identical to the
/// historical pop-then-push loop.
///
/// Every event that does not stay in the queue is counted once in
/// `dropped_events`, and once by [`count_event`] since the flusher will
/// never see it: each one an eviction actually removes, and the new
/// event itself if every retry loses the race for the freed slot.
fn push_evicting(
    queue: &ShardedQueue,
    metrics: &TuiMetrics,
    event: LogEvent,
) {
    let Err(mut rejected) = queue.push(event) else {
        return;
    };
    for _ in 0..3 {
        if let Some(evicted) = queue.pop_local() {
            count_dropped(metrics, &evicted);
        }
        match queue.push(rejected) {
            Ok(()) => return,
            Err(e) => rejected = e,
        }
    }
    count_dropped(metrics, &rejected);
}

/// Count an event that left the engine without reaching the sink.
fn count_dropped(metrics: &TuiMetrics, event: &LogEvent) {
    count_event(metrics, event);
    metrics.inc_dropped();
}

/// Add one event to the totals, level, error and format counters.
///
/// Runs where an event leaves the queue (on the flusher, or on the
/// rare eviction path), never on every `ingest`: the counters then
/// have one writer in the common case, and producers do not contend
/// on their cache line.
fn count_event(metrics: &TuiMetrics, event: &LogEvent) {
    metrics.inc_events();
    metrics.inc_level(event.level);
    if event.level_num >= LogLevel::ERROR.to_numeric() {
        metrics.inc_errors();
    }
    metrics.inc_format(event.log.format);
}

/// Start the `rlg-flusher` thread over `queue`.
#[cfg(not(miri))]
fn spawn_flusher(
    queue: Arc<ShardedQueue>,
    shutdown: Arc<AtomicBool>,
    idle: Arc<AtomicBool>,
    metrics: Arc<TuiMetrics>,
) -> thread::JoinHandle<()> {
    // Lightweight OS thread: runtime agnostic.
    thread::Builder::new()
        .name("rlg-flusher".into())
        .spawn(move || run_flusher(&queue, &shutdown, &idle, &metrics))
        .expect("Failed to spawn rlg-flusher background thread")
}

/// The flusher loop: drain a batch, format and emit it, and stop once
/// shutdown is flagged and the queue is empty.
#[cfg(not(miri))]
fn run_flusher(
    queue: &ShardedQueue,
    shutdown: &AtomicBool,
    idle: &AtomicBool,
    metrics: &TuiMetrics,
) {
    let mut sink = PlatformSink::native();
    let mut fmt_buf = Vec::with_capacity(512);
    loop {
        let mut batch: [Option<LogEvent>; MAX_DRAIN_BATCH_SIZE] =
            std::array::from_fn(|_| None);
        drain_into(queue, &mut batch);
        for event in batch.iter().flatten() {
            count_event(metrics, event);
            emit(&mut sink, &mut fmt_buf, event);
        }
        if shutdown.load(Ordering::Relaxed) && queue.is_empty() {
            break;
        }
        park_when_idle(queue, idle);
    }
}

/// Park until a producer wakes the flusher, or 5 ms pass. The flag is
/// raised before the final emptiness check, so a producer that pushed
/// after that check sees it and wakes the thread; a wake-up lost in the
/// narrow race between the two is covered by the timeout.
#[cfg(not(miri))]
fn park_when_idle(queue: &ShardedQueue, idle: &AtomicBool) {
    idle.store(true, Ordering::SeqCst);
    fence(Ordering::SeqCst);
    if queue.is_empty() {
        thread::park_timeout(Duration::from_millis(5));
    }
    idle.store(false, Ordering::Release);
}

/// Fill `batch` from the front with queued events until either runs out.
#[cfg(not(miri))]
fn drain_into(queue: &ShardedQueue, batch: &mut [Option<LogEvent>]) {
    for slot in batch {
        let Some(event) = queue.pop() else { break };
        *slot = Some(event);
    }
}

/// Format one event into `fmt_buf` and hand it to the sink.
#[cfg(not(miri))]
fn emit(
    sink: &mut PlatformSink,
    fmt_buf: &mut Vec<u8>,
    event: &LogEvent,
) {
    use std::io::Write;
    fmt_buf.clear();
    let _ = writeln!(fmt_buf, "{}", event.log);
    sink.emit(event.level.as_str(), fmt_buf);
}

impl LockFreeEngine {
    /// Create a new engine with the given buffer capacity and spawn the flusher.
    ///
    /// # Panics
    ///
    /// Panics if the OS cannot spawn the background flusher thread.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let queue = Arc::new(ShardedQueue::new(capacity));
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let metrics = Arc::new(TuiMetrics::default());
        let filter_level = AtomicU8::new(0); // Default to ALL
        let flusher_idle = Arc::new(AtomicBool::new(false));

        // Under MIRI, skip spawning background threads to avoid
        // "main thread terminated without waiting" errors.
        #[cfg(not(miri))]
        let flusher_handle = {
            let handle = spawn_flusher(
                Arc::clone(&queue),
                Arc::clone(&shutdown_flag),
                Arc::clone(&flusher_idle),
                Arc::clone(&metrics),
            );
            // Spawn the TUI dashboard thread if RLG_TUI=1
            if std::env::var("RLG_TUI").is_ok_and(|v| v == "1") {
                spawn_tui_thread(
                    metrics.clone(),
                    shutdown_flag.clone(),
                );
            }
            Some(handle)
        };

        #[cfg(miri)]
        let flusher_handle: Option<thread::JoinHandle<()>> = None;

        let flusher_thread_handle =
            flusher_handle.as_ref().map(|h| h.thread().clone());

        Self {
            queue,
            shutdown_flag,
            metrics,
            filter_level,
            flusher_thread_handle,
            flusher_idle,
            flusher_join: Mutex::new(flusher_handle),
        }
    }

    /// Appends an event to the ring buffer.
    ///
    /// If the buffer is full, the oldest event is evicted to make room.
    /// Dropped events are tracked via `TuiMetrics::dropped_events`.
    ///
    /// The event, level, error and format counters are updated as the
    /// flusher drains each event (or when one is dropped), so they
    /// trail `ingest` by at most one flush.
    pub fn ingest(&self, event: LogEvent) {
        if event.level_num < self.filter_level.load(Ordering::Acquire) {
            return;
        }
        push_evicting(&self.queue, &self.metrics, event);
        self.wake_flusher();
    }

    /// Wake the flusher if it is parked. Only the first producer to see
    /// it idle pays for the `unpark`; the rest only read the flag, so
    /// producers do not contend on the flusher's park state.
    fn wake_flusher(&self) {
        if self.flusher_idle.load(Ordering::SeqCst)
            && self.flusher_idle.swap(false, Ordering::AcqRel)
            && let Some(thread) = &self.flusher_thread_handle
        {
            thread.unpark();
        }
    }

    /// Sets the global log level filter.
    pub fn set_filter(&self, level: u8) {
        self.filter_level.store(level, Ordering::Release);
    }

    /// Returns the current global log level filter.
    #[must_use]
    pub fn filter_level(&self) -> u8 {
        self.filter_level.load(Ordering::Relaxed)
    }

    /// Increments the format counter in the TUI metrics.
    ///
    /// The engine counts each event's format itself as it drains it;
    /// call this only for records that bypass [`Self::ingest`].
    pub fn inc_format(&self, format: crate::log_format::LogFormat) {
        self.metrics.inc_format(format);
    }

    /// Increments the active span count in the TUI metrics.
    pub fn inc_spans(&self) {
        self.metrics.inc_spans();
    }

    /// Decrements the active span count in the TUI metrics.
    pub fn dec_spans(&self) {
        self.metrics.dec_spans();
    }

    /// Returns the current number of active spans.
    #[must_use]
    pub fn active_spans(&self) -> usize {
        self.metrics.active_spans.load(Ordering::Relaxed)
    }

    /// Applies configuration settings to the engine.
    ///
    /// Sets the log level filter from the config. File sink construction
    /// and rotation are handled by the flusher thread at startup via
    /// [`PlatformSink::from_config`](crate::sink::PlatformSink::from_config).
    pub fn apply_config(&self, config: &crate::config::Config) {
        self.set_filter(config.log_level.to_numeric());
    }

    /// Safely halts the background thread, flushing pending logs.
    ///
    /// Signals the flusher thread to stop and waits for it to finish
    /// draining any remaining events from the queue.
    pub fn shutdown(&self) {
        self.shutdown_flag.store(true, Ordering::SeqCst);
        // Wake the flusher so it can drain and exit.
        if let Some(thread) = &self.flusher_thread_handle {
            thread.unpark();
        }
        if let Ok(mut guard) = self.flusher_join.lock()
            && let Some(handle) = guard.take()
        {
            let _ = handle.join();
        }
    }
}

/// Zero-Allocation Serializer Helper
#[derive(Debug, Clone, Copy)]
pub struct FastSerializer;

impl FastSerializer {
    /// Appends a u64 integer to a buffer using `itoa` without allocating a String.
    pub fn append_u64(buf: &mut Vec<u8>, val: u64) {
        let mut buffer = itoa::Buffer::new();
        buf.extend_from_slice(buffer.format(val).as_bytes());
    }

    /// Appends an f64 float to a buffer using `ryu` without allocating a String.
    pub fn append_f64(buf: &mut Vec<u8>, val: f64) {
        let mut buffer = ryu::Buffer::new();
        buf.extend_from_slice(buffer.format(val).as_bytes());
    }
}

#[cfg(test)]
#[cfg_attr(miri, allow(unused_imports))]
mod tests;
