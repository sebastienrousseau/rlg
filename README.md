<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/rlg/v1/logos/rlg.svg" alt="RLG logo" width="128" />
</p>

<h1 align="center">rlg — RustLogs</h1>

<p align="center">
  Near-lock-free structured logging for Rust, plus an ecosystem of
  companion crates: CLI tooling, MCP server, OTLP exporter, tower
  middleware, WebAssembly bindings, PII redaction, test utilities,
  and aggregation reports.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/rlg/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rlg/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://github.com/sebastienrousseau/rlg/actions/workflows/miri.yml"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rlg/miri.yml?style=for-the-badge&logo=rust&label=Miri" alt="Miri" /></a>
  <a href="https://crates.io/crates/rlg"><img src="https://img.shields.io/crates/v/rlg.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/rlg"><img src="https://img.shields.io/badge/docs.rs-rlg-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://codecov.io/gh/sebastienrousseau/rlg"><img src="https://img.shields.io/codecov/c/github/sebastienrousseau/rlg?style=for-the-badge&logo=codecov" alt="Coverage" /></a>
  <a href="https://lib.rs/crates/rlg"><img src="https://img.shields.io/badge/lib.rs-rlg-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
</p>

---

This is the Cargo workspace root. The library lives at
[`crates/rlg`](crates/rlg). Nine companion crates ship from this
workspace, all at lockstep version `0.0.11`.

## The rlg ecosystem

| Crate | What it does | Use case |
|---|---|---|
| **[`rlg`](crates/rlg/README.md)** | Near-lock-free structured logging engine. 65k-slot ring buffer, deferred formatting, 14 output formats, native OS sinks (`os_log` via `syslog(3)`, `journald`). | Embed structured logging in any Rust binary or library. |
| **[`rlg-cli`](crates/rlg-cli/README.md)** | `rlg` binary — `jq` for structured logs. Tail, filter, convert across all 14 formats. | Pipe `my-service \| rlg --min-level error --format ecs` from the shell. |
| **[`rlg-mcp`](crates/rlg-mcp/README.md)** | Model Context Protocol server exposing rlg streams as tools to LLM agents. | Claude Desktop, Cursor, mcp.run agents reading your logs. |
| **[`rlg-otlp`](crates/rlg-otlp/README.md)** | OpenTelemetry network exporter (OTLP/HTTP JSON). | Ship records to Honeycomb, Datadog, Tempo, Jaeger, otelcol. |
| **[`rlg-tower`](crates/rlg-tower/README.md)** | `tower::Layer` emitting per-request structured access logs. | axum, tonic, hyper, `lambda_runtime` — any `tower::Service`. |
| **[`rlg-wasm`](crates/rlg-wasm/README.md)** | `wasm-bindgen` wrapper for browser / Deno / Cloudflare Workers / Bun. | Structured logging from JS via `RlgWasm`. |
| **[`rlg-redact`](crates/rlg-redact/README.md)** | PII / secret redaction between `Log::fire()` and the sink. | Compliance, GDPR, audit-trail safety. Built-in patterns for cards, JWTs, bearers, emails, IPs, AWS keys. |
| **[`rlg-test`](crates/rlg-test/README.md)** | Test utilities — capture rlg records in a `#[test]` scope and assert on them with `assert_logged!`. | Downstream library tests verifying their structured log output. |
| **[`rlg-report`](crates/rlg-report/README.md)** | Log digest / analytics: count by level, top components, top errors, latency percentiles. CLI + library mode. | Operational dashboards, daily error reports, oncall triage. |
| `xtask` *(internal, unpublished)* | `cargo xtask check-all / coverage / audit / doc / examples / bench / release`. | Maintainer automation. |

### Install one, install all

```toml
[dependencies]
rlg         = "0.0.11"
rlg-otlp    = "0.0.11"  # ship to an OTLP collector
rlg-tower   = "0.0.11"  # HTTP middleware
rlg-redact  = "0.0.11"  # PII redaction

[dev-dependencies]
rlg-test    = "0.0.11"  # assertions in your tests
```

CLI binaries:

```bash
cargo install rlg-cli       # the `rlg` binary
cargo install rlg-mcp       # the `rlg-mcp` MCP server
cargo install rlg-report    # the `rlg-report` digest tool
```

## Workspace layout

```text
.
├── Cargo.toml            # workspace manifest (profiles, members)
├── README.md             # ← you are here
├── CHANGELOG.md / CONTRIBUTING.md / SECURITY.md
├── LICENSE-{APACHE,MIT}
├── crates/
│   ├── rlg/              # core library
│   ├── rlg-cli/          # `rlg` binary + filter/render lib
│   ├── rlg-mcp/          # MCP server
│   ├── rlg-otlp/         # OTLP/HTTP exporter
│   ├── rlg-tower/        # tower::Layer
│   ├── rlg-wasm/         # wasm-bindgen wrapper
│   ├── rlg-redact/       # PII / secret scrubber
│   ├── rlg-test/         # test utilities
│   ├── rlg-report/       # log digest binary + lib
│   └── xtask/            # internal automation (publish = false)
└── .github/
    └── workflows/
        ├── ci.yml        # delegates to sebastienrousseau/pipelines
        └── release.yml   # workspace-aware crates.io publisher
```

## Quick links

- **Core docs:** [`crates/rlg/README.md`](crates/rlg/README.md) · [docs.rs/rlg](https://docs.rs/rlg)
- **Contributing & signing policy:** [`CONTRIBUTING.md`](CONTRIBUTING.md)
- **Security policy:** [`SECURITY.md`](SECURITY.md)
- **Release notes:** [`CHANGELOG.md`](CHANGELOG.md)

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.
