// src/lib.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! # RLG (`RustLogs`) — High-Performance Lock-Free Logging Engine
//!
//! `rlg` is a structured logging library built on a lock-free ring buffer (LMAX Disruptor pattern).
//! It delivers sub-microsecond ingestion latency (~1.4µs) with deferred formatting on a background
//! flusher thread, and native platform sinks for macOS `os_log` and Linux `journald`.
//!
//! ## Design Principles
//! - **Fluent API:** Chainable builder pattern for ergonomic log construction.
//! - **14 structured formats:** JSON, OTLP, MCP, ECS, CEF, GELF, Logfmt, and more.
//! - **Lock-free concurrency:** MIRI-compliant safety, no mutex contention on the hot path.
//!
//! ## Feature Matrix
//!
//! | Feature | Default | Description |
//! |---------|:-------:|-------------|
//! | `default` | &mdash; | No default features; all modules are always compiled. |
//! | `debug_enabled` | No | Enables verbose internal engine diagnostics. |
//! | `miette` | No | Pretty diagnostic error reports via `miette`. |
//! | `tokio` | No | Async config loading, async file utilities (`load_async`, `hot_reload_async`). |
//! | `tracing-layer` | No | Composable `tracing_subscriber::Layer` via `RlgLayer`. |
//!
//! ## Quick Start: The Liquid Fluent API
//!
//! ```rust,no_run
//! use rlg::log::Log;
//! use rlg::log_format::LogFormat;
//!
//! // Fire-and-forget logging with sub-10ns handoff to the background engine.
//! Log::info("User successfully authenticated")
//!     .component("auth-service")
//!     .with("user_id", 42)
//!     .with("session_uuid", "a1b2c3d4")
//!     .format(LogFormat::MCP)
//!     .fire();
//! ```
//!
//! ## Architectural Overview
//! The heart of `rlg` is a lock-free ring buffer (65k capacity) that decouples log emission from
//! formatting and I/O. Serialization is performed on a dedicated background flusher thread
//! using stack-based buffers, ensuring that the critical path remains allocation-free.

#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    rust_2018_idioms
)]
#![warn(missing_docs)]
#![allow(clippy::module_name_repetitions)]

/// Configuration management for the logging engine.
pub mod config;
/// The core lock-free ingestion and flushing engine.
pub mod engine;
/// Custom error types for the RLG ecosystem.
pub mod error;
/// Zero-config initialization API.
pub mod init;
/// Log entry structures and the Liquid Fluent API.
pub mod log;
/// Exhaustive support for industry-standard log formats.
pub mod log_format;
/// Severity level definitions and parsing.
pub mod log_level;
/// Bridge from the `log` crate facade into the RLG engine.
pub mod logger;
/// Convenience macros for ergonomic logging.
pub mod macros;
/// Native platform-specific logging sinks.
pub mod sink;
/// Integration with the `tracing` ecosystem.
pub mod tracing;
/// Terminal UI dashboard for real-time metrics during local development.
pub mod tui;
/// Utility functions for timestamps, file operations, and sanitization.
pub mod utils;

/// Shared ecosystem utilities from euxis-commons.
pub use euxis_commons as commons;

// Re-exports for a flattened, intuitive API.
pub use crate::error::{RlgError, RlgResult};
pub use crate::init::{InitError, RlgBuilder, builder, init};
pub use crate::log::Log;
pub use crate::log_format::LogFormat;
pub use crate::log_level::LogLevel;
pub use crate::logger::RlgLogger;
pub use crate::sink::PlatformSink;
pub use crate::tracing::RlgSubscriber;

#[cfg(feature = "tracing-layer")]
pub use crate::tracing::RlgLayer;

/// The version of the `rlg` crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
