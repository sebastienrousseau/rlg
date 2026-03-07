// engine.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Lock-free ingestion engine backed by a bounded ring buffer.

use crate::log_level::LogLevel;
use crate::sink::PlatformSink;
use crate::tui::{TuiMetrics, spawn_tui_thread};
use crossbeam_queue::ArrayQueue;
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::thread;
use std::time::Duration;

/// Capacity of the lock-free ring buffer (number of log events).
const RING_BUFFER_CAPACITY: usize = 65_536;

/// Maximum number of events drained per flusher wake-up cycle.
const MAX_DRAIN_BATCH_SIZE: usize = 64;

/// A structured log event optimized for zero-allocation handoff.
///
/// Formatting is deferred to the flusher thread — the caller only pays
/// the cost of a `Log` move (~128-byte memcpy), not serialization.
#[derive(Debug, Clone)]
pub struct LogEvent {
    /// The log level severity.
    pub level: LogLevel,
    /// The numeric log level for filtering.
    pub level_num: u8,
    /// Raw structured log data, formatted on the flusher thread.
    pub log: crate::log::Log,
}

/// The Lock-Free Engine handling background log flushes.
pub struct LockFreeEngine {
    /// Lock-free queue for log events.
    queue: Arc<ArrayQueue<LogEvent>>,
    /// Flag to signal shutdown.
    shutdown_flag: Arc<AtomicBool>,
    /// Metrics for the TUI dashboard.
    metrics: Arc<TuiMetrics>,
    /// Global log level filter.
    filter_level: AtomicU8,
    /// Handle to the background flusher thread.
    flusher_thread: Mutex<Option<thread::JoinHandle<()>>>,
}

impl fmt::Debug for LockFreeEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LockFreeEngine")
            .field("queue", &self.queue)
            .field("shutdown_flag", &self.shutdown_flag)
            .field("metrics", &self.metrics)
            .field("filter_level", &self.filter_level)
            .field(
                "flusher_thread",
                &self
                    .flusher_thread
                    .lock()
                    .ok()
                    .and_then(|g| g.as_ref().map(|h| h.thread().id())),
            )
            .finish()
    }
}

/// Global lazy-initialized lock-free engine.
pub static ENGINE: LazyLock<LockFreeEngine> =
    LazyLock::new(|| LockFreeEngine::new(RING_BUFFER_CAPACITY));

impl LockFreeEngine {
    /// Initializes the lock-free engine and spawns the background flusher.
    ///
    /// # Panics
    ///
    /// This function panics if the flusher background thread fails to spawn.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let queue = Arc::new(ArrayQueue::new(capacity));
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let metrics = Arc::new(TuiMetrics::default());
        let filter_level = AtomicU8::new(0); // Default to ALL

        // Under MIRI, skip spawning background threads to avoid
        // "main thread terminated without waiting" errors.
        #[cfg(not(miri))]
        let flusher_handle = {
            let flusher_queue = queue.clone();
            let flusher_shutdown = shutdown_flag.clone();

            // Spawn lightweight OS thread (Runtime Agnostic)
            let handle = thread::Builder::new()
                .name("rlg-flusher".into())
                .spawn(move || {
                    use std::io::Write;
                    let mut sink = PlatformSink::native();
                    let mut fmt_buf = Vec::with_capacity(512);

                    loop {
                        let mut batch: [Option<LogEvent>;
                            MAX_DRAIN_BATCH_SIZE] =
                            std::array::from_fn(|_| None);
                        let mut count = 0;
                        while count < MAX_DRAIN_BATCH_SIZE {
                            match flusher_queue.pop() {
                                Some(event) => {
                                    batch[count] = Some(event);
                                    count += 1;
                                }
                                None => break,
                            }
                        }
                        for event in batch.iter().flatten() {
                            fmt_buf.clear();
                            let _ =
                                writeln!(fmt_buf, "{}", &event.log);
                            sink.emit(event.level.as_str(), &fmt_buf);
                        }

                        if flusher_shutdown.load(Ordering::Relaxed)
                            && flusher_queue.is_empty()
                        {
                            break;
                        }

                        // Park briefly as fallback; real wakeup comes from unpark() in ingest().
                        thread::park_timeout(Duration::from_millis(5));
                    }
                })
                .expect("Failed to spawn rlg-flusher background thread");

            // Spawn the TUI dashboard thread if RLG_TUI=1
            if std::env::var("RLG_TUI")
                .map(|v| v == "1")
                .unwrap_or(false)
            {
                spawn_tui_thread(metrics.clone(), shutdown_flag.clone());
            }

            Some(handle)
        };

        #[cfg(miri)]
        let flusher_handle = None;

        Self {
            queue,
            shutdown_flag,
            metrics,
            filter_level,
            flusher_thread: Mutex::new(flusher_handle),
        }
    }

    /// Appends an event to the ring buffer.
    ///
    /// If the buffer is full, the oldest event is evicted to make room.
    /// Dropped events are tracked via `TuiMetrics::dropped_events`.
    pub fn ingest(&self, event: LogEvent) {
        if event.level_num < self.filter_level.load(Ordering::Acquire) {
            return;
        }

        self.metrics.inc_events();
        self.metrics.inc_level(event.level);

        if event.level_num >= LogLevel::ERROR.to_numeric() {
            self.metrics.inc_errors();
        }

        // If the buffer is full, evict the oldest event and retry.
        if let Err(err) = self.queue.push(event) {
            let _ = self.queue.pop();
            self.metrics.inc_dropped();
            let _ = self.queue.push(err);
        }

        // Wake the flusher thread for sub-microsecond latency.
        if let Ok(guard) = self.flusher_thread.lock()
            && let Some(handle) = guard.as_ref()
        {
            handle.thread().unpark();
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
    pub fn apply_config(&self, config: &crate::config::Config) {
        self.set_filter(config.log_level.to_numeric());
    }

    /// Safely halts the background thread, flushing pending logs.
    ///
    /// Signals the flusher thread to stop and waits for it to finish
    /// draining any remaining events from the queue.
    pub fn shutdown(&self) {
        self.shutdown_flag.store(true, Ordering::SeqCst);
        if let Ok(mut guard) = self.flusher_thread.lock()
            && let Some(handle) = guard.take()
        {
            handle.thread().unpark();
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
