# CLAUDE.md — RLG Contributor Guide

## Project Overview

RLG (RustLogs) is a high-performance structured logging library for Rust built on a near-lock-free ring buffer (LMAX Disruptor pattern).

## Architecture

```
Application Thread → Log::fire() → ArrayQueue (65k ring buffer)
                                         ↓
                               Background Flusher Thread
                                         ↓
                               PlatformSink (os_log / journald / stdout / file)
```

- **Hot path** (`ingest()`): Near-lock-free — only atomic operations, no Mutex.
- **Mutex**: Reserved solely for `shutdown()` to join the flusher thread.
- **Formatting**: Deferred to the flusher thread (never on the caller's thread).

## Key Design Decisions

- `session_id` is `u64` (not String) to avoid allocation on the hot path.
- `component` and `time` use `Cow<'static, str>` — static strings stay on the stack.
- Config files are TOML (both load and save).
- `notify` and `terminal_size` are optional dependencies behind `tokio` and `tui` features.

## Development Workflow

```bash
# Check compilation
cargo check --all-features

# Run tests
cargo test --all-features

# Lint
cargo clippy --all-features --tests --benches -- -D warnings

# Format
cargo fmt --check

# Benchmarks
cargo bench --bench competitive_bench
```

## Module Map

| Module | Purpose |
|--------|---------|
| `engine.rs` | Ring buffer, flusher thread, global `ENGINE` |
| `log.rs` | `Log` struct, fluent API, 14-format `Display` impl |
| `config.rs` | TOML config loading, validation, hot-reload |
| `sink.rs` | Platform-native sinks (os_log, journald, file, stdout) |
| `rotation.rs` | Log rotation policies (size, time, date, count) |
| `init.rs` | Zero-config `init()`, `FlushGuard`, RUST_LOG parsing |
| `tui.rs` | Terminal dashboard (opt-in via `RLG_TUI=1`) |
| `logger.rs` | Bridge from `log` crate facade |
| `tracing.rs` | Bridge from `tracing` ecosystem |

## Conventions

- Edition 2024, MSRV 1.88.0
- `#![deny(clippy::all, clippy::pedantic, clippy::nursery)]`
- `unsafe_code = "deny"` (except platform FFI in `sink.rs`)
- Tests use `#[cfg_attr(miri, ignore)]` for thread-spawning tests
- All public items require doc comments (`missing_docs = "warn"`)
