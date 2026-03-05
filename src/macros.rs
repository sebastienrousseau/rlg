// macros.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// ======================
// Macros for Log Creation
// ======================

/// This macro simplifies the creation of log entries with specific parameters.
#[macro_export]
#[doc = "Macro to create a new log easily"]
#[deprecated(since = "0.0.7", note = "Use the fluent builder API: rlg::Log::info().fire()")]
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
#[macro_export]
#[doc = "Macro for info log with default session id and format"]
#[deprecated(since = "0.0.7", note = "Use the fluent builder API: rlg::Log::info().fire()")]
macro_rules! macro_info_log {
    ($time:expr, $component:expr, $description:expr) => {
        $crate::log::Log::info($description)
            .component($component)
            .time($time)
            .format($crate::log_format::LogFormat::CLF)
    };
}

/// This macro asynchronously logs a message to a file.
#[macro_export]
#[doc = "Async log message to file"]
#[deprecated(since = "0.0.7", note = "Use the fluent builder API: rlg::Log::info().fire()")]
macro_rules! macro_log_to_file {
    ($log:expr) => {{
        $log.fire();
        Ok::<(), $crate::error::RlgError>(())
    }};
}

/// This macro creates a `WARN` level log entry with a default session ID and format.
#[macro_export]
#[doc = "Macro for warn log with default session id and format"]
#[deprecated(since = "0.0.7", note = "Use the fluent builder API: rlg::Log::warn().fire()")]
macro_rules! macro_warn_log {
    ($time:expr, $component:expr, $description:expr) => {
        $crate::log::Log::warn($description)
            .component($component)
            .time($time)
            .format($crate::log_format::LogFormat::CLF)
    };
}

/// This macro creates an `ERROR` level log entry with a default session ID and format.
#[macro_export]
#[doc = "Macro for error log with default session id and format"]
#[deprecated(since = "0.0.7", note = "Use the fluent builder API: rlg::Log::error().fire()")]
macro_rules! macro_error_log {
    ($time:expr, $component:expr, $description:expr) => {
        $crate::log::Log::error($description)
            .component($component)
            .time($time)
            .format($crate::log_format::LogFormat::CLF)
    };
}

/// This macro creates a `TRACE` level log entry with a default session ID and format.
#[macro_export]
#[doc = "Macro for trace log with default session id and format"]
#[deprecated(since = "0.0.7", note = "Use the fluent builder API: rlg::Log::trace().fire()")]
macro_rules! macro_trace_log {
    ($time:expr, $component:expr, $description:expr) => {
        $crate::log::Log::trace($description)
            .component($component)
            .time($time)
            .format($crate::log_format::LogFormat::CLF)
    };
}

/// This macro creates a `FATAL` level log entry with a default session ID and format.
#[macro_export]
#[doc = "Macro for fatal log with default session id and format"]
#[deprecated(since = "0.0.7", note = "Use the fluent builder API: rlg::Log::fatal().fire()")]
macro_rules! macro_fatal_log {
    ($time:expr, $component:expr, $description:expr) => {
        $crate::log::Log::fatal($description)
            .component($component)
            .time($time)
            .format($crate::log_format::LogFormat::CLF)
    };
}

/// This macro creates a `DEBUG` level log entry with a default session ID and format.
#[macro_export]
#[doc = "Conditional debug logging based on feature flag"]
#[deprecated(since = "0.0.7", note = "Use the fluent builder API: rlg::Log::debug().fire()")]
macro_rules! macro_debug_log {
    ($log:expr) => {
        #[cfg(feature = "debug_enabled")]
        {
            println!("{}", $log.description);
        }
        #[cfg(not(feature = "debug_enabled"))]
        {
            // Do nothing
        }
    };
}

/// This macro prints a log entry to the console.
#[macro_export]
#[doc = "Print log to stdout"]
#[deprecated(since = "0.0.7", note = "Use the fluent builder API: rlg::Log::info().fire()")]
macro_rules! macro_print_log {
    ($log:expr) => {
        println!("{}", $log.description);
    };
}

/// This macro sets the log format to CLF.
#[macro_export]
#[doc = "Macro to set log format to CLF"]
#[deprecated(since = "0.0.7", note = "Use the fluent builder API: rlg::Log::info().format(LogFormat::CLF)")]
macro_rules! macro_set_log_format_clf {
    ($log:expr) => {
        $log.format = $crate::log_format::LogFormat::CLF;
    };
}

/// This macro creates a log entry with custom metadata.
#[macro_export]
#[doc = "Macro to create a log with metadata"]
#[deprecated(since = "0.0.7", note = "Use the fluent builder API: rlg::Log::info().with()")]
macro_rules! macro_log_with_metadata {
    ($session_id:expr, $time:expr, $level:expr, $component:expr, $description:expr, $format:expr) => {
        format!(
            "{{\"SessionID\":\"{}\",\"Timestamp\":\"{}\",\"Level\":\"{:?}\",\"Component\":\"{}\",\"Description\":\"{}\",\"Format\":\"{:?}\"}}",
            $session_id, $time, $level, $component, $description, $format
        )
    };
}

/// This macro conditionally creates a log entry.
#[macro_export]
#[doc = "Macro to conditionally create a log"]
#[deprecated(since = "0.0.7", note = "Use conditional logic with the fluent API.")]
macro_rules! macro_log_if {
    ($condition:expr, $log:expr) => {
        if $condition {
            let _ = $log.fire();
        }
    };
}
