// macros.rs
// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// ======================
// Macros for Log Creation
// ======================

/// This macro simplifies the creation of log entries with specific parameters.
/// It returns a new `Log` instance based on the provided session ID, time, level,
/// component, description, and format.
///
/// # Parameters
/// - `session_id`: A unique identifier for the log session.
/// - `time`: The timestamp of the log entry.
/// - `level`: The severity level of the log.
/// - `component`: The system component that generated the log.
/// - `description`: A textual description of the log event.
/// - `format`: The format in which the log will be recorded.
///
/// # Example
/// ```
/// use rlg::{macro_log, log_level::LogLevel, log_format::LogFormat};
/// let log = macro_log!("id", "2022-01-01", &LogLevel::INFO, "app", "message", &LogFormat::JSON);
/// ```
/// Usage:
/// let log = macro_log!(session_id, time, level, component, description, format);
#[macro_export]
#[doc = "Macro to create a new log easily"]
macro_rules! macro_log {
    ($session_id:expr, $time:expr, $level:expr, $component:expr, $description:expr, $format:expr) => {
        $crate::log::Log::new(
            $session_id,
            $time,
            $level,
            $component,
            $description,
            $format,
        )
    };
}

/// This macro creates an `INFO` level log entry with a default session ID and format.
/// The session ID is generated randomly and the log format defaults to CLF.
///
/// # Parameters
/// - `time`: The timestamp of the log entry.
/// - `component`: The system component that generated the log.
/// - `description`: A textual description of the log event.
///
/// # Example
/// ```
/// use rlg::macro_info_log;
/// let log = macro_info_log!("2024-08-29T12:00:00Z", "Auth", "User login");
/// ```
/// Usage:
/// let log = macro_info_log!(time, component, description);
#[macro_export]
#[doc = "Macro for info log with default session id and format"]
macro_rules! macro_info_log {
    ($time:expr, $component:expr, $description:expr) => {
        $crate::log::Log::new(
            &vrd::random::Random::default()
                .int(0, 1_000_000_000)
                .to_string(),
            $time,
            &$crate::log_level::LogLevel::INFO,
            $component,
            $description,
            &$crate::log_format::LogFormat::CLF,
        )
    };
}

/// This macro asynchronously logs a message to a file.
/// It returns the result of the logging operation, which could be
/// used to check the success or failure of the logging action.
///
/// # Parameters
/// - `log`: The log entry to be saved to a file.
///
/// # Example
/// ```
/// use rlg::{macro_log,macro_log_to_file,macro_info_log};
/// use rlg::log_format::LogFormat;
/// use rlg::log_level::LogLevel;
/// let log = macro_info_log!("2022-01-01", "app", "message");
/// async {
/// let result = macro_log_to_file!(log);
/// };
/// ```
/// Usage:
/// let result = macro_log_to_file!(log);
#[macro_export]
#[doc = "Async log message to file"]
macro_rules! macro_log_to_file {
    ($log:expr) => {{
        let result = $log.log().await;
        result
    }};
}

/// This macro creates a `WARN` level log entry with a default session ID and format.
/// The session ID is generated randomly and the log format defaults to CLF.
///
/// # Parameters
/// - `time`: The timestamp of the log entry.
/// - `component`: The system component that generated the log.
/// - `description`: A textual description of the log event.
///
/// # Example
/// ```
/// use rlg::{macro_warn_log, macro_log};
/// use rlg::log_level::LogLevel;
/// use rlg::log_format::LogFormat;
/// let log = macro_warn_log!("2024-08-29T12:00:00Z", "Auth", "Invalid password attempt");
/// ```
/// Usage:
/// let log = macro_warn_log!(time, component, description);
#[macro_export]
#[doc = "Macro for warn log with default session id and format"]
macro_rules! macro_warn_log {
    ($time:expr, $component:expr, $description:expr) => {
        $crate::macro_log!(
            &vrd::random::Random::default()
                .int(0, 1_000_000_000)
                .to_string(),
            $time,
            &$crate::log_level::LogLevel::WARN,
            $component,
            $description,
            &$crate::log_format::LogFormat::CLF
        )
    };
}

