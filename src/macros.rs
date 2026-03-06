// macros.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Modernized macros for RLG (v0.0.7).
//! All macros route through the zero-latency LMAX Disruptor engine.

// ======================
// Legacy Macros (Rerouted)
// ======================

/// This macro simplifies the creation of log entries with specific parameters.
#[macro_export]
#[deprecated(
    since = "0.0.7",
    note = "Use the fluent builder API: rlg::Log::info().fire()"
)]
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
#[deprecated(
    since = "0.0.7",
    note = "Use the fluent builder API: rlg::Log::info().fire()"
)]
macro_rules! macro_info_log {
    ($time:expr, $component:expr, $description:expr) => {
        $crate::log::Log::info($description)
            .component($component)
            .time($time)
            .format($crate::log_format::LogFormat::CLF)
    };
}

/// This macro asynchronously logs a message.
#[macro_export]
#[deprecated(
    since = "0.0.7",
    note = "Use the fluent builder API: rlg::Log::info().fire()"
)]
macro_rules! macro_log_to_file {
    ($log:expr) => {{
        $log.fire();
        Ok::<(), $crate::error::RlgError>(())
    }};
}

/// This macro creates a `WARN` level log entry.
#[macro_export]
#[deprecated(
    since = "0.0.7",
    note = "Use the fluent builder API: rlg::Log::warn().fire()"
)]
macro_rules! macro_warn_log {
    ($time:expr, $component:expr, $description:expr) => {
        $crate::log::Log::warn($description)
            .component($component)
            .time($time)
            .format($crate::log_format::LogFormat::CLF)
    };
}

/// This macro creates an `ERROR` level log entry.
#[macro_export]
#[deprecated(
    since = "0.0.7",
    note = "Use the fluent builder API: rlg::Log::error().fire()"
)]
macro_rules! macro_error_log {
    ($time:expr, $component:expr, $description:expr) => {
        $crate::log::Log::error($description)
            .component($component)
            .time($time)
            .format($crate::log_format::LogFormat::CLF)
    };
}

/// This macro creates a `TRACE` level log entry.
#[macro_export]
#[deprecated(
    since = "0.0.7",
    note = "Use the fluent builder API: rlg::Log::trace().fire()"
)]
macro_rules! macro_trace_log {
    ($time:expr, $component:expr, $description:expr) => {
        $crate::log::Log::trace($description)
            .component($component)
            .time($time)
            .format($crate::log_format::LogFormat::CLF)
    };
}

/// This macro creates a `FATAL` level log entry.
#[macro_export]
#[deprecated(
    since = "0.0.7",
    note = "Use the fluent builder API: rlg::Log::fatal().fire()"
)]
macro_rules! macro_fatal_log {
    ($time:expr, $component:expr, $description:expr) => {
        $crate::log::Log::fatal($description)
            .component($component)
            .time($time)
            .format($crate::log_format::LogFormat::CLF)
    };
}

/// This macro routes debug logs through the lock-free engine when the feature is active.
#[macro_export]
#[deprecated(
    since = "0.0.7",
    note = "Use the fluent builder: rlg::Log::debug().fire()"
)]
macro_rules! macro_debug_log {
    ($log:expr) => {
        #[cfg(feature = "debug_enabled")]
        $log.fire();
        #[cfg(not(feature = "debug_enabled"))]
        {
            let _ = &$log;
        }
    };
}

/// This macro prints a log entry through the non-blocking engine.
#[macro_export]
#[deprecated(
    since = "0.0.7",
    note = "Route through the engine: rlg::Log::info().fire()"
)]
macro_rules! macro_print_log {
    ($log:expr) => {
        $log.fire();
    };
}

/// This macro sets the log format to CLF.
#[macro_export]
#[deprecated(
    since = "0.0.7",
    note = "Pass the format dynamically: .format(LogFormat::CLF)"
)]
macro_rules! macro_set_log_format_clf {
    ($log:expr) => {
        $log.format = $crate::log_format::LogFormat::CLF;
    };
}

/// This macro creates a log entry with custom metadata.
#[macro_export]
#[deprecated(
    since = "0.0.7",
    note = "Use the fluent builder API: rlg::Log::info().with()"
)]
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
#[deprecated(
    since = "0.0.7",
    note = "Use conditional logic with the fluent API."
)]
macro_rules! macro_log_if {
    ($condition:expr, $log:expr) => {
        if $condition {
            $log.fire();
        }
    };
}

// ======================
// 2026 Liquid Macros
// ======================

/// Injects OTLP context and executes a block of code.
#[macro_export]
macro_rules! rlg_span {
    ($name:expr, $block:block) => {{
        let span_id = $crate::utils::generate_span_id();
        $crate::engine::ENGINE.inc_spans();
        $crate::log::Log::info($name)
            .with("span_id", &span_id)
            .format($crate::log_format::LogFormat::OTLP)
            .fire();
        let result = $block;
        $crate::engine::ENGINE.dec_spans();
        result
    }};
}

/// Measures latency and emits a Logfmt profile metric.
#[macro_export]
macro_rules! rlg_time_it {
    ($action:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let elapsed = start.elapsed().as_micros();

        $crate::log::Log::info(&format!("{} completed", $action))
            .with("latency_us", elapsed as u64)
            .format($crate::log_format::LogFormat::Logfmt)
            .fire();

        result
    }};
}

/// Forces MCP format for AI state synchronization.
#[macro_export]
macro_rules! rlg_mcp_notify {
    ($state_key:expr, $state_val:expr) => {
        $crate::log::Log::info("State transition")
            .with($state_key, $state_val)
            .format($crate::log_format::LogFormat::MCP)
            .fire();
    };
}
