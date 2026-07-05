// kani_proofs.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Kani model-checked proofs. Only compiled under `--cfg kani`,
// which the `cargo kani` invocation sets automatically.
//
// See `docs/adr/0004-kani-verified-invariants.md`.

use crate::log_level::LogLevel;
use std::sync::atomic::{AtomicU64, Ordering};

/// The 11 LogLevel variants map to numeric values 0..=10. Kani
/// exhaustively verifies that `from_numeric(to_numeric(x)) == Some(x)`
/// for every representable input via symbolic execution.
#[kani::proof]
fn from_numeric_round_trip_matches_to_numeric() {
    let disc: u8 = kani::any();
    kani::assume(disc <= 10);

    // Every value in [0, 10] must round-trip.
    let level = LogLevel::from_numeric(disc)
        .expect("in-range values must map to Some");
    let round = level.to_numeric();
    assert!(round == disc);
    assert!(round <= 10);
}

/// Values outside [0, 10] must produce `None`. This proves the
/// `from_numeric` guard clause is exhaustive.
#[kani::proof]
fn from_numeric_returns_none_for_out_of_range() {
    let disc: u8 = kani::any();
    kani::assume(disc > 10);

    let level = LogLevel::from_numeric(disc);
    assert!(level.is_none());
}

/// A monotonic counter's fetch_add produces distinct successive
/// values under any non-wraparound execution. This proves the
/// contract the `SESSION_COUNTER` in `crate::log` relies on.
///
/// Kani models `AtomicU64::fetch_add` faithfully for
/// single-threaded execution; the wraparound case is bounded away
/// by the assume clause because we care about the practical
/// invariant, not overflow behaviour.
#[kani::proof]
fn atomic_fetch_add_yields_distinct_ids() {
    let start: u64 = kani::any();
    kani::assume(start < u64::MAX - 1); // guard against wraparound

    let counter = AtomicU64::new(start);
    let a = counter.fetch_add(1, Ordering::AcqRel);
    let b = counter.fetch_add(1, Ordering::AcqRel);

    // Post-condition: two distinct, sequential IDs.
    assert!(a == start);
    assert!(b == start + 1);
    assert!(a != b);
    assert!(counter.load(Ordering::SeqCst) == start + 2);
}
