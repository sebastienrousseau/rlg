<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/rlg/v1/logos/rlg.svg" alt="rlg logo" width="128" />
</p>

<h1 align="center">rlg</h1>

<p align="center">
  Near-lock-free structured logging for Rust: records leave the application thread with a few atomic operations and are formatted and written by a background flusher.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/rlg/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rlg/ci.yml?style=for-the-badge&logo=github" alt="Build" height="28" /></a>
  <a href="https://crates.io/crates/rlg"><img src="https://img.shields.io/crates/v/rlg.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Registry" height="28" /></a>
  <a href="https://docs.rs/rlg"><img src="https://img.shields.io/badge/docs.rs-rlg-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs" height="28" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/rlg"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/rlg?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" height="28" /></a>
  <a href="https://www.bestpractices.dev/projects/15160"><img src="https://img.shields.io/cii/level/15160?style=for-the-badge&label=OpenSSF%20Best%20Practices&logo=openssf" alt="OpenSSF Best Practices" height="28" /></a>
  <a href="#license"><img src="https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg?style=for-the-badge" alt="License: Apache-2.0 OR MIT" height="28" /></a>
  <a href="https://github.com/sebastienrousseau/rlg/blob/main/docs/POLICIES.md"><img src="https://img.shields.io/badge/MSRV-1.89.0-93450a.svg?style=for-the-badge&logo=rust" alt="Minimum supported Rust version 1.89.0" height="28" /></a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/sebastienrousseau/rlg/main/.github/demo.gif" alt="The rlg CLI filtering a service log to errors in Elastic Common Schema, then rlg-report summarising the same file by level, component and message" width="100%" />
</p>

---

## Contents

**Getting started**

