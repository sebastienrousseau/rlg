// enrich.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Demonstrates attaching per-process context (pid/tid/uid) to
// every rlg record before firing it.
//
// Run with: cargo run -p rlg-ebpf --example enrich

#![allow(missing_docs)]

use rlg::log::Log;
use rlg_ebpf::{Enricher, ProcessEnricher};

fn main() {
    let enricher = ProcessEnricher::new();

    // Any log the app produces routes through `enricher.enrich`
    // before dispatch. In a production pipeline you'd wrap this
    // in a helper that the whole codebase calls.
    let raw = Log::info("checkout completed in 142ms")
        .component("orders")
        .with("order_id", 9001_u64);

    let enriched = enricher.enrich(raw);

    println!("── enriched attributes ─────");
    for (k, v) in &enriched.attributes {
        println!("  {k:<12} {v}");
    }
    println!(
        "\ndescription : {}\nlevel       : {}",
        enriched.description, enriched.level
    );
}
