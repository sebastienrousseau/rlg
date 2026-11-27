// write.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The hand-written JSON writers behind `Log`'s `Display`.
//! They write straight to the formatter, with no intermediate
//! `serde_json::Value`, because formatting runs on the flusher thread
//! for every record.

use std::collections::BTreeMap;
use std::fmt;

/// Writes a JSON-escaped string (with surrounding quotes) to the formatter.
fn write_json_str(f: &mut fmt::Formatter<'_>, s: &str) -> fmt::Result {
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
    fn write(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
pub(super) enum Part<'a> {
    /// Written as-is: keys, punctuation, constant values.
    Raw(&'a str),
    /// A JSON string, quoted and escaped.
    Str(&'a str),
    /// An attribute map, as a JSON object.
    Map(&'a BTreeMap<String, serde_json::Value>),
    /// An unsigned number.
    Num(u64),
    /// A JSON value, as `serde_json` renders it.
    Value(&'a serde_json::Value),
}
impl Part<'_> {
    fn write(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
pub(super) fn write_parts(
    f: &mut fmt::Formatter<'_>,
    parts: &[Part<'_>],
) -> fmt::Result {
    parts.iter().try_for_each(|part| part.write(f))
}

/// Writes a `BTreeMap<String, serde_json::Value>` as a JSON object.
fn write_json_map(
    f: &mut fmt::Formatter<'_>,
    map: &BTreeMap<String, serde_json::Value>,
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
    f: &mut fmt::Formatter<'_>,
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
    f: &mut fmt::Formatter<'_>,
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
    f: &mut fmt::Formatter<'_>,
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
