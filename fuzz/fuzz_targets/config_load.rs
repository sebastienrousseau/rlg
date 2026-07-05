// config_load.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Fuzz target: the TOML deserialisation path used by
// `rlg::config::Config::load`.
//
// Contract under fuzz: for every input, `toml::from_str::<Config>`
// either returns `Ok(Config)` or an error. Never panics.
//
// We fuzz `toml::from_str` directly instead of going through
// `Config::load` (which requires a real file path) because the file
// I/O has no fuzz-interesting behaviour of its own — it either
// reads the same bytes or fails at the OS boundary. The deserialise
// step is where corpus-driven inputs discover parser + validation
// bugs.

#![no_main]

use libfuzzer_sys::fuzz_target;
use rlg::config::Config;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = toml::from_str::<Config>(s);
    }
});
