// src/lib.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! # RLG — Near-Lock-Free Structured Logging for Rust
//!
//! `rlg` pushes structured log events through a 65k-slot ring buffer
//! ([LMAX Disruptor](https://lmax-exchange.github.io/disruptor/) pattern)
//! in ~1.4 µs. A background flusher thread handles serialization and
//! dispatch to platform-native sinks (`os_log`, `journald`, files, stdout).
//!
//! ## Why RLG
//!
//! - **No Mutex on the hot path.** `ingest()` uses atomic operations only.
//! - **Deferred formatting.** Serialization runs on the flusher thread.
//! - **14 output formats.** JSON, MCP, OTLP, ECS, CEF, GELF, Logfmt, and more.
//! - **MIRI-verified.** Zero undefined behaviour under strict provenance.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! // Initialize once at the top of main. Hold the guard.
//! let _guard = rlg::init().unwrap();
//!
//! use rlg::log::Log;
//! use rlg::log_format::LogFormat;
//!
//! Log::info("User authenticated")
//!     .component("auth-service")
//!     .with("user_id", 42)
//!     .with("session_uuid", "a1b2c3d4")
//!     .format(LogFormat::MCP)
//!     .fire();
//! ```
//!
//! ## Features
//!
//! No features are enabled by default.
//!
//! | Feature | Effect |
//! |---------|--------|
//! | `tokio` | Async config loading, hot-reload via `notify`. |
//! | `tui` | Live terminal dashboard via `terminal_size`. |
//! | `miette` | Pretty diagnostic error reports. |
//! | `tracing-layer` | Composable `tracing_subscriber::Layer`. |
//! | `debug_enabled` | Verbose internal engine diagnostics. |
//!
//! ## Architecture
//!
//! ```text
//! Application Thread → Log::fire() → ArrayQueue (65k)
//!                                         ↓
//!                              Background Flusher Thread
//!                                         ↓
//!                              PlatformSink (os_log / journald / file / stdout)
//! ```
//!
//! The flusher drains events in batches of 64. Fields use `Cow<str>` and
//! `u64` session IDs to minimize heap allocations on the hot path.

#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    rust_2018_idioms
)]
#![allow(clippy::module_name_repetitions)]
// Enable `#[doc(cfg(feature = "…"))]` under docs.rs so feature-gated
// items advertise the flag that enables them. The `docsrs` cfg is set
// by `[package.metadata.docs.rs]`.
#![cfg_attr(docsrs, feature(doc_cfg))]

/// TOML-based configuration, validation, and hot-reload.
pub mod config;
/// Internal ISO 8601 timestamp helpers (replaces the historical `dtt` dep).
pub mod datetime;
/// Ring buffer engine: ingestion, flushing, and the global `ENGINE`.
pub mod engine;
/// Error types and the `RlgResult` alias.
pub mod error;
/// Zero-config `init()`, builder API, and `FlushGuard`.
pub mod init;
/// `Log` struct, fluent builder, and per-format `Display` impls.
pub mod log;
/// 14 structured output formats (JSON, MCP, OTLP, ECS, CEF, ...).
pub mod log_format;
/// Severity levels: `ALL` through `DISABLED`, with `FromStr` parsing.
pub mod log_level;
/// Bridge from the `log` crate facade into the RLG engine.
pub mod logger;
/// Macros: `rlg_span!`, `rlg_time_it!`, `rlg_mcp_notify!`.
pub mod macros;
/// Log rotation policies: size, time, date, and count-based.
pub mod rotation;
/// Platform-native sinks: `os_log` (macOS), `journald` (Linux), file, stdout.
pub mod sink;
/// `tracing` integration: `RlgSubscriber` and optional `RlgLayer`.
pub mod tracing;
/// Opt-in terminal dashboard for live metrics (`RLG_TUI=1`).
pub mod tui;
/// Timestamps, file I/O helpers, and input sanitization.
pub mod utils;

/// Kani model-checked proofs. Only compiled under `--cfg kani`
/// (set automatically by `cargo kani`). See
/// `docs/adr/0004-kani-verified-invariants.md`.
#[cfg(kani)]
mod kani_proofs;

/// Shared utilities from `euxis-commons`.
pub use euxis_commons as commons;

// --- Flattened re-exports ---
pub use crate::error::{RlgError, RlgResult};
pub use crate::init::{
    FlushGuard, InitError, RlgBuilder, builder, init,
};
pub use crate::log::Log;
pub use crate::log_format::LogFormat;
pub use crate::log_level::LogLevel;
pub use crate::logger::RlgLogger;
pub use crate::sink::PlatformSink;
pub use crate::tracing::RlgSubscriber;

#[cfg(feature = "tracing-layer")]
#[cfg_attr(docsrs, doc(cfg(feature = "tracing-layer")))]
pub use crate::tracing::RlgLayer;

/// Crate version, injected at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
