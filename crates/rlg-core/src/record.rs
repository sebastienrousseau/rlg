// record.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{Format, LogLevel};
use alloc::collections::BTreeMap;
use alloc::string::String;
use core::fmt;

mod escape;
mod json;
mod text;

/// Structured key-value context on a record, as `rlg::Log` stores it.
pub type Attributes = BTreeMap<String, serde_json::Value>;

/// No attributes, for records built with [`Record::new`].
static NO_ATTRIBUTES: Attributes = BTreeMap::new();

/// A borrowed view of one log record: the fields of `rlg::Log`, plus
/// the host name the Apache access-log format prints.
///
/// Rendering it ([`Record::write_to`], or `Display`) writes the record
/// in [`Record::format`], byte for byte as `rlg::Log` does. Nothing is
/// allocated except for logfmt array and object attribute values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Record<'a> {
    /// Monotonic session identifier.
    pub session_id: u64,
    /// Timestamp, written as given.
    pub time: &'a str,
    /// Severity level.
    pub level: LogLevel,
    /// Originating service or module name.
    pub component: &'a str,
    /// Human-readable message body.
    pub description: &'a str,
    /// Output format.
    pub format: Format,
    /// Key-value attributes.
    pub attributes: &'a Attributes,
    /// Host name, written only by [`Format::ApacheAccessLog`].
    pub host: &'a str,
}

/// Writes one record in one format.
type Writer = fn(&Record<'_>, &mut dyn fmt::Write) -> fmt::Result;

/// One writer per [`Format`], indexed by [`Format::index`]. The
/// `[_; 14]` type and the exhaustive match in `format.rs`'s tests keep
/// it in step with the enum.
const WRITERS: [Writer; Format::ALL.len()] = [
    text::clf,
    json::json,
    text::cef,
    text::elf,
    text::w3c,
    json::gelf,
    text::apache,
    json::logstash,
    text::log4j_xml,
    json::ndjson,
    json::mcp,
    json::otlp,
    text::logfmt,
    json::ecs,
];

impl<'a> Record<'a> {
    /// A record at `level` with `description`, session 0, an empty
    /// timestamp and host, component `"default"`, no attributes, in
    /// [`Format::MCP`] (the defaults of `rlg::Log::build`, minus the
    /// clock and counter, which need `std`).
    #[must_use]
    pub const fn new(level: LogLevel, description: &'a str) -> Self {
        Self {
            session_id: 0,
            time: "",
            level,
            component: "default",
            description,
            format: Format::MCP,
            attributes: &NO_ATTRIBUTES,
            host: "",
        }
    }

    /// Set the session identifier.
    #[must_use]
    pub const fn session_id(mut self, session_id: u64) -> Self {
        self.session_id = session_id;
        self
    }

    /// Set the timestamp.
    #[must_use]
    pub const fn time(mut self, time: &'a str) -> Self {
        self.time = time;
        self
    }

    /// Set the component.
    #[must_use]
    pub const fn component(mut self, component: &'a str) -> Self {
        self.component = component;
        self
    }

    /// Set the output format.
    #[must_use]
    pub const fn format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    /// Set the attributes.
    #[must_use]
    pub const fn attributes(
        mut self,
        attributes: &'a Attributes,
    ) -> Self {
        self.attributes = attributes;
        self
    }

    /// Set the host name the Apache access-log format prints.
    #[must_use]
    pub const fn host(mut self, host: &'a str) -> Self {
        self.host = host;
        self
    }

    /// Write this record in [`Record::format`] to `out`.
    ///
    /// # Errors
    ///
    /// Whatever `out` returns; the formatting itself cannot fail.
    pub fn write_to(&self, out: &mut dyn fmt::Write) -> fmt::Result {
        WRITERS[self.format.index()](self, out)
    }
}

impl fmt::Display for Record<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_to(f)
    }
}

#[cfg(test)]
mod tests;
