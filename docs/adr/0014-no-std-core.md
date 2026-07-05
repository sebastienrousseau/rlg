<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR 0014 — `no_std` Core (Phase 23: strategy + gate)

- **Status:** Accepted
- **Date:** 2026-07-05
- **Phase:** 23 (per `docs/IMPLEMENTATION-PLAN-v0.1.0.md`)
- **Deciders:** repository maintainers

## Context

Embedded and IoT deployments running on Cortex-M, RISC-V, and
similar targets need structured logging. `defmt` owns that
segment today. rlg cannot reach it because the flagship crate
depends unconditionally on `std::fs`, `std::thread`,
`std::time::Instant`, and other host services.

The plan called for a `no_std` + `alloc` mode covering the
type-only surface (`Log`, `LogFormat`, `LogLevel`, their
`Display` impls) — enough for a `defmt`-adjacent embedded
adopter to render a rlg record on-device, ship the bytes over a
transport of their choosing, and reassemble host-side.

## Decision

Phase 23 lands the strategy document, the target-matrix gate,
and the manifest structure. The actual `#[cfg(feature = "std")]`
gating across the source tree is Phase 23.1.

Landing the strategy alone is a real deliverable — it commits
the workspace to a concrete target list, an MSRV contract for
embedded targets, and a review checklist for any new dep bump.
Phase 23.1 executes the mechanical `#[cfg]` sprinkle.

### In scope for eventual `no_std` compilation

- `crate::log_level::LogLevel` — plain enum, `Display`, `FromStr`.
- `crate::log_format::LogFormat` — plain enum, 14 variants,
  `Display`, `FromStr`.
- `crate::log::Log` — struct + fluent builder, `Display`
  dispatched per format. Uses `Cow<'static, str>` + `String`
  (via `alloc`).
- `crate::error::RlgError` — thiserror-derived, no I/O.

### Explicitly out of scope

- `crate::engine::LockFreeEngine` — spawns OS threads, requires
  `std::sync::Mutex`.
- `crate::sink::PlatformSink` — every variant hits an OS
  primitive.
- `crate::config::Config` — uses `std::fs`, TOML load.
- `crate::init::init()` — global engine bootstrap.
- `crate::rotation::RotatingFile` — file I/O.
- `crate::tui` — terminal I/O.
- `crate::tracing::RlgSubscriber` — thread-local state.

### Target matrix (Phase 23.1 CI addition)

- `thumbv7em-none-eabihf` — Cortex-M4F, no_std sanity check.
- `riscv32imac-unknown-none-elf` — RISC-V 32-bit, no_std sanity
  check.
- `x86_64-unknown-linux-gnu` (default) — std baseline unchanged.

## What Phase 23 does NOT deliver

- The `default = ["std"]` split. Requires touching every module.
- The Cortex-M4 demo crate. Requires QEMU setup in CI.
- The MSRV bump justification if `no_std` requires nightly
  features.

## Why scope this way

Same reason Phases 19c, 20, 21, 22 shipped scaffolds first:
mechanical refactor work is safer done in isolation, with the
strategy contract already in place to review against. Phase 23.1
executes against a fixed target — no drift, no re-scoping mid-
refactor.

## Alternatives considered

- **Skip `no_std` entirely.** Rejected — `defmt` owns the
  embedded segment today; not reaching for it foreclos on a
  first-class deployment target.
- **Ship `default = ["std"]` in one commit.** Rejected on
  CI-risk grounds. Every `use std::` becomes a candidate for
  `use alloc::` or `use core::`; the diff would touch every
  module and iterate on clippy/miri/loom feedback for hours.
- **`no_std + alloc` on the whole crate.** Rejected — the
  engine, sinks, config, rotation, and TUI legitimately require
  `std`. Splitting them into separate crates is a v0.2.0 concern.

## Phase 23.1 execution checklist

1. Add `default = ["std"]` to `crates/rlg/Cargo.toml`.
2. Add `std = []` feature.
3. `#[cfg_attr(not(feature = "std"), no_std)]` at crate root.
4. `extern crate alloc;` under the same cfg.
5. Feature-gate every std-touching module with
   `#[cfg(feature = "std")]`.
6. CI matrix: add `cargo check --no-default-features --target
   thumbv7em-none-eabihf` and RISC-V equivalent.
7. Docs: update `crates/rlg/README.md` with an "Embedded /
   `no_std`" section.

## References

- [The Rust Embedded Book](https://docs.rust-embedded.org/book/)
- [`defmt` project](https://github.com/knurling-rs/defmt) —
  incumbent embedded logging framework.
- [`no_std` chapter of the Rust book](https://doc.rust-lang.org/cargo/reference/features.html#the-default-feature)
