// json.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The hand-written JSON writers behind the JSON-family formats.
//! They write straight to the output, with no intermediate
//! `serde_json::Value`, because formatting runs for every record.

use super::Attributes;
use crate::Record;
use Part::{Map, Num, Raw, Str, Value};
use alloc::string::String;
use core::fmt;

/// Writes a JSON-escaped string (with surrounding quotes).
fn write_json_str(f: &mut dyn fmt::Write, s: &str) -> fmt::Result {
    f.write_str("\"")?;
    // Unescaped runs are written as slices, escapes one at a time.
    let mut start = 0;
    for (i, c) in s.char_indices() {
        if let Some(escape) = json_escape(c) {
            f.write_str(&s[start..i])?;
            escape.write(f)?;
            start = i + c.len_utf8();
        }
    }
    f.write_str(&s[start..])?;
    f.write_str("\"")
}

/// How a character must appear inside a JSON string, if not as itself.
enum JsonEscape {
    /// A two-character escape such as `\n`.
    Short(&'static str),
    /// Any other control character, as `\u00XX`.
    Unicode(u32),
}

impl JsonEscape {
    fn write(&self, f: &mut dyn fmt::Write) -> fmt::Result {
        match self {
            Self::Short(s) => f.write_str(s),
            Self::Unicode(n) => write!(f, "\\u{n:04x}"),
        }
    }
}

fn json_escape(c: char) -> Option<JsonEscape> {
    match c {
        '"' => Some(JsonEscape::Short("\\\"")),
        '\\' => Some(JsonEscape::Short("\\\\")),
        '\n' => Some(JsonEscape::Short("\\n")),
        '\r' => Some(JsonEscape::Short("\\r")),
        '\t' => Some(JsonEscape::Short("\\t")),
        c if c.is_control() => Some(JsonEscape::Unicode(c as u32)),
        _ => None,
    }
}

/// One piece of a hand-written JSON record: literal text, or a value
/// written with the right escaping.
enum Part<'a> {
    /// Written as-is: keys, punctuation, constant values.
    Raw(&'a str),
    /// A JSON string, quoted and escaped.
    Str(&'a str),
    /// An attribute map, as a JSON object.
    Map(&'a Attributes),
    /// An unsigned number.
    Num(u64),
    /// A JSON value, as `serde_json` renders it.
    Value(&'a serde_json::Value),
}
impl Part<'_> {
    fn write(&self, f: &mut dyn fmt::Write) -> fmt::Result {
        match self {
            Self::Raw(s) => f.write_str(s),
            Self::Str(s) => write_json_str(f, s),
            Self::Map(m) => write_json_map(f, m),
            Self::Num(n) => write!(f, "{n}"),
            Self::Value(v) => write_json_value(f, v),
        }
    }
}

/// Write `parts` in order.
fn write_parts(
    f: &mut dyn fmt::Write,
    parts: &[Part<'_>],
) -> fmt::Result {
    parts.iter().try_for_each(|part| part.write(f))
}

/// Writes an attribute map as a JSON object.
fn write_json_map(
    f: &mut dyn fmt::Write,
    map: &Attributes,
) -> fmt::Result {
    f.write_str("{")?;
    let mut first = true;
    for (key, value) in map {
        if !first {
            f.write_str(",")?;
        }
        first = false;
        write_json_str(f, key)?;
        f.write_str(":")?;
        write_json_value(f, value)?;
    }
    f.write_str("}")
}

/// Writes a JSON value. Strings go through [`write_json_str`], which
/// escapes every control character, DEL and C1 included; `serde_json`'s
/// own `Display` leaves those two raw.
fn write_json_value(
    f: &mut dyn fmt::Write,
    value: &serde_json::Value,
) -> fmt::Result {
    use serde_json::Value;
    match value {
        Value::String(s) => write_json_str(f, s),
        Value::Array(items) => write_json_array(f, items),
        Value::Object(map) => write_json_object(f, map),
        other => write!(f, "{other}"),
    }
}

fn write_json_array(
    f: &mut dyn fmt::Write,
    items: &[serde_json::Value],
) -> fmt::Result {
    f.write_str("[")?;
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            f.write_str(",")?;
        }
        write_json_value(f, item)?;
    }
    f.write_str("]")
}

fn write_json_object(
    f: &mut dyn fmt::Write,
    map: &serde_json::Map<String, serde_json::Value>,
) -> fmt::Result {
    f.write_str("{")?;
    for (i, (key, value)) in map.iter().enumerate() {
        if i > 0 {
            f.write_str(",")?;
        }
        write_json_str(f, key)?;
        f.write_str(":")?;
        write_json_value(f, value)?;
    }
    f.write_str("}")
}

// --- The JSON-family formats ---

pub(super) fn json(
    r: &Record<'_>,
    f: &mut dyn fmt::Write,
) -> fmt::Result {
    write_parts(
        f,
        &[
            Raw("{\"Attributes\":"),
            Map(r.attributes),
            Raw(",\"Component\":"),
            Str(r.component),
            Raw(",\"Description\":"),
            Str(r.description),
            Raw(",\"Format\":\"JSON\",\"Level\":"),
            Str(r.level.as_str()),
            Raw(",\"SessionID\":"),
            Num(r.session_id),
            Raw(",\"Timestamp\":"),
            Str(r.time),
            Raw("}"),
        ],
    )
}

pub(super) fn gelf(
    r: &Record<'_>,
    f: &mut dyn fmt::Write,
) -> fmt::Result {
    write_parts(
        f,
        &[
            Raw("{\"_attributes\":"),
            Map(r.attributes),
            Raw(",\"_session_id\":"),
            Num(r.session_id),
            Raw(",\"full_message\":"),
            Str(r.description),
            Raw(",\"host\":"),
            Str(r.component),
            Raw(",\"level\":"),
            Num(u64::from(r.level.to_numeric())),
            Raw(",\"short_message\":"),
            Str(r.description),
            Raw(",\"timestamp\":"),
            Str(r.time),
            Raw(",\"version\":\"1.1\"}"),
        ],
    )
}

pub(super) fn logstash(
    r: &Record<'_>,
    f: &mut dyn fmt::Write,
) -> fmt::Result {
    write_parts(
        f,
        &[
            Raw("{\"@timestamp\":"),
            Str(r.time),
            Raw(",\"attributes\":"),
            Map(r.attributes),
            Raw(",\"component\":"),
            Str(r.component),
            Raw(",\"level\":"),
            Str(r.level.as_str()),
            Raw(",\"message\":"),
            Str(r.description),
            Raw(",\"session_id\":"),
            Num(r.session_id),
            Raw("}"),
        ],
    )
}

pub(super) fn ndjson(
    r: &Record<'_>,
    f: &mut dyn fmt::Write,
) -> fmt::Result {
    write_parts(
        f,
        &[
            Raw("{\"attributes\":"),
            Map(r.attributes),
            Raw(",\"component\":"),
            Str(r.component),
            Raw(",\"level\":"),
            Str(r.level.as_str()),
            Raw(",\"message\":"),
            Str(r.description),
            Raw(",\"timestamp\":"),
            Str(r.time),
            Raw("}"),
        ],
    )
}

pub(super) fn mcp(
    r: &Record<'_>,
    f: &mut dyn fmt::Write,
) -> fmt::Result {
    write_parts(
        f,
        &[
            Raw(
                "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/log\",\"params\":{\"data\":{\"attributes\":",
            ),
            Map(r.attributes),
            Raw(",\"component\":"),
            Str(r.component),
            Raw(",\"description\":"),
            Str(r.description),
            Raw(",\"session_id\":"),
            Num(r.session_id),
            Raw(",\"time\":"),
            Str(r.time),
            Raw("},\"level\":"),
            Str(r.level.as_str_lowercase()),
            Raw("}}"),
        ],
    )
}

pub(super) fn otlp(
    r: &Record<'_>,
    f: &mut dyn fmt::Write,
) -> fmt::Result {
    let empty = serde_json::Value::String(String::new());
    let trace_id = r.attributes.get("trace_id").unwrap_or(&empty);
    let span_id = r.attributes.get("span_id").unwrap_or(&empty);
    write_parts(
        f,
        &[
            Raw("{\"attributes\":"),
            Map(r.attributes),
            Raw(",\"body\":{\"stringValue\":"),
            Str(r.description),
            Raw("},\"severityNumber\":"),
            Num(u64::from(r.level.to_numeric())),
            Raw(",\"severityText\":"),
            Str(r.level.as_str()),
            Raw(",\"spanId\":"),
            Value(span_id),
            Raw(",\"timeUnixNano\":"),
            Str(r.time),
            Raw(",\"traceId\":"),
            Value(trace_id),
            Raw("}"),
        ],
    )
}

pub(super) fn ecs(
    r: &Record<'_>,
    f: &mut dyn fmt::Write,
) -> fmt::Result {
    write_parts(
        f,
        &[
            Raw("{\"@timestamp\":"),
            Str(r.time),
            Raw(",\"labels\":"),
            Map(r.attributes),
            Raw(",\"log.level\":"),
            Str(r.level.as_str_lowercase()),
            Raw(",\"log.logger\":\"rlg\",\"message\":"),
            Str(r.description),
            Raw(",\"process.name\":"),
            Str(r.component),
            Raw("}"),
        ],
    )
}
