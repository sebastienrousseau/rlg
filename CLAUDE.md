# CLAUDE.md — RLG Contributor Guide

## Project

RLG (RustLogs) is a near-lock-free structured logging library for Rust, built on a 65k-slot ring buffer (LMAX Disruptor pattern).

## Architecture

```text
Application Thread → Log::fire() → ArrayQueue (65k ring buffer)
                                         ↓
                              Background Flusher Thread
                                         ↓
                              PlatformSink (os_log / journald / stdout / file)
```

- **Hot path** (`ingest()`): atomic operations only. No Mutex.
- **Mutex**: reserved for `shutdown()` to join the flusher thread.
- **Formatting**: deferred to the flusher thread.

## Key Design Decisions

- `session_id` is `u64` — avoids allocation on the hot path.
- `component` and `time` use `Cow<'static, str>` — static strings stay on the stack.
- Config files use TOML for both load and save.
- `notify` and `terminal_size` are optional, gated behind `tokio` and `tui` features.

## Development

```bash
cargo check --all-features               # Compile check
cargo test --all-features                 # Run all tests
cargo clippy --all-features --tests --benches -- -D warnings  # Lint
cargo fmt --check                         # Format check
cargo bench --bench competitive_bench     # Benchmarks
```

## Module Map

| Module | Purpose |
|--------|---------|
| `engine.rs` | Ring buffer, flusher thread, global `ENGINE` |
| `log.rs` | `Log` struct, fluent builder, 14-format `Display` impl |
| `config.rs` | TOML config loading, validation, hot-reload |
| `sink.rs` | Platform sinks: `os_log`, `journald`, file, stdout |
| `rotation.rs` | Log rotation: size, time, date, count-based |
| `init.rs` | `init()`, `FlushGuard`, `RUST_LOG` parsing |
| `tui.rs` | Terminal dashboard (opt-in via `RLG_TUI=1`) |
| `logger.rs` | Bridge from `log` crate facade |
| `tracing.rs` | Bridge from `tracing` ecosystem |

## Conventions

- Edition 2024, MSRV 1.88.0
- `#![deny(clippy::all, clippy::pedantic, clippy::nursery)]`
- `unsafe_code = "deny"` (exception: platform FFI in `sink.rs`)
- Thread-spawning tests use `#[cfg_attr(miri, ignore)]`
- All public items require doc comments (`missing_docs = "warn"`)
