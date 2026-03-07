// tracing.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Integration with the `tracing` ecosystem.
//!
//! Provides both a standalone [`RlgSubscriber`] and, behind the
//! `tracing-layer` feature, a composable [`RlgLayer`].

use crate::log::Log;
use crate::log_level::LogLevel;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing_core::field::{Field, Visit};
use tracing_core::{Event, Level, Metadata, Subscriber};

/// Maps a [`tracing_core::Level`] to an RLG [`LogLevel`].
fn map_tracing_level(level: Level) -> LogLevel {
    if level == Level::ERROR {
        LogLevel::ERROR
    } else if level == Level::WARN {
        LogLevel::WARN
    } else if level == Level::INFO {
        LogLevel::INFO
    } else if level == Level::DEBUG {
        LogLevel::DEBUG
    } else {
        LogLevel::TRACE
    }
}

/// Monotonic span ID counter for unique span identification.
static SPAN_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

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
        map_tracing_level(*metadata.level()).to_numeric()
            >= crate::engine::ENGINE.filter_level()
    }

    fn new_span(
        &self,
        _span: &tracing_core::span::Attributes<'_>,
    ) -> tracing_core::span::Id {
        tracing_core::span::Id::from_u64(
            SPAN_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
        )
    }

    fn record(
        &self,
        _span: &tracing_core::span::Id,
        _values: &tracing_core::span::Record<'_>,
    ) {
    }

    fn record_follows_from(
        &self,
        _span: &tracing_core::span::Id,
        _follows: &tracing_core::span::Id,
    ) {
    }

    fn event(&self, event: &Event<'_>) {
        let metadata = event.metadata();
        let level = map_tracing_level(*metadata.level());

        let mut visitor = RlgVisitor::default();
        event.record(&mut visitor);

        let mut log = Log::build(level, &visitor.message);
        log.component =
            std::borrow::Cow::Owned(metadata.target().to_string());

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

macro_rules! impl_record_field {
    ($method:ident, $ty:ty) => {
        fn $method(&mut self, field: &Field, value: $ty) {
            self.fields.insert(
                field.name().to_string(),
                serde_json::json!(value),
            );
        }
    };
    (stringify $method:ident, $ty:ty) => {
        fn $method(&mut self, field: &Field, value: $ty) {
            self.fields.insert(
                field.name().to_string(),
                serde_json::json!(value.to_string()),
            );
        }
    };
}

impl Visit for RlgVisitor {
    fn record_debug(
        &mut self,
        field: &Field,
        value: &dyn std::fmt::Debug,
    ) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        } else {
            self.fields.insert(
                field.name().to_string(),
                serde_json::json!(format!("{value:?}")),
            );
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            self.fields.insert(
                field.name().to_string(),
                serde_json::json!(value),
            );
        }
    }

    fn record_error(
        &mut self,
        field: &Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::json!(value.to_string()),
        );
    }

    impl_record_field!(record_u64, u64);
    impl_record_field!(record_i64, i64);
    impl_record_field!(record_bool, bool);
    impl_record_field!(record_f64, f64);
    impl_record_field!(stringify record_u128, u128);
    impl_record_field!(stringify record_i128, i128);
}

// ---------------------------------------------------------------------------
// Composable tracing Layer (behind `tracing-layer` feature)
// ---------------------------------------------------------------------------

/// A composable [`tracing_subscriber::Layer`] that routes events into the RLG engine.
///
/// This allows RLG to be used alongside other tracing layers in a
/// `tracing_subscriber::Registry` stack.
///
/// # Example
///
/// ```rust,ignore
/// use tracing_subscriber::prelude::*;
/// use rlg::tracing::RlgLayer;
///
/// tracing_subscriber::registry()
///     .with(RlgLayer::new())
///     .init();
/// ```
#[cfg(feature = "tracing-layer")]
#[derive(Debug, Clone, Copy)]
pub struct RlgLayer {
    format: crate::log_format::LogFormat,
}

#[cfg(feature = "tracing-layer")]
impl Default for RlgLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "tracing-layer")]
impl RlgLayer {
    /// Creates a new `RlgLayer` with the default MCP format.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            format: crate::log_format::LogFormat::MCP,
        }
    }

    /// Sets the log output format for this layer.
    #[must_use]
    pub const fn with_format(
        mut self,
        format: crate::log_format::LogFormat,
    ) -> Self {
        self.format = format;
        self
    }
}

#[cfg(feature = "tracing-layer")]
impl<S> tracing_subscriber::Layer<S> for RlgLayer
where
    S: Subscriber
        + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn enabled(
        &self,
        metadata: &Metadata<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        map_tracing_level(*metadata.level()).to_numeric()
            >= crate::engine::ENGINE.filter_level()
    }

    fn on_event(
        &self,
        event: &Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let level = map_tracing_level(*metadata.level());

        let mut visitor = RlgVisitor::default();
        event.record(&mut visitor);

        let mut log = Log::build(level, &visitor.message);
        log.component =
            std::borrow::Cow::Owned(metadata.target().to_string());
        log.format = self.format;

        for (key, value) in visitor.fields {
            log = log.with(&key, value);
        }

        log.fire();
    }

    fn on_new_span(
        &self,
        _attrs: &tracing_core::span::Attributes<'_>,
        _id: &tracing_core::span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        crate::engine::ENGINE.inc_spans();
    }

    fn on_close(
        &self,
        _id: tracing_core::span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        crate::engine::ENGINE.dec_spans();
    }
}
