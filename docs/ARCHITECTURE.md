<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Architecture

How rlg is put together, for contributors. For how to use it, start
with the [introduction](introduction.md); for the reasoning behind
individual decisions, read the [ADRs](adr/).

## The workspace

Ten publishable crates share one version and are released together.

| Crate | Role | Depends on |
| :--- | :--- | :--- |
| `rlg` | The logging engine: records, formats, sinks, config | — |
| `rlg-cli` | `rlg` binary: parse, filter and render log files | `rlg` |
| `rlg-report` | `rlg-report` binary: summaries of a log file | `rlg`, `rlg-cli` |
| `rlg-mcp` | MCP server exposing log files as tools | `rlg`, `rlg-cli` |
| `rlg-otlp` | OTLP/HTTP exporter to an OpenTelemetry Collector | `rlg` |
| `rlg-redact` | Redaction of secrets and PII before a record is written | `rlg` |
| `rlg-tower` | `tower::Layer` emitting per-request access logs | `rlg` |
| `rlg-test` | Assertions over captured records in tests | `rlg` |
| `rlg-wasm` | WebAssembly bindings | `rlg` |
| `rlg-ebpf` | Enrichment of records with kernel context | `rlg` |

`crates/xtask` holds maintainer automation and is never published.

## The engine (`rlg`)

```text
application thread                 flusher thread (rlg-flusher)
──────────────────                 ────────────────────────────
Log::info("…").fire()
  └─ ENGINE.ingest(event)          loop:
       ├─ level filter (atomic)      drain ≤ 64 events
       ├─ ShardedQueue::push  ────▶  format each (Display)
       └─ unpark flusher             PlatformSink::emit
                                     park (5 ms fallback)
```

The split is the design: the application thread does one atomic level
check, one queue push and one `unpark`, and never formats, allocates a
string or takes a lock. Everything expensive happens on the flusher.

- **Records** (`log.rs`): `Log` is built through a fluent API and carries
  level, component, description, time, a `u64` session id and a
  `BTreeMap` of attributes. `component` and `time` are
  `Cow<'static, str>`, so static strings are never copied.
- **Queue** (`engine.rs`, `sharded_queue.rs`): a 65,536-slot ring buffer
  of `crossbeam::ArrayQueue`, one shard by default, eight with the
  `fast-queue` feature ([ADR 0009](adr/0009-sharded-producer-queue.md)).
  A full shard evicts its oldest record. The shutdown handshake and
  session-id monotonicity are checked by Loom
  ([ADR 0001](adr/0001-loom-verified-ring-buffer.md)) and Kani
  ([ADR 0004](adr/0004-kani-verified-invariants.md)).
- **Formats** (`log.rs`, `log/write.rs`): fourteen output formats (JSON,
  NDJSON, ECS, GELF, Logstash, OTLP, MCP, logfmt, CLF, CEF, ELF, W3C,
  Apache access log, Log4j XML) written straight to the formatter with
  no intermediate `serde_json::Value`. Their shape is property-tested
  ([ADR 0003](adr/0003-property-tested-formats.md)).
- **Sinks** (`sink.rs`): `os_log` on macOS through FFI (the one place
  `unsafe` is allowed), `journald` over its datagram socket on Linux, a
  file, or stdout. `io_uring` is an opt-in file sink on Linux
  ([ADR 0011](adr/0011-io-uring-file-sink.md)).
- **Configuration** (`config.rs` and `config/`): TOML loaded with
  `Config::load` or `load_async`, validated, and optionally hot-reloaded
  by polling the file (`config/hot_reload.rs`, `tokio` feature).
  Rotation policies (`size:N`, `time:N`, `date`, `count:N`) parse in
  `config/log_rotation.rs` and run in `rotation.rs`.
- **Bridges** (`logger.rs`, `tracing.rs`): `rlg::init()` installs a
  `log::Log` implementation; the `tracing-layer` feature adds a
  `tracing_subscriber::Layer`. Both feed the same engine.
- **Dashboard** (`tui.rs`): an opt-in terminal view of throughput,
  levels and formats, started with `RLG_TUI=1`.

## The satellites

- **`rlg-mcp`** serves four tools (`tail_log`, `filter_log`,
  `summarize_errors`, `tail_logs_glob`), one prompt and two resources
  through the official MCP SDK. `ops.rs` holds the operations as plain
  functions, `model.rs` the tool arguments and results, `lib.rs` the
  server, and `transport.rs` (shared verbatim across the suite's MCP
  servers) the stdio, streamable HTTP and HTTP+SSE transports.
- **`rlg-otlp`** sends OTLP/HTTP JSON to a local Collector, which owns
  TLS ([ADR 0015](adr/0015-otlp-local-collector-transport.md)). Both
  exporters use an in-house HTTP/1.1 client (`http.rs`), the blocking
  one over `std::net` and the async one over Tokio, and share retry,
  jitter and circuit-breaking
  from `backoff.rs` ([ADR 0010](adr/0010-otlp-pluggable-transport.md)).
- **`rlg-redact`** scans each value once, against a single regex that
  fuses every built-in pattern into one alternation
  ([ADR 0008](adr/0008-fused-redaction-automaton.md)).
- **`rlg-wasm`** and **`rlg-ebpf`** are scaffolds on their way to full
  implementations ([ADR 0013](adr/0013-wasi-0.2-component.md),
  [ADR 0012](adr/0012-ebpf-enricher.md)).

## Invariants the gates hold

| Invariant | Enforced by |
| :--- | :--- |
| No undefined behaviour in the engine | Miri on every push |
| Shutdown and ordering under concurrency | Loom proofs |
| Level and counter invariants | Kani proofs |
| Parsers survive hostile input | cargo-fuzz targets ([ADR 0002](adr/0002-fuzz-strategy.md)) |
| Dependencies are licensed, unique and reviewed | cargo-deny over all features, cargo-vet |
| Public API changes are deliberate | cargo-semver-checks |
| Functions and files stay small | `scripts/complexity-gate.py` against a baseline |
| Coverage stays above 95% | tarpaulin in CI |

Run all of them locally with `make verify`; see
[`DEVELOPMENT.md`](https://github.com/sebastienrousseau/rlg/blob/main/DEVELOPMENT.md).
