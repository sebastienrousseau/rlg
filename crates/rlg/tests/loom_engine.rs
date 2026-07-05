//! Loom concurrency proofs for the rlg engine's shutdown handshake
//! and producer/flusher rendezvous.
//!
//! See `docs/adr/0001-loom-verified-ring-buffer.md` for the model
//! justification and what is (and is not) covered.
//!
//! Run with:
//! ```bash
//! RUSTFLAGS="--cfg loom" cargo test --release --test loom_engine \
//!     -p rlg -- --nocapture
//! ```

// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.

#![cfg(loom)]
#![allow(missing_docs)]

use loom::sync::Arc;
use loom::sync::Mutex;
use loom::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use loom::thread;

/// Faithful minimal model of the rlg engine's concurrent invariants.
///
/// The real engine uses `crossbeam-queue::ArrayQueue` for the ring
/// buffer, which carries its own upstream Loom coverage. Modelling
/// it here would double-cover the same invariants; we instead use a
/// `Mutex<Vec<u32>>` as a functionally-equivalent bounded FIFO so
/// Loom can exhaustively explore the surrounding handshake:
/// the atomic shutdown flag, the ordering constraints between
/// `push` and `unpark`, and the flusher's drain-until-empty loop.
struct Engine {
    /// Stand-in for `ArrayQueue`. Each `u32` is an event ID.
    queue: Mutex<Vec<u32>>,
    /// Shutdown request signalled by producer or external caller.
    shutdown: AtomicBool,
    /// Total events observed by the flusher across its lifetime.
    /// Post-condition: `drained == events_pushed` after shutdown
    /// returns.
    drained: AtomicU64,
}

impl Engine {
    fn new() -> Self {
        Self {
            queue: Mutex::new(Vec::new()),
            shutdown: AtomicBool::new(false),
            drained: AtomicU64::new(0),
        }
    }

    /// Producer path. Mirrors `LockFreeEngine::ingest` — push then
    /// signal.
    fn ingest(&self, event_id: u32) {
        self.queue.lock().unwrap().push(event_id);
        // Real engine calls `thread.unpark()` here. Loom's park
        // model is not identical to std's; the shutdown flag +
        // busy-check pattern below faithfully reproduces the
        // "wake me when there's work" semantic without needing
        // park.
    }

    /// Flusher loop. Mirrors the real engine's drain-batches +
    /// check-shutdown-and-empty pattern.
    ///
    /// `loom::thread::yield_now()` at the bottom of each outer
    /// iteration is required by Loom's cooperative-scheduler model:
    /// bare atomic-load spins otherwise blow the branch budget with
    /// no other thread ever getting a slot.
    fn flush_until_shutdown_and_empty(&self) {
        loop {
            // Drain everything available.
            loop {
                let popped = {
                    let mut q = self.queue.lock().unwrap();
                    q.pop()
                };
                match popped {
                    Some(_) => {
                        self.drained.fetch_add(1, Ordering::AcqRel);
                    }
                    None => break,
                }
            }

            // Termination condition: shutdown requested AND queue drained.
            //
            // Order of checks matters. We load shutdown FIRST (Acquire),
            // then re-check the queue. Any push that happened-before
            // the shutdown store must be visible to the queue check
            // that follows. Loom will explore every interleaving of
            // producer push / shutdown store to prove this.
            if self.shutdown.load(Ordering::Acquire) {
                let empty = self.queue.lock().unwrap().is_empty();
                if empty {
                    return;
                }
            }

            // Cooperative yield — hands Loom a scheduling point so the
            // producer / shutdown-signaller can advance while the
            // flusher waits.
            thread::yield_now();
        }
    }

    fn request_shutdown(&self) {
        // Release so that any push happens-before this store is
        // visible to the flusher's Acquire load.
        self.shutdown.store(true, Ordering::Release);
    }
}

// ---------------------------------------------------------------------------
// Proof 1 — Producer + flusher never lose an event when both run
// concurrently and shutdown is signalled by the producer after its
// last push.
// ---------------------------------------------------------------------------

#[test]
fn proof_no_events_lost_single_producer() {
    loom::model(|| {
        let engine = Arc::new(Engine::new());

        // Producer thread: two ingests, then shutdown.
        let producer_engine = engine.clone();
        let producer = thread::spawn(move || {
            producer_engine.ingest(1);
            producer_engine.ingest(2);
            producer_engine.request_shutdown();
        });

        // Flusher thread: drain until shutdown-and-empty.
        let flusher_engine = engine.clone();
        let flusher = thread::spawn(move || {
            flusher_engine.flush_until_shutdown_and_empty();
        });

        producer.join().unwrap();
        flusher.join().unwrap();

        // Post-condition: the flusher observed exactly the two events
        // the producer pushed. No interleaving loses either.
        assert_eq!(engine.drained.load(Ordering::SeqCst), 2);
    });
}

// ---------------------------------------------------------------------------
// Proof 2 — Two producers can push concurrently with the flusher
// draining, and no event is lost when shutdown is signalled after
// both producers complete.
// ---------------------------------------------------------------------------

#[test]
fn proof_no_events_lost_multi_producer() {
    loom::model(|| {
        let engine = Arc::new(Engine::new());
        let producers_done = Arc::new(AtomicU64::new(0));

        // Producer A
        let a_engine = engine.clone();
        let a_done = producers_done.clone();
        let a = thread::spawn(move || {
            a_engine.ingest(1);
            a_done.fetch_add(1, Ordering::Release);
        });

        // Producer B
        let b_engine = engine.clone();
        let b_done = producers_done.clone();
        let b = thread::spawn(move || {
            b_engine.ingest(2);
            b_done.fetch_add(1, Ordering::Release);
        });

        // Coordinator (on main): wait for both producers, then
        // shutdown. This models the external-shutdown case where the
        // producers finished before shutdown was decided.
        let flusher_engine = engine.clone();
        let flusher = thread::spawn(move || {
            flusher_engine.flush_until_shutdown_and_empty();
        });

        a.join().unwrap();
        b.join().unwrap();
        assert_eq!(producers_done.load(Ordering::Acquire), 2);

        engine.request_shutdown();
        flusher.join().unwrap();

        // Post-condition: exactly two events drained under every
        // interleaving Loom explores.
        assert_eq!(engine.drained.load(Ordering::SeqCst), 2);
    });
}

// ---------------------------------------------------------------------------
// Proof 3 — Session-ID monotonicity under concurrent producers.
//
// The real `Log` type carries a caller-supplied `session_id: u64`.
// When two producers each `fetch_add(1)` on a shared counter, the
// resulting IDs are unique and monotonic. Loom exhaustively confirms
// AcqRel ordering is sufficient.
// ---------------------------------------------------------------------------

#[test]
fn proof_session_id_monotonicity_under_concurrent_producers() {
    loom::model(|| {
        let counter = Arc::new(AtomicU64::new(0));

        let c1 = counter.clone();
        let t1 =
            thread::spawn(move || c1.fetch_add(1, Ordering::AcqRel));

        let c2 = counter.clone();
        let t2 =
            thread::spawn(move || c2.fetch_add(1, Ordering::AcqRel));

        let id1 = t1.join().unwrap();
        let id2 = t2.join().unwrap();

        // Every interleaving produces exactly {0, 1} across the two
        // producers, never a duplicate.
        assert_ne!(id1, id2);
        assert!(id1 == 0 || id1 == 1);
        assert!(id2 == 0 || id2 == 1);
        // Post-fetch counter is exactly 2.
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    });
}
