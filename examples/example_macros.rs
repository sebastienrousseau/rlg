// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # RustLogs (RLG) Macros Examples
//!
//! This example demonstrates the usage of the fluent API and modern macros
//! in the RustLogs (RLG) library.

use rlg::log::Log;
use rlg::log_format::LogFormat;

/// Entry point for the RustLogs macros examples.
///
/// This function demonstrates the fluent API and modern macros for logging.
pub(crate) fn main() {
    println!("🦀 RustLogs Macros Examples 🦀\n");

    fluent_api_example();
    conditional_log_example();
    rlg_span_example();
    rlg_time_it_example();
    rlg_mcp_notify_example();
    unicode_log_example();

    println!("\n🎉 All examples completed successfully!");
}

/// Demonstrates the fluent builder API.
fn fluent_api_example() {
    println!("🦀  **Fluent API Example**");
    println!("---------------------------------------------");

    Log::info("This is an info message")
        .component("component")
        .time("2022-01-01T12:00:00Z")
        .format(LogFormat::JSON)
        .fire();

    Log::warn("This is a warning").component("component").fire();

    Log::error("This is an error").component("component").fire();

    Log::trace("This is a trace").component("component").fire();

    Log::fatal("This is fatal").component("component").fire();
}

/// Demonstrates conditional logging.
fn conditional_log_example() {
    println!("\n🦀  **Conditional Log Example**");
    println!("---------------------------------------------");

    let should_log = true;
    if should_log {
        Log::info("Conditional log fired")
            .component("component")
            .fire();
    }
    println!("(Log with true condition should have fired)");

    let should_not_log = false;
    if should_not_log {
        Log::warn("This should not appear")
            .component("component")
            .fire();
    }
    println!("(Log with false condition should not have fired)");
}

/// Demonstrates the rlg_span! macro.
fn rlg_span_example() {
    println!("\n🦀  **rlg_span! Example**");
    println!("---------------------------------------------");

    let result = rlg::rlg_span!("Compute Task", {
        let x = 10;
        let y = 20;
        x + y
    });
    println!("    Span result: {}", result);
}

/// Demonstrates the rlg_time_it! macro.
fn rlg_time_it_example() {
    println!("\n🦀  **rlg_time_it! Example**");
    println!("---------------------------------------------");

    let result = rlg::rlg_time_it!("Database Query", {
        let x = 100;
        x * 2
    });
    println!("    Timed result: {}", result);
}

/// Demonstrates the rlg_mcp_notify! macro.
fn rlg_mcp_notify_example() {
    println!("\n🦀  **rlg_mcp_notify! Example**");
    println!("---------------------------------------------");

    rlg::rlg_mcp_notify!("user_status", "logged_in");
    println!("    MCP notification sent");
}

/// Demonstrates logging messages with Unicode characters.
fn unicode_log_example() {
    println!("\n🦀  **Unicode Log Example**");
    println!("---------------------------------------------");

    Log::info("こんにちは RustLogs! 🦀")
        .component("component")
        .time("2022-01-01T12:00:00Z")
        .fire();
}
