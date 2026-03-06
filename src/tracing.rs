// tracing.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Integration with the `tracing` ecosystem.

use crate::log::Log;
use crate::log_level::LogLevel;
use tracing_core::{Event, Level, Metadata, Subscriber};
use tracing_core::field::{Field, Visit};

/// A `tracing::Subscriber` that routes events to the `RLG` engine.
#[derive(Debug, Default, Clone, Copy)]
pub struct RlgSubscriber;

impl RlgSubscriber {
    /// Create a new `RlgSubscriber`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Subscriber for RlgSubscriber {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        // We defer to the engine's global filter
        let level = match *metadata.level() {
            Level::ERROR => LogLevel::ERROR,
            Level::WARN => LogLevel::WARN,
            Level::INFO => LogLevel::INFO,
            Level::DEBUG => LogLevel::DEBUG,
            Level::TRACE => LogLevel::TRACE,
        };
        level.to_numeric() >= crate::engine::ENGINE.filter_level()
    }

    fn new_span(&self, _span: &tracing_core::span::Attributes<'_>) -> tracing_core::span::Id {
        tracing_core::span::Id::from_u64(1) // Simple placeholder for now
    }

    fn record(&self, _span: &tracing_core::span::Id, _values: &tracing_core::span::Record<'_>) {}

    fn record_follows_from(&self, _span: &tracing_core::span::Id, _follows: &tracing_core::span::Id) {}

    fn event(&self, event: &Event<'_>) {
        let metadata = event.metadata();
        let level = match *metadata.level() {
            Level::ERROR => LogLevel::ERROR,
            Level::WARN => LogLevel::WARN,
            Level::INFO => LogLevel::INFO,
            Level::DEBUG => LogLevel::DEBUG,
            Level::TRACE => LogLevel::TRACE,
        };

        let mut visitor = RlgVisitor::default();
        event.record(&mut visitor);

        let mut log = Log::build(level, &visitor.message);
        log.component = metadata.target().to_string();
        
        for (key, value) in visitor.fields {
            log = log.with(&key, value);
        }

        log.fire();
    }

    fn enter(&self, _span: &tracing_core::span::Id) {}

    fn exit(&self, _span: &tracing_core::span::Id) {}
}

#[derive(Default)]
struct RlgVisitor {
    message: String,
    fields: std::collections::BTreeMap<String, serde_json::Value>,
}

impl Visit for RlgVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        } else {
            self.fields.insert(field.name().to_string(), serde_json::json!(format!("{value:?}")));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            self.fields.insert(field.name().to_string(), serde_json::json!(value));
        }
    }
}
