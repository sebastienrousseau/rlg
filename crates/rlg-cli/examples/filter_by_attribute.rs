// filter_by_attribute.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Demonstrates the `Filter::attribute(key, value)` builder — the
// third filter axis alongside `.min_level()` and `.component()`.
// Useful when you want "every record where request-id == X" or
// "every record for customer = acme".
//
// Run with: cargo run -p rlg-cli --example filter_by_attribute

#![allow(missing_docs)]

use rlg::log_format::LogFormat;
use rlg_cli::{Filter, parse_record, render};

const NDJSON: &[&str] = &[
    r#"{"session_id":1,"time":"t","level":"INFO","component":"api","description":"start","format":"JSON","attributes":{"request_id":"req-1","tenant":"acme"}}"#,
    r#"{"session_id":2,"time":"t","level":"INFO","component":"api","description":"start","format":"JSON","attributes":{"request_id":"req-2","tenant":"widgets"}}"#,
    r#"{"session_id":3,"time":"t","level":"INFO","component":"api","description":"end","format":"JSON","attributes":{"request_id":"req-1","tenant":"acme"}}"#,
];

fn main() {
    // Keep only records that carry `tenant = acme` in their attribute
    // map. The remaining records are dropped.
    let filter = Filter::new().attribute("tenant", "acme".into());

    for line in NDJSON {
        if let Ok(record) = parse_record(line)
            && filter.matches(&record)
        {
            println!("{}", render(record, LogFormat::JSON));
        }
    }
}
