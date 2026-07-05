// sharded_queue.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Sharded producer queue backing the engine's ring buffer.
//!
//! Wraps `N` `crossbeam-queue::ArrayQueue<LogEvent>` shards behind a
//! single facade with the same `push` / `pop` / `is_empty` surface
//! `crate::engine::LockFreeEngine` was previously using directly.
//!
//! `N` is a compile-time constant chosen by the feature set:
//!
//! - Default: **1 shard** — semantically identical to the pre-Phase-18
//!   direct `ArrayQueue` use. Zero overhead for the single-producer /
//!   single-thread case.
//! - `fast-queue` feature enabled: **8 shards** — reduces producer-side
//!   cache-line contention on the underlying atomic tag when many
//!   threads ingest concurrently.
//!
//! Producers pick a shard once per thread via a thread-local index
//! assigned round-robin from a shared `AtomicUsize` counter. Sticky
//! per-thread assignment amortises the shard-selection cost to zero
//! past the first `push` call from a given thread.
//!
//! The flusher drains all shards in rotation on every wake.
//!
//! See `docs/adr/0009-sharded-producer-queue.md`.

use crate::engine::LogEvent;
use crossbeam_queue::ArrayQueue;
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Number of shards. Compile-time constant.
#[cfg(feature = "fast-queue")]
pub(crate) const SHARD_COUNT: usize = 8;

/// Number of shards. Compile-time constant.
#[cfg(not(feature = "fast-queue"))]
pub(crate) const SHARD_COUNT: usize = 1;

// Compile-time invariant: at least one shard.
const _: () = assert!(SHARD_COUNT > 0, "SHARD_COUNT must be > 0");

/// Round-robin counter used to assign a sticky shard index to each
/// producer thread on first `push` call.
static NEXT_SHARD: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    /// Sticky shard index for this thread. Lazily initialised on
    /// first `push` call — round-robin assignment via `NEXT_SHARD`.
    static SHARD_INDEX: Cell<Option<usize>> = const { Cell::new(None) };
}

fn thread_shard() -> usize {
    SHARD_INDEX.with(|slot| {
        slot.get().unwrap_or_else(|| {
            let idx = NEXT_SHARD.fetch_add(1, Ordering::Relaxed)
                % SHARD_COUNT;
            slot.set(Some(idx));
            idx
        })
    })
}

/// Sharded bounded queue with the minimal `push` / `pop` / `is_empty`
/// surface that `LockFreeEngine` requires.
#[derive(Debug)]
pub(crate) struct ShardedQueue {
    shards: Box<[ArrayQueue<LogEvent>]>,
}

impl ShardedQueue {
    /// Construct a sharded queue with an overall capacity of
    /// `total_capacity` events, split evenly across [`SHARD_COUNT`]
    /// shards. Per-shard capacity is `total_capacity / SHARD_COUNT`,
    /// with any remainder distributed to the first shards.
    pub(crate) fn new(total_capacity: usize) -> Self {
        let base = total_capacity / SHARD_COUNT;
        let remainder = total_capacity % SHARD_COUNT;
        let shards = (0..SHARD_COUNT)
            .map(|i| {
                let cap = base + usize::from(i < remainder);
                // ArrayQueue requires capacity >= 1; guard against
                // pathological `total_capacity < SHARD_COUNT`.
                ArrayQueue::new(cap.max(1))
            })
            .collect::<Box<[_]>>();
        Self { shards }
    }

    /// Push an event. Each producer thread has a sticky shard index
    /// assigned round-robin on first call, so subsequent pushes from
    /// the same thread hit the same shard with no contention on the
    /// shard-selection path.
    ///
    /// # Errors
    /// Returns the event back to the caller if the thread-local
    /// shard is at capacity — the same contract as
    /// [`crossbeam_queue::ArrayQueue::push`].
    pub(crate) fn push(&self, event: LogEvent) -> Result<(), LogEvent> {
        self.shards[thread_shard()].push(event)
    }

    /// Pop from the caller's thread-local shard. Used only by the
    /// retry-eviction path in `LockFreeEngine::ingest` so the pop
    /// happens on the same shard as the failed push.
    pub(crate) fn pop_local(&self) -> Option<LogEvent> {
        self.shards[thread_shard()].pop()
    }

    /// Pop any available event across all shards. Called by the
    /// flusher thread's drain loop.
    ///
    /// `cfg_attr(miri, allow(dead_code))` — the flusher spawn path
    /// in `engine.rs` is `#[cfg(not(miri))]`, so under Miri nothing
    /// calls this and the dead-code lint fires. The engine still
    /// constructs a `ShardedQueue` under Miri (the constructor is
    /// exercised), so the method must exist; only the lint needs
    /// suppressing.
    #[cfg_attr(miri, allow(dead_code))]
    pub(crate) fn pop(&self) -> Option<LogEvent> {
        for shard in &self.shards {
            if let Some(event) = shard.pop() {
                return Some(event);
            }
        }
        None
    }

    /// True when every shard is empty. See [`Self::pop`] for the
    /// `cfg_attr(miri, allow(dead_code))` rationale.
    #[cfg_attr(miri, allow(dead_code))]
    pub(crate) fn is_empty(&self) -> bool {
        self.shards.iter().all(ArrayQueue::is_empty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LogLevel;
    use crate::log::Log;

    fn make_event(level: LogLevel) -> LogEvent {
        LogEvent {
            level,
            level_num: level.to_numeric(),
            log: Log::info("test"),
        }
    }

    #[test]
    fn empty_reports_no_events() {
        let q = ShardedQueue::new(16);
        assert!(q.is_empty());
    }

    #[test]
    fn push_then_pop_round_trips() {
        let q = ShardedQueue::new(8);
        q.push(make_event(LogLevel::INFO)).unwrap();
        assert!(!q.is_empty());
        let e = q.pop().unwrap();
        assert_eq!(e.level, LogLevel::INFO);
        assert!(q.is_empty());
    }

    #[test]
    #[cfg_attr(miri, ignore)] // spawns threads
    fn concurrent_producers_share_shards() {
        use std::sync::Arc;
        use std::thread;

        let q = Arc::new(ShardedQueue::new(1024));
        let mut handles = Vec::new();
        for _ in 0..4 {
            let q = q.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..64 {
                    let _ = q.push(make_event(LogLevel::INFO));
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        let mut drained = 0;
        while q.pop().is_some() {
            drained += 1;
        }
        assert_eq!(drained, 4 * 64);
    }
}
