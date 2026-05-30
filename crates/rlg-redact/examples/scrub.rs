// scrub.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Demonstrates redacting PII / secrets from rlg records before they
// are dispatched to a sink.
//
// Run with: cargo run -p rlg-redact --example scrub

#![allow(missing_docs)]

use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg_redact::Redactor;

fn main() {
    let redactor = Redactor::with_defaults();

    let raw = Log::info(
        "card 4111-1111-1111-1111 declined for alice@example.com",
    )
    .component("payments")
    .with("client_ip", "192.0.2.42")
    .with("session_token", "Bearer abc123XYZdef")
    .with("order_id", 9001_u64) // numeric — preserved as-is
    .format(LogFormat::JSON);

    let safe = redactor.scrub(raw);

    println!("description : {}", safe.description);
    for (k, v) in &safe.attributes {
        println!("  {k:<16}{v}");
    }

    assert!(safe.description.contains("[REDACTED]"));
    assert!(!safe.description.contains("4111"));
    assert!(!safe.description.contains("alice@example.com"));
}
