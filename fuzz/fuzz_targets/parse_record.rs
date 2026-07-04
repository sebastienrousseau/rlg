// parse_record.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Fuzz target: `rlg_cli::parse_record`.
//
// Contract under fuzz: for every input byte string, `parse_record`
// either returns `Ok(Log)` or `Err(serde_json::Error)`. It must
// never panic, abort, or exhibit UB.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // The public API takes `&str`; funnel only valid UTF-8 through.
    // Non-UTF-8 inputs are rejected at the boundary — fuzzing that
    // rejection would exercise `std::str::from_utf8`, not our code.
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = rlg_cli::parse_record(s);
    }
});
