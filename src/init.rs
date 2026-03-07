// init.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Zero-config initialization for the RLG observability engine.
//!
//! ```rust,no_run
//! // Sensible defaults (INFO level, MCP format)
//! rlg::init().unwrap();
//!
//! // Custom configuration
//! rlg::builder()
//!     .level(rlg::LogLevel::DEBUG)
//!     .format(rlg::LogFormat::JSON)
//!     .init()
//!     .unwrap();
//! ```

use crate::engine::ENGINE;
use crate::log_format::LogFormat;
use crate::log_level::LogLevel;
use crate::logger::{RlgLogger, to_log_level_filter};
use std::fmt;
use std::sync::OnceLock;

/// Detects the default log format based on output context.
///
/// - **TTY** → `Logfmt` (human-readable key=value)
/// - **Pipe/file/CI** → `JSON` (structured, machine-parseable)
/// - **`RLG_ENV=production`** → `JSON`
fn detect_default_format() -> LogFormat {
    if std::env::var("RLG_ENV")
        .map(|v| v == "production")
        .unwrap_or(false)
    {
        return LogFormat::JSON;
    }
    if atty_stdout() {
        LogFormat::Logfmt
    } else {
        LogFormat::JSON
    }
}

/// Returns `true` if stdout is connected to a terminal.
fn atty_stdout() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

/// Parses `RUST_LOG` for a simple level filter (e.g., `RUST_LOG=debug`).
///
/// Supports `RUST_LOG=<level>` and `RUST_LOG=<crate>=<level>` (the crate
/// filter is ignored for now — we apply the most permissive level found).
fn parse_rust_log() -> Option<LogLevel> {
    let val = std::env::var("RUST_LOG").ok()?;
    let mut most_permissive: Option<LogLevel> = None;
    for directive in val.split(',') {
        let level_str = directive
            .split('=')
            .next_back()
            .unwrap_or(directive)
            .trim();
        if let Ok(level) = level_str.parse::<LogLevel>() {
            match most_permissive {
                None => most_permissive = Some(level),
                Some(current)
                    if level.to_numeric() < current.to_numeric() =>
                {
                    most_permissive = Some(level);
                }
                _ => {}
            }
        }
    }
    most_permissive
}

/// Guard to prevent double initialization.
static INIT_GUARD: OnceLock<()> = OnceLock::new();

/// Static logger instance providing `&'static` lifetime for `log::set_logger`.
static LOGGER: OnceLock<RlgLogger> = OnceLock::new();

/// Errors that can occur during initialization.
#[derive(Debug, Clone, Copy)]
pub enum InitError {
    /// The `log` crate global logger was already set.
    LoggerAlreadySet,
    /// The `tracing` global subscriber was already set.
    SubscriberAlreadySet,
    /// `rlg::init()` or `rlg::builder().init()` was already called.
    AlreadyInitialized,
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LoggerAlreadySet => {
                f.write_str("a log crate logger was already set")
            }
            Self::SubscriberAlreadySet => {
                f.write_str("a tracing subscriber was already set")
            }
            Self::AlreadyInitialized => {
                f.write_str("rlg was already initialized")
            }
        }
    }
}

impl std::error::Error for InitError {}

/// Builder for customizing the RLG initialization.
#[derive(Debug, Clone, Copy)]
pub struct RlgBuilder {
    level: LogLevel,
    format: LogFormat,
    install_log: bool,
    install_tracing: bool,
}

impl Default for RlgBuilder {
    fn default() -> Self {
        Self {
            level: LogLevel::INFO,
            format: detect_default_format(),
            install_log: true,
            install_tracing: true,
        }
    }
}

impl RlgBuilder {
    /// Sets the minimum log level.
    #[must_use]
    pub const fn level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    /// Sets the default log output format.
    #[must_use]
    pub const fn format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Disables the `log` crate facade integration.
    #[must_use]
    pub const fn without_log(mut self) -> Self {
        self.install_log = false;
        self
    }

    /// Disables the `tracing` subscriber installation.
    #[must_use]
    pub const fn without_tracing(mut self) -> Self {
        self.install_tracing = false;
        self
    }

    /// Installs the `log` crate facade bridge.
    ///
    /// # Errors
    ///
    /// Returns `InitError::LoggerAlreadySet` if a logger was already registered.
    pub(crate) fn install_log_facade(
        format: LogFormat,
        level: LogLevel,
    ) -> Result<(), InitError> {
        let logger = LOGGER.get_or_init(|| RlgLogger::new(format));
        log::set_logger(logger)
            .map_err(|_| InitError::LoggerAlreadySet)?;
        log::set_max_level(to_log_level_filter(level));
        Ok(())
    }

