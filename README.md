# RLG (RustLogs) — v0.0.7

[![Crates.io](https://img.shields.io/crates/v/rlg.svg)](https://crates.io/crates/rlg)
[![Documentation](https://docs.rs/rlg/badge.svg)](https://docs.rs/rlg)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

**RLG (RustLogs)** is a brutalist, lock-free observability engine for Rust. Engineered for the 2026 industry standards, it delivers sub-microsecond ingestion latency, AI-native structured formatting (MCP/OTLP), and zero-allocation critical paths.

---

## 🚀 Key Performance Deltas
- **Latency:** ~1.4µs ingestion (Lock-free LMAX Disruptor pattern).
- **Handoff:** <10ns thread-to-engine handoff.
- **Serialization:** Zero-allocation via stack-based `itoa` and `ryu`.
- **Native Sinks:** Direct binary-level FFI for macOS `os_log` and Linux `journald`.

## 💎 Liquid Glass DX
Designed with Apple-standard developer experience in mind, `rlg` provides a chainable "Liquid" Fluent API that makes observability feel effortless.

```rust
use rlg::log::Log;
use rlg::log_format::LogFormat;

Log::info("Cloud instance scaled successfully")
    .component("orchestrator")
    .with("cpu_load", 0.85)
    .with("region", "us-east-1")
    .format(LogFormat::OTLP)
    .fire();
```

## 🛠️ Generative TUI Dashboard
See your application's heartbeat in real-time. Enable the asynchronous 60FPS dashboard during development:

```bash
export RLG_TUI=1
cargo run
```

## 🤖 AI-First Observability
`rlg` is built for the era of AI coding assistants. By supporting **Model Context Protocol (MCP)** and **OpenTelemetry (OTLP)** natively, your logs are immediately digestible by LLM-based orchestrators and modern observability stacks like Grafana and Honeycomb.

## 🛡️ Reliability & Safety
- **MIRI-Compliant:** Verified against memory provenance violations.
- **Enterprise Rigor:** 95%+ code coverage.
- **Agnostic:** Works across macOS, Linux, and WSL without runtime lock-in.

---

## 📖 Documentation
For tutorials, how-to guides, and architecture deep-dives, visit the [RLG Documentation Portal](docs/SUMMARY.md).

## 📄 License
Licensed under either [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
