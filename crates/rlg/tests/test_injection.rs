// test_injection.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! No input can forge a record: every format renders a record whose
//! fields carry line breaks, terminal escapes and delimiters as one
//! line with those characters escaped.

#![allow(missing_docs)]

use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;

const ALL: [LogFormat; 14] = [
    LogFormat::CLF,
    LogFormat::CEF,
    LogFormat::ELF,
    LogFormat::W3C,
    LogFormat::ApacheAccessLog,
    LogFormat::Log4jXML,
    LogFormat::JSON,
    LogFormat::GELF,
    LogFormat::Logstash,
    LogFormat::NDJSON,
    LogFormat::MCP,
    LogFormat::OTLP,
    LogFormat::Logfmt,
    LogFormat::ECS,
];

fn hostile(format: LogFormat) -> String {
    let log = Log::build(
        LogLevel::INFO,
        "ok\nlevel=error msg=\"forged\"\r\x1b[2J\u{85}done",
    )
    .component("svc|x=1<b>\n")
    .time("2026-11-27T00:00:00Z\nInjected: yes")
    .with("k\ney", "v\nw")
    .with("quote", "a\"b\\c")
    .format(format);
    log.to_string()
}

#[test]
fn no_format_lets_a_field_break_the_line() {
    for format in ALL {
        let out = hostile(format);
        for bad in ['\n', '\r', '\x1b', '\u{85}'] {
            assert!(
                !out.contains(bad),
                "{format:?} leaks {bad:?}: {out}"
            );
        }
    }
}

#[test]
fn cef_escapes_its_delimiters() {
    let out = hostile(LogFormat::CEF);
    // `CEF:0|a|b|c|d|e|CEF`: six separators, so exactly six
    // unescaped pipes; the one inside the component is escaped.
    let unescaped = out
        .match_indices('|')
        .filter(|(i, _)| !out[..*i].ends_with('\\'))
        .count();
    assert_eq!(unescaped, 6, "{out}");
    assert!(out.contains(r"svc\|x=1"), "{out}");
}

#[test]
fn pipe_formats_escape_pipes() {
    for format in [LogFormat::ELF, LogFormat::W3C] {
        let out = hostile(format);
        assert!(out.contains(r"svc\|x=1"), "{format:?}: {out}");
    }
}

#[test]
fn log4j_xml_escapes_markup() {
    let out = hostile(LogFormat::Log4jXML);
    assert!(out.contains("svc|x=1&lt;b&gt;"), "{out}");
    assert!(!out.contains("<b>"), "{out}");
    assert_eq!(out.matches("<log4j:message>").count(), 1, "{out}");
}

#[test]
fn logfmt_quotes_and_escapes_values_and_keys() {
    let out = hostile(LogFormat::Logfmt);
    assert!(out.contains(r#"msg="ok\nlevel=error msg=\"forged\"\r\u{1b}[2J\u{85}done""#), "{out}");
    assert!(out.contains(r#"quote="a\"b\\c""#), "{out}");
    // A key with a line break is percent-encoded, not split.
    assert!(out.contains("k%0Aey=\"v\\nw\""), "{out}");
    // Exactly one `level=` pair: the forged one sits inside msg's quotes.
    assert_eq!(
        out.matches(" level=").count()
            + usize::from(out.starts_with("level=")),
        1,
        "{out}"
    );
}

#[test]
fn apache_escapes_the_quoted_request() {
    let out = hostile(LogFormat::ApacheAccessLog);
    assert!(out.contains(r#"\"forged\""#), "{out}");
}

#[test]
fn clean_values_render_as_before() {
    let log = Log::build(LogLevel::INFO, "plain message")
        .component("svc")
        .time("2026-11-27T00:00:00Z")
        .with("k", "v");
    let clf = log.clone().format(LogFormat::CLF).to_string();
    assert!(
        clf.contains(
            "Description=plain message Level=INFO Component=svc"
        ),
        "{clf}"
    );
    let logfmt = log.format(LogFormat::Logfmt).to_string();
    assert!(
        logfmt.contains(r#"msg="plain message""#)
            && logfmt.contains(" k=v"),
        "{logfmt}"
    );
}

/// Found by the `render_one_line` fuzz target: `serde_json` writes DEL
/// and C1 controls raw inside attribute strings.
#[test]
fn json_attribute_values_escape_del_and_c1() {
    for format in [LogFormat::JSON, LogFormat::MCP, LogFormat::ECS] {
        let out = Log::build(LogLevel::INFO, "x")
            .with("k", "a\u{7f}b\u{85}c")
            .with("list", vec!["\u{7f}"])
            .format(format)
            .to_string();
        assert!(
            !out.chars().any(char::is_control),
            "{format:?}: {out:?}"
        );
        assert!(out.contains("\\u007f"), "{format:?}: {out}");
        let parsed: serde_json::Value =
            serde_json::from_str(&out).unwrap();
        assert!(parsed.is_object());
    }
}
