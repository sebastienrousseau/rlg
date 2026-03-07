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
            format: LogFormat::MCP,
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
    /// # Errors
    ///
    /// Returns an error if a logger or subscriber was already installed, or
    /// if RLG was already initialized.
    pub fn init(self) -> Result<(), InitError> {
        if INIT_GUARD.set(()).is_err() {
            return Err(InitError::AlreadyInitialized);
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

        Ok(())
    }
}

/// Creates a new [`RlgBuilder`] for custom initialization.
#[must_use]
pub fn builder() -> RlgBuilder {
    RlgBuilder::default()
}

/// Initializes RLG with sensible defaults (INFO level, MCP format).
///
/// Installs RLG as the global `log` logger and `tracing` subscriber.
///
/// # Errors
///
/// Returns an error if a logger or subscriber was already installed.
pub fn init() -> Result<(), InitError> {
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
        assert_eq!(
            format!("{b:?}"),
            "RlgBuilder { level: INFO, format: MCP, install_log: true, install_tracing: true }"
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
        assert_eq!(b.format, LogFormat::MCP);
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
