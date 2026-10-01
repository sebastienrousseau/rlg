<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Comparison

Where rlg sits among Rust logging crates. This page compares
capabilities, not speed; measured numbers are in
[BENCHMARKS.md](BENCHMARKS.md). "Add-on" means the capability exists
through a separate crate rather than the crate itself.

| Capability | rlg | `tracing` + `tracing-subscriber` | `slog` | `log` + `env_logger` |
| :--- | :---: | :---: | :---: | :---: |
| Structured key-value records | yes | yes | yes | key-values behind a feature |
| Hierarchical spans as the data model | no (events only) | yes | no | no |
| Formatting off the caller's thread by default | yes | add-on (`tracing-appender`) | add-on (`slog-async`) | no |
| Built-in output formats | 14 (JSON, ECS, GELF, OTLP, MCP, logfmt, CLF, …) | text and JSON | add-on drains | text |
| `journald` / `os_log` sinks built in | yes | add-on | add-on | no |
| OTLP export | `rlg-otlp` (via a local Collector) | add-on (`opentelemetry` crates) | add-on | no |
| Log files exposed to AI agents over MCP | `rlg-mcp` | no | no | no |
| CLI to filter and convert log files | `rlg` (`rlg-cli`) | no | no | no |
| Maturity | `0.0.x`, one maintainer | widely adopted | established | the ecosystem facade |

## Choosing

- **Pick rlg** when records should leave the application thread
  quickly, land in the platform's native log store, or be read by
  agents and tools in one of many formats.
- **Pick `tracing`** when spans and their context are the model you
  want, or you need its large ecosystem of layers.
- **Pick `log` with a small backend** for low-volume tools where a
  synchronous write is simplest.

rlg bridges both facades: `rlg::init()` installs a `log` logger, and the
`tracing-layer` feature adds a `tracing_subscriber::Layer`, so a program
can keep its existing macros and route them through rlg. The migration
guides cover [`log`](migration/from-log.md),
[`slog`](migration/from-slog.md) and
[`tracing`](migration/from-tracing.md).