- [Install](#install): Cargo for the libraries, `cargo install` for the CLIs and the MCP server
- [Requirements](#requirements): toolchain floor, platforms
- [Quick Start](#quick-start): one structured record, fired and flushed

**The rlg ecosystem**

- [The rlg ecosystem](#the-rlg-ecosystem): `rlg`, `rlg-core`, `rlg-cli`, `rlg-report`, `rlg-mcp`, `rlg-otlp`, `rlg-redact`, `rlg-tower`, `rlg-test`, `rlg-wasm`, `rlg-ebpf`, and the `rustlogs.github.io` website

**Library reference**

- [Capabilities at a glance](#capabilities-at-a-glance): the current surface by theme
- [Ecosystem comparison](#ecosystem-comparison): short matrix; full table at [`docs/COMPARISON.md`](docs/COMPARISON.md)
- [Benchmarks](#benchmarks): headline numbers; full table at [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md)
- [Features](#features): module-level capability list
- [Configuration](#configuration): core options
- [Examples](#examples): runnable example index

**Operational**

- [When not to use rlg](#when-not-to-use-rlg): limitations
- [Development](#development): make targets, fuzzing, CI
- [Security](#security): guarantees and compliance
- [Documentation](#documentation): all reference docs
- [Stability guarantees](#stability-guarantees): SemVer axis, output stability, minimum toolchain discipline
- [License](#license)

---

## Install

### As a Rust library

```toml
[dependencies]
rlg = "0.0.23"
```

Satellites install the same way, all at the same version:

```toml
[dependencies]
rlg-otlp   = "0.0.23"  # ship records to an OpenTelemetry Collector
rlg-tower  = "0.0.23"  # per-request access logs for tower services
rlg-redact = "0.0.23"  # scrub secrets and PII before they are written

[dev-dependencies]
rlg-test   = "0.0.23"  # assert on captured records in tests
```

### Command-line tools

```bash
cargo install rlg-cli      # `rlg`: filter and convert log files
cargo install rlg-report   # `rlg-report`: summaries by level, component and message
cargo install rlg-mcp      # `rlg-mcp`: log files as tools for AI agents
```

From a checkout, `make install` builds the three binaries and installs
them with their manpages and bash, zsh and fish completions under
`/usr/local` (`PREFIX` and `DESTDIR` are honoured; `make uninstall`
reverses it). The binaries generate both themselves:
`rlg --manpage > rlg.1`, `rlg --completions zsh > _rlg`.

`rlg-mcp` is also published as a container image,
`ghcr.io/sebastienrousseau/rlg-mcp`, and listed in the MCP registry.

---

## Requirements

- **Rust 1.89.0 or newer** (edition 2024). CI builds every library and
  binary on 1.89.0; see the [toolchain policy](docs/POLICIES.md).
- **macOS or Linux** for the native sinks (`os_log`, `journald`).
  Everywhere else, including Windows, rlg writes to a file or stdout.

---

## Quick Start

```rust
use rlg::log::Log;
use rlg::log_format::LogFormat;

fn main() {
    // Keep the guard for the life of the program: dropping it flushes
    // pending records and stops the background thread.
    let _guard = rlg::init().unwrap();

    Log::info("User authenticated")
        .component("auth-service")
        .with("user_id", 42)
        .format(LogFormat::JSON)
        .fire();
}
```

`fire()` checks the level, pushes the record into a ring buffer and
returns; the flusher thread formats it as JSON and writes it to the
platform sink. This block is compiled and run by `cargo test`.

---

## The rlg ecosystem

Eleven crates and the website, released together at lockstep version `0.0.23`.

| Component | Purpose | Use case |
| :--- | :--- | :--- |
| [`rlg`](crates/rlg/README.md) | The engine: 65,536-slot ring buffer, background flusher, 14 formats, native sinks | Structured logging in any Rust program |
| [`rlg-core`](crates/rlg-core/README.md) | The `no_std` + `alloc` core: `LogLevel`, a borrowed `Record` and the 14 formats over `core::fmt::Write`; rlg re-exports it | Logging on bare-metal and embedded targets |
| [`rlg-cli`](crates/rlg-cli/README.md) | The `rlg` binary: filter by level, component or attribute, convert between formats | `my-service \| rlg --min-level error --format ecs` |
| [`rlg-report`](crates/rlg-report/README.md) | The `rlg-report` binary and library: counts by level and component, top messages, latency percentiles | On-call triage, daily error digests |
| [`rlg-mcp`](crates/rlg-mcp/README.md) | MCP server: four tools, one prompt, two resources over stdio, streamable HTTP or HTTP+SSE | AI agents reading production logs |
| [`rlg-otlp`](crates/rlg-otlp/README.md) | OTLP/HTTP exporter to a local OpenTelemetry Collector | Honeycomb, Datadog, Grafana through a Collector |
| [`rlg-redact`](crates/rlg-redact/README.md) | Scrubs cards, JWTs, bearer tokens, emails, IPv4 addresses and AWS keys in one pass | GDPR and audit-trail safety |
| [`rlg-tower`](crates/rlg-tower/README.md) | `tower::Layer` emitting one access-log record per request | axum, hyper and other `tower` services |
| [`rlg-test`](crates/rlg-test/README.md) | Capture records in a test and assert on them | Libraries testing their own log output |
| [`rlg-wasm`](crates/rlg-wasm/README.md) | `wasm-bindgen` bindings and a WASI 0.2 component interface | Browsers, Deno, Workers, wasmtime hosts |
| [`rlg-ebpf`](crates/rlg-ebpf/README.md) | Process enrichment, portable; an eBPF enricher scaffold on Linux | Host context on every record |
| [`rustlogs.github.io`](https://github.com/sebastienrousseau/rustlogs.github.io) | The project website, [rustlogs.com](https://rustlogs.com/), in lockstep since 0.0.22 | Finding rlg, its guides and its releases |

---

## Capabilities at a glance

| Area | Capability | Status |
| :--- | :--- | :--- |
| Ingestion | Atomic level filter, ring-buffer push, no lock on the caller's thread | Stable |
| Formats | JSON, NDJSON, ECS, GELF, Logstash, OTLP, MCP, logfmt, CLF, CEF, ELF, W3C, Apache, Log4j XML | Stable |
| `no_std` | Levels, a borrowed record and all 14 formats in `#![no_std]` + `alloc` (`rlg-core`) | New |
| Sinks | `os_log` (macOS), `journald` (Linux), file, stdout | Stable |
| Sinks | `io_uring` file sink (Linux, `uring` feature) | Scaffold |
| Configuration | TOML load, validation, polling hot-reload (`tokio` feature) | Stable |
| Bridges | `log` facade via `rlg::init()`; `tracing` layer (`tracing-layer` feature) | Stable |
| Export | OTLP/HTTP to a local Collector (`rlg-otlp`) | Stable |
| Agents | MCP server over stdio, streamable HTTP and HTTP+SSE (`rlg-mcp`) | Stable |
| Enrichment | eBPF kernel context (`rlg-ebpf`, Linux) | Scaffold |

---

## Ecosystem comparison

rlg trades `tracing`'s span model for an event-only engine that keeps
formatting and I/O off the application thread and ships native sinks,
many formats and an agent interface in the box.

| Project | Formatting off the caller's thread | Built-in formats | Native `journald` / `os_log` |
| :--- | :---: | :---: | :---: |
| **rlg** | yes | 14 | yes |
| `tracing` + `tracing-subscriber` | add-on | text, JSON | add-on |
| `slog` | add-on | add-on | add-on |
| `log` + `env_logger` | no | text | no |

See [`docs/COMPARISON.md`](docs/COMPARISON.md) for the evidence and complete matrix.

---

## Benchmarks

The `competitive_bench` suite times rlg's `fire()` against
`tracing::info!` and `log::info!` for single records, records with
attributes, 10,000-record bursts and latency distribution. It runs in
CI on every release tag, and its results are published as workflow
artifacts.

| Scenario | Result | Environment |
| :--- | ---: | :--- |
| rlg `fire()`, one record | 598 ns | GitHub-hosted `ubuntu-latest`, release profile |
| `tracing::info!` to a discarding writer | 591 ns | same run |
| rlg `fire()` with 3 attributes | 947 ns (`tracing`: 1,117 ns) | same run |

On the calling thread rlg now costs about what `tracing` costs, without
formatting there; what it buys is that the caller never waits on the
sink's I/O.

See [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) for methodology and full results.

---

## Features

Every Cargo feature of every crate, with its default. Two are on by
default: `config` in `rlg` and `blocking` in `rlg-otlp`. The crates
not listed (`rlg-cli`, `rlg-report`, `rlg-mcp`, `rlg-tower`,
`rlg-test`, `rlg-wasm`) have no features.

| Feature | Crate | Default | Adds |
| :--- | :--- | :--- | :--- |
| `config` | `rlg` | on | `Config` load, save and `set` from TOML and the environment |
| `tokio` | `rlg` | off | `Config::load_async` and polling hot-reload |
| `tui` | `rlg` | off | A live terminal dashboard, started with `RLG_TUI=1` |
| `tracing-layer` | `rlg` | off | `RlgLayer`, a `tracing_subscriber::Layer` |
| `fast-queue` | `rlg` | off | An eight-shard ring buffer for many concurrent producers |
| `uring` | `rlg` | off | The `io_uring` file sink on Linux |
| `debug_enabled` | `rlg` | off | Nothing yet: declared, but no code reads it |
| `blocking` | `rlg-otlp` | on | Nothing extra: the synchronous exporter over `std::net` is always built |
| `async` | `rlg-otlp` | off | `AsyncOtlpExporter` on Tokio |
| `tracing` | `rlg-otlp` | off | `SpanExporter` for closed `RlgLayer` spans |
| `tracing` | `rlg-redact` | off | `Scrub` for `Redactor`, so `RlgLayer` redacts records and spans |
| `ebpf` | `rlg-ebpf` | off | The `EbpfEnricher` scaffold, which delegates to process enrichment today |

---

## Configuration

`Config` loads from TOML, validates, and creates the log paths it needs.

```toml
# rlg.toml
version              = "1.0"
profile              = "production"
log_file_path        = "/var/log/rlg.log"
log_level            = "INFO"
logging_destinations = [
    { type = "File", value = "/var/log/rlg.log" },
    { type = "Stdout" },
]
log_rotation         = { Size = 10485760 }  # 10 MiB
```

Load it with `Config::load(Some("rlg.toml"))`, or with the `tokio`
feature `Config::load_async`, and follow edits with
`Config::hot_reload_async`. `RUST_LOG` sets the level too. The
[crate README](crates/rlg/README.md#configuration) lists every field.

---

## Examples

| Example | Shows | Run |
| :--- | :--- | :--- |
| `example_log_format` | All 14 formats side by side | `cargo run -p rlg --example example_log_format` |
| `example_config` | TOML config, async load, hot-reload | `cargo run -p rlg --example example_config --features tokio` |
| `example_macros` | The logging macros | `cargo run -p rlg --example example_macros` |
| `serve_a_session` | A full MCP session against `rlg-mcp` | `cargo run -p rlg-mcp --example serve_a_session` |
| `honeycomb` | Exporting through a local Collector | `cargo run -p rlg-otlp --example honeycomb` |

CI runs every example on each pull request (`examples-smoke.yml`), except
`example_config` and `example_utils`, async demos whose behaviour the
crate's own tests cover.

---

## When not to use rlg

- **Low volume.** Below about a hundred records a second, a background
  thread and a 65k-slot buffer buy nothing a synchronous logger does not
  already give you.
- **Spans are your model.** rlg records events. If hierarchical spans are
  what you analyse, use `tracing` directly; rlg's layer only bridges its
  events.
- **Windows Event Log.** There is no native Windows sink; rlg writes to a
  file or stdout there.
- **Every record must survive a crash.** Records wait in memory until the
  flusher writes them; a process killed before the `FlushGuard` drops
  loses what is still buffered.
- **You need a stable API.** rlg is at `0.0.x`: any release may break,
  and it has one maintainer.

---

## Development

```bash
make verify   # everything CI runs on a pull request
make demo     # re-render .github/demo.gif from .github/demo.tape
```

[`DEVELOPMENT.md`](DEVELOPMENT.md) lists every gate with its local
command: Miri, Loom and Kani for the engine's concurrency and memory
safety, four fuzz targets, cargo-deny and cargo-vet for the dependency
tree, cargo-semver-checks, a 95% coverage floor and complexity ceilings.
Contribution and signing rules are in [`CONTRIBUTING.md`](CONTRIBUTING.md)
and [`AGENTS.md`](AGENTS.md).

---

## Security

- **Reporting**: privately, by email or GitHub's advisory form; never in a
  public issue. The process and response time are in `SECURITY.md`.
- **Memory safety**: `unsafe` is denied across the workspace except the
  documented macOS `os_log` FFI in `crates/rlg/src/sink.rs`. The engine
  runs under Miri on every push.
- **Resource limits**: the ring buffer is bounded at 65,536 records and
  evicts the oldest when full; `rlg-otlp` caps a collector's response
  headers at 16 KiB and times out every request.
- **Fuzzing**: five cargo-fuzz targets (record parsing, format names,
  config loading, redaction, single-line rendering). On every pull
  request and every push to `main`, each target replays its committed
  corpus where one exists (today `render_one_line`), then fuzzes for
  30 seconds; a manual run can pick one target and a longer budget.
  See [`docs/OSS-FUZZ.md`](docs/OSS-FUZZ.md) for OSS-Fuzz status.
- **Supply chain**: every dependency passes cargo-deny and cargo-vet;
  releases publish through crates.io Trusted Publishing with
  sigstore-signed SBOMs ([`pkg/VERIFY.md`](pkg/VERIFY.md)).

Report vulnerabilities according to [`SECURITY.md`](SECURITY.md).

---

## Documentation

| Document | For |
| :--- | :--- |
| [User manual](https://doc.rustlogs.com/manual/) | Tutorials, how-to guides, architecture, ADRs ([source](docs/)) |
| [API reference](https://docs.rs/rlg) | Every public item, per crate on docs.rs |
| [`DEVELOPMENT.md`](DEVELOPMENT.md) | Toolchain, local gates, test layout, release model |
| [Ecosystem map](#the-rlg-ecosystem) | Every crate and the [rustlogs.com](https://rustlogs.com/) website, released in lockstep |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | How the engine and satellites fit together |
| [`CHANGELOG.md`](CHANGELOG.md) | What changed in each release |
| [Migration guides](docs/migration/) | Coming from `log`, `slog` or `tracing` |

---

## Stability guarantees

- **SemVer axis**: rlg is at `0.0.x`, so under Cargo's rules every release
  may be breaking. Breaking changes are marked **Breaking** in the
  CHANGELOG, and cargo-semver-checks runs on every pull request so none
  slips through unannounced.
- **Deprecation window**: a removal or contract change is deprecated
  first and keeps working, with a warning, for at least 90 days and one
  release; open deprecations and their dates are in
  [DEPRECATIONS.md](DEPRECATIONS.md), the policy in
  [docs/POLICIES.md](docs/POLICIES.md#deprecation).
- **Output stability**: the bytes a format produces are an interface. A
  change to them for the same record is breaking even when no signature
  moves, and is recorded the same way.
- **Minimum toolchain**: Rust 1.89.0, enforced in CI; a rise is its own
  CHANGELOG entry with the reason. [`docs/POLICIES.md`](docs/POLICIES.md)
  has the full policy.

---

## License

Dual-licensed under either of

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE))
- MIT license ([`LICENSE-MIT`](LICENSE-MIT))

at your option. Unless you state otherwise, any contribution you submit
for inclusion in rlg is dual-licensed as above, without additional terms.
