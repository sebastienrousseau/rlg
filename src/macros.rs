// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// Macro to create a new log easily
/// Usage:
/// let log = macro_log!(session_id, time, level, component, description, format);
#[macro_export]
macro_rules! macro_log {
    ($session_id:expr, $time:expr, $level:expr, $component:expr, $description:expr, $format:expr) => {
        $crate::Log::new(
            $session_id,
            $time,
            $level,
            $component,
            $description,
            $format,
        )
    };
}

/// Macro for info log with default session id and format
/// Usage:
/// let log = macro_info_log!(time, component, description);
#[macro_export]
macro_rules! macro_info_log {
    ($time:expr, $component:expr, $description:expr) => {
        $crate::Log::new(
            &vrd::Random::default().int(0, 1_000_000_000).to_string(),
            $time,
            &$crate::LogLevel::INFO,
            $component,
            $description,
            &$crate::LogFormat::CLF,
        )
    };
}

/// Print log to stdout
/// Usage:
/// macro_print_log!(log);
#[macro_export]
macro_rules! macro_print_log {
    ($log:expr) => {
        println!("{}", $log);
    };
}

/// Async log message to file
/// Usage:
/// let result = macro_log_to_file!(log);
#[macro_export]
macro_rules! macro_log_to_file {
    ($log:expr) => {{
        let result = $log.log().await;
        result
    }};
}

/// Macro for warn log
#[macro_export]
macro_rules! macro_warn_log {
    ($time:expr, $component:expr, $description:expr) => {
        macro_log!(
            &vrd::Random::default().int(0, 1_000_000_000).to_string(),
            $time,
            &LogLevel::WARNING,
            $component,
            $description,
            &LogFormat::CLF
        )
    };
}

/// Macro for error log with default format
#[macro_export]
macro_rules! macro_error_log {
    ($time:expr, $component:expr, $description:expr) => {
        macro_log!(
            &vrd::Random::default().int(0, 1_000_000_000).to_string(),
            $time,
            &LogLevel::ERROR,
            $component,
            $description,
            &LogFormat::CLF
        )
    };
}

/// Set log format if not already defined
/// Usage:
/// macro_set_log_format_clf!(log);
#[macro_export]
macro_rules! macro_set_log_format_clf {
    ($log:expr) => {
        if $log.format != $crate::LogFormat::CLF {
            $log.format = $crate::LogFormat::CLF;
        }
    };
}

/// Conditional debug logging
/// Logs if `debug_enabled` feature flag set
#[cfg(feature = "debug_enabled")]
#[macro_export]
macro_rules! macro_debug_log {
    ($log:expr) => {
        macro_print_log!($log);
    };
}

/// Conditional debug logging
/// Logs if `debug_enabled` feature flag set
#[cfg(not(feature = "debug_enabled"))]
#[macro_export]
macro_rules! macro_debug_log {
    ($log:expr) => {
        // Do nothing if `debug_enabled` feature flag is not set
    };
}

/// Macro for trace log
#[macro_export]
macro_rules! macro_trace_log {
    ($time:expr, $component:expr, $description:expr) => {
        macro_log!(
            &vrd::Random::default().int(0, 1_000_000_000).to_string(),
            $time,
            &LogLevel::TRACE,
            $component,
            $description,
            &LogFormat::CLF
        )
    };
}

/// Macro for fatal log
#[macro_export]
macro_rules! macro_fatal_log {
    ($time:expr, $component:expr, $description:expr) => {
        macro_log!(
            &vrd::Random::default().int(0, 1_000_000_000).to_string(),
            $time,
            &LogLevel::FATAL,
            $component,
            $description,
            &LogFormat::CLF
        )
    };
}

/// Conditional logging based on a predicate
/// Usage:
/// macro_log_if!(predicate, log);
#[macro_export]
macro_rules! macro_log_if {
    ($predicate:expr, $log:expr) => {
        if $predicate {
            macro_print_log!($log);
        }
    };
}

/// Macro for logging with metadata
/// Usage:
/// let log = macro_log_with_metadata!(session_id, time, level, component, description, format);
/// println!("{log} | Metadata: {metadata}");
#[macro_export]
macro_rules! macro_log_with_metadata {
    ($session_id:expr, $time:expr, $level:expr, $component:expr, $description:expr, $format:expr) => {
        {
            let log = $crate::Log::new(
                $session_id,
                $time,
                $level,
                $component,
                $description,
                $format,
            );
            // Replace keys in the log message with consistent ones
            let log_message = log.to_string()
                .replace("\"component\"", "\"component\"")
                .replace("\"session_id\"", "\"session_id\"");
            log_message
        }
    };
}