    /// Installs the `tracing` global subscriber.
    ///
    /// # Errors
    ///
    /// Returns `InitError::SubscriberAlreadySet` if a subscriber was already registered.
    pub(crate) fn install_tracing_subscriber() -> Result<(), InitError>
    {
        let subscriber = crate::tracing::RlgSubscriber::new();
        let dispatch =
            tracing_core::dispatcher::Dispatch::new(subscriber);
        tracing_core::dispatcher::set_global_default(dispatch)
            .map_err(|_| InitError::SubscriberAlreadySet)?;
        Ok(())
    }

    /// Finalizes the builder and installs RLG as the global logger/subscriber.
    ///
    /// Respects `RUST_LOG` for level overrides and auto-detects the output
    /// format when no explicit format was set (TTY → Logfmt, pipe → JSON).
    ///
    /// # Errors
    ///
    /// Returns an error if a logger or subscriber was already installed, or
    /// if RLG was already initialized.
    pub fn init(mut self) -> Result<FlushGuard, InitError> {
        if INIT_GUARD.set(()).is_err() {
            return Err(InitError::AlreadyInitialized);
        }

        // Apply RUST_LOG level override.
        if let Some(env_level) = parse_rust_log() {
            self.level = env_level;
        }

        // Set engine filter level
        ENGINE.set_filter(self.level.to_numeric());

        // Install log facade
        if self.install_log {
            Self::install_log_facade(self.format, self.level)?;
        }

        // Install tracing subscriber
        if self.install_tracing {
            Self::install_tracing_subscriber()?;
        }

        Ok(FlushGuard { _private: () })
    }
}

/// Creates a new [`RlgBuilder`] for custom initialization.
#[must_use]
pub fn builder() -> RlgBuilder {
    RlgBuilder::default()
}

/// A guard that calls [`ENGINE.shutdown()`](crate::engine::LockFreeEngine::shutdown)
/// when dropped, ensuring buffered events are flushed before process exit.
///
/// Returned by [`init`] and [`RlgBuilder::init`]. Hold it in `main()`:
///
/// ```rust,no_run
/// let _guard = rlg::init().unwrap();
/// // … application code …
/// // guard drops here, flushing all pending logs
/// ```
#[derive(Debug)]
pub struct FlushGuard {
    _private: (),
}

impl Drop for FlushGuard {
    fn drop(&mut self) {
        ENGINE.shutdown();
    }
}

