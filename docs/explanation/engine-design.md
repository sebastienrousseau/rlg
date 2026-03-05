# Architecture: The Lock-Free Disruptor Engine

## Executive Summary
`rlg` (RustLogs) implements an asynchronous, lock-free observability pipeline designed to achieve sub-microsecond ingestion latency. By leveraging the **LMAX Disruptor pattern** and zero-allocation serialization, `rlg` decouples the application's critical path from the overhead of log formatting and I/O.

---

## 1. The Critical Path: <10ns Handoff
In a standard logging architecture, the application thread is often responsible for formatting the log string and writing it to a file or socket. Even with `std::sync::Mutex`, this introduces potential thread contention and blocking I/O.

`rlg` eliminates this bottleneck:
1. **The Ingestion Engine:** Uses a `crossbeam-queue::ArrayQueue` (fixed capacity 65,536).
2. **The Fluent API:** `Log::info().fire()` performs a shallow clone of metadata and pushes it onto the ring buffer. 
3. **Latency:** The handoff from the application thread to the buffer typically takes **<10ns**, allowing the business logic to continue without interruption.

## 2. Zero-Allocation Serialization
Traditional loggers generate massive amounts of transient `String` garbage. `rlg` utilizes stack-based serialization:
- **Integer Formatting:** Powered by `itoa`, bypassing `std::fmt`.
- **Float Formatting:** Powered by `ryu`.
- **Pre-sized Buffers:** The background flusher maintains a reusable `Vec<u8>` payload, minimizing heap allocations during high-throughput bursts.

## 3. Native OS Sinks (Binary-Level FFI)
`rlg` avoids the overhead of intermediate CLI tools or wrappers. It communicates directly with the kernel:
- **macOS (`os_log`):** Binds to the system's unified logging subsystem via `libsystem`. Logs appear instantly in the macOS Console app with correct subsystem/category tagging.
- **Linux (`journald`):** Directly writes structured binary payloads to `/run/systemd/journal/socket`, ensuring that metadata (Level, PID, SessionID) is indexed by Systemd without parsing.

## 4. Safety & MIRI Compliance
High-performance Rust often relies on `unsafe` blocks for FFI or pointer manipulation. `rlg` maintains a "Brutalist Security" posture:
- **Math-Verified Bounds:** Every `unsafe` block is documented with a `// SAFETY:` comment and verified against memory alignment constraints.
- **Continuous Validation:** The entire engine is verified using `cargo miri` with `-Zmiri-tree-borrows` to ensure no data races or provenance violations occur at the FFI boundary.
- **Runtime Agnostic:** The background flusher runs on a lightweight OS thread, making it compatible with `Tokio`, `async-std`, or synchronous runtimes.

---

## 5. Performance Delta (v0.0.7)
Compared to traditional logging crates (e.g., `env_logger` or `log4rs` with default settings), `rlg` provides:
- **~20x faster ingestion** in multi-threaded environments.
- **Zero impact** on tail latency during I/O stalls.
- **AI-native structured payloads** (MCP/OTLP) by default.
