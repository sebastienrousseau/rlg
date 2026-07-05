<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/rlg/v1/logos/rlg.svg" alt="RLG logo" width="128" />
</p>

<h1 align="center">rlg</h1>

<p align="center">
  Near-lock-free structured logging for Rust. Sub-microsecond ingestion
  via a 65k-slot ring buffer, deferred formatting, and native OS sinks.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/rlg/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rlg/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/rlg"><img src="https://img.shields.io/crates/v/rlg.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/rlg"><img src="https://img.shields.io/badge/docs.rs-rlg-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://codecov.io/gh/sebastienrousseau/rlg"><img src="https://img.shields.io/codecov/c/github/sebastienrousseau/rlg?style=for-the-badge&logo=codecov" alt="Coverage" /></a>
  <a href="https://lib.rs/crates/rlg"><img src="https://img.shields.io/badge/lib.rs-rlg-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
</p>

---

## Contents

**Getting started**

- [Install](#install) — Cargo, source, MSRV, features
- [Quick Start](#quick-start) — initialise and emit in ten lines

**Library reference**

- [Why this approach?](#why-this-approach) — design rationale
- [Capabilities in 0.0.11](#capabilities-in-0011) — release inventory
- [The fluent API](#the-fluent-api) — `Log::info(...).with(...).fire()`
- [Output formats](#output-formats) — every `LogFormat` variant
- [Sinks](#sinks) — `os_log`, `journald`, file, stdout
- [Configuration](#configuration) — TOML, env vars, hot-reload
- [Log rotation](#log-rotation) — size, time, date, count
- [Bridging existing facades](#bridging-existing-facades) — `log`, `tracing`
- [Examples](#examples) — runnable example index

**Operational**

- [When not to use rlg](#when-not-to-use-rlg) — limitations
- [Development](#development) — local verification, fuzzing, CI
- [Security](#security) — guarantees and compliance
- [Documentation](#documentation) — all reference docs
- [License](#license)

---

## Install

### As a Rust library (crates.io)

```toml
[dependencies]
rlg = "0.0.11"
```

### Build from source

```bash
git clone https://github.com/sebastienrousseau/rlg.git
cd rlg
cargo check --all-features
cargo test  --all-features
```

`rlg` targets Rust **1.88.0** (MSRV) and edition 2024. It runs
on macOS, Linux, and WSL; Windows is supported on a best-effort
basis via the stdout fallback sink.

### Cargo features

All optional integrations are off by default. Enable only what
the application needs.

| Feature | Pulls in | Adds | Documented in |
| :--- | :--- | :--- | :--- |
| `tokio` | `tokio` + `notify` | `Config::load_async`, file-watcher hot-reload | [Configuration](#configuration), `examples/example_config.rs` |
| `tui` | `terminal_size` | Live terminal dashboard at `RLG_TUI=1` | [Capabilities](#capabilities-in-0011) |
| `miette` | `miette` 7 | Pretty diagnostic error reports | [Library reference](#capabilities-in-0011) |
| `tracing-layer` | `tracing-subscriber` | `RlgLayer` for composable `tracing` setups | [Bridging existing facades](#bridging-existing-facades) |
| `debug_enabled` | — | Verbose internal engine diagnostics | — |

```toml
# Example: async config loading + tracing bridge
[dependencies]
rlg = { version = "0.0.11", features = ["tokio", "tracing-layer"] }
```

---

## Quick Start

```rust
use rlg::log::Log;
use rlg::log_format::LogFormat;

fn main() {
    // Hold the FlushGuard for the lifetime of `main`. Dropping it
    // flushes pending events and joins the background thread.
    let _guard = rlg::init().unwrap();

    Log::info("User authenticated")
        .component("auth-service")
        .with("user_id", 42)
        .with("session_uuid", "a1b2c3d4")
        .format(LogFormat::MCP)
        .fire();
}
```

Output (MCP / JSON-RPC 2.0 notification):

```json
{"jsonrpc":"2.0","method":"notifications/log","params":{"data":{"attributes":{"caller":"src/main.rs:9","session_uuid":"a1b2c3d4","user_id":42},"component":"auth-service","description":"User authenticated","session_id":1,"time":"2026-05-29T22:18:04.123456789Z"},"level":"info"}}
```

The default ingestion path runs in **~1.4 µs** — `Log::fire()`
pushes a fully-built event into the `crossbeam::ArrayQueue` and
returns. A dedicated flusher thread picks the event up, runs the
chosen `LogFormat`'s `Display` impl, and dispatches to the
configured `PlatformSink`.

---

## Why this approach?

rlg targets the niche `log` / `tracing` / `env_logger` /
`fern` occupy — emit structured records from application
threads, route them somewhere durable — and is written
**lock-free on the hot path** against the LMAX Disruptor
pattern. The engine runs MIRI-clean under
`-Zmiri-tree-borrows`; 99.07 % of source lines and 99.30 % of
functions are covered by tests.

Two architectural choices motivate the design:

1. **Atomic ingestion, deferred formatting.** `Log::fire()`
   only does the work that *cannot* be deferred: capture
   `file:line` via `#[track_caller]`, increment the per-format
   metrics counter, and push into the ring buffer. The
   serialisation (`fmt_json`, `fmt_mcp`, `fmt_otlp`, …) and the
   `os_log` / `journald` / `write_all` syscalls all run on the
   flusher thread, off the caller's critical path. The pattern
   that mainstream Rust loggers use — *take a Mutex, format
   into a String, write to a Writer* — is ~20 µs at p50 and
   pathologically variable under contention. rlg measures
   ~1.4 µs at p50 with no Mutex anywhere on the hot path.

2. **POSIX `syslog(3)` for the macOS sink, not `_os_log_impl`.**
   Apple's `os_log` macro expands into a binary-trailer
   calling convention that cannot be reproduced from Rust
   without inline assembly. Calling the private
   `_os_log_impl` symbol directly with raw bytes — as several
   Rust wrappers do — is undefined behaviour and crashes
   sporadically. rlg routes through `syslog(3)` instead, which
   on macOS Sierra+ is gateway'd into `os_log` automatically;
   records still appear in Console.app and `log stream`, the
   ABI is stable, and the FFI surface is one
   `extern "C" fn syslog(c_int, *const c_char, *const c_char)`
   call with a static `c"%s"` format.

A few features built on top of those choices:

- **65k-slot ring buffer.** `crossbeam_queue::ArrayQueue<LogEvent>`
  with capacity tuned for typical service throughput; overflow
  evicts the oldest event and increments `TuiMetrics::dropped`.
- **Low-allocation serialisation.** `session_id` is `u64`;
  `component` and `time` are `Cow<'static, str>` so static
  strings stay on the stack; `itoa` / `ryu` format integers and
  floats without allocating.
- **Native sinks, not file wrappers.** `os_log` (macOS via
  `syslog(3)`) and `journald` (Linux via the Unix datagram
  socket) integrate at the syslog protocol level — `journalctl
  -u my-service`, Console.app, `log stream --predicate
  'subsystem == "rlg"'` all light up automatically.

The runtime default profile carries **seven runtime crates**
plus the well-vetted `serde` family. Disabling all optional
features keeps the engine compiling to the same seven; the
`tokio` runtime, `terminal_size` for the TUI, `miette` for
diagnostics, and `tracing-subscriber` for the layer bridge
are strictly opt-in.

---

## Capabilities in 0.0.11

- **14 output formats.** CLF, CEF, ELF, W3C, Apache Access,
  Log4j XML, JSON, GELF, Logstash, NDJSON, MCP, OTLP, Logfmt,
  ECS. Switch with `.format(LogFormat::X)` per-entry or set the
  default via `RlgBuilder::format`.
- **Native platform sinks.** `os_log` on macOS via POSIX
  `syslog(3)`; `journald` on Linux via the
  `/run/systemd/journal/socket` Unix datagram; stdout and
  rotating files as fallbacks.
- **Background flusher thread.** Single OS thread spawned at
  init, drained on `FlushGuard::drop` / `ENGINE.shutdown()`.
  `#[cfg_attr(miri, ignore)]` on tests that spawn it; MIRI
  itself never sees the flusher.
- **`#[track_caller]` everywhere it matters.** `Log::fire()`
  records `file:line` for every entry. The caller string ends
  up in the `caller` attribute alongside your other
  `.with(...)` keys.
- **AI-native formats.** **MCP** is JSON-RPC 2.0 over
  `notifications/log` — designed for Model Context Protocol
  agents (Claude Desktop, Cursor, mcp.run). **OTLP** maps to
  OpenTelemetry's `severityNumber` / `severityText` /
  `spanId` / `traceId` so an `otelcol` pipeline picks up rlg
  records without an adapter.
- **TOML configuration with hot-reload.** `Config::load_async`
  + the `notify` file watcher (behind the `tokio` feature)
  picks up `/etc/rlg.toml` mutations without a restart.
- **Bridges for `log` and `tracing`.** `rlg::init()`
  installs a `log::Log` implementation; the `tracing-layer`
  feature exposes a `tracing_subscriber::Layer` you can stack
  with the rest of your subscriber.
- **99.07 % line coverage.** Measured by `cargo llvm-cov`.
  Run on every PR via the centralised
  [`sebastienrousseau/pipelines`](https://github.com/sebastienrousseau/pipelines)
  reusable workflows.

---

## The fluent API

```rust
use rlg::log::Log;
use rlg::log_format::LogFormat;

Log::error("Service degraded")
    .component("orchestrator")
    .with("region",     "us-east-1")
    .with("cpu_load",   0.85)
    .with("queue_size", 4096_u64)
    .format(LogFormat::OTLP)
    .fire();
```

| Method | Effect |
| :--- | :--- |
| `Log::info("…")` | Create at INFO level. Also: `warn`, `error`, `debug`, `trace`, `fatal`, `critical`, `verbose`. |
| `Log::build(level, "…")` | Create at an explicit `LogLevel`. |
| `.component("name")` | Tag the originating service or module. |
| `.with("key", value)` | Attach a key-value attribute. `value: T: Serialize` covers strings, numbers, booleans, arrays, structs. |
| `.time("…")` | Override the auto-captured ISO 8601 timestamp. |
| `.session_id(u64)` | Override the auto-assigned monotonic ID. |
| `.format(LogFormat::X)` | Pick the wire format for this entry. |
| `.fire()` | Consume the builder, capture `file:line` via `#[track_caller]`, push into the ring buffer. |

Every method returns `Self`, so chains compose freely.

---

## Output formats

The 14 variants of `LogFormat` cover most ingestion targets
without an adapter:

| Format | Use case | Example consumer |
| :--- | :--- | :--- |
| **`MCP`** *(default)* | LLM agent telemetry over JSON-RPC 2.0 | Claude Desktop, Cursor, mcp.run |
| **`OTLP`** | OpenTelemetry-native | `otelcol`, Honeycomb, Datadog OTel |
| **`JSON`** | Structured JSON for ingest pipelines | Vector, Fluent Bit, Loki |
| **`NDJSON`** | One record per line | Loki, ClickHouse |
| **`ECS`** | Elastic Common Schema | Elasticsearch, OpenSearch |
| **`Logstash`** | Logstash-flavoured JSON | Logstash, OpenSearch Pipelines |
| **`GELF`** | Graylog Extended Log Format | Graylog |
| **`CEF`** | Common Event Format | SIEMs (Splunk, ArcSight) |
| **`CLF`** | Common Log Format | nginx-style access logs |
| **`ELF`** | Extended Log Format | Legacy collectors |
| **`W3C`** | W3C Extended Log Format | Microsoft / legacy IIS |
| **`ApacheAccessLog`** | Apache combined log | Apache, awstats |
| **`Log4jXML`** | Log4j XML events | Java enterprise stacks |
| **`Logfmt`** | Human-readable `key=value` pairs | Heroku, terminal viewing |

Switch per entry: `.format(LogFormat::OTLP)`. Switch as a
process-wide default: `RlgBuilder::format(LogFormat::JSON)`.

---

## Sinks

`PlatformSink::native()` picks the best-available native sink
for the host. Override via `Config::logging_destinations`.

| Variant | Active when | Mechanism |
| :--- | :--- | :--- |
| `OsLog` | macOS | POSIX `syslog(3)` → routed into `os_log` by the system gateway. Visible in Console.app + `log stream`. |
| `Journald(Some(_))` | Linux + systemd | Unix datagram to `/run/systemd/journal/socket`. `journalctl` shows records immediately. |
| `Journald(None)` | Linux without journald | Falls back to stdout. |
| `File(_)` | Explicit `logging_destinations = [{ type = "file", path = "..." }]` | `OpenOptions::new().create(true).append(true)`. |
| `Stdout` | Explicit fallback or `RLG_FALLBACK_STDOUT=1` | `std::io::stdout().write_all`. |

`RLG_FALLBACK_STDOUT=1` (or `GITHUB_ACTIONS=1`) forces the
stdout sink — useful for CI runs and integration tests that
shouldn't pollute the real syslog.

---

## Configuration

`Config` deserialises from TOML or environment variables (via
`envy`). All fields have serde defaults so an empty file or
unset environment is valid.

```toml
# rlg.toml
version              = "1.0"
profile              = "production"
log_level            = "INFO"
log_format           = "%level - %message"
logging_destinations = [
    { type = "file",   path = "/var/log/rlg.log" },
    { type = "stdout" },
]

[log_rotation]
type      = "size"
threshold = 10485760            # 10 MiB
```

```rust,ignore
use rlg::config::Config;

// Sync load.
let cfg = Config::load(Some("/etc/rlg.toml"))?;

// Async load + file-watcher hot-reload (requires `tokio` feature).
#[cfg(feature = "tokio")]
let cfg = Config::load_async(Some("/etc/rlg.toml")).await?;
#[cfg(feature = "tokio")]
Config::hot_reload_async("/etc/rlg.toml", &cfg)?;
```

| Field | Type | Notes |
| :--- | :--- | :--- |
| `version` | `String` | Schema version. `1.0` is the only currently-accepted value. |
| `profile` | `String` | Free-form deployment tag. |
| `log_level` | `LogLevel` | Engine-wide filter level. Lower-severity records are dropped at `ingest()`. |
| `log_format` | `String` | Default formatting template (CLF/Logfmt). |
| `logging_destinations` | `Vec<LoggingDestination>` | Ordered list; `PlatformSink::from_config` picks the first openable one. |
| `log_rotation` | `Option<LogRotation>` | See [Log rotation](#log-rotation). |
| `env_vars` | `HashMap<String, String>` | `${NAME}` substituted from process env. |

The `RUST_LOG` environment variable is honoured too —
`RlgBuilder::init` overrides `self.level` with the most
permissive directive found (e.g. `RUST_LOG=warn,my_crate=debug`
yields `DEBUG`).

---

## Log rotation

`RotatingFile` enforces a policy on a wrapped `std::fs::File`.

| Policy | Triggers on | Configured by |
| :--- | :--- | :--- |
| `Size(NonZeroU64)` | bytes written ≥ threshold | `log_rotation = { type = "size", threshold = 10485760 }` |
| `Time(NonZeroU64)` | seconds elapsed since open ≥ threshold | `log_rotation = { type = "time", threshold = 3600 }` |
| `Date` | local date string changes | `log_rotation = { type = "date" }` |
| `Count(u32)` | events written ≥ threshold | `log_rotation = { type = "count", threshold = 10000 }` |

On rotation the current file is renamed to
`<stem>.<YYYYMMDD-HHMMSS>.<ext>` and a fresh file is opened at
the original path.

---

## Bridging existing facades

`rlg::init()` installs a global `log::Log` implementation —
existing code that calls `log::info!`, `log::warn!`, etc. is
re-routed through the rlg ring buffer with no source changes.

```rust,ignore
use log::{info, warn};

fn main() {
    let _guard = rlg::init().unwrap();
    info!("hello from the log crate facade");   // routes through rlg
    warn!(target: "my-component", "with-target works too");
}
```

For `tracing`, enable the `tracing-layer` feature and stack
`RlgLayer` with your existing subscriber:

```rust,ignore
use tracing_subscriber::prelude::*;

tracing_subscriber::registry()
    .with(rlg::tracing::RlgLayer::new())
    .with(tracing_subscriber::fmt::layer())
    .init();
```

`RlgLayer` mirrors every `tracing::event!` and span open/close
into the rlg ring buffer; span IDs surface as the `span_id`
attribute on emitted records.

---

## Examples

Seven runnable examples ship under `examples/`:

| Example | Demonstrates | Required features |
| :--- | :--- | :--- |
| `example.rs` | End-to-end usage walkthrough (every public API) | none |
| `example_lib.rs` | Library-style embedding | none |
| `example_macros.rs` | The `info!` / `warn!` / `error!` macros | none |
| `example_log_format.rs` | All 14 `LogFormat` variants side by side | none |
| `example_log_level.rs` | Level filtering + `LogLevel::includes` semantics | none |
| `example_config.rs` | TOML config + async loading + hot-reload | `tokio` |
| `example_utils.rs` | Datetime, span/trace ID, file utilities | `tokio` |

```bash
cargo run --example example_log_format
cargo run --example example_config --features tokio
```

---

## When not to use rlg

- **You're emitting fewer than 100 records per second.** The
  ring buffer + background flusher pay a fixed setup cost
  (one OS thread, ~256 KB ring) that only amortises at moderate
  throughput. For low-throughput tools, a synchronous logger
  like `env_logger` is simpler and just as fast at that
  volume.
- **You need stdout-only output and can tolerate Mutex
  contention.** `env_logger` and `simplelog` are 50 lines of
  dependency; rlg's ring buffer is overkill if you don't need
  the platform sinks or the deferred formatting.
- **You need to log under MIRI.** rlg's engine spawns a real
  OS thread on `LockFreeEngine::new` and the platform sinks
  call libc FFI. Most rlg tests are gated by
  `#[cfg_attr(miri, ignore)]` for that reason. For
  MIRI-clean inner-loop logging, write to a `Vec<u8>` and
  inspect it at the end.
- **You need Windows Event Log as a native sink.** rlg falls
  back to stdout on Windows. A native ETW sink is on the
  roadmap (`PlatformSink::WindowsEvent`) but not yet
  implemented.
- **You want `tracing`'s span-based recording model as the
  primary API.** The `tracing-layer` feature bridges
  `tracing` events into rlg, but rlg's primary API is
  per-event (`Log::info(...).fire()`). If you need
  hierarchical spans as a first-class data model, use
  `tracing-subscriber::fmt` directly.

If you hit a case that should be on this list, please open an
issue — that's how it gets fixed or moved into the supported set.

---

## Development

```bash
cargo fmt --check                                              # format
cargo clippy --all-features --tests --benches -- -D warnings   # lint
cargo test  --all-features                                     # unit + integration
cargo bench --bench competitive_bench                          # perf-sensitive changes only
cargo llvm-cov --all-features --summary-only                   # coverage
cargo doc   --all-features --no-deps                           # API docs
```

On macOS, the engine routes through `syslog(3)` by default —
add `RLG_FALLBACK_STDOUT=1` for tests that should never touch
the real system log.

### CI

| Workflow | Trigger | Purpose |
| :--- | :--- | :--- |
| `ci.yml` | push, PR | Delegates to `sebastienrousseau/pipelines/rust-ci.yml` (Clippy, fmt, test matrix, coverage), `security.yml` (cargo-audit + dependency review), and `docs.yml` (deploy API docs on `main`). |

The reusable workflows live in the centralised
[`sebastienrousseau/pipelines`](https://github.com/sebastienrousseau/pipelines)
repo. See [`CONTRIBUTING.md`](../../CONTRIBUTING.md) for the
signed-commit policy and PR flow.

---

## Security

### Memory safety

- `unsafe_code = "deny"` across the crate — the only `unsafe`
  block is the macOS `syslog(3)` FFI in `src/sink.rs`, which
  is fully documented and gated behind `#[cfg(target_os =
  "macos")]`.
- The FFI surface is a single
  `extern "C" fn syslog(c_int, *const c_char, *const c_char)`
  call with a static `c"%s"` format string and exactly one
  argument — no varargs UB, no
  `_os_log_impl`-style private-symbol calls.
- 99.07 % line coverage on the engine path, including the
  concurrent queue retry, the shutdown idempotency, and the
  `OsLog` priority mapping.

### Supply chain

- `cargo audit` clean — zero advisories. RUSTSEC-2024-0436
  (`paste` unmaintained, via `dtt`) was closed in 0.0.9 by
  inlining the ISO 8601 helper.
- Dependency count: **241** transitive crates. Down from 251
  before the `dtt` removal.
- Signed commits enforced — see [`SECURITY.md`](../../SECURITY.md)
  for the SSH-signing policy and reporting channel.

### Reporting

Vulnerabilities go through the private channel documented in
[`SECURITY.md`](../../SECURITY.md). Do not file public issues
for security problems.

---

## Documentation

| Document | Covers |
| :--- | :--- |
| [`doc/introduction.md`](doc/introduction.md) | Motivation and design overview. |
| [`doc/tutorials/getting-started.md`](doc/tutorials/getting-started.md) | Step-by-step first integration. |
| [`doc/how-to/fluent-api.md`](doc/how-to/fluent-api.md) | Building entries with the fluent builder. |
| [`doc/explanation/engine-design.md`](doc/explanation/engine-design.md) | LMAX Disruptor pattern as applied in rlg. |
| [`doc/explanation/safety.md`](doc/explanation/safety.md) | UB-free FFI design, MIRI posture. |
| [`SECURITY.md`](../../SECURITY.md) | Disclosure policy, supported versions, contact. |
| [`CONTRIBUTING.md`](../../CONTRIBUTING.md) | Signed-commit policy, PR guidelines, local-test recipe. |

API documentation is published at
[docs.rs/rlg](https://docs.rs/rlg) on every release.

---

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.

<p align="right"><a href="#contents">Back to Top</a></p>
