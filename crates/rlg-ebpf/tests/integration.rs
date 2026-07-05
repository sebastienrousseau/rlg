// integration.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Cross-target integration coverage for `Enricher` and
//! `ProcessEnricher`. Unix-specific paths gate on `cfg(unix)`.

#![allow(missing_docs)]

use rlg::log::Log;
use rlg_ebpf::{Chain, Enricher, ProcessEnricher};

#[test]
fn process_enricher_attaches_pid_across_platforms() {
    let e = ProcessEnricher::new();
    let out = e.enrich(Log::info("hello"));
    let pid = out.attributes.get("pid").expect("pid missing");
    assert!(pid.is_u64());
    assert!(pid.as_u64().unwrap() > 0);
}

#[cfg(unix)]
#[test]
fn process_enricher_attaches_uid_and_tid_on_unix() {
    let e = ProcessEnricher::new();
    let out = e.enrich(Log::info("hello"));
    assert!(out.attributes.contains_key("tid"));
    assert!(out.attributes.contains_key("uid"));
}

#[cfg(unix)]
#[test]
fn tid_differs_between_threads() {
    // Two threads should produce different TIDs; PID is shared.
    let (t1_tid, t2_tid) = std::thread::scope(|s| {
        let h1 = s.spawn(|| {
            let e = ProcessEnricher::new();
            let out = e.enrich(Log::info("h1"));
            out.attributes.get("tid").unwrap().as_u64().unwrap()
        });
        let h2 = s.spawn(|| {
            let e = ProcessEnricher::new();
            let out = e.enrich(Log::info("h2"));
            out.attributes.get("tid").unwrap().as_u64().unwrap()
        });
        (h1.join().unwrap(), h2.join().unwrap())
    });
    assert_ne!(
        t1_tid, t2_tid,
        "distinct threads must have distinct TIDs"
    );
}

#[test]
fn chain_applies_enrichers_in_order() {
    struct AppendMarker(&'static str);
    impl Enricher for AppendMarker {
        fn enrich(&self, mut log: Log) -> Log {
            let existing = log
                .attributes
                .get("markers")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string();
            let updated = format!("{existing}{}", self.0);
            log.attributes.insert(
                "markers".into(),
                serde_json::Value::from(updated),
            );
            log
        }
    }

    let chain = Chain::new(AppendMarker("A"), AppendMarker("B"));
    let out = chain.enrich(Log::info("test"));
    assert_eq!(
        out.attributes.get("markers").unwrap().as_str(),
        Some("AB")
    );
}

#[test]
fn enricher_preserves_log_level_and_description() {
    let e = ProcessEnricher::new();
    let log = Log::error("boom");
    let out = e.enrich(log);
    assert_eq!(out.description, "boom");
    assert_eq!(out.level, rlg::log_level::LogLevel::ERROR);
}
