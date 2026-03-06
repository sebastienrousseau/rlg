// src/lib.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! # RLG (`RustLogs`) — High-Performance Lock-Free Observability Engine
//!
//! `rlg` is a brutalist, zero-allocation logging library designed for the 2026 observability landscape.
//! Built on the LMAX Disruptor pattern, it delivers sub-microsecond ingestion latency (~1.4µs)
//! with native platform integration for macOS `os_log` and Linux `journald`.
//!
//! ## Core Philosophies
//! - **Liquid DX (Apple-Standard):** A chainable, fluent API that makes high-performance logging feel effortless.
//! - **AI-Native (Google-Standard):** Optimized for ingestion by LLMs and MCP-compliant orchestrators via structured formats (OTLP, MCP, ECS).
//! - **Enterprise Rigor (IBM-Standard):** MIRI-compliant safety, lock-free concurrency, and direct OS-native binary FFI.
//!
//! ## Feature Matrix
//!
//! | Feature | Default | Description |
//! |---------|:-------:|-------------|
//! | `default` | Yes | Core engine, standard sinks, and terminal dashboard. |
//! | `reqwest` | No | Enables OTLP/HTTP exports for remote observability. |
//! | `syslog` | No | Enables legacy RFC 5424 syslog support. |
//! | `debug_enabled` | No | Enables verbose internal engine diagnostics. |
//!
//! ## Quick Start: The Liquid Fluent API
//!
//! ```rust
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
/// Log entry structures and the Liquid Fluent API.
pub mod log;
/// Exhaustive support for industry-standard log formats.
pub mod log_format;
/// Severity level definitions and parsing.
pub mod log_level;
/// Convenience macros for ergonomic logging.
pub mod macros;
/// Native platform-specific logging sinks.
pub mod sink;
/// Integration with the `tracing` ecosystem.
pub mod tracing;
/// Terminal UI Dashboard for generative local development.
pub mod tui;
/// High-performance utility functions.
pub mod utils;

// Re-exports for a flattened, intuitive API.
pub use crate::error::{RlgError, RlgResult};
pub use crate::log::Log;
pub use crate::log_format::LogFormat;
pub use crate::log_level::LogLevel;
pub use crate::sink::PlatformSink;
pub use crate::tracing::RlgSubscriber;

/// The version of the `rlg` crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
