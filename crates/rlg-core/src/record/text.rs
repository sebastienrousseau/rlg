// text.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The line-oriented text formats. Every field that comes from the
//! caller goes through [`Escaped`] or the logfmt rules, so a record
//! is always one line and no field can forge another.

use super::escape::{Escaped, LogfmtKey, Mode, logfmt_needs_quotes};
use crate::Record;
use alloc::string::ToString;
use core::fmt;

pub(super) fn clf(
    r: &Record<'_>,
    f: &mut dyn fmt::Write,
) -> fmt::Result {
    write!(
        f,
        "SessionID={} Timestamp={} Description={} Level={} Component={}",
        r.session_id,
        Escaped(r.time, Mode::Text),
        Escaped(r.description, Mode::Text),
        r.level,
        Escaped(r.component, Mode::Text),
    )
}

/// The shared shape of the `|`-delimited formats.
fn piped(
    r: &Record<'_>,
    f: &mut dyn fmt::Write,
    tag: &str,
) -> fmt::Result {
    write!(
        f,
        "{tag}:0|{}|{}|{}|{}|{}|{tag}",
        r.session_id,
        Escaped(r.time, Mode::Pipe),
        r.level,
        Escaped(r.component, Mode::Pipe),
        Escaped(r.description, Mode::Pipe),
    )
}

pub(super) fn cef(
    r: &Record<'_>,
    f: &mut dyn fmt::Write,
) -> fmt::Result {
    piped(r, f, "CEF")
}

pub(super) fn elf(
    r: &Record<'_>,
    f: &mut dyn fmt::Write,
) -> fmt::Result {
    piped(r, f, "ELF")
}

pub(super) fn w3c(
    r: &Record<'_>,
    f: &mut dyn fmt::Write,
) -> fmt::Result {
    piped(r, f, "W3C")
}

pub(super) fn apache(
    r: &Record<'_>,
    f: &mut dyn fmt::Write,
) -> fmt::Result {
    write!(
        f,
        "{} - - [{}] \"{}\" {} {}",
        Escaped(r.host, Mode::Text),
        Escaped(r.time, Mode::Text),
        Escaped(r.description, Mode::Quoted),
        r.level,
        Escaped(r.component, Mode::Text),
    )
}

pub(super) fn log4j_xml(
    r: &Record<'_>,
    f: &mut dyn fmt::Write,
) -> fmt::Result {
    write!(
        f,
        r#"<log4j:event logger="{}" timestamp="{}" level="{}" thread="{}"><log4j:message>{}</log4j:message></log4j:event>"#,
        Escaped(r.component, Mode::Xml),
        Escaped(r.time, Mode::Xml),
        r.level,
        r.session_id,
        Escaped(r.description, Mode::Xml),
    )
}

pub(super) fn logfmt(
    r: &Record<'_>,
    f: &mut dyn fmt::Write,
) -> fmt::Result {
    write!(
        f,
        "level={} msg=\"{}\" session_id={} component=\"{}\"",
        r.level.as_str_lowercase(),
        Escaped(r.description, Mode::Quoted),
        r.session_id,
        Escaped(r.component, Mode::Quoted),
    )?;
    for (key, value) in r.attributes {
        write!(f, " {}=", LogfmtKey(key))?;
        write_logfmt_value(f, value)?;
    }
    Ok(())
}

/// A logfmt value: numbers, booleans and null bare; a string bare
/// when it needs no quoting; anything else (strings that need it,
/// arrays, objects) quoted and escaped.
fn write_logfmt_value(
    f: &mut dyn fmt::Write,
    value: &serde_json::Value,
) -> fmt::Result {
    use serde_json::Value;
    match value {
        Value::String(s) if !logfmt_needs_quotes(s) => f.write_str(s),
        Value::String(s) => {
            write!(f, "\"{}\"", Escaped(s, Mode::Quoted))
        }
        Value::Array(_) | Value::Object(_) => {
            write!(
                f,
                "\"{}\"",
                Escaped(&value.to_string(), Mode::Quoted)
            )
        }
        other => write!(f, "{other}"),
    }
}
