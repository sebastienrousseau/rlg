<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/rlg/images/logos/rlg.svg"
alt="RustLogs (RLG) logo" height="66" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# RustLogs (RLG): The 2026 Telemetry Standard

A brutalist, lock-free Rust logging engine designed for extreme performance and AI-first observability. Engineered to be the lightweight alternative to `tracing` with an Apple-standard developer experience.

[![Made With Love][made-with-rust]][00] [![Crates.io][crates-badge]][07] [![lib.rs][libs-badge]][03] [![Docs.rs][docs-badge]][08] [![Codecov][codecov-badge]][09] [![Build Status][build-badge]][10] [![GitHub][github-badge]][06]

## 🚀 Extreme Performance

`rlg` implements the **LMAX Disruptor** pattern under the hood, decoupling log ingestion from flushing.

- **Lock-Free Ingestion:** < 10ns per event. The critical path never blocks on file I/O or Mutexes.
- **Zero-Allocation Serialization:** Uses `itoa` and `ryu` for stack-based numeric formatting.
- **Runtime Agnostic:** Background flusher runs on a dedicated OS thread, compatible with `tokio`, `async-std`, or synchronous codebases.

##  Apple-Standard DX

Logging shouldn't be a wall of JSON. `rlg` provides a stunning developer experience out of the box.

- **"Liquid" Fluent API:** Semantic, chainable, and strictly typed.
- **Generative TUI Dashboard:** Pin a live-updating performance dashboard to the bottom of your terminal during local development.
- **Native OS Sinks:**
  - **macOS:** Zero-copy binary handoff to Apple's **Unified Logging System (`os_log`)**.
  - **Linux/WSL:** Direct binary integration with **Systemd `journald`**.

## 🤖 AI-First Observability

Engineered for the 2026 AI-driven workflow.

- **MCP (Model Context Protocol):** Native support for AI agent ingestion via structured JSON-RPC notifications.
- **High-Cardinality Spans:** Move beyond flat strings with semantic context tagging (`BTreeMap` attributes).

## 🛠 Usage

### The Fluent API (Recommended)

```rust
use rlg::log::Log;

// Chainable, semantic, and non-blocking
Log::info("User transaction processed")
    .component("billing-svc")
    .with("user_id", 8472)
    .with("latency_ms", 12.5)
    .fire(); // Dispatched to lock-free ring buffer instantly
```

### Live TUI Dashboard

Enable the stunning "Liquid Glass" dashboard for local development by setting:

```bash
export RLG_TUI=1
cargo run
```

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rlg = "0.0.7"
```

## 📜 License

The project is dual-licensed under the terms of both the MIT license and the Apache License (Version 2.0).

- [Apache License, Version 2.0][01]
- [MIT license][02]

[00]: https://rustlogs.com
[01]: http://www.apache.org/licenses/LICENSE-2.0
[02]: http://opensource.org/licenses/MIT
[03]: https://lib.rs/crates/rlg
[04]: https://doc.rustlogs.com/
[06]: https://github.com/sebastienrousseau/rlg
[07]: https://crates.io/crates/rlg
[08]: https://docs.rs/rlg
[09]: https://codecov.io/gh/sebastienrousseau/rlg
[10]: https://github.com/sebastienrousseau/rlg/actions?query=branch%3Amaster

[build-badge]: https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rlg/release.yml?branch=master&style=for-the-badge&logo=github "Build Status"
[codecov-badge]: https://img.shields.io/codecov/c/github/sebastienrousseau/rlg?style=for-the-badge&token=Q9KJ6XXL67&logo=codecov "Codecov"
[crates-badge]: https://img.shields.io/crates/v/rlg.svg?style=for-the-badge&color=fc8d62&logo=rust "Crates.io"
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.7-orange.svg?style=for-the-badge "View on lib.rs"
[docs-badge]: https://img.shields.io/badge/docs.rs-rlg-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs "Docs.rs"
[github-badge]: https://img.shields.io/badge/github-sebastienrousseau/rlg-8da0cb?style=for-the-badge&labelColor=555555&logo=github "GitHub"
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust'