/// Initializes RLG with sensible defaults.
///
/// Auto-detects the output format (TTY → Logfmt, pipe → JSON) and respects
/// `RUST_LOG` for level overrides.
///
/// Returns a [`FlushGuard`] that flushes pending events on drop.
///
/// # Errors
///
/// Returns an error if a logger or subscriber was already installed.
pub fn init() -> Result<FlushGuard, InitError> {
    builder().init()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_error_display_logger_already_set() {
        let err = InitError::LoggerAlreadySet;
        assert_eq!(
            err.to_string(),
            "a log crate logger was already set"
        );
    }

    #[test]
    fn test_init_error_display_subscriber_already_set() {
        let err = InitError::SubscriberAlreadySet;
        assert_eq!(
            err.to_string(),
            "a tracing subscriber was already set"
        );
    }

    #[test]
    fn test_init_error_display_already_initialized() {
        let err = InitError::AlreadyInitialized;
        assert_eq!(err.to_string(), "rlg was already initialized");
    }

    #[test]
    fn test_init_error_debug() {
        let err = InitError::LoggerAlreadySet;
        assert_eq!(format!("{err:?}"), "LoggerAlreadySet");
    }

    #[test]
    fn test_init_error_clone_copy() {
        let err = InitError::AlreadyInitialized;
        let cloned = err;
        assert_eq!(format!("{err:?}"), format!("{cloned:?}"));
    }

    #[test]
    fn test_init_error_is_error() {
        let err = InitError::LoggerAlreadySet;
        // Verify it implements std::error::Error
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_builder_defaults() {
        let b = RlgBuilder::default();
        assert_eq!(b.level, LogLevel::INFO);
        assert!(b.install_log);
        assert!(b.install_tracing);
        // Format is auto-detected (Logfmt for TTY, JSON for pipe/CI)
        assert!(
            b.format == LogFormat::JSON
                || b.format == LogFormat::Logfmt
        );
    }

    #[test]
    fn test_builder_level() {
        let b = builder().level(LogLevel::DEBUG);
        assert_eq!(b.level, LogLevel::DEBUG);
    }

    #[test]
    fn test_builder_format() {
        let b = builder().format(LogFormat::JSON);
        assert_eq!(b.format, LogFormat::JSON);
    }

    #[test]
    fn test_builder_without_log() {
        let b = builder().without_log();
        assert!(!b.install_log);
        assert!(b.install_tracing);
    }

    #[test]
    fn test_builder_without_tracing() {
        let b = builder().without_tracing();
        assert!(b.install_log);
        assert!(!b.install_tracing);
    }

    #[test]
    fn test_builder_chaining() {
        let b = builder()
            .level(LogLevel::TRACE)
            .format(LogFormat::ECS)
            .without_log()
            .without_tracing();
        assert_eq!(b.level, LogLevel::TRACE);
        assert_eq!(b.format, LogFormat::ECS);
        assert!(!b.install_log);
        assert!(!b.install_tracing);
    }

    #[test]
    fn test_builder_clone_copy() {
        let b = builder().level(LogLevel::WARN);
        let b2 = b;
        // Both usable since RlgBuilder is Copy
        assert_eq!(b.level, b2.level);
        assert_eq!(b.format, b2.format);
    }

    #[test]
    fn test_builder_without_facades_configuration() {
        let b = builder().without_log().without_tracing();
        assert!(!b.install_log);
        assert!(!b.install_tracing);
    }

    #[test]
    fn test_builder_fn() {
        let b = builder();
        assert_eq!(b.level, LogLevel::INFO);
        // Format is auto-detected based on output context
        assert!(
            b.format == LogFormat::JSON
                || b.format == LogFormat::Logfmt
        );
        assert!(b.install_log);
        assert!(b.install_tracing);
    }

    #[test]
    fn test_init_error_source() {
        let err = InitError::LoggerAlreadySet;
        // std::error::Error::source should return None
        assert!(std::error::Error::source(&err).is_none());
    }

    #[test]
    fn test_builder_default_impl() {
        let b1 = RlgBuilder::default();
        let b2 = builder();
        assert_eq!(b1.level, b2.level);
        assert_eq!(b1.format, b2.format);
        assert_eq!(b1.install_log, b2.install_log);
        assert_eq!(b1.install_tracing, b2.install_tracing);
    }

    #[test]
    fn test_init_error_all_display_variants() {
        // Exercise all three Display paths
        let msgs: Vec<String> = vec![
            InitError::LoggerAlreadySet,
            InitError::SubscriberAlreadySet,
            InitError::AlreadyInitialized,
        ]
        .into_iter()
        .map(|e| e.to_string())
        .collect();
        assert_eq!(msgs.len(), 3);
        assert!(msgs[0].contains("log"));
        assert!(msgs[1].contains("tracing"));
        assert!(msgs[2].contains("already initialized"));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_init_guard_static() {
        // Exercise the OnceLock guard
        // First attempt may succeed or fail depending on test ordering
        let _ = INIT_GUARD.set(());
        // Second attempt should always fail
        assert!(INIT_GUARD.set(()).is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_logger_static() {
        // Exercise the LOGGER OnceLock
        let logger =
            LOGGER.get_or_init(|| RlgLogger::new(LogFormat::JSON));
        assert!(format!("{logger:?}").contains("RlgLogger"));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_install_log_facade() {
        // First call may succeed or fail (test ordering is non-deterministic)
        let r1 = RlgBuilder::install_log_facade(
            LogFormat::JSON,
            LogLevel::INFO,
        );
        assert!(
            r1.is_ok()
                || matches!(r1, Err(InitError::LoggerAlreadySet))
        );
        // Second call should definitely fail
        let r2 = RlgBuilder::install_log_facade(
            LogFormat::MCP,
            LogLevel::DEBUG,
        );
        assert!(matches!(r2, Err(InitError::LoggerAlreadySet)));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_install_tracing_subscriber() {
        // First call may succeed or fail (test ordering is non-deterministic)
        let r1 = RlgBuilder::install_tracing_subscriber();
        assert!(
            r1.is_ok()
                || matches!(r1, Err(InitError::SubscriberAlreadySet))
        );
        // Second call should definitely fail
        let r2 = RlgBuilder::install_tracing_subscriber();
        assert!(matches!(r2, Err(InitError::SubscriberAlreadySet)));
    }
}
