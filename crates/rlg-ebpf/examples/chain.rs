// chain.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Demonstrates composing enrichers with `Chain` — apply a
// portable ProcessEnricher first, then layer on an application-
// specific enricher that adds a static service tag.
//
// Run with: cargo run -p rlg-ebpf --example chain

#![allow(missing_docs)]

use rlg::log::Log;
use rlg_ebpf::{Chain, Enricher, ProcessEnricher};
use serde_json::Value;

// A minimal application-specific enricher — attaches the service
// name so every log line downstream is unambiguous even if the
// process is `Envoy` or `unpin` or another shared binary.
struct ServiceTag {
    name: &'static str,
}

impl Enricher for ServiceTag {
    fn enrich(&self, mut log: Log) -> Log {
        log.attributes
            .insert("service.name".into(), Value::from(self.name));
        log
    }
}

fn main() {
    // Chain applies enrichers left-to-right. `first` (Process) runs
    // first and adds `pid`/`tid`/`uid`; `second` (ServiceTag)
    // then layers on `service.name`.
    let pipeline = Chain::new(
        ProcessEnricher::new(),
        ServiceTag { name: "checkout" },
    );

    let raw = Log::info("charged customer 9001");
    let enriched = pipeline.enrich(raw);

    println!("── chained attributes ─────");
    for (k, v) in &enriched.attributes {
        println!("  {k:<16} {v}");
    }
    assert!(enriched.attributes.contains_key("pid"));
    assert_eq!(
        enriched
            .attributes
            .get("service.name")
            .and_then(Value::as_str),
        Some("checkout")
    );
}
