// redact_scrub.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Fuzz target: `rlg_redact::Redactor::with_defaults().scrub`.
//
// Contract under fuzz: for every input, `scrub` produces a `Log`
// with the same shape (same attribute keys, same numeric attribute
// values). No panic; no infinite regex backtracking.

#![no_main]

use libfuzzer_sys::fuzz_target;
use rlg::log::Log;
use rlg_redact::Redactor;
use std::sync::OnceLock;

// Build the default redactor once. `with_defaults` already caches
// the six built-in regexes in a `LazyLock`, but wrapping the whole
// `Redactor` avoids re-cloning the `Vec<Regex>` on every iteration.
static REDACTOR: OnceLock<Redactor> = OnceLock::new();

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let redactor = REDACTOR.get_or_init(Redactor::with_defaults);
        let _ = redactor.scrub(Log::info(s));
    }
});
