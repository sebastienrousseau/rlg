<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/rlg/v1/logos/rlg.svg" alt="RLG logo" width="128" />
</p>

<h1 align="center">rlg-tower</h1>

<p align="center">
  <code>tower::Layer</code> that emits structured <code>rlg</code> records
  for every HTTP request. Works with axum, tonic, hyper,
  <code>lambda_runtime</code> — anything that speaks
  <code>tower::Service&lt;http::Request&lt;_&gt;&gt;</code>.
</p>

<p align="center">
  <a href="https://crates.io/crates/rlg-tower"><img src="https://img.shields.io/crates/v/rlg-tower.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/rlg-tower"><img src="https://img.shields.io/badge/docs.rs-rlg--tower-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
</p>

---

## Install

```toml
[dependencies]
rlg        = "0.0.10"
rlg-tower  = "0.0.10"
```

## Quick start (axum)

```rust,ignore
use axum::{Router, routing::get};
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg_tower::RlgLayer;

let app: Router = Router::new()
    .route("/", get(|| async { "hello" }))
    .layer(
        RlgLayer::new()
            .level(LogLevel::INFO)
            .format(LogFormat::JSON)
            .component("api")
            .header("traceparent"),    // extract W3C trace-context
    );
```

## What gets logged

One record per response, with these attributes:

| Attribute | Source |
| --- | --- |
| `component` | `"rlg-tower"` (override with `.component(...)`) |
| `description` | `"<METHOD> <path> -> <status>"` |
| `http.method` | request method |
| `http.path` | request URI path |
| `http.status` | response status code |
| `http.latency_ms` | wall-clock milliseconds, poll-ready → ready response |
| `trace_id` | extracted from a configured request header (W3C `traceparent`, B3 `x-b3-traceid`) if set |

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.
