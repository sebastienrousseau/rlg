// text.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The line-oriented text formats. Every field that comes from the
//! caller goes through [`Escaped`] or the logfmt rules, so a record
//! is always one line and no field can forge another.

use super::escape::{Escaped, LogfmtKey, Mode, logfmt_needs_quotes};
use super::{CACHED_HOSTNAME, Log};
use std::fmt;

impl Log {
    pub(super) fn fmt_clf(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(
            f,
            "SessionID={} Timestamp={} Description={} Level={} Component={}",
            self.session_id,
            Escaped(&self.time, Mode::Text),
            Escaped(&self.description, Mode::Text),
            self.level,
            Escaped(&self.component, Mode::Text),
        )
    }

    /// The shared shape of the `|`-delimited formats.
    fn fmt_piped(
        &self,
        f: &mut fmt::Formatter<'_>,
        tag: &str,
    ) -> fmt::Result {
        write!(
            f,
            "{tag}:0|{}|{}|{}|{}|{}|{tag}",
            self.session_id,
            Escaped(&self.time, Mode::Pipe),
            self.level,
            Escaped(&self.component, Mode::Pipe),
            Escaped(&self.description, Mode::Pipe),
        )
    }

    pub(super) fn fmt_cef(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.fmt_piped(f, "CEF")
    }

    pub(super) fn fmt_elf(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.fmt_piped(f, "ELF")
    }

    pub(super) fn fmt_w3c(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.fmt_piped(f, "W3C")
    }

    pub(super) fn fmt_apache(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(
            f,
            "{} - - [{}] \"{}\" {} {}",
            Escaped(&CACHED_HOSTNAME, Mode::Text),
            Escaped(&self.time, Mode::Text),
            Escaped(&self.description, Mode::Quoted),
            self.level,
            Escaped(&self.component, Mode::Text),
        )
    }

    pub(super) fn fmt_log4j_xml(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(
            f,
            r#"<log4j:event logger="{}" timestamp="{}" level="{}" thread="{}"><log4j:message>{}</log4j:message></log4j:event>"#,
            Escaped(&self.component, Mode::Xml),
            Escaped(&self.time, Mode::Xml),
            self.level,
            self.session_id,
            Escaped(&self.description, Mode::Xml),
        )
    }

    pub(super) fn write_logfmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(
            f,
            "level={} msg=\"{}\" session_id={} component=\"{}\"",
            self.level.as_str_lowercase(),
            Escaped(&self.description, Mode::Quoted),
            self.session_id,
            Escaped(&self.component, Mode::Quoted),
        )?;
        for (key, value) in &self.attributes {
            write!(f, " {}=", LogfmtKey(key))?;
            write_logfmt_value(f, value)?;
        }
        Ok(())
    }
}

/// A logfmt value: numbers, booleans and null bare; a string bare
/// when it needs no quoting; anything else (strings that need it,
/// arrays, objects) quoted and escaped.
fn write_logfmt_value(
    f: &mut fmt::Formatter<'_>,
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
