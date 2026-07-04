<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# rlg — Implementation Plan to v0.1.0

> **Status:** Draft for review. Every phase is scoped as one signed commit,
> pushed to a working branch, verified green on CI before the next phase
> starts.
>
> **Audience:** repository maintainers, contributors preparing to pick up
> a phase, and enterprise adopters auditing forward-looking commitments.
>
> **Non-goals for this document:** replacing the CHANGELOG, replacing
> per-crate ADRs (each phase that changes an architectural invariant
> ships its own ADR under `docs/adr/`), or replacing the release runbook
> in `pkg/PUBLISH.md`.

---

## 0. Guiding principles

Every phase in this plan must satisfy the seven invariants below. A phase that
would violate one is either re-scoped or split.

1. **Signed commits, CI green.** Every phase lands as one or more SSH-signed
   commits. `cargo fmt --check`, `cargo clippy --workspace --all-features
   --tests --benches -- -D warnings`, and `cargo test --workspace
   --all-features` must pass locally before push. GitHub Actions must be green
   before the next phase begins.
2. **No public API break without a `semver-checks` justification.**
   `cargo semver-checks` (Phase 8) gates the workspace once installed. Any
   intentional break carries a `docs/adr/` entry naming the caller-side
   migration.
3. **Documentation is part of the definition of done.** A phase does not merge
   until every new public item has `///` docs including `# Errors`, `# Panics`,
   `# Safety`, and `# Examples` sections where applicable. `missing_docs`
   moves from `warn` to `forbid` at Phase 8.
4. **Every new public API item ships either a doctest or an `examples/`
   entry.** Phase 24 is the completion pass; every phase before it is
   responsible for its own additions.
5. **CI verifies examples run.** Phase 24 adds an `examples-smoke` CI job that
   runs every `examples/*.rs` with a deterministic input and asserts on exit
   status. From that point onward, every new example is verified on every PR.
6. **Benchmarks are the source of truth for performance claims.** No
   performance claim ships to READMEs, docs, or blog posts without a Criterion
   report published under `rustlogs.com/bench/`.
7. **Correctness proofs precede performance rewrites.** Phase 9 (Miri),
   Phase 10 (Loom), and Phase 13 (Kani) land before the hot-path rewrites in
   Phase 17 (Aho-Corasick) and Phase 18 (rtrb-sharded). Rewriting
   concurrency-critical code without prior proof machinery is a false economy.

---

## 1. Executive summary

The v0.0.11 branch closed the immediate security, hygiene, and coverage gaps.
v0.1.0 is the next major milestone. It closes the gaps identified in the
2026 Strategic Audit across five waves:

| Wave | Phases | Theme | Landing target |
|------|--------|-------|----------------|
| 1 | 8 → 16 | Correctness, compliance, supply-chain moat | v0.0.12 → v0.0.13 |
| 2 | 17 → 20 | Performance & concurrency rewrites | v0.0.14 |
| 3 | 21 → 23 | Ecosystem expansion (eBPF, WASI 0.2, `no_std`) | v0.0.15 → v0.0.17 |
| 4 | 24 → 25 | 100 % docs, examples, README parity | v0.0.18 |
| 5 | 26 → 28 | Positioning, DevRel, publish observability | v0.1.0 |

**Total scope**: 21 phases, one signed commit per phase minimum, estimated
150–200 files touched, ~15 k LOC added / ~2 k LOC modified. Each phase is
independently reviewable and independently revertable.

---

## Wave 1 — Correctness, Compliance, Supply-Chain Moat

Goal: earn enterprise trust before touching performance-critical code.

### Phase 8 — Documentation lint gate & docs.rs polish

**Objective.** Make missing docs a hard error and gate future work with
`semver-checks`. Fix the docs.rs discoverability of feature-gated items.

**Files touched.**
- Every `crates/*/Cargo.toml` — flip `missing_docs = "warn"` to `"forbid"` in
  `[lints.rust]`. Add `clippy::missing_docs_in_private_items = "warn"` in
  `[lints.clippy]`.
- Every optional item in `crates/rlg/src/*.rs`, `crates/rlg-otlp/src/lib.rs`,
  `crates/rlg-tower/src/lib.rs`, `crates/rlg-wasm/src/lib.rs` — annotate with
  `#[cfg_attr(docsrs, doc(cfg(feature = "…")))]`.
- `crates/rlg/src/lib.rs` — add top-of-file `#[doc(alias = "log")]`,
  `#[doc(alias = "logging")]`, `#[doc(alias = "structured logs")]`,
  `#[doc(alias = "observability")]` for docs.rs search.
- `.github/workflows/ci.yml` — add `cargo semver-checks check-release`
  job on every PR against `main`.

**Public API.** None (documentation-only + lint gate).

