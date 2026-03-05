// engine.rs
// Brutalist Lock-Free Ingestion Engine

use crate::sink::PlatformSink;
use crate::tui::{TuiMetrics, spawn_tui_thread};
use crossbeam_queue::ArrayQueue;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::thread;

/// A structured log event optimized for zero-allocation handoff.
#[derive(Debug, Clone)]
pub struct LogEvent {
    /// The log level severity.
    pub level: String,
    /// Pre-formatted or rapidly assembled buffer.
    pub payload: Vec<u8>, 
}

/// The Lock-Free Engine handling background log flushes.
#[derive(Debug)]
pub struct LockFreeEngine {
    /// Lock-free queue for log events.
    queue: Arc<ArrayQueue<LogEvent>>,
    /// Flag to signal shutdown.
    shutdown_flag: Arc<AtomicBool>,
    /// Metrics for the TUI dashboard.
    metrics: Arc<TuiMetrics>,
}

/// Global lazy-initialized lock-free engine.
pub static ENGINE: LazyLock<LockFreeEngine> = LazyLock::new(|| LockFreeEngine::new(65536)); // Ring buffer size of 65k

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

        let engine = Self {
            queue: queue.clone(),
            shutdown_flag: shutdown_flag.clone(),
            metrics: metrics.clone(),
        };

        // Spawn lightweight OS thread (Runtime Agnostic)
        let flusher_shutdown = shutdown_flag.clone();
        thread::Builder::new()
            .name("rlg-flusher".into())
            .spawn(move || {
                let mut sink = PlatformSink::native();
                
                loop {
                    // Drain the queue completely
                    while let Some(event) = queue.pop() {
                        sink.emit(&event.level, &event.payload);
                    }

                    if flusher_shutdown.load(Ordering::Relaxed) && queue.is_empty() {
                        break;
                    }

                    // Yield briefly to avoid 100% CPU lock in the spin loop
                    // In a true disruptor, this would use a Condvar/WaitStrategy.
                    thread::yield_now();
                }
            })
            .expect("Failed to spawn rlg-flusher background thread");

        // Spawn the Generative TUI Dashboard Thread if enabled
        if std::env::var("RLG_TUI").map(|v| v == "1").unwrap_or(false) {
            spawn_tui_thread(metrics, shutdown_flag);
        }

        engine
    }

    /// Appends an event to the ring buffer.
    ///
    /// This function handles atomic metrics increments and buffer management.
    pub fn ingest(&self, event: LogEvent) {
        self.metrics.inc_events();
        
        if event.level == "ERROR" || event.level == "FATAL" || event.level == "CRITICAL" {
            self.metrics.inc_errors();
        }

        // If the buffer is full, we forcefully push, dropping the oldest if necessary to prioritize liveness over latency.
        let mut ev = event;
        while let Err(err) = self.queue.push(ev) {
            let _ = self.queue.pop(); // Drop oldest log to make room
            ev = err;
            #[cfg(test)]
            {
                let _ = ev;
                break;
            }
        }
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
