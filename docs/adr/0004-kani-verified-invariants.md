<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR 0004 — Kani-Verified Invariants

- **Status:** Accepted
- **Date:** 2026-07-05
- **Phase:** 13 (per `docs/IMPLEMENTATION-PLAN-v0.1.0.md`)
- **Deciders:** repository maintainers
- **Related:** ADR 0001 (Loom-verified ring buffer) — same
  invariant surface, orthogonal exploration.
  ADR 0003 (property tests) — same invariants, weaker (statistical)
  exploration.

## Context

Three narrow invariants underpin correctness of the workspace's
public surface:

1. **`LogLevel::from_numeric` and `to_numeric` are inverses on
   `[0, 10]`.** Every downstream comparison, filter, and
   deserialisation depends on this bijection.
2. **`LogLevel::from_numeric` returns `None` for values outside
   `[0, 10]`.** No silent fallback, no wrap.
3. **The session-ID counter's `fetch_add(1, AcqRel)` produces
   distinct successive values.** Downstream aggregation
   (`rlg-mcp::filter_log`, `rlg-report`) trusts this.

Proptests (ADR 0003) validate these statistically. Kani proves
them **exhaustively** by symbolic execution — every representable
`u8` for the numeric bijection, every valid start value for the
counter — in ~seconds per proof.

## Decision

Adopt Kani (0.55+) as the model-checked prover for these three
invariants. Author the harnesses in a `#[cfg(kani)]`-gated module
at `crates/rlg/src/kani_proofs.rs`, wired from `lib.rs` with
`#[cfg(kani)] mod kani_proofs;`. `cargo kani` sets `--cfg kani`
automatically; standard builds never compile the module.

Kani runs via the official `model-checking/kani-github-action@v1`
GHA action on:

- **Push to `main`** — verifies every merge that touches invariant
  surfaces.
- **Weekly cron (Monday 06:00 UTC)** — catches regressions in
  Kani's own upstream (nightly-tracked model checker).
- **`workflow_dispatch`** — on-demand for maintainers.

Kani is *not* run per-PR. It is heavyweight (~10 min per proof
in current sizing), and its guarantees do not accrete faster than
per-merge. Miri, Loom, proptest, and semver-checks carry the per-PR
correctness surface.

## The three proofs

- **`from_numeric_round_trip_matches_to_numeric`** — for every
  `disc: u8` with `disc <= 10`,
  `LogLevel::from_numeric(disc).unwrap().to_numeric() == disc`.
  Proves the bijection.

- **`from_numeric_returns_none_for_out_of_range`** — for every
  `disc: u8` with `disc > 10`, `LogLevel::from_numeric(disc)` is
  `None`. Proves the guard clause is exhaustive.

- **`atomic_fetch_add_yields_distinct_ids`** — for any `start: u64`
  bounded away from `u64::MAX`, two successive
  `AtomicU64::fetch_add(1, Ordering::AcqRel)` calls yield
  `(start, start + 1)` and the post-fetch counter equals
  `start + 2`. Proves the monotonicity contract the session
  counter relies on.

## What Kani does NOT cover here

- **Ring-buffer concurrency.** Loom (ADR 0001) covers producer /
  flusher interleavings under exhaustive scheduler exploration.
  Kani's concurrency model is single-threaded — the atomic proof
  above is sequential-only.
- **`u64::MAX` wraparound.** Bounded away by `kani::assume`. The
  practical invariant is what matters; wraparound is unreachable
  at ~500-year fetch_add rates.
- **String parsing.** `LogLevel::from_str` involves
  `to_uppercase()` allocation, which Kani struggles to model.
  Property tests (ADR 0003) carry that coverage.
- **`ArrayQueue` push semantics.** Third-party trusted; verified
  upstream by `crossbeam-queue`'s own test suite.

## Consequences

- **CI cost.** Weekly + on-merge. Two Kani jobs at ~10 min each
  = ~20 min per week. Negligible.
- **Contributor cost.** Local reproducer:
  ```bash
  cargo install --locked kani-verifier
  cargo kani setup
  cd crates/rlg && cargo kani --tests
  ```
  Documented in `CONTRIBUTING.md`.
- **Toolchain pinning.** Kani ships its own rustc build. This is
  contained to the `kani` job; the rest of CI runs on stable.
- **False positives.** Kani occasionally reports issues from
  upstream (nightly-tracked). Weekly cron catches drift; failures
  file GitHub issues automatically per the action's default.

## Alternatives considered

- **Prusti / Creusot** — richer contract language but weaker
  ergonomics on stable Rust. Rejected for v0.1.0.
- **TLA+ spec of the ring buffer** — over-budget for v0.1.0 (see
  plan §6, "Out of scope"). Loom provides the practical guarantee.
- **Skip Kani entirely** — proptest gives good statistical
  coverage. But the numeric bijection is trivially amenable to
  exhaustive proof, and shipping "verified" as a workspace claim
  requires an actual verifier in the loop. Kani is that verifier.

## References

- [Kani book](https://model-checking.github.io/kani/)
- Kani's `AtomicU64` support matrix:
  <https://model-checking.github.io/kani/rust-feature-support.html>
- ADR 0001 (Loom) and ADR 0003 (proptest) in this directory.
