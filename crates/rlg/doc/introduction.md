# RLG — RustLogs

A high-performance structured logging library for Rust.

RLG pushes log events into a lock-free ring buffer and formats them on a background thread. Your application thread never blocks on I/O.

## Core Features

- **14 output formats** — JSON, NDJSON, OTLP, MCP, GELF, CEF, ECS, Logfmt, CLF, W3C, Syslog, Logstash, Log4j-XML, Apache Error
- **Fluent builder API** — `Log::info("msg").with("key", val).fire()`
- **Platform-native sinks** — macOS `os_log`, Linux `journald`, file, stdout
- **`log` and `tracing` bridges** — drop-in replacement for existing Rust logging
- **TUI dashboard** — real-time throughput and error metrics in-terminal
- **Log rotation** — size, time, date, or count-based policies

## Quick Start

```toml
[dependencies]
rlg = "0.0.7"
```

```rust
use rlg::init;
use rlg::log::Log;

fn main() {
    let _guard = init::init().expect("failed to initialise RLG");

    Log::info("Service started")
        .with("version", "0.0.7")
        .fire();
}
// FlushGuard drops here — all buffered events flush automatically.
```

## Navigation

- **[Getting Started](tutorials/getting-started.md)** — install, configure, and emit your first log
- **[Fluent API](how-to/fluent-api.md)** — chain `.with()`, `.component()`, `.format()`, then `.fire()`
- **[Engine Design](explanation/engine-design.md)** — how the ring buffer and background flusher work
- **[Safety](explanation/safety.md)** — MIRI verification and FFI boundary guarantees
- **[API Reference](api/rlg/index.html)** — auto-generated Rustdoc
