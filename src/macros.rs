// macros.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Convenience macros for span tracking, latency measurement, and MCP notifications.
//! All macros route through the lock-free ingestion engine.

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