/// This macro creates an `ERROR` level log entry with a default session ID and format.
/// The session ID is generated randomly and the log format defaults to CLF.
///
/// # Parameters
/// - `time`: The timestamp of the log entry.
/// - `component`: The system component that generated the log.
/// - `description`: A textual description of the log event.
///
/// # Example
/// ```
/// use rlg::{macro_error_log, macro_log};
/// use rlg::log_level::LogLevel;
/// use rlg::log_format::LogFormat;
/// let log = macro_error_log!("2024-08-29T12:00:00Z", "Database", "Connection failed");
/// ```
/// Usage:
/// let log = macro_error_log!(time, component, description);
#[macro_export]
#[doc = "Macro for error log with default session id and format"]
macro_rules! macro_error_log {
    ($time:expr, $component:expr, $description:expr) => {
        $crate::macro_log!(
            &vrd::random::Random::default()
                .int(0, 1_000_000_000)
                .to_string(),
            $time,
            &$crate::log_level::LogLevel::ERROR,
            $component,
            $description,
            &$crate::log_format::LogFormat::CLF
        )
    };
}

/// This macro creates a `TRACE` level log entry with a default session ID and format.
/// The session ID is generated randomly and the log format defaults to CLF.
///
/// # Parameters
/// - `time`: The timestamp of the log entry.
/// - `component`: The system component that generated the log.
/// - `description`: A textual description of the log event.
///
/// # Example
/// ```
/// use rlg::{macro_trace_log, macro_log};
/// use rlg::log_level::LogLevel;
/// use rlg::log_format::LogFormat;
/// let log = macro_trace_log!("2024-08-29T12:00:00Z", "Auth", "Tracing user activity");
/// ```
/// Usage:
/// let log = macro_trace_log!(time, component, description);
#[macro_export]
#[doc = "Macro for trace log with default session id and format"]
macro_rules! macro_trace_log {
    ($time:expr, $component:expr, $description:expr) => {
        $crate::macro_log!(
            &vrd::random::Random::default()
                .int(0, 1_000_000_000)
                .to_string(),
            $time,
            &$crate::log_level::LogLevel::TRACE,
            $component,
            $description,
            &$crate::log_format::LogFormat::CLF
        )
    };
}

/// This macro creates a `FATAL` level log entry with a default session ID and format.
/// The session ID is generated randomly and the log format defaults to CLF.
///
/// # Parameters
/// - `time`: The timestamp of the log entry.
/// - `component`: The system component that generated the log.
/// - `description`: A textual description of the log event.
///
/// # Example
/// ```
/// use rlg::log_level::LogLevel;
/// use rlg::log_format::LogFormat;
/// use rlg::{macro_fatal_log, macro_log};
/// let log = macro_fatal_log!("2024-08-29T12:00:00Z", "System", "Critical failure");
/// ```
/// Usage:
/// let log = macro_fatal_log!(time, component, description);
#[macro_export]
#[doc = "Macro for fatal log with default session id and format"]
macro_rules! macro_fatal_log {
    ($time:expr, $component:expr, $description:expr) => {
        $crate::macro_log!(
            &vrd::random::Random::default()
                .int(0, 1_000_000_000)
                .to_string(),
            $time,
            &$crate::log_level::LogLevel::FATAL,
            $component,
            $description,
            &$crate::log_format::LogFormat::CLF
        )
    };
}

// ========================
// Macros for Log Formatting
// ========================

/// This macro sets the log format to CLF if it is not already defined.
///
/// # Parameters
/// - `log`: The log entry whose format is to be set.
///
/// # Example
/// ```
/// use rlg::macro_set_log_format_clf;
/// use rlg::macro_info_log;
/// let mut log = macro_info_log!("2022-01-01", "app", "message");
/// macro_set_log_format_clf!(log);
/// ```
/// Usage:
/// macro_set_log_format_clf!(log);
#[macro_export]
#[doc = "Set log format to CLF if not already defined"]
macro_rules! macro_set_log_format_clf {
    ($log:expr) => {
        if $log.format != $crate::log_format::LogFormat::CLF {
            $log.format = $crate::log_format::LogFormat::CLF;
        }
    };
}