**Tests.** Existing suites must continue to pass. New: `cargo semver-checks`
runs on every PR.

**Docs.** N/A — this phase produces the enforcement layer.

**CI.** New job `semver-checks`. New job `docs-build` runs
`cargo doc --workspace --all-features --no-deps -- -D warnings`.

**Success criteria.**
- `cargo doc --workspace --all-features` completes with zero warnings.
- `cargo semver-checks check-release` passes on the PR that introduces it.
- `docs.rs` renders `rlg` with feature-gate annotations visible.

**Estimated size.** ~1 commit, ~30 files, +150/-30 LOC.

---

### Phase 9 — Miri gate in CI

**Objective.** Run the standard test suite under Miri on Linux and macOS to
catch UB in the ring-buffer hot path and the `sink.rs` FFI boundary.

**Files touched.**
- `.github/workflows/ci.yml` (or the reusable `pipelines/rust-ci.yml`) — new
  job `miri` matrix over `ubuntu-latest`, `macos-latest`; runs
  `cargo +nightly miri test -p rlg --lib --all-features`.
- Any test in `crates/rlg/tests/` that spawns a thread already carries
  `#[cfg_attr(miri, ignore)]` per `CLAUDE.md`. Audit the four crates added
  in Phase 4 (`rlg-mcp`, `rlg-redact`, `rlg-test`, `rlg-otlp`) and add the
  same attr where needed.

**Public API.** None.

**Tests.** Every test that stays inside a single thread and does not open a
`std::fs::File` runs under Miri. Expected pass count: ~120 of ~200 total
tests. Rest are legitimately Miri-skipped due to thread spawn, FFI, or file
I/O.

**Docs.** Update `CONTRIBUTING.md` with the `cargo miri test` invocation.

**CI.** ~7 min added per PR on Linux, ~10 min on macOS. Runs in parallel with
the existing matrix, so wall-clock impact is zero.

**Success criteria.**
- New Miri job green on the introducing PR.
- README badge added: `Miri` status.

**Estimated size.** ~1 commit, ~10 files, +80/-20 LOC.

---

### Phase 10 — Loom concurrency proofs for the ring buffer

**Objective.** Prove producer / flusher / shutdown interleavings in the
engine are race-free.

**Files touched.**
- `crates/rlg/Cargo.toml` — new `[target.'cfg(loom)'.dev-dependencies]`
  block adding `loom = "0.7"`.
- `crates/rlg/tests/loom_engine.rs` — new file. Three `#[cfg(loom)]`
  proofs:
  1. Producer + flusher never lose a record when queue capacity ≥ 2.
  2. Shutdown never drops in-flight records.
  3. `session_id` monotonicity holds under concurrent `ingest()`.
- `.github/workflows/ci.yml` — new job `loom` that sets
  `RUSTFLAGS="--cfg loom"` and runs `cargo test --test loom_engine`.

**Public API.** None.

**Tests.** Three Loom proofs. Each proof explores 10⁴–10⁶ interleavings and
completes in <90 s locally.

**Docs.** ADR: `docs/adr/0001-loom-verified-ring-buffer.md` describing the
proved invariants and known model limitations.

**CI.** ~3 min added, runs on the Linux matrix only.

**Success criteria.**
- All three Loom proofs pass.
- Adding a deliberate race (verified locally, not committed) causes at
  least one proof to fail.

**Estimated size.** ~1 commit, ~4 files, +250/-0 LOC.

---

### Phase 11 — `cargo-fuzz` targets + OSS-Fuzz onboarding

**Objective.** Continuous fuzzing of every parser and every redaction regex.

**Files touched.**
- `fuzz/` (new top-level workspace) — cargo-fuzz layout:
  - `fuzz/Cargo.toml`
  - `fuzz/fuzz_targets/parse_record.rs` — driver for `rlg_cli::parse_record`.
  - `fuzz/fuzz_targets/log_format_from_str.rs` — driver for
    `LogFormat::from_str`.
  - `fuzz/fuzz_targets/config_load.rs` — driver for `Config::from_toml`.
  - `fuzz/fuzz_targets/redact_scrub.rs` — driver for
    `Redactor::with_defaults().scrub()`.
- `.github/workflows/fuzz-smoke.yml` — new workflow. Runs each fuzz target
  for 30 s on every PR. Non-zero exit fails the check.
- `docs/OSS-FUZZ.md` — onboarding runbook, PR template for the
  `google/oss-fuzz` submission.

**Public API.** None.

**Tests.** Four fuzz targets. Corpus seeded from the integration test
fixtures.

**Docs.** ADR: `docs/adr/0002-fuzz-strategy.md` — targets, corpus policy,
crash-triage runbook.

**CI.** ~2 min per target × 4 = 8 min per PR for the smoke fuzz.

