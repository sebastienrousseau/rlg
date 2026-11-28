<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Dependencies

rlg keeps its third-party dependencies to the strict minimum
(standards `AGENTS.md` section 11). Every one is code we ship without
having written it, so each must earn its place, and the set may only
shrink.

`scripts/dependency-gate.py` enforces this in `make verify` and CI. It
counts four kinds against `scripts/dependency-baseline.json` and
prints how many of each there are:

| Kind | Counted from |
| --- | --- |
| Crates | third-party packages in `Cargo.lock`, transitive included |
| Actions | `uses:` in `.github/` |
| Container images | `FROM` lines and the dev container |
| Tools | install-action `tool:`, `cargo install`, `npx`, `pipx run`, pre-commit repos |

The gate fails on any name not in the baseline, and on any direct crate
below that has no row here. Removing a dependency always passes; run
`scripts/dependency-gate.py --update` to record the smaller baseline.

## Adding one

No new dependency of any kind lands without the maintainer's explicit
approval of that named dependency, after seeing what it does, why
building it here is worse, how many packages the lockfile gains, who
publishes them and their advisory history. Record the decision as a
row below (or an ADR) in the same change that runs `--update`. An
approval never covers a second dependency. Updating an existing one
for a fix is expected; an update that brings new transitive packages
counts as adding them.

Cryptography, TLS and other protocol stacks with a security history,
the toolchain and operating-system interfaces stay dependencies: they
are safer from an audited source than written here.

## Approved additions

Each row records the maintainer's approval of one named dependency.

| Dependency | Kind | Approved | Why |
| --- | --- | --- | --- |
| `reuse` 6.3.0 | tool (CI, `pipx run`) | 2026-11-28 | REUSE compliance lint (REPO-STANDARD section 7); no in-house equivalent of the specification's checker is worth owning |
| `codespell` 2.4.3 | tool (CI `pipx run`, pre-commit hook) | 2026-11-28 | Spelling in the docs-lint job and pre-commit (REPO-STANDARD section 8) |
| `cargo-hack` 0.6.46 | tool (CI, install-action) | 2026-11-28 | Each feature alone per push, the feature powerset weekly (REPO-STANDARD section 6) |

## Direct crates

"Shrink" marks the candidates to replace in-house first, worst-first:
a single use, a large tree for what it gives, or a pin kept only to
avoid an advisory.

| Crate | Used by | Why | Shrink |
| --- | --- | --- | --- |
| `anyhow` | rlg-cli, rlg-report, xtask | Error context in the binaries' `main` | Candidate: a small error enum would do |
| `assert_cmd` | rlg-cli, rlg-report (dev) | Runs the built binaries in integration tests | |
| `axum` | rlg-mcp | HTTP and SSE transport for the MCP server | |
| `clap` | rlg-cli, rlg-report, xtask | Command-line parsing | |
| `clap_complete` | rlg-cli, rlg-report | Shell completions generated from the CLI definition | |
| `clap_mangen` | rlg-cli, rlg-report | Manpages generated from the CLI definition | |
| `config` | rlg (`config` feature) | Layered config loading (file, environment) | Candidate: TOML plus env is all rlg reads |
| `criterion` | every crate (dev) | Benchmarks | |
| `crossbeam-queue` | rlg | The lock-free `ArrayQueue` behind the ring buffer | |
| `env_logger` | rlg (dev) | Comparison baseline in benchmarks | |
| `envy` | rlg (`config` feature) | Environment variables into `Config` | Candidate: one function, two call sites |
| `euxis-commons` | rlg | Shared error, validation, id and counter types of the family | |
| `futures` | rlg-mcp | Stream combinators for the SSE transport | Candidate: `futures-util` subset or std |
| `glob` | rlg-mcp | Glob patterns in `tail_logs_glob`, resolved under the log root | Candidate: small pattern matcher |
| `hostname` | rlg | Host name on every record | Candidate: one call; `libc`/`rustix` already present |
| `http` | rlg-tower | Request and response types for the middleware | |
| `http-body-util` | rlg-tower (dev) | Bodies in middleware tests | |
| `io-uring` | rlg (`uring` feature) | io_uring file sink on Linux (ADR 0011) | |
| `itoa` | rlg | Integer formatting on the flusher without allocation | |
| `libc` | rlg-ebpf | Process and host context on Linux | |
| `libfuzzer-sys` | fuzz | cargo-fuzz harness | |
| `log` | rlg | Bridge from the `log` facade | |
| `loom` | rlg (dev) | Model-checks the engine's concurrency | |
| `parking_lot` | rlg, rlg-test | Locks off the hot path (config, test capture) | Candidate: `std::sync` |
| `pin-project-lite` | rlg-tower | Pin projection in the response future | |
| `predicates` | rlg-cli, rlg-report (dev) | Output assertions with `assert_cmd` | |
| `proptest` | rlg, rlg-cli (dev) | Property tests of formats and parsing | |
| `regex` | rlg, rlg-redact | PII patterns in rlg-redact; format validation | |
| `rmcp` | rlg-mcp | Model Context Protocol server implementation | |
| `rustix` | rlg (`tui` feature) | Pinned to 1.1.6 or later so the `terminal_size` tree avoids an advisory | Pin only: drops with `terminal_size` |
| `ryu` | rlg | Float formatting on the flusher without allocation | |
| `schemars` | rlg-mcp | JSON Schema for the MCP tool arguments | |
| `serde` | rlg, rlg-mcp, rlg-otlp, rlg-report | Serialization of config, records and OTLP payloads | |
| `serde_json` | every crate but rlg-tower | JSON output and parsing | |
| `serial_test` | rlg (dev) | Serialises tests that share the global engine | |
| `tempfile` | rlg, rlg-cli, rlg-mcp, rlg-otlp, rlg-report (dev) | Temporary files in tests | |
| `terminal_size` | rlg (`tui` feature) | Terminal width for the dashboard | Candidate: one ioctl via `rustix` |
| `thiserror` | rlg, rlg-otlp | Error type derives | |
| `tokio` | rlg (`tokio` feature), rlg-mcp, rlg-otlp | Async runtime for the MCP server, the OTLP exporter and the async API | |
| `tokio-test` | rlg (dev) | Async test helpers | |
| `tokio-util` | rlg-mcp | Cancellation and codecs for the transports | |
| `toml` | rlg (`config` feature), fuzz | TOML config load and save | |
| `tower` | rlg-tower (dev) | `ServiceExt` and a real service stack in middleware tests | |
| `tower-layer` | rlg-tower | `Layer` trait | |
| `tower-service` | rlg-tower | `Service` trait | |
| `tracing` | rlg, rlg-otlp, rlg-redact (dev) | Emits spans and events in bridge tests | |
| `tracing-core` | rlg | The `tracing` layer (`RlgLayer`) | |
| `tracing-subscriber` | rlg (`tracing-layer` feature), rlg-otlp, rlg-redact (dev) | Registry the layer plugs into | |
| `uuid` | rlg-mcp | MCP session ids | Candidate: random id from `euxis-commons` |
| `version_check` | rlg (build) | Toolchain feature detection in `build.rs` | |
| `wasm-bindgen` | rlg-wasm | JavaScript bindings for the WebAssembly build | |
