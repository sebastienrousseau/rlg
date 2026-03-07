// logger.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Bridge from the [`log`](https://docs.rs/log) crate facade into the RLG engine.

use crate::engine::ENGINE;
use crate::log::Log;
use crate::log_format::LogFormat;
use crate::log_level::LogLevel;

/// Maps a [`log::Level`] to an RLG [`LogLevel`].
#[must_use]
pub const fn map_log_level(level: log::Level) -> LogLevel {
    match level {
        log::Level::Error => LogLevel::ERROR,
        log::Level::Warn => LogLevel::WARN,
        log::Level::Info => LogLevel::INFO,
        log::Level::Debug => LogLevel::DEBUG,
        log::Level::Trace => LogLevel::TRACE,
    }
}

/// Converts an RLG [`LogLevel`] to a [`log::LevelFilter`].
#[must_use]
pub const fn to_log_level_filter(level: LogLevel) -> log::LevelFilter {
    match level {
        LogLevel::ALL | LogLevel::TRACE => log::LevelFilter::Trace,
        LogLevel::DEBUG => log::LevelFilter::Debug,
        LogLevel::VERBOSE | LogLevel::INFO => log::LevelFilter::Info,
        LogLevel::WARN => log::LevelFilter::Warn,
        LogLevel::ERROR | LogLevel::FATAL | LogLevel::CRITICAL => {
            log::LevelFilter::Error
        }
        LogLevel::NONE | LogLevel::DISABLED => log::LevelFilter::Off,
    }
}

/// A [`log::Log`] implementation that routes messages into the RLG lock-free engine.
#[derive(Debug, Clone, Copy)]
pub struct RlgLogger {
    format: LogFormat,
}

impl RlgLogger {
    /// Creates a new `RlgLogger` with the given output format.
    #[must_use]
    pub const fn new(format: LogFormat) -> Self {
        Self { format }
    }
}

impl log::Log for RlgLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        map_log_level(metadata.level()).to_numeric()
            >= ENGINE.filter_level()
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level = map_log_level(record.level());
        let mut entry = Log::build(level, &record.args().to_string());
        entry.component =
            std::borrow::Cow::Owned(record.target().to_string());
        entry.format = self.format;

        if let Some(file) = record.file() {
            entry = entry.with("file", file);
        }
        if let Some(line) = record.line() {
            entry = entry.with("line", line);
        }
        if let Some(module) = record.module_path() {
            entry = entry.with("module", module);
        }

        entry.fire();
    }

    fn flush(&self) {
        // The background flusher thread handles I/O.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_log_level_all_variants() {
        assert_eq!(map_log_level(log::Level::Error), LogLevel::ERROR);
        assert_eq!(map_log_level(log::Level::Warn), LogLevel::WARN);
        assert_eq!(map_log_level(log::Level::Info), LogLevel::INFO);
        assert_eq!(map_log_level(log::Level::Debug), LogLevel::DEBUG);
        assert_eq!(map_log_level(log::Level::Trace), LogLevel::TRACE);
    }

    #[test]
    fn test_to_log_level_filter_all_variants() {
        assert_eq!(
            to_log_level_filter(LogLevel::ALL),
            log::LevelFilter::Trace
        );
        assert_eq!(
            to_log_level_filter(LogLevel::TRACE),
            log::LevelFilter::Trace
        );
        assert_eq!(
            to_log_level_filter(LogLevel::DEBUG),
            log::LevelFilter::Debug
        );
        assert_eq!(
            to_log_level_filter(LogLevel::VERBOSE),
            log::LevelFilter::Info
        );
        assert_eq!(
            to_log_level_filter(LogLevel::INFO),
            log::LevelFilter::Info
        );
        assert_eq!(
            to_log_level_filter(LogLevel::WARN),
            log::LevelFilter::Warn
        );
        assert_eq!(
            to_log_level_filter(LogLevel::ERROR),
            log::LevelFilter::Error
        );
        assert_eq!(
            to_log_level_filter(LogLevel::FATAL),
            log::LevelFilter::Error
        );
        assert_eq!(
            to_log_level_filter(LogLevel::CRITICAL),
            log::LevelFilter::Error
        );
        assert_eq!(
            to_log_level_filter(LogLevel::NONE),
            log::LevelFilter::Off
        );
        assert_eq!(
            to_log_level_filter(LogLevel::DISABLED),
            log::LevelFilter::Off
        );
    }

    #[test]
    fn test_rlg_logger_new() {
        let logger = RlgLogger::new(LogFormat::JSON);
        assert_eq!(format!("{logger:?}"), "RlgLogger { format: JSON }");
    }

    #[test]
    fn test_rlg_logger_clone_copy() {
        let logger = RlgLogger::new(LogFormat::MCP);
        let cloned = logger;
        // Both are valid since RlgLogger is Copy
        let _ = format!("{logger:?}");
        let _ = format!("{cloned:?}");
    }

    #[test]
    fn test_rlg_logger_enabled() {
        let logger = RlgLogger::new(LogFormat::JSON);
        // Default filter is 0 (ALL), so everything is enabled
        let metadata = log::MetadataBuilder::new()
            .level(log::Level::Trace)
            .build();
        assert!(log::Log::enabled(&logger, &metadata));

        let metadata = log::MetadataBuilder::new()
            .level(log::Level::Error)
            .build();
        assert!(log::Log::enabled(&logger, &metadata));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_rlg_logger_log_with_metadata() {
        let logger = RlgLogger::new(LogFormat::JSON);

        // Build a record with file/line/module metadata
        let record = log::RecordBuilder::new()
            .args(format_args!("test log message"))
            .level(log::Level::Info)
            .target("test_target")
            .file(Some("test_file.rs"))
            .line(Some(42))
            .module_path(Some("test_module"))
            .build();

        log::Log::log(&logger, &record);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_rlg_logger_log_without_metadata() {
        let logger = RlgLogger::new(LogFormat::MCP);

        // Build a record without optional metadata
        let record = log::RecordBuilder::new()
            .args(format_args!("minimal message"))
            .level(log::Level::Warn)
            .target("minimal_target")
            .build();

        log::Log::log(&logger, &record);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_rlg_logger_log_all_levels() {
        let logger = RlgLogger::new(LogFormat::JSON);

        for level in &[
            log::Level::Error,
            log::Level::Warn,
            log::Level::Info,
            log::Level::Debug,
            log::Level::Trace,
        ] {
            let record = log::RecordBuilder::new()
                .args(format_args!("level test"))
                .level(*level)
                .target("level_test")
                .build();
            log::Log::log(&logger, &record);
        }
    }

    #[test]
    fn test_rlg_logger_flush() {
        let logger = RlgLogger::new(LogFormat::JSON);
        log::Log::flush(&logger); // Should be a no-op
    }
}