/// This macro logs with metadata.
/// It replaces specific keys in the log message with consistent ones.
///
/// # Parameters
/// - `session_id`: A unique identifier for the log session.
/// - `time`: The timestamp of the log entry.
/// - `level`: The severity level of the log.
/// - `component`: The system component that generated the log.
/// - `description`: A textual description of the log event.
/// - `format`: The format in which the log will be recorded.
///
/// # Example
/// ```
/// use rlg::{macro_log_with_metadata, log_level::LogLevel, log_format::LogFormat};
/// let log = macro_log_with_metadata!("id", "2022-01-01", &LogLevel::INFO, "app", "message", &LogFormat::JSON);
/// println!("{log} | Metadata: <metadata>");
/// ```
/// Usage:
/// let log = macro_log_with_metadata!(session_id, time, level, component, description, format);
#[macro_export]
#[doc = "Macro for logging with metadata"]
macro_rules! macro_log_with_metadata {
    ($session_id:expr, $time:expr, $level:expr, $component:expr, $description:expr, $format:expr) => {{
        let log = $crate::log::Log::new(
            $session_id,
            $time,
            $level,
            $component,
            $description,
            $format,
        );
        // Replace keys in the log message with consistent ones
        let log_message = log
            .to_string()
            .replace("\"component\"", "\"component\"")
            .replace("\"session_id\"", "\"session_id\"");
        log_message
    }};
}

// =========================
// Macros for Log Conditions
// =========================

/// This macro conditionally logs a message based on a predicate.
///
/// # Parameters
/// - `predicate`: A boolean expression that determines whether to log.
/// - `log`: The log entry to be conditionally logged.
///
/// # Example
/// ```
/// use rlg::{macro_log_if, macro_print_log};
/// use rlg::macro_info_log;
/// let log = macro_info_log!("2022-01-01", "app", "message");
/// macro_log_if!(true, log);
/// ```
/// Usage:
/// macro_log_if!(predicate, log);
#[macro_export]
#[doc = "Conditional logging based on a predicate"]
macro_rules! macro_log_if {
    ($predicate:expr, $log:expr) => {
        if $predicate {
            macro_print_log!($log);
        }
    };
}

/// This macro conditionally logs a debug message if the `debug_enabled` feature flag is set.
///
/// # Parameters
/// - `log`: The log entry to be conditionally logged.
///
/// # Example
/// ```
/// use rlg::macro_info_log;
/// use rlg::macro_debug_log;
/// use rlg::macro_print_log;
/// let log = macro_info_log!("2022-01-01", "app", "message");
/// macro_debug_log!(log);
/// ```
/// Usage:
/// macro_debug_log!(log);
#[cfg(feature = "debug_enabled")]
#[macro_export]
#[doc = "Conditional debug logging based on feature flag"]
macro_rules! macro_debug_log {
    ($log:expr) => {
        macro_print_log!($log);
    };
}

#[cfg(not(feature = "debug_enabled"))]
#[macro_export]
#[doc = "Conditional debug logging does nothing if feature flag is not set"]
macro_rules! macro_debug_log {
    ($log:expr) => {
        // Do nothing if `debug_enabled` feature flag is not set
    };
}

// =======================
// Macros for Log Output
// =======================

/// This macro prints a log entry to the standard output (stdout).
/// It is useful for debugging or simple logging to the console.
///
/// # Parameters
/// - `log`: The log entry to be printed.
///
/// # Example
/// ```
/// use rlg::{macro_print_log,macro_info_log};
/// let log = macro_info_log!("2022-01-01", "app", "message");
/// macro_print_log!(log);
/// ```
/// Usage:
/// macro_print_log!(log);
#[macro_export]
#[doc = "Print log to stdout"]
macro_rules! macro_print_log {
    ($log:expr) => {
        println!("{}", $log);
    };
}
