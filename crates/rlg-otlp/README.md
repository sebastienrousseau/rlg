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

The wire format is OTLP/HTTP with JSON encoding.

## Install

```toml
[dependencies]
rlg       = "0.0.14"
rlg-otlp  = "0.0.14"
```

Requires Rust **1.88.0** or newer (edition 2024).

## Usage

```rust
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg_otlp::OtlpExporter;

// Defaults to a Collector on this host: http://localhost:4318/v1/logs
let exporter = OtlpExporter::builder().timeout_secs(10).build();

let record = Log::error("payment-service down")
    .component("orders")
    .with("trace_id", "abc123")
    .format(LogFormat::OTLP);

exporter.export_one(&record).unwrap();
```

The `async` feature adds `AsyncOtlpExporter`, the same API on Tokio.

## Deployment: send to a local Collector

`rlg-otlp` carries no TLS stack and no crypto dependencies. It speaks
plain OTLP/HTTP to an [OpenTelemetry Collector][otelcol] (or any
OTLP/HTTP forwarder) on the same host or in the same pod, and the
Collector owns everything between it and the backend: TLS, API keys,
batching, retries and buffering during an outage.

```text
app (rlg + rlg-otlp) --http--> Collector :4318 --https--> Honeycomb / Datadog / Grafana / …
```

An `https://` endpoint is rejected when the exporter is built instead
of being sent in the clear. A minimal Collector configuration that
forwards to Honeycomb:

```yaml
receivers:
  otlp:
    protocols:
      http:
        endpoint: 127.0.0.1:4318
exporters:
  otlphttp:
    endpoint: https://api.honeycomb.io
    headers:
      x-honeycomb-team: ${env:HONEYCOMB_API_KEY}
service:
  pipelines:
    logs:
      receivers: [otlp]
      exporters: [otlphttp]
```

On Kubernetes, run the Collector as a sidecar or DaemonSet and point
the exporter at it (`http://localhost:4318/v1/logs` for a sidecar,
`http://$(NODE_IP):4318/v1/logs` for a DaemonSet).

[otelcol]: https://opentelemetry.io/docs/collector/

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.
