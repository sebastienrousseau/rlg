// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # RustLogs (RLG) Example Entry Point
//!
//! This file serves as an entry point for running all the RustLogs (RLG) examples,
//! demonstrating logging levels, formats, macros, and library functionality.

mod example_lib;
mod example_log_format;
mod example_log_level;
mod example_macros;

/// Entry point to run all RustLogs examples.
///
/// This function calls all the individual examples for log levels, log formats, macros, and library functionality.
fn main() {
    println!("🦀 Running RustLogs (RLG) Examples 🦀\n");

    let _ = example_log_format::main();
    example_log_level::main().expect("Log level examples failed");
    example_macros::main();
    example_lib::main();

    println!("\n🎉 All RustLogs examples completed successfully!");
}
