// escape.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Escaping for the text formats, so that no field can end a record,
//! start a forged one, or smuggle a delimiter or terminal escape.
//!
//! Every mode escapes `\` and every control character (C0, DEL and
//! C1, which covers `\n`, `\r`, ESC and NEL): the short forms `\n`,
//! `\r` and `\t`, and `\u{XX}` for the rest. Each mode then adds the
//! delimiters its format gives meaning to. Clean runs are written as
//! slices, so a value with nothing to escape costs one `write_str`.

use core::fmt;

/// Which format's rules to apply.
#[derive(Clone, Copy)]
pub(super) enum Mode {
    /// A space-separated text field (CLF, Apache host and fields).
    Text,
    /// Inside `"..."` (Apache request, logfmt values): also `"`.
    Quoted,
    /// A `|`-delimited header field (CEF, ELF, W3C): also `|`.
    Pipe,
    /// XML text or attribute value (Log4j XML): entities.
    Xml,
}

/// `s`, escaped for `mode`, as a `Display` value.
pub(super) struct Escaped<'a>(pub(super) &'a str, pub(super) Mode);

impl fmt::Display for Escaped<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(s, mode) = *self;
        let mut start = 0;
        for (i, c) in s.char_indices() {
            let Some(esc) = escape(mode, c) else { continue };
            f.write_str(&s[start..i])?;
            esc.write(f)?;
            start = i + c.len_utf8();
        }
        f.write_str(&s[start..])
    }
}

/// How one character is written, when not as itself.
enum Esc {
    Lit(&'static str),
    Code(u32),
}

impl Esc {
    fn write(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lit(s) => f.write_str(s),
            Self::Code(n) => write!(f, "\\u{{{n:x}}}"),
        }
    }
}

fn escape(mode: Mode, c: char) -> Option<Esc> {
    match (mode, c) {
        (Mode::Xml, _) => xml(c),
        (_, '\\') => Some(Esc::Lit("\\\\")),
        (Mode::Quoted, '"') => Some(Esc::Lit("\\\"")),
        (Mode::Pipe, '|') => Some(Esc::Lit("\\|")),
        _ => control(c),
    }
}

fn control(c: char) -> Option<Esc> {
    match c {
        '\n' => Some(Esc::Lit("\\n")),
        '\r' => Some(Esc::Lit("\\r")),
        '\t' => Some(Esc::Lit("\\t")),
        c if c.is_control() => Some(Esc::Code(u32::from(c))),
        _ => None,
    }
}

/// XML 1.0: the five entities, character references for the three
/// whitespace controls XML allows, and `\u{XX}` text for the controls
/// it forbids.
fn xml(c: char) -> Option<Esc> {
    const XML: [(char, &str); 8] = [
        ('&', "&amp;"),
        ('<', "&lt;"),
        ('>', "&gt;"),
        ('"', "&quot;"),
        ('\'', "&apos;"),
        ('\n', "&#xA;"),
        ('\r', "&#xD;"),
        ('\t', "&#x9;"),
    ];
    XML.iter()
        .find(|(k, _)| *k == c)
        .map(|(_, v)| Esc::Lit(v))
        .or_else(|| c.is_control().then(|| Esc::Code(u32::from(c))))
}

/// A logfmt key: characters outside `[A-Za-z0-9_.-]` are written as
/// `%XX` per UTF-8 byte, so a key can never hold a space, `=` or a
/// line break.
pub(super) struct LogfmtKey<'a>(pub(super) &'a str);

impl fmt::Display for LogfmtKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.0;
        let mut start = 0;
        for (i, c) in s.char_indices() {
            if c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-')
            {
                continue;
            }
            f.write_str(&s[start..i])?;
            let mut buf = [0u8; 4];
            for b in c.encode_utf8(&mut buf).bytes() {
                write!(f, "%{b:02X}")?;
            }
            start = i + c.len_utf8();
        }
        f.write_str(&s[start..])
    }
}

/// Whether a logfmt value must be quoted: empty, or holding a space,
/// `=`, `"`, `\` or a control character.
pub(super) fn logfmt_needs_quotes(s: &str) -> bool {
    s.is_empty()
        || s.chars().any(|c| {
            matches!(c, ' ' | '=' | '"' | '\\') || c.is_control()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::{String, ToString};

    fn esc(s: &str, mode: Mode) -> String {
        Escaped(s, mode).to_string()
    }

    #[test]
    fn controls_and_backslash_are_escaped_in_every_mode() {
        for mode in [Mode::Text, Mode::Quoted, Mode::Pipe] {
            assert_eq!(
                esc("a\nb\r\tc\\d\x1b\u{85}\x7f", mode),
                r"a\nb\r\tc\\d\u{1b}\u{85}\u{7f}"
            );
        }
    }

    #[test]
    fn each_mode_adds_its_delimiters() {
        assert_eq!(esc(r#"a"b|c"#, Mode::Text), r#"a"b|c"#);
        assert_eq!(esc(r#"a"b|c"#, Mode::Quoted), r#"a\"b|c"#);
        assert_eq!(esc(r#"a"b|c"#, Mode::Pipe), r#"a"b\|c"#);
        assert_eq!(
            esc("<a href=\"x\">'&'\n\x01", Mode::Xml),
            "&lt;a href=&quot;x&quot;&gt;&apos;&amp;&apos;&#xA;\\u{1}"
        );
    }

    #[test]
    fn clean_text_is_untouched() {
        assert_eq!(
            esc("plain ünïcode text", Mode::Pipe),
            "plain ünïcode text"
        );
    }

    #[test]
    fn logfmt_keys_are_percent_encoded() {
        assert_eq!(LogfmtKey("ok_key.1-a").to_string(), "ok_key.1-a");
        assert_eq!(
            LogfmtKey("k\ney x=é").to_string(),
            "k%0Aey%20x%3D%C3%A9"
        );
    }

    #[test]
    fn logfmt_quoting_rule() {
        assert!(logfmt_needs_quotes(""));
        assert!(logfmt_needs_quotes("a b"));
        assert!(logfmt_needs_quotes("a=b"));
        assert!(logfmt_needs_quotes("a\\b"));
        assert!(logfmt_needs_quotes("a\nb"));
        assert!(!logfmt_needs_quotes("plain"));
    }
}
