<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR 0001 — Loom-Verified Shutdown Handshake

- **Status:** Accepted
- **Date:** 2026-07-04
- **Phase:** 10 (per `docs/IMPLEMENTATION-PLAN-v0.1.0.md`)
- **Deciders:** repository maintainers
- **Related:** ADR 0009 (sharded producer queue) — future work that
  will re-verify against these proofs.

## Context

`rlg::engine::LockFreeEngine` uses a bounded ring buffer
(`crossbeam-queue::ArrayQueue`) as the producer/consumer conduit
between application threads and a single dedicated flusher thread.
The concurrent contract we care about — and that no amount of unit
testing will exhaustively prove — is:

1. **Every event pushed by a producer is eventually observed by the
   flusher.** No memory-ordering interleaving may cause a push to be
   invisible to a subsequent drain-and-check-empty pass.
2. **`shutdown()` drains all in-flight events before returning.** If
   producers have completed their `ingest()` calls before
   `shutdown()` is called, the flusher's drain loop terminates only
   after the queue is empty.
3. **`session_id: u64` monotonicity holds under concurrent producers.**
   Two producers calling `fetch_add(1, AcqRel)` on a shared counter
   never observe the same value.

Unit tests can demonstrate the *happy path* for each. They cannot
exhaustively enumerate every scheduler interleaving.

## Decision

Adopt Loom (0.7) as the exhaustive concurrency-model checker for
these three invariants. Author the proofs as a standalone integration
test file at `crates/rlg/tests/loom_engine.rs`, guarded by
`#![cfg(loom)]` so it never compiles into the standard `cargo test`
runs and does not affect ordinary contributor workflows.

CI job `.github/workflows/loom.yml` runs the proofs with
`RUSTFLAGS="--cfg loom"` on every PR that touches the engine, the
proofs themselves, or the Cargo manifest.

## Model faithfulness

Loom exhaustively explores interleavings of *its own* atomic and
threading primitives. Our proofs model:

- **The queue** — `Mutex<Vec<u32>>` stands in for `ArrayQueue`. Both
  are bounded FIFOs with atomic `pop` / `push` semantics. Modelling
  `ArrayQueue` directly would double-cover the invariants that
  `crossbeam-queue` already verifies upstream; using a simpler
  substitute focuses Loom on the *surrounding handshake* — the
  atomic shutdown flag and the drain-until-empty loop — which is
  rlg's own code.
- **The shutdown flag** — `AtomicBool` used with `Release` on the
  store and `Acquire` on the flusher's load, matching the real
  engine's ordering.
- **The drain-until-empty pattern** — flusher pops until empty, then
  loads the shutdown flag (Acquire); if set, re-checks the queue
  (this second check is critical to the safety proof) and only
  terminates if both conditions hold.

## What is proven

- **`proof_no_events_lost_single_producer`** — one producer,
  two events, then shutdown. Flusher observes exactly 2 events under
  every interleaving.
- **`proof_no_events_lost_multi_producer`** — two producers, one
  event each, then external shutdown. Flusher observes exactly 2
  events under every interleaving.
- **`proof_session_id_monotonicity_under_concurrent_producers`** —
  two producers `fetch_add` on a shared `AtomicU64`. Results are
  distinct and post-fetch counter equals 2, under every interleaving.

## What is not proven

- The behaviour of `crossbeam-queue::ArrayQueue` itself — trusted
  upstream, verified separately by that crate.
- The behaviour of `std::thread::park` / `unpark` — Loom's shims for
  park do not perfectly match std's semantics (spurious wakes,
  timeout coalescing). Our proofs use the shutdown-flag +
  drain-check pattern instead of park to model the wake condition,
  which is a strictly weaker (i.e. more pessimistic) coverage that
  cannot false-positive.
- Interactions with the TUI thread (opt-in behind `RLG_TUI=1`) —
  out of scope for the engine's core contract.
- The scenario where a producer starts an `ingest()` call *after*
  `shutdown()` has been observed by the flusher. The engine's
  documented API contract is that `shutdown()` drains events pushed
  *before* the shutdown was signalled; overlapping producers are the
  caller's contract to prevent.

## Consequences

- **CI cost.** ~5 min added on the Loom job. Cancels in-progress runs
  on the same ref; bounded by `LOOM_MAX_PREEMPTIONS=3` and
  `LOOM_MAX_BRANCHES=200000`.
- **Contributor cost.** Local reproducer:
  ```bash
  RUSTFLAGS="--cfg loom" cargo test --release --test loom_engine -p rlg
  ```
  Documented in `CONTRIBUTING.md`.
- **Refactor gate.** Phase 18 (sharded producer queue) will replace
  `ArrayQueue` with `rtrb` behind a `fast-queue` feature. The Loom
  proofs will be extended to cover the new queue variant *before*
  it becomes default. This ADR is the contract that gate must meet.

## Alternatives considered

- **Refactor `engine.rs` to use `loom::sync` shims conditionally** —
  the standard pattern for full Loom coverage of a production module.
  Rejected for Phase 10 because it materially widens the diff and
  introduces a `cfg(loom)` fork in the hot path. Adopted in Phase
  10.1 (planned) once the standalone proofs stabilise.
- **TLA+ / Coq spec** — over-budget for v0.1.0 (see plan §6, "Out of
  scope"). Kani (Phase 13) covers the subset of invariants amenable
  to bounded model checking.

## References

- [Loom 0.7 docs](https://docs.rs/loom)
- Tokio's Loom-tested runtime primitives, upstream reference:
  <https://github.com/tokio-rs/tokio/tree/master/tokio/src/loom>
- Crossbeam's own Loom coverage of `ArrayQueue`:
  <https://github.com/crossbeam-rs/crossbeam/blob/master/crossbeam-queue/tests>
