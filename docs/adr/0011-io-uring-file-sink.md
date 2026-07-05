<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR 0011 — io_uring File Sink (Phase 20: scaffold)

- **Status:** Accepted
- **Date:** 2026-07-05
- **Phase:** 20 (per `docs/IMPLEMENTATION-PLAN-v0.1.0.md`)
- **Deciders:** repository maintainers
- **Related:** ADR 0010 (OTLP pluggable transport) — same
  scaffold-then-fill pattern used for the gRPC transport.

## Context

Linux 5.1+ ships `io_uring`, a submission-queue/completion-queue
async I/O interface that eliminates the per-syscall context-switch
overhead of `write(2)` for high-throughput writers. rlg's file
sink writes formatted log payloads through
`std::fs::File::write_all`, which is one `write(2)` syscall per
payload — fine at 10 k events/sec, throughput-capped at 500 k+.

Enterprise adopters targeting Linux want the io_uring path. The
plan called for a new `PlatformSink::UringFile` variant behind a
`uring` feature.

## Decision

Land the scaffold in Phase 20 and fill in the submission-queue
integration in Phase 20.1 (planned):

### Phase 20 (this commit) — scaffold

- New Cargo feature `uring` on `rlg`.
- `io-uring 0.7` dep pinned under
  `[target.'cfg(target_os = "linux")'.dependencies]`. Only pulls
  on Linux; other targets get a resolved-but-inert feature.
- New `PlatformSink::UringFile(std::fs::File)` variant, gated by
  `#[cfg(all(target_os = "linux", feature = "uring"))]`. Compiles
  only on Linux, only with the feature.
- `PlatformSink::emit` handles the variant. The current
  implementation delegates to the sync `write_all` path for
  correctness — no io_uring SQE loop yet. The variant exists so
  consumers can select it today and the type signature is fixed;
  the wire path is the follow-up.
- No public API change to the sink constructors — the variant is
  only produced when a consumer explicitly selects it.

### Phase 20.1 (follow-up) — full submission-queue integration

- Introduce a per-flusher-thread `io_uring::IoUring` instance
  with an SQ depth of 128 (batch-size × 2 headroom).
- Batch outstanding writes into a single `submit_and_wait` call
  per flusher wake, matching the existing 64-event drain batch.
- Handle short writes (partial-completion CQE) with a retry loop
  bounded by the same 3-retry policy the queue already uses.
- Benchmark methodology matches Phase 18's sharded queue: run
  the file-sink benches with and without `--features uring` and
  publish deltas to `rustlogs.com/bench/`.

## Model

The scaffold's model is deliberately conservative:

- **Variant compiles only on Linux.** Non-Linux targets never see
  a `UringFile` in a `match` arm; the sink's cross-platform
  usability is unaffected.
- **Enum uses `std::fs::File` as the backing type**, matching the
  `File` variant. Phase 20.1 replaces this with an `io_uring`-owned
  file descriptor plus a per-thread submission queue.
- **`emit` writes synchronously.** The write path calls
  `File::write_all` — the io_uring SQE submission is deferred.
  Consumers who select `UringFile` today get the same throughput
  as `File`; they select it to future-proof, not for a Phase 20
  performance win.

This is the same scaffold-then-fill pattern used for the gRPC
transport in Phase 19c (ADR 0010): the type layout, feature
flag, and dep tree land now; the wire path fills in when the
follow-up phase brings the ~200 LOC needed to do it well.

## What Phase 20 is NOT

- **Not a performance win.** The scaffold doesn't move any bytes
  through io_uring. Users who want the win today wire the
  submission queue themselves against the underlying `File` —
  documented in the variant's rustdoc.
- **Not a cross-platform sink.** The variant is Linux-only, both
  because io_uring is Linux-only and because
  `[target.'cfg(target_os = "linux")']` gates the dep.
- **Not benchmarked yet.** Phase 20.1 lands the benches.

## Consequences

- **Zero regression by default.** The feature is off. The variant
  doesn't compile. Users who never set `--features uring` see
  identical behaviour to pre-Phase-20 rlg.
- **API future-proofing.** Consumers targeting Linux today can
  select `PlatformSink::UringFile` and know the enum variant name
  is stable; Phase 20.1 changes the internals only.
- **Cold-build time.** ~50 LOC of new dependency graph on Linux
  (io-uring 0.7). Negligible.
- **CI cost.** No new CI job — the existing Linux matrix leg
  already tests both feature combinations under
  `cargo test --workspace --all-features`.

## Alternatives considered

- **`tokio-uring`.** Original plan text. Rejected in favour of the
  raw `io-uring` crate because tokio-uring requires a
  tokio-uring-managed runtime, which is incompatible with rlg's
  `std::thread` flusher model. `io-uring 0.7` is the low-level
  submission-queue API that works from any thread.
- **Full Phase 20 in one commit.** Rejected on CI-risk grounds —
  the SQE integration needs benches, per-thread runtime state
  management, and error-recovery machinery. Landing it alongside
  Phases 19b/19c would collide with the concurrency work and
  balloon merge conflicts.
- **`glommio`.** A userspace runtime built on io_uring with
  first-class file I/O primitives. Rejected — pulls a runtime
  dependency; conflicts with the "runtime-agnostic" positioning
  the workspace maintains.
- **Skip io_uring entirely.** Rejected — the enterprise linux
  segment is a first-class rlg deployment target and io_uring is
  the industry standard for high-throughput file writes there
  from 2024 onwards.

## References

- [`io-uring` crate docs](https://docs.rs/io-uring/)
- [Linux kernel io_uring documentation](https://kernel.dk/io_uring.pdf)
- ADR 0010 — OTLP pluggable transport (same scaffold-then-fill
  pattern for the gRPC transport).
