<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR 0002 — Fuzz Strategy

- **Status:** Accepted
- **Date:** 2026-07-05
- **Phase:** 11 (per `docs/IMPLEMENTATION-PLAN-v0.1.0.md`)
- **Deciders:** repository maintainers
- **Related:** ADR 0003 (property tests) — same invariant surface,
  different exploration strategy.

## Context

Four public API entry points in the workspace deserialise or scan
untrusted input:

- **`rlg_cli::parse_record`** — parses a single JSON-shape record from
  a stream line. Called by `rlg-cli`, `rlg-mcp`, and `rlg-report` on
  every input line.
- **`<LogFormat as FromStr>::from_str`** — parses the format
  identifier for the `--format` flag and for the MCP tool arguments.
- **`rlg::config::Config` deserialisation** — parses TOML config files
  loaded at process start.
- **`rlg_redact::Redactor::with_defaults().scrub`** — scans arbitrary
  log strings against six built-in regexes. A pathological input
  could trigger regex catastrophic backtracking or an unexpected
  panic in the regex engine.

Unit and integration tests exercise the happy path and a handful of
edge cases. Neither systematically explores the input space.

## Decision

Adopt `cargo-fuzz` (libFuzzer-backed) as the fuzz driver. Author
four targets — one per entry point above — under `fuzz/`, excluded
from the workspace so `libfuzzer-sys` and nightly-only build flags
never leak into the normal `cargo build` / `cargo test` toolchain.

CI job `.github/workflows/fuzz-smoke.yml` runs each target for **30
seconds per PR** on Ubuntu with nightly. This is a smoke gate, not
a bug-hunt — 30 s catches regressions on the well-explored corpus,
not novel bugs.

Long-running fuzzing lives in **OSS-Fuzz** (see `docs/OSS-FUZZ.md`).
Once Google accepts the submission, OSS-Fuzz runs each target for
hours per day against the shared corpus and files crashes as
private security issues automatically.

## Target contracts

Every fuzz target satisfies:

- **`#![no_main]`** — libfuzzer-sys entrypoint.
- **UTF-8 gate** — non-UTF-8 bytes are rejected at the boundary via
  `std::str::from_utf8` before touching workspace code. Fuzzing that
  rejection would exercise `std` internals, not our code.
- **No panics allowed** — every wrapped API is documented as
  fallible. `Result::Err` is the correct response to invalid input.
  A panic under any input is a bug.
- **Deterministic** — no clock reads, no thread spawns, no
  filesystem writes. Fuzz targets must be pure functions of their
  input.

## Corpus policy

- **Initial seeds** for each target are drawn from the integration
  test fixtures in `crates/rlg-cli/tests/`, `crates/rlg-mcp/tests/`,
  `crates/rlg-redact/tests/`. Every green test line is a valid
  seed input.
- **New crashes** are triaged within one working day. The fix ships
  as a regular PR with a regression test derived from the crash
  artefact, added to the crate's `tests/` and to the fuzz corpus.
- **Corpus size cap:** 10 MB per target. Beyond that, run
  `cargo fuzz cmin` (corpus minimisation) as part of the fix PR.

## OSS-Fuzz integration

Onboarding runbook: `docs/OSS-FUZZ.md`. Summary:

1. Draft the `project.yaml` naming the fuzz targets and the
   maintainer email.
2. Draft the `Dockerfile` that clones this repo and installs the
   nightly toolchain.
3. Draft the `build.sh` that compiles each target with `cargo fuzz
   build --release`.
4. Open a PR against `google/oss-fuzz` referencing this ADR.
5. Once accepted, Google runs the fuzz corpus continuously and
   files crashes as GitHub Security Advisories.

Timeline for OSS-Fuzz acceptance is Google's — typically 2–6 weeks.
Phase 11 lands the local + smoke-gate coverage regardless.

## What is not covered

- **Concurrency bugs.** Fuzz targets are single-threaded by design.
  Concurrent invariants belong to Loom (ADR 0001).
- **Panics inside `crossbeam-queue`, `serde_json`, `regex`, or
  `toml`.** Third-party crates carry their own fuzz coverage.
  A crash discovered in a transitive dep gets reported upstream.
- **Long-tail input patterns.** 30 s per PR is a smoke gate. Deep
  bug-hunting is OSS-Fuzz's role.

## Consequences

- **CI cost.** ~2 min per PR (30 s × 4 targets, plus nightly install
  + cache priming).
- **Contributor cost.** Local reproducer:
  ```bash
  cargo install cargo-fuzz --locked
  cd fuzz && cargo +nightly fuzz run parse_record
  ```
  Documented in `CONTRIBUTING.md` and `fuzz/README.md`.
- **Nightly dependency.** libFuzzer requires nightly. This is
  contained to the fuzz workflow — no impact on the rest of CI
  which runs on stable.
- **Excluded workspace.** `fuzz/` cannot use workspace-wide
  `[lints]` or `[patch]`. It sets its own minimal lints in
  `fuzz/Cargo.toml`.

## Alternatives considered

- **AFL++.** Slower to instrument in Rust than libFuzzer; harder
  to integrate with OSS-Fuzz. Rejected.
- **Property tests only (Phase 12).** Complementary, not equivalent.
  Proptest generates structured inputs; fuzzing generates raw byte
  strings. Both catch different bug classes.
- **In-repo continuous fuzzing without OSS-Fuzz.** GitHub Actions
  minutes budget does not sustain hours-per-day per-target fuzzing
  affordably. OSS-Fuzz is free for open-source projects.

## References

- [`cargo-fuzz` book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [OSS-Fuzz new-project process](https://google.github.io/oss-fuzz/getting-started/new-project-guide/)
- Rustsec advisories with root cause in `serde_json::from_str`
  panics — historical precedent for this class of finding.
