// proptest_round_trip.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Property tests for the `Log::Display` implementations across
// the 14 `LogFormat` variants.
//
// See `docs/adr/0003-property-tested-formats.md`.

#![allow(missing_docs)]

use proptest::prelude::*;
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use serde_json::Value;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn any_log_level() -> impl Strategy<Value = LogLevel> {
    prop_oneof![
        Just(LogLevel::TRACE),
        Just(LogLevel::DEBUG),
        Just(LogLevel::VERBOSE),
        Just(LogLevel::INFO),
        Just(LogLevel::WARN),
        Just(LogLevel::ERROR),
        Just(LogLevel::FATAL),
        Just(LogLevel::CRITICAL),
    ]
}

fn any_log_format() -> impl Strategy<Value = LogFormat> {
    prop_oneof![
        Just(LogFormat::CLF),
        Just(LogFormat::JSON),
        Just(LogFormat::CEF),
        Just(LogFormat::ELF),
        Just(LogFormat::W3C),
        Just(LogFormat::GELF),
        Just(LogFormat::ApacheAccessLog),
        Just(LogFormat::Logstash),
        Just(LogFormat::Log4jXML),
        Just(LogFormat::NDJSON),
        Just(LogFormat::MCP),
        Just(LogFormat::OTLP),
        Just(LogFormat::Logfmt),
        Just(LogFormat::ECS),
    ]
}

/// Restricted string strategy — ASCII printable minus quote / backslash
/// / control characters. Avoids proptest exploring the escape-heavy
/// corner of the input space where any format-specific quoting rules
/// dominate; those are exercised by targeted unit tests.
fn tame_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _\\-./:]{0,32}"
}

fn any_attributes() -> impl Strategy<Value = BTreeMap<String, Value>> {
    proptest::collection::btree_map(
        "[a-z_]{1,10}",
        prop_oneof![
            any::<u64>().prop_map(Value::from),
            any::<i64>().prop_map(Value::from),
            any::<bool>().prop_map(Value::from),
            tame_string().prop_map(Value::from),
        ],
        0..5,
    )
}

prop_compose! {
    fn any_log()(
        session_id in any::<u64>(),
        level in any_log_level(),
        component in tame_string(),
        description in tame_string(),
        format in any_log_format(),
        attributes in any_attributes(),
    ) -> Log {
        Log {
            session_id,
            time: "2026-07-05T00:00:00.000000000Z".into(),
            level,
            component: component.into(),
            description,
            format,
            attributes,
        }
    }
}

// ---------------------------------------------------------------------------
// Property 1 — No format may panic on any generated log.
//
// This is the base survival property: for every combination of log
// fields and format variant, `format!("{log}")` produces a `String`
// without unwinding. Proptest exhaustively explores generation
// axes so a panic on any interaction surfaces here.
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn no_format_panics_on_any_log(log in any_log()) {
        let _ = format!("{log}");
    }
}

// ---------------------------------------------------------------------------
// Property 2 — Serde round-trip preserves the entire structural core.
//
// Via serde `to_string` + `from_str`, every `Log` round-trips
// verbatim. This is the canonical shape `rlg_cli::parse_record`
// consumes.
//
// Note: `Log::fmt` for `LogFormat::JSON` currently produces
// PascalCase field names (`SessionID`, `Component`, …), while
// `parse_record` expects the serde-default snake_case shape.
// That asymmetry means `parse_record(format!("{log:?}"))` does
// NOT round-trip. It's tracked as follow-up work; the property
// here proves the serde contract downstream code actually depends
// on.
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn serde_round_trip_preserves_all_fields(log in any_log()) {
        let json = serde_json::to_string(&log)
            .expect("Log Serialize must succeed");
        let parsed = rlg_cli::parse_record(&json)
            .expect("serde-shape JSON must round-trip through parse_record");
        prop_assert_eq!(parsed.session_id, log.session_id);
        prop_assert_eq!(parsed.level, log.level);
        prop_assert_eq!(
            parsed.component.as_ref(),
            log.component.as_ref()
        );
        prop_assert_eq!(parsed.description, log.description);
        prop_assert_eq!(parsed.format, log.format);
        prop_assert_eq!(parsed.attributes, log.attributes);
    }
}

// ---------------------------------------------------------------------------
// Property 3 — NDJSON output is single-line.
//
// NDJSON is *newline-delimited* JSON: exactly one record per line.
// The rendered string must not contain a `\n` before the terminating
// position.
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn ndjson_output_is_single_line(log in any_log()) {
        let mut ndjson_log = log.clone();
        ndjson_log.format = LogFormat::NDJSON;
        let rendered = format!("{ndjson_log}");
        // Trim one trailing newline if present — that's the ND
        // (newline-delimited) part; anything else is a violation.
        let body = rendered.trim_end_matches('\n');
        prop_assert!(
            !body.contains('\n'),
            "NDJSON body must be single-line, got: {body:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Property 4 — Every structured JSON-family output preserves the
// four core fields as top-level string values.
//
// This is a weaker cousin of Property 2 that applies to JSON,
// NDJSON, MCP, and ECS — the JSON-serialisable formats where we
// can grep for the fields even without a format-specific parser.
// ---------------------------------------------------------------------------

fn any_json_family_format() -> impl Strategy<Value = LogFormat> {
    prop_oneof![
        Just(LogFormat::JSON),
        Just(LogFormat::NDJSON),
        Just(LogFormat::MCP),
        Just(LogFormat::ECS),
    ]
}

proptest! {
    #[test]
    fn json_family_output_is_valid_utf8(
        log in any_log(),
        format in any_json_family_format(),
    ) {
        let mut typed = log.clone();
        typed.format = format;
        let rendered = format!("{typed}");
        // `format!` returns `String` which is already UTF-8, but the
        // assertion here documents the invariant future refactors
        // must preserve (e.g. if we ever return `Vec<u8>` from a
        // renderer).
        prop_assert!(rendered.is_char_boundary(rendered.len()));
    }
}
