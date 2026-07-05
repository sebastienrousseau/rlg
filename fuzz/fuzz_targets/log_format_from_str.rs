// log_format_from_str.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Fuzz target: `<rlg::log_format::LogFormat as FromStr>::from_str`.
//
// Contract under fuzz: every input parses to one of the 14 known
// variants or returns an error. Never panics.

#![no_main]

use libfuzzer_sys::fuzz_target;
use rlg::log_format::LogFormat;
use std::str::FromStr;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = LogFormat::from_str(s);
    }
});