**Success criteria.**
- Four fuzz targets build and run.
- OSS-Fuzz submission PR opened (may not merge in this phase; landing is
  Google's timeline).

**Estimated size.** ~1 commit, ~8 files, +300/-0 LOC.

---

### Phase 12 — Property tests for the 14 `Display` impls

**Objective.** Prove `render(parse(x)) == x` after canonicalisation for the
formats where round-trip is meaningful (JSON, NDJSON, Logfmt, MCP, OTLP,
ECS).

**Files touched.**
- `crates/rlg/Cargo.toml` — add `proptest = "1"` to `[dev-dependencies]`.
- `crates/rlg/tests/proptest_round_trip.rs` — new file. One `proptest!` per
  round-trippable format. Strategy: generate a `Log` with arbitrary
  `session_id`, level, component, description, attributes; format it; parse
  it back; assert equality post-canonicalisation.
- `crates/rlg-cli/tests/proptest_filter.rs` — new file. Prove
  `Filter::matches` is monotone in `min_level`.

**Public API.** None.

**Tests.** Six round-trip proptests + two filter proptests. Each runs
1 024 cases by default.

**Docs.** ADR: `docs/adr/0003-property-tested-formats.md`.

**CI.** ~30 s added.

**Success criteria.**
- All property tests pass with default case counts.
- Increasing the case count to 100 000 in a local run still passes.

**Estimated size.** ~1 commit, ~3 files, +400/-0 LOC.

---

### Phase 13 — Kani proof harnesses

**Objective.** Prove two invariants formally:
1. `Log::ingest()` never leaves the ring buffer in an inconsistent state.
2. `session_id: u64` wraparound cannot violate the monotonicity contract
   that the flusher relies on.

**Files touched.**
- `crates/rlg/kani/Cargo.toml` — sub-package layout per Kani convention.
- `crates/rlg/kani/proofs/ring_buffer.rs` — `#[kani::proof]` harnesses.
- `crates/rlg/kani/proofs/session_id.rs` — `#[kani::proof]` for u64
  arithmetic invariants.
- `.github/workflows/kani.yml` — new workflow. Runs `cargo kani`.
- `docs/adr/0004-kani-verified-invariants.md`.

**Public API.** None.

**Tests.** Two Kani proofs, each budgeted to ≤10 min on a 4-vCPU runner.

**Docs.** ADR + a `KANI.md` in `crates/rlg/kani/` describing the harness
model and what is *not* verified.

**CI.** ~20 min added. Runs on `push` to `main` and weekly cron, not on
every PR (too slow).

**Success criteria.**
- Both Kani proofs complete without a counter-example.
- Introducing a deliberate off-by-one (verified locally, not committed)
  produces a Kani counter-example.

**Estimated size.** ~1 commit, ~6 files, +500/-0 LOC.

---

### Phase 14 — SBOM + sigstore/cosign on releases

**Objective.** Make every published binary and every crate artefact
verifiable end-to-end.

**Files touched.**
- `.github/workflows/release.yml` — new steps:
  1. `cargo sbom` (or `cargo cyclonedx`) generates CycloneDX SBOM per crate.
  2. `cosign sign-blob --yes --output-signature <artefact>.sig` on every
     release-tarball and every published `.crate`.
  3. Upload SBOM + signature bundle to the GitHub Release assets.
- `pkg/VERIFY.md` — new file. Consumer-side verification instructions.
- `Makefile` — install target verifies signature before installing.
- `SECURITY.md` — update with the sigstore trust root and the reporting
  matrix for SBOM discrepancies.

**Public API.** None.

**Tests.** Manual: pull the release artefact, verify signature, verify SBOM
against `cargo audit`.

**Docs.** ADR: `docs/adr/0005-sigstore-and-sbom.md`.

**CI.** ~2 min added on release only.

**Success criteria.**
- First release under this phase carries a `cosign`-verifiable signature.
- CycloneDX SBOM lists every dependency version present in `Cargo.lock`.
- The `Makefile install` target refuses to install an artefact with a
  broken signature.

**Estimated size.** ~1 commit, ~5 files, +250/-30 LOC.

---

### Phase 15 — `cargo-vet` audit chain

**Objective.** Verifiable provenance for every transitive dependency.

**Files touched.**
- `supply-chain/config.toml` — bootstrap importing the Google, Mozilla, and
  Bytecode Alliance audit sets.
- `supply-chain/audits.toml` — audits authored in this workspace.
- `supply-chain/imports.lock` — machine-generated.
- `.github/workflows/ci.yml` — new step: `cargo vet --locked`.

**Public API.** None.

**Tests.** `cargo vet` passes.

**Docs.** ADR: `docs/adr/0006-cargo-vet-adoption.md`.

**CI.** ~15 s per PR.

**Success criteria.**
- `cargo vet` is clean.
- New dependencies fail CI until audited.

**Estimated size.** ~1 commit, ~3 files, +300/-0 LOC (mostly imports).

---

### Phase 16 — `cargo-deny` hardening

**Objective.** Turn advisory-mode dependency policy into enforced policy.

**Files touched.**
- `deny.toml`:
  - `[bans] multiple-versions = "deny"` (was `"warn"`).
  - `deny = [{ name = "openssl-sys" }, { name = "native-tls" }, { name = "chrono", wrappers = ["hyper"] }]` — force `rustls` everywhere and pin transitive uses of chrono to explicit wrappers.
  - `[sources]` block whitelisting `crates.io` and the workspace path
    dependencies only.
- Fix every duplicate-version warning surfaced by `cargo deny check` before
  flipping the switch. Historical friction here: `syn 1` / `syn 2`
  duplicates via legacy dev-deps, `hashbrown` variants.

**Public API.** None (potentially breaks the build until duplicates
resolved).

**Tests.** `cargo deny check` runs green.

**Docs.** ADR: `docs/adr/0007-cargo-deny-hardened.md`.

**CI.** No change (`cargo deny check` already runs via
`pipelines/security.yml`).

**Success criteria.**
- `cargo deny check` green with the tightened policy.

**Estimated size.** ~1 commit, ~1 file + Cargo.lock churn, +30/-5 LOC.

---

## Wave 2 — Performance & Concurrency

Only starts after **every proof-machinery phase (9, 10, 11, 12, 13) is green
on `main`.**

### Phase 17 — Aho-Corasick fused redaction

**Objective.** Replace the six-regex loop in `rlg-redact::Redactor::scrub`
with a single `regex-automata::meta::Regex` (DFA-fused Aho-Corasick).
Publish before/after Criterion charts.

**Files touched.**
- `crates/rlg-redact/src/lib.rs` — rewrite the `Redactor` internals; keep
  the public API surface identical.
- `crates/rlg-redact/benches/scrub.rs` — extend with a comparative case
  set.
- `docs/adr/0008-fused-redaction-automaton.md` — describes the DFA
  compilation model and why the API is stable.

**Public API.** No breaking changes. `Redactor::with_pattern` continues to
work; internally the pattern is folded into the automaton at construction
time.

**Tests.** Existing `rlg-redact/tests/integration.rs` (13 tests from
Phase 4) must remain green. Add three tests specifically exercising the
Aho-Corasick fusion boundary (overlapping matches, alternation
correctness).

**Docs.** README section: link the Criterion report.

**CI.** No change.

**Success criteria.**
- All 16 existing tests continue to pass.
- Criterion shows ≥3× throughput on `heavy_pii_match`.
- Criterion shows ≤0 % regression on `no_pii_match`.

**Estimated size.** ~1 commit, ~4 files, +200/-150 LOC.

---

### Phase 18 — Sharded producer queue

**Objective.** Replace `crossbeam-queue::ArrayQueue` on the ingest hot path
with per-producer `rtrb` SPSC rings, aggregated by the flusher.

**Files touched.**
- `crates/rlg/src/engine.rs` — new module `engine::sharded` behind a
  `fast-queue` feature (default off). Retain the ArrayQueue path as the
  default for one release cycle.
- `crates/rlg/Cargo.toml` — new optional dep `rtrb = "0.3"`; new feature
  `fast-queue = ["dep:rtrb"]`.
- `crates/rlg/benches/competitive_bench.rs` — add a sharded-queue case
  set.
- `crates/rlg/tests/loom_engine.rs` (Phase 10) — extend Loom proofs to
  cover the sharded variant.

**Public API.** New feature flag. No visible surface change.

**Tests.** Existing engine tests must pass with and without
`--features fast-queue`. Loom proofs extended.

**Docs.** ADR: `docs/adr/0009-sharded-producer-queue.md`.

**Success criteria.**
- Loom proofs cover both variants.
- Criterion shows ≥1.4× ingest throughput at 4 producers on Skylake+ / M-series
  vs. the ArrayQueue baseline.
- No regression on the single-producer case.

**Estimated size.** ~1 commit, ~6 files, +600/-50 LOC.

---

### Phase 19 — Async OTLP + gRPC scaffold

**Objective.** Add pluggable transport to `rlg-otlp`. Keep sync `ureq` path
as `blocking` feature (default). Add `async` feature using `reqwest` +
`rustls`. Add `grpc` feature using `tonic`. Wire retry-with-jitter + a
tokens-per-window circuit breaker.

**Files touched.**
- `crates/rlg-otlp/src/lib.rs` — introduce a `Transport` trait.
- `crates/rlg-otlp/src/transport/blocking.rs` — existing ureq path.
- `crates/rlg-otlp/src/transport/async_http.rs` — new reqwest-based.
- `crates/rlg-otlp/src/transport/grpc.rs` — new tonic-based against
  `opentelemetry-proto`.
- `crates/rlg-otlp/src/backoff.rs` — retry policy + circuit breaker.
- `crates/rlg-otlp/Cargo.toml` — new optional deps: `reqwest`, `tonic`,
  `opentelemetry-proto`.
- `crates/rlg-otlp/tests/integration.rs` — extend with a mock-server test
  per transport (using `wiremock`).
- `crates/rlg-otlp/examples/honeycomb.rs` (existing) — update to
  demonstrate async transport.
- `crates/rlg-otlp/examples/grpc_collector.rs` — new example against a
  local `otelcol` (documented as manual).

**Public API.** Additive: new `Transport` trait, new
`OtlpExporter::builder().transport(...)`. Existing `export_one` /
`export_batch` remain.

**Tests.** Add ~10 integration tests per new transport, all against
`wiremock`. Circuit-breaker property test.

**Docs.** ADR: `docs/adr/0010-otlp-pluggable-transport.md`. README updates
across `rlg-otlp/README.md`.

**Success criteria.**
- `wiremock` tests green.
- Bench shows async transport competitive with sync at 1× record; wins at
  ≥16× parallel exports.
- `grpc` feature builds and passes a mock-tonic smoke test.

**Estimated size.** ~1 commit (or split as 3 sub-phases 19a/19b/19c if the
review is dense), ~15 files, +1 500/-100 LOC.

---

### Phase 20 — `io_uring` file sink (Linux)

**Objective.** Add a Linux-only `uring` feature that swaps
`std::fs::File::write_all` for `tokio-uring`.

**Files touched.**
- `crates/rlg/src/sink.rs` — new `PlatformSink::UringFile(...)` variant
  behind `#[cfg(all(target_os = "linux", feature = "uring"))]`.
- `crates/rlg/Cargo.toml` — new optional dep `tokio-uring`, new feature
  `uring`.
- `crates/rlg/benches/file_sink_bench.rs` — new file. Comparative bench
  vs. the existing `File` path.
- `docs/adr/0011-io-uring-file-sink.md`.

**Public API.** New feature flag; existing enum grows a variant behind
`#[cfg]`.

**Tests.** New Linux-only integration test in
`crates/rlg/tests/uring_smoke.rs`. Skipped on non-Linux.

**CI.** New Linux matrix leg with `--features uring`.

**Success criteria.**
- Criterion shows ≥1.3× throughput at ≥100 k records/s file writes on
  Linux 6.x.
- macOS + Windows builds unaffected.

**Estimated size.** ~1 commit, ~5 files, +400/-30 LOC.

---

## Wave 3 — Ecosystem Expansion

### Phase 21 — `rlg-ebpf` (Linux context enrichment)

**Objective.** New crate `rlg-ebpf` that attaches PID / TID / cgroup / UID
/ optional network 4-tuple to every record. Ships as `PlatformSink::Ebpf`
adapter or a separate `Enricher` trait.

**Files touched.**
- `crates/rlg-ebpf/Cargo.toml`, `crates/rlg-ebpf/src/lib.rs`,
  `crates/rlg-ebpf/tests/`, `crates/rlg-ebpf/README.md`,
  `crates/rlg-ebpf/examples/enrich.rs`.
- `Cargo.toml` — add to `[workspace] members`.
- Choice: `aya` (pure Rust) or `libbpf-rs` (bindings). Recommend `aya`
  for build hygiene.

**Public API.** New crate. Public trait `Enricher` with one blanket impl.

**Tests.** Integration tests behind a `#[cfg(target_os = "linux")]` gate,
plus an all-platforms unit test for the trait.

**Docs.** ADR: `docs/adr/0012-ebpf-enricher.md`. README section on
capability requirements (`CAP_BPF`).

**Success criteria.**
- Compiles on Linux stable and nightly.
- Enrichment test attaches expected `pid`, `tid`, `uid` fields.
- Criterion bench under `crates/rlg-ebpf/benches/enrich.rs` shows
  <5 µs per record overhead.

**Estimated size.** ~1 commit, ~10 files, +900/-0 LOC.

---

### Phase 22 — WASI 0.2 component model target for `rlg-wasm`

**Objective.** Publish `rlg-wasm` as a WASI 0.2 component exporting
`wasi:logging/logging` and consuming `wasi:cli/stderr`.

**Files touched.**
- `crates/rlg-wasm/wit/rlg.wit` — WIT interface.
- `crates/rlg-wasm/src/wasi.rs` — implementation.
- `crates/rlg-wasm/Cargo.toml` — new target section for
  `wasm32-wasip2`; adds `wit-bindgen`.
- `crates/rlg-wasm/README.md` — new "WASI 0.2" section with the
  `wasmtime run --component` invocation.
- `crates/rlg-wasm/examples/wasi_component.rs` — buildable example.

**Public API.** Additive: new module `wasi`.

**Tests.** CI job that builds the component and runs a `wasmtime`
smoke against it.

**Docs.** ADR: `docs/adr/0013-wasi-0.2-component.md`.

**Success criteria.**
- `wasm32-wasip2` build produces a component.
- `wasmtime` smoke test runs.

**Estimated size.** ~1 commit, ~7 files, +500/-20 LOC.

---

### Phase 23 — `no_std` + `alloc` mode for the core crate

**Objective.** Feature-gate `std` usage in `rlg` so a subset compiles under
`no_std`. Explicit scope: **the core `Log` type + its `Display` impls +
`LogFormat` + `LogLevel`.** Out of scope: engine, sinks, config, TUI —
those legitimately require `std`.

**Files touched.**
- `crates/rlg/src/lib.rs` — `#![cfg_attr(not(feature = "std"), no_std)]`.
- `crates/rlg/Cargo.toml` — new `default = ["std"]`, new `std` feature,
  everything currently in `[dependencies]` migrated behind conditional
  compilation.
- New `crates/rlg-embedded-demo/` — an example targeting a Cortex-M4
  under QEMU that emits a Logfmt record via `defmt-uart` or `semihosting`.
  Verifies the `no_std` path works end-to-end.

**Public API.** No breakage; `std`-requiring items become gated with
`#[cfg(feature = "std")]`.

**Tests.** New CI matrix leg: `cargo check --no-default-features
--target thumbv7em-none-eabihf` and equivalent RISC-V `riscv32imac`.

**Docs.** README: new "Embedded / `no_std`" section. ADR:
`docs/adr/0014-no-std-core.md`.

**Success criteria.**
- Cortex-M4 target compiles.
- Feature matrix passes for `default`, `std`, `no_std` combinations.

**Estimated size.** ~1 commit, ~15 files, +400/-100 LOC.

---

## Wave 4 — Documentation & Testing Completeness

### Phase 24 — 100 % example coverage + `examples-smoke` CI

**Objective.** Every public function, struct, and trait either carries a
runnable doctest or has an entry under `examples/`. CI verifies every
example runs to a clean exit.

**Files touched.**
- Every `crates/*/src/lib.rs` and its sub-modules — audit and add doctests
  where missing.
- New `examples/` entries where the function is too complex for a doctest.
- `.github/workflows/ci.yml` — new job `examples-smoke`. Iterates every
  `[[example]]` in every workspace `Cargo.toml`; runs
  `cargo run --release --example <name>`; asserts exit code 0.
- `xtask/src/main.rs` — new sub-command `xtask verify-examples`
  parametrising the CI job.
- New `docs/EXAMPLES-INDEX.md` — hand-curated catalogue by capability.

**Public API.** None.

**Tests.** The CI job itself is the verification. Local dev runs
`cargo xtask verify-examples`.

**Docs.** ADR: `docs/adr/0015-examples-are-tests.md`.

**Success criteria.**
- Every `examples/*.rs` file across the workspace runs green under CI.
- A coverage-tracker script (`xtask coverage-examples`) reports 100 % of
  public items either doc-tested or example-covered.

**Estimated size.** ~2–3 commits (large; naturally splits per crate),
~40 files, +2 000/-100 LOC.

---

### Phase 25 — README currency, migration guides, ADR index

**Objective.** Bring every README into perfect sync with the public API and
publish first-class migration guides.

**Files touched.**
- Every `crates/*/README.md` — regenerate the Install / Feature / Usage
  sections against the current Cargo.toml. Add a Benchmarks section
  linking `rustlogs.com/bench/`. Add a "Related" section pointing at
  sibling crates.
- `README.md` (workspace root) — refresh with the v0.0.11 → v0.1.0
  narrative, the MCP-native positioning, and the workspace map.
- `docs/migration/from-tracing.md` — name-for-name mapping.
- `docs/migration/from-slog.md` — same.
- `docs/migration/from-log.md` — same.
- `docs/adr/README.md` — index of every ADR authored in Phases 8–24.
- `docs/BENCHMARKS.md` — pointer to the published Criterion reports,
  reproducibility instructions.
- `xtask src/main.rs` — new sub-command `xtask verify-readmes` that lints
  every `README.md` for Install-section version drift against
  `Cargo.toml`.

**Public API.** None.

**Tests.** `cargo xtask verify-readmes` runs in CI.

**Docs.** This *is* the docs phase.

**Success criteria.**
- `xtask verify-readmes` green.
- Every crate README has: badges row (5 badges), MSRV, Install, Quick
  Start, Features, Examples index, Benchmarks link, License.
- Three migration guides published.

**Estimated size.** ~2 commits, ~25 files, +1 800/-500 LOC.

---

## Wave 5 — Positioning, DevRel, Publish Observability

### Phase 26 — Positioning refresh + Whitepaper 1

**Objective.** Reposition the flagship around MCP-native observability and
publish the first authority-building whitepaper.

**Files touched.**
- `README.md` — new tagline; hero paragraph pivots to MCP.
- GitHub repository description — updated to reflect the pivot (already
  done partially in the last session; refresh again with the
  Whitepaper 1 link).
- `docs/whitepapers/01-logs-as-mcp-tools.md` — 4-part deep dive.
- `crates/rlg-mcp/README.md` — add the "Why MCP" section referencing the
  whitepaper.
- Publish HTML rendering to `rustlogs.com/whitepapers/01-mcp-tools`.

**Public API.** None.

**Tests.** N/A (content).

**Docs.** The whitepaper *is* the deliverable.

**Success criteria.**
- Whitepaper published, discoverable from the workspace README, and
  cross-posted to at least two Rust community channels (r/rust,
  This Week in Rust, or a Rust newsletter).

**Estimated size.** ~1 commit for the docs; the whitepaper itself is
~4 000 words.

---

### Phase 27 — Publish Criterion HTML reports

**Objective.** Continuously publish bench results.

**Files touched.**
- `.github/workflows/bench-publish.yml` — new workflow. On tag push, runs
  `cargo criterion --workspace --message-format=json`, converts to HTML,
  syncs to `rustlogs.com/bench/<tag>/`, and updates `bench/latest/` to
  point at the new tag.
- `docs/BENCHMARKS.md` — link the live URL.

**Public API.** None.

**Tests.** N/A.

**Docs.** Update the workspace README with the live bench URL.

**Success criteria.**
- First tag under this phase publishes reports at the live URL.
- The workspace README displays the throughput number pulled from the
  latest report.

**Estimated size.** ~1 commit, ~2 files, +100/-0 LOC of workflow YAML.

---

### Phase 28 — Meta-gates: semver + coverage + Renovate

**Objective.** Wire the final policy gates that keep v0.1.0-and-beyond
regression-proof.

**Files touched.**
- `.github/workflows/ci.yml` — add the codecov PR gate (fail on ≥2 %
  coverage drop).
- `.github/renovate.json` — Renovate config with batched dependency PRs
  for the workspace, replacing the current Dependabot grouping.
- `.github/dependabot.yml` — remove or dial down to security-only.
- `Makefile` — add `make verify` target that runs everything a contributor
  needs before opening a PR (fmt, clippy, test, miri, semver-checks,
  vet, deny, examples-smoke, verify-readmes).

**Public API.** None.

**Tests.** All gates run green on the introducing PR.

**Docs.** Update `CONTRIBUTING.md` with the `make verify` step.

**Success criteria.**
- Codecov gate blocks a synthetic coverage-drop PR.
- Renovate opens the first batched dep PR.

**Estimated size.** ~1 commit, ~5 files, +150/-40 LOC.

---

## 2. Cross-cutting invariants

### Documentation

Every phase in Waves 1–5 must satisfy:

- Every new `pub fn`, `pub struct`, `pub enum`, `pub trait` has `///`
  docs including `# Errors` (if fallible), `# Panics` (if any), `# Safety`
  (if `unsafe`), and `# Examples` (always, unless it's covered by an
  `examples/` file linked from the docs).
- `missing_docs = "forbid"` (from Phase 8) prevents drift.
- ADRs live under `docs/adr/NNNN-slug.md` with a stable header:
  `Status: Accepted | Superseded by NNNN | Deprecated`.

### Testing

Every phase must ship, in this order of preference for a given code path:

1. **Doctests** — cheapest, run under `cargo test --doc`.
2. **Unit tests** — inline `#[cfg(test)] mod tests`, small and colocated
   with the code.
3. **Integration tests** — under `tests/`, black-box against the public
   API.
4. **Property tests** — where round-trip / invariant properties exist.
5. **Loom tests** — where concurrency matters.
6. **Fuzz targets** — where parsing untrusted input.
7. **Kani proofs** — for the load-bearing invariants only.

Coverage floor: **90 % line coverage** measured by tarpaulin, enforced
via the Codecov gate from Phase 28.

### Benchmarks

Every phase that claims a performance win publishes a Criterion report
under `rustlogs.com/bench/<tag>/`. No unverified performance claims land
in READMEs or blog posts.

### Examples

By the end of Phase 24, the invariant is:

- Every `pub fn` has either a `# Examples` doctest **or** an
  `examples/*.rs` file that exercises it.
- Every `examples/*.rs` runs to a clean exit under CI's `examples-smoke`
  job.
- The `docs/EXAMPLES-INDEX.md` catalogue lists every example with its
  covered API surface.

### Backwards compatibility

- Phase 8 gates all future changes with `cargo semver-checks`.
- Phase 15 gates all new dependencies with `cargo vet`.
- Any intentional break carries an ADR + a migration section in the
  crate README.

---

## 3. Rollout order and dependency chain

```
Phase 8 ─→ 9 ─→ 10 ─→ 11 ─→ 12 ─→ 13 ──┐
                                        │
                             14 ─→ 15 ─→ 16 ──┐
                                              │
Wave 1 complete ──────────────────────────────┴─→ 17 ─→ 18
                                                       ├─→ 19
                                                       └─→ 20
Wave 2 complete ──────────────────────────────────────────────→ 21 ─→ 22 ─→ 23
Wave 3 complete ─────────────────────────────────────────────────────────────→ 24 ─→ 25
Wave 4 complete ─────────────────────────────────────────────────────────────────────→ 26 ─→ 27 ─→ 28
```

Notes:

- Phase 18 (sharded queue) requires Phase 10 (Loom) merged. Non-negotiable.
- Phase 17 (Aho-Corasick redact) requires Phase 11 (fuzz) and Phase 12
  (proptest) merged. Otherwise a subtle DFA compilation bug ships silently.
- Phase 21 (eBPF) is independent of Phases 17–20 and can run in parallel
  once Wave 1 is done.
- Phase 24 (examples coverage) can begin at Phase 17 but only *closes* once
  Wave 3 is done — every phase adds new public items that need coverage.

---

## 4. Risk register

| Risk | Impact | Likelihood | Mitigation |
|---|---|---|---|
| Kani proofs (Phase 13) exceed 20 min CI budget | Cron-only fallback | Medium | Budget each proof to ≤10 min; run only on `main` + weekly cron. |
| Aho-Corasick fusion (Phase 17) breaks pattern semantics for custom regex | Silent scrub misses | Low | Property tests from Phase 12 cover this. Enforcement: block Phase 17 on Phase 12 landing. |
| Sharded queue (Phase 18) regresses single-producer case | Common case degrades | Medium | Behind `fast-queue` feature, default off, for one release cycle. Criterion gate on the introducing PR. |
| Async OTLP (Phase 19) grows the dependency graph significantly | Cold-build time inflates | High | Every new transport behind its own feature. Default `blocking` remains. `cargo-udeps` gate. |
| `no_std` (Phase 23) breaks published binaries via feature-graph mistake | Cascade | Low–Medium | Add `check-features` xtask that iterates every feature combination. |
| WASI 0.2 (Phase 22) chases a moving target (WASI 0.3 preview) | Rework | Medium | Track spec stability; do not publish until WASI 0.2 preview 3 is confirmed final. |
| Ripple churn — Phases 8, 25, 28 touch every crate | Merge conflicts | High | Rebase-clean-often discipline; land each in its own PR against a fresh HEAD. |

---

## 5. Acceptance for v0.1.0

Ship v0.1.0 when **every** row is green:

- [ ] Phases 8–28 landed on `main`.
- [ ] `cargo audit`, `cargo deny check`, `cargo vet`, `cargo
      semver-checks`, `cargo miri test`, Loom proofs, Kani proofs, fuzz
      smoke — all green.
- [ ] Tarpaulin coverage ≥ 90 %.
- [ ] `cargo xtask verify-examples` — every example runs.
- [ ] `cargo xtask verify-readmes` — every README in sync.
- [ ] Criterion report published at `rustlogs.com/bench/v0.1.0/`.
- [ ] Whitepaper 1 published; whitepapers 2 and 3 outlined.
- [ ] SBOM emitted for every release artefact; every artefact
      `cosign`-verifiable.
- [ ] All 10 sub-crate READMEs on the standardised skeleton with a
      Benchmarks section linking the live report.

---

## 6. Out of scope for v0.1.0

Explicitly deferred:

- GPU regex offload (Moonshot in the audit).
- Post-quantum TLS default in `rlg-otlp` — wait for `rustls` PQ hybrid to
  ship as stable feature.
- Formal TLA+ / Coq spec of the ring buffer — Kani proofs cover the
  practical safety envelope; TLA+ is over-budget for v0.1.0.
- First-class Cloudflare Workers persistence backend.
- Live LLM-narration hook (agentic monitoring) — parked until MCP client
  patterns in the wider ecosystem stabilise.

---

## 7. How to review this document

- Comment inline on the section that concerns you.
- If a phase is mis-scoped (too big, too small, wrong dependency), flag
  it and propose a re-shape.
- If a phase is missing something the audit called out, name the audit
  item and propose the insertion point.
- If a phase's success criteria are too soft or too strict, propose an
  amendment.

Once approved, each phase becomes its own PR against `main`, following
the workflow codified in the `CLAUDE.md` contributor guide.
