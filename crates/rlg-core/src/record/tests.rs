// tests.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Unit tests for `record.rs`: every format's exact bytes, and
//! rendering into a fixed-size, allocation-free buffer.

use super::*;
use alloc::string::ToString;
use core::fmt::Write;

fn attributes() -> Attributes {
    let mut map = Attributes::new();
    map.insert("n".into(), serde_json::json!(1));
    map
}

fn render(format: Format, attrs: &Attributes) -> String {
    Record::new(LogLevel::WARN, "low \"bat\"")
        .session_id(7)
        .time("T")
        .component("fw")
        .host("h")
        .attributes(attrs)
        .format(format)
        .to_string()
}

const EXPECTED: [(Format, &str); 14] = [
    (
        Format::CLF,
        r#"SessionID=7 Timestamp=T Description=low "bat" Level=WARN Component=fw"#,
    ),
    (
        Format::JSON,
        r#"{"Attributes":{"n":1},"Component":"fw","Description":"low \"bat\"","Format":"JSON","Level":"WARN","SessionID":7,"Timestamp":"T"}"#,
    ),
    (Format::CEF, r#"CEF:0|7|T|WARN|fw|low "bat"|CEF"#),
    (Format::ELF, r#"ELF:0|7|T|WARN|fw|low "bat"|ELF"#),
    (Format::W3C, r#"W3C:0|7|T|WARN|fw|low "bat"|W3C"#),
    (
        Format::GELF,
        r#"{"_attributes":{"n":1},"_session_id":7,"full_message":"low \"bat\"","host":"fw","level":7,"short_message":"low \"bat\"","timestamp":"T","version":"1.1"}"#,
    ),
    (
        Format::ApacheAccessLog,
        r#"h - - [T] "low \"bat\"" WARN fw"#,
    ),
    (
        Format::Logstash,
        r#"{"@timestamp":"T","attributes":{"n":1},"component":"fw","level":"WARN","message":"low \"bat\"","session_id":7}"#,
    ),
    (
        Format::Log4jXML,
        r#"<log4j:event logger="fw" timestamp="T" level="WARN" thread="7"><log4j:message>low &quot;bat&quot;</log4j:message></log4j:event>"#,
    ),
    (
        Format::NDJSON,
        r#"{"attributes":{"n":1},"component":"fw","level":"WARN","message":"low \"bat\"","timestamp":"T"}"#,
    ),
    (
        Format::MCP,
        r#"{"jsonrpc":"2.0","method":"notifications/log","params":{"data":{"attributes":{"n":1},"component":"fw","description":"low \"bat\"","session_id":7,"time":"T"},"level":"warn"}}"#,
    ),
    (
        Format::OTLP,
        r#"{"attributes":{"n":1},"body":{"stringValue":"low \"bat\""},"severityNumber":7,"severityText":"WARN","spanId":"","timeUnixNano":"T","traceId":""}"#,
    ),
    (
        Format::Logfmt,
        r#"level=warn msg="low \"bat\"" session_id=7 component="fw" n=1"#,
    ),
    (
        Format::ECS,
        r#"{"@timestamp":"T","labels":{"n":1},"log.level":"warn","log.logger":"rlg","message":"low \"bat\"","process.name":"fw"}"#,
    ),
];

#[test]
fn every_format_writes_its_exact_bytes() {
    let attrs = attributes();
    for (format, want) in EXPECTED {
        assert_eq!(render(format, &attrs), want, "{format:?}");
    }
}

#[test]
fn nested_attribute_values_are_escaped() {
    let mut attrs = Attributes::new();
    attrs.insert(
        "k ey".into(),
        serde_json::json!({"a": ["x\u{7f}", true, null]}),
    );
    assert_eq!(
        render(Format::NDJSON, &attrs),
        r#"{"attributes":{"k ey":{"a":["x\u007f",true,null]}},"component":"fw","level":"WARN","message":"low \"bat\"","timestamp":"T"}"#
    );
    assert!(
        render(Format::Logfmt, &attrs)
            .ends_with(r#" k%20ey="{\"a\":[\"x\u{7f}\",true,null]}""#)
    );
}

#[test]
fn otlp_lifts_trace_context() {
    let mut attrs = Attributes::new();
    attrs.insert("trace_id".into(), serde_json::json!("abc"));
    attrs.insert("span_id".into(), serde_json::json!("def"));
    let out = render(Format::OTLP, &attrs);
    assert!(out.contains(r#""spanId":"def""#));
    assert!(out.ends_with(r#""traceId":"abc"}"#));
}

#[test]
fn new_uses_the_log_build_defaults() {
    let r = Record::new(LogLevel::INFO, "x");
    assert_eq!(
        (r.session_id, r.time, r.component, r.format, r.host),
        (0, "", "default", Format::MCP, "")
    );
    assert!(r.attributes.is_empty());
}

/// A fixed-capacity `fmt::Write` sink: what a `no_std` target with no
/// allocator in the hot path would use.
struct FixedBuf<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> FixedBuf<N> {
    const fn new() -> Self {
        Self {
            buf: [0; N],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap()
    }
}

impl<const N: usize> Write for FixedBuf<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len + s.len();
        if end > N {
            return Err(fmt::Error);
        }
        self.buf[self.len..end].copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

#[test]
fn writes_into_a_fixed_buffer() {
    let record = Record::new(LogLevel::ERROR, "disk full")
        .session_id(3)
        .component("fs")
        .format(Format::CEF);
    let mut out = FixedBuf::<64>::new();
    record.write_to(&mut out).unwrap();
    assert_eq!(out.as_str(), "CEF:0|3||ERROR|fs|disk full|CEF");

    let mut tiny = FixedBuf::<8>::new();
    assert!(record.write_to(&mut tiny).is_err());
}
