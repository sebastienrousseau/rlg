// render_one_line.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Fuzz target: `Display for rlg::log::Log`, in every format.
//
// Contract under fuzz: whatever the description, component, time and
// attributes hold, every format renders the record as one line with
// no control character (no line break, carriage return, ESC or NEL),
// so no field can end the record or forge another. No panic.

#![no_main]

use libfuzzer_sys::fuzz_target;
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;

const FORMATS: [LogFormat; 14] = [
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

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else { return };
    // Split the input into four fields so each can carry hostile text.
    let mut parts = s.splitn(4, '\u{1f}');
    let description = parts.next().unwrap_or_default();
    let component = parts.next().unwrap_or_default();
    let time = parts.next().unwrap_or_default();
    let attr = parts.next().unwrap_or_default();
    let base = Log::build(LogLevel::INFO, description)
        .component(component)
        .time(time)
        .with(attr, attr);
    for format in FORMATS {
        let out = base.clone().format(format).to_string();
        assert!(
            !out.chars().any(char::is_control),
            "{format:?} rendered a control character: {out:?}"
        );
    }
});
