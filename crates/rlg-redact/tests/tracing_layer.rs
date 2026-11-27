// tracing_layer.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `Redactor` as the `RlgLayer` scrubber: span attributes and log
//! records are redacted before they leave the layer.

#![allow(missing_docs)]

use rlg::log::Log;
use rlg::tracing::{RlgLayer, Scrub, SpanRecord, SpanSink};
use rlg_redact::Redactor;
use std::sync::{Arc, Mutex};
use tracing_subscriber::prelude::*;

struct Sink(Arc<Mutex<Vec<SpanRecord>>>);

impl SpanSink for Sink {
    fn on_span_end(&self, span: SpanRecord) {
        self.0.lock().unwrap().push(span);
    }
}

#[test]
fn redactor_scrubs_through_the_trait() {
    let log = Log::info("mail ops@example.com")
        .with("token", "Bearer abc.def");
    let out = Scrub::scrub(&Redactor::with_defaults(), log);
    assert_eq!(out.description, "mail [REDACTED]");
    assert_eq!(out.attributes["token"], "[REDACTED]");
}

#[test]
#[cfg_attr(miri, ignore)]
fn redactor_scrubs_closed_span_attributes() {
    let spans = Arc::new(Mutex::new(Vec::new()));
    let layer = RlgLayer::new()
        .with_scrub(Redactor::with_defaults())
        .with_span_sink(Sink(Arc::clone(&spans)));
    tracing::subscriber::with_default(
        tracing_subscriber::registry().with(layer),
        || {
            drop(tracing::error_span!(
                "request",
                user = "ops@example.com",
                route = "/v1/allocate"
            ));
        },
    );
    let span = spans.lock().unwrap()[0].clone();
    assert_eq!(span.attributes["user"], "[REDACTED]");
    assert_eq!(span.attributes["route"], "/v1/allocate");
}
