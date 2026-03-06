// engine.rs
// Brutalist Lock-Free Ingestion Engine

use crate::log_level::LogLevel;
use crate::sink::PlatformSink;
use crate::tui::{spawn_tui_thread, TuiMetrics};
use crossbeam_queue::ArrayQueue;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, LazyLock};
use std::thread;
use std::time::Duration;

/// A structured log event optimized for zero-allocation handoff.
#[derive(Debug, Clone)]
pub struct LogEvent {
    /// The log level severity.
    pub level: LogLevel,
    /// The numeric log level for filtering.
    pub level_num: u8,
    /// Pre-formatted or rapidly assembled buffer.
    pub payload: Vec<u8>,
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
    flusher_thread: Option<thread::JoinHandle<()>>,
}

impl fmt::Debug for LockFreeEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LockFreeEngine")
            .field("queue", &self.queue)
            .field("shutdown_flag", &self.shutdown_flag)
            .field("metrics", &self.metrics)
            .field("filter_level", &self.filter_level)
            .field("flusher_thread", &self.flusher_thread.as_ref().map(|h| h.thread().id()))
            .finish()
    }
}

use std::fmt;

/// Global lazy-initialized lock-free engine.
pub static ENGINE: LazyLock<LockFreeEngine> =
    LazyLock::new(|| LockFreeEngine::new(65536)); // Ring buffer size of 65k

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

        // Clone Arcs for the flusher thread
        let flusher_queue = queue.clone();
        let flusher_shutdown = shutdown_flag.clone();

        // Spawn lightweight OS thread (Runtime Agnostic)
        let handle = thread::Builder::new()
            .name("rlg-flusher".into())
            .spawn(move || {
                let mut sink = PlatformSink::native();

                loop {
                    // Batch drain: dequeue up to 64 events at a time
                    let mut batch: [Option<LogEvent>; 64] =
                        std::array::from_fn(|_| None);
                    let mut count = 0;
                    while count < 64 {
                        match flusher_queue.pop() {
                            Some(event) => {
                                batch[count] = Some(event);
                                count += 1;
                            }
                            None => break,
                        }
                    }
                    for event in batch.iter().flatten() {
                        sink.emit(event.level.as_str(), &event.payload);
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

        // Spawn the Generative TUI Dashboard Thread if enabled
        if std::env::var("RLG_TUI").map(|v| v == "1").unwrap_or(false) {
            spawn_tui_thread(metrics.clone(), shutdown_flag.clone());
        }

        Self {
            queue,
            shutdown_flag,
            metrics,
            filter_level,
            flusher_thread: Some(handle),
        }
    }

    /// Appends an event to the ring buffer.
    ///
    /// This function handles atomic metrics increments and buffer management.
    pub fn ingest(&self, event: LogEvent) {
        if event.level_num < self.filter_level.load(Ordering::Acquire) {
            return;
        }

        self.metrics.inc_events();

        if event.level_num >= LogLevel::ERROR.to_numeric() {
            self.metrics.inc_errors();
        }

        // If the buffer is full, pop oldest and retry once.
        if let Err(err) = self.queue.push(event) {
            let _ = self.queue.pop();
            let _ = self.queue.push(err);
        }

        // Wake the flusher thread for sub-microsecond latency.
        if let Some(handle) = &self.flusher_thread {
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
    pub fn shutdown(&self) {
        self.shutdown_flag.store(true, Ordering::SeqCst);
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
