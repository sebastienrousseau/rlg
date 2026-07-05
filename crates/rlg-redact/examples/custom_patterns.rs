// custom_patterns.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Demonstrates the low-level Redactor knobs — start empty, load
// only specific patterns, add an ad-hoc pattern, and override the
// redaction marker.
//
// Run with: cargo run -p rlg-redact --example custom_patterns

#![allow(missing_docs)]

use rlg::log::Log;
use rlg_redact::{EMAIL, JWT, Redactor};

fn main() -> Result<(), regex::Error> {
    // A locked-down redactor that only scrubs emails and JWTs, and
    // marks matches with `<PII>` instead of the default
    // `[REDACTED]`. `.empty()` starts with zero patterns; each
    // `.with_pattern(...)` recompiles the fused alternation regex
    // and can fail if the pattern doesn't parse.
    let redactor = Redactor::empty()
        .with_pattern(EMAIL)?
        .with_pattern(JWT)?
        // Bespoke pattern — an internal ticket ID that should never
        // reach the sink verbatim.
        .with_pattern(r"TCK-[0-9]{6}")?
        .marker("<PII>");

    let raw = Log::info(
        "user alice@example.com opened ticket TCK-123456 with token \
         eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.abcdefghijklmnopqrstuvwx",
    );

    let safe = redactor.scrub(raw);

    println!("scrubbed description:\n  {}", safe.description);

    assert!(safe.description.contains("<PII>"));
    assert!(!safe.description.contains("alice@example.com"));
    assert!(!safe.description.contains("TCK-123456"));
    assert!(!safe.description.contains("eyJhbGciOiJIUzI1NiJ9"));
    Ok(())
}
