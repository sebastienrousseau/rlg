<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/rlg/v1/logos/rlg.svg" alt="RLG logo" width="128" />
</p>

<h1 align="center">rlg-otlp</h1>

<p align="center">
  OpenTelemetry network exporter for <code>rlg</code>. Ships records
  over OTLP/HTTP to any OTel-compatible collector.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/rlg/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rlg/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/rlg-otlp"><img src="https://img.shields.io/crates/v/rlg-otlp.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/rlg-otlp"><img src="https://img.shields.io/badge/docs.rs-rlg--otlp-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://lib.rs/crates/rlg-otlp"><img src="https://img.shields.io/badge/lib.rs-rlg--otlp-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/rlg"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/rlg?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" /></a>
</p>

---

## Why

The core `rlg` crate renders records in `LogFormat::OTLP` shape but only writes them to local sinks (stdout, file, `os_log`, `journald`). To actually *ship* the records to a collector you need a network transport. `rlg-otlp` is that transport.

OTLP/HTTP JSON is the v0.0.11 wire format. Protobuf is on the v0.0.12 roadmap.

## Install

```toml
[dependencies]
rlg       = "0.0.11"
rlg-otlp  = "0.0.11"
```

Requires Rust **1.88.0** or newer (edition 2024).

## Usage

```rust
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg_otlp::OtlpExporter;

let exporter = OtlpExporter::builder()
    .endpoint("https://api.honeycomb.io/v1/logs")
    .header("x-honeycomb-team", std::env::var("HONEYCOMB_API_KEY").unwrap())
    .timeout_secs(10)
    .build();

let record = Log::error("payment-service down")
    .component("orders")
    .with("trace_id", "abc123")
    .format(LogFormat::OTLP);

exporter.export_one(&record).unwrap();
```

## Endpoint examples

| Collector | URL |
| --- | --- |
| Honeycomb | `https://api.honeycomb.io/v1/logs` (set `x-honeycomb-team`) |
| Datadog | `https://http-intake.logs.datadoghq.com/api/v2/logs` (set `dd-api-key`) |
| Grafana Tempo | `https://tempo-prod-04-prod-us-east-0.grafana.net/v1/logs` |
| Jaeger | `http://jaeger-collector:4318/v1/logs` |
| Otelcol | `http://localhost:4318/v1/logs` |

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.
