// tests.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Unit tests for `lib.rs`.

use super::*;
use serde_json::{Value, json};
use std::io::Write;

fn write_log(content: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f
}

const INFO: &str = r#"{"session_id":1,"time":"t","level":"INFO","component":"svc","description":"hi","format":"JSON","attributes":{}}"#;
const ERROR: &str = r#"{"session_id":2,"time":"t","level":"ERROR","component":"db","description":"boom","format":"JSON","attributes":{}}"#;
const FATAL: &str = r#"{"session_id":3,"time":"t","level":"FATAL","component":"db","description":"down","format":"JSON","attributes":{}}"#;

/// Call a tool the way a request reaches it: JSON arguments,
/// deserialised into the tool's parameter type.
fn parse<T: serde::de::DeserializeOwned>(v: Value) -> T {
    serde_json::from_value(v).expect("arguments")
}

fn call(tool: &str, args: Value) -> CallToolResult {
    let server = LogServer::new();
    let result = match tool {
        "tail_log" => server.tail_log_tool(Parameters(parse(args))),
        "filter_log" => server.filter_log_tool(Parameters(parse(args))),
        "summarize_errors" => {
            server.summarize_errors_tool(Parameters(parse(args)))
        }
        "tail_logs_glob" => {
            server.tail_logs_glob_tool(Parameters(parse(args)))
        }
        other => panic!("no such tool {other}"),
    };
    result.expect("a tool failure is a result, not a protocol error")
}

fn text_of(r: &CallToolResult) -> &str {
    r.content
        .first()
        .and_then(ContentBlock::as_text)
        .map(|t| t.text.as_str())
        .expect("text content")
}

fn is_error(r: &CallToolResult) -> bool {
    r.is_error == Some(true)
}

#[test]
fn tail_log_returns_last_n() {
    let f = write_log(&format!("{INFO}\n{ERROR}\n{FATAL}\n"));
    let out = tail_log(f.path(), 2).unwrap();
    assert_eq!(out.len(), 2);
    assert!(out[0].contains("boom"));
    assert!(out[1].contains("down"));
}

#[test]
fn tail_log_handles_short_files() {
    let f = write_log(&format!("{INFO}\n"));
    let out = tail_log(f.path(), 100).unwrap();
    assert_eq!(out.len(), 1);
}

#[test]
fn filter_log_drops_below_min_level() {
    let f = write_log(&format!("{INFO}\n{ERROR}\n{FATAL}\n"));
    let filter = Filter::new().min_level(LogLevel::ERROR);
    let out = filter_log(f.path(), &filter, LogFormat::Logfmt).unwrap();
    assert_eq!(out.len(), 2);
}

#[test]
fn summarize_errors_groups_by_component() {
    let f = write_log(&format!("{INFO}\n{ERROR}\n{FATAL}\n"));
    let buckets = summarize_errors(f.path()).unwrap();
    assert_eq!(buckets.get("db"), Some(&2));
    assert_eq!(buckets.get("svc"), None);
}

#[test]
fn tail_logs_glob_merges_matched_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.log"), format!("{INFO}\n"))
        .unwrap();
    std::fs::write(
        dir.path().join("b.log"),
        format!("{ERROR}\n{FATAL}\n"),
    )
    .unwrap();
    let pattern = format!("{}/*.log", dir.path().to_str().unwrap());
    let out = tail_logs_glob(&pattern, 100, None).unwrap();
    assert_eq!(out.len(), 3);
    let joined = out.join("\n");
    assert!(joined.contains("hi"));
    assert!(joined.contains("boom"));
    assert!(joined.contains("down"));
}

#[test]
fn tail_logs_glob_filters_by_level() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("x.log"),
        format!("{INFO}\n{ERROR}\n{FATAL}\n"),
    )
    .unwrap();
    let pattern = format!("{}/*.log", dir.path().to_str().unwrap());
    let out =
        tail_logs_glob(&pattern, 100, Some(LogLevel::ERROR)).unwrap();
    assert_eq!(out.len(), 2); // INFO dropped, ERROR + FATAL kept
}

#[test]
fn tail_logs_glob_respects_n() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("x.log"),
        format!("{INFO}\n{ERROR}\n{FATAL}\n"),
    )
    .unwrap();
    let pattern = format!("{}/*.log", dir.path().to_str().unwrap());
    let out = tail_logs_glob(&pattern, 1, None).unwrap();
    assert_eq!(out.len(), 1);
    assert!(out[0].contains("down"));
}

#[test]
fn tail_logs_glob_rejects_bad_pattern() {
    let err = tail_logs_glob("a/**b[", 10, None).unwrap_err();
    assert!(err.contains("invalid glob pattern"));
}

#[test]
fn tail_logs_glob_reports_unreadable_match() {
    // A pattern matching a directory: read_to_string fails on it.
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("sub")).unwrap();
    let pattern = format!("{}/sub", dir.path().to_str().unwrap());
    let err = tail_logs_glob(&pattern, 10, None).unwrap_err();
    assert!(err.contains("read"));
}

#[test]
fn every_tool_is_registered_with_schema_and_annotations() {
    let tools = LogServer::tool_router().list_all();
    let mut names: Vec<&str> =
        tools.iter().map(|t| t.name.as_ref()).collect();
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "filter_log",
            "summarize_errors",
            "tail_log",
            "tail_logs_glob"
        ]
    );
    for t in &tools {
        assert!(
            t.description.is_some(),
            "{} has no description",
            t.name
        );
        assert_eq!(
            t.input_schema.get("type").and_then(Value::as_str),
            Some("object")
        );
        let props =
            t.input_schema.get("properties").expect("properties");
        for (name, schema) in props.as_object().expect("object") {
            assert!(
                schema.get("examples").is_some(),
                "{}.{name} has no example: {schema}",
                t.name
            );
            assert!(
                schema.get("description").is_some(),
                "{}.{name} has no description",
                t.name
            );
        }
        assert!(
            t.output_schema.is_some(),
            "{} has no outputSchema",
            t.name
        );
        let a = t.annotations.as_ref().expect("annotations");
        assert_eq!(a.read_only_hint, Some(true), "{}", t.name);
        assert_eq!(a.destructive_hint, Some(false), "{}", t.name);
    }
}

#[test]
fn input_schemas_keep_the_field_names_defaults_and_enums() {
    let tools = LogServer::tool_router().list_all();
    let find = |name: &str| {
        tools.iter().find(|t| t.name == name).expect("tool").clone()
    };

    let tail = find("tail_log");
    let props = &tail.input_schema["properties"];
    assert_eq!(props["n"]["default"], 100, "{props}");
    assert_eq!(props["n"]["minimum"], 1, "{props}");
    assert_eq!(tail.input_schema["required"], json!(["path"]));

    let filter = find("filter_log");
    let props = &filter.input_schema["properties"];
    assert_eq!(props["format"]["default"], "Logfmt", "{props}");
    assert_eq!(props["min_level"]["type"], "string", "{props}");
    let levels = props["min_level"]["enum"].as_array().expect("enum");
    assert_eq!(levels.len(), 8, "{props}");
    assert!(levels.contains(&json!("ERROR")));
    assert_eq!(filter.input_schema["required"], json!(["path"]));

    let glob = find("tail_logs_glob");
    assert_eq!(glob.input_schema["required"], json!(["glob_pattern"]));
    assert_eq!(
        glob.input_schema["properties"]["lines"]["default"],
        100
    );
}

#[test]
fn tail_log_tool_returns_text_and_structured_content() {
    let f = write_log(&format!("{INFO}\n{ERROR}\n"));
    let r = call(
        "tail_log",
        json!({ "path": f.path().to_str().unwrap(), "n": 5 }),
    );
    assert!(!is_error(&r));
    assert!(text_of(&r).contains("hi"));
    assert!(text_of(&r).contains("boom"));
    let s = r.structured_content.expect("structured");
    assert_eq!(s["count"], 2);
    assert_eq!(s["records"].as_array().map(Vec::len), Some(2));
}

#[test]
fn n_defaults_to_one_hundred() {
    let f = write_log(&format!("{INFO}\n"));
    let r =
        call("tail_log", json!({ "path": f.path().to_str().unwrap() }));
    assert!(!is_error(&r));
    assert_eq!(r.structured_content.expect("structured")["count"], 1);
}

#[test]
fn an_empty_result_says_so() {
    let f = write_log("not a record\n");
    let r =
        call("tail_log", json!({ "path": f.path().to_str().unwrap() }));
    assert!(!is_error(&r));
    assert!(text_of(&r).contains("No parseable"), "{}", text_of(&r));
    assert_eq!(r.structured_content.expect("structured")["count"], 0);
}

#[test]
fn a_missing_file_is_a_result_the_model_can_read() {
    let r = call(
        "tail_log",
        json!({ "path": "/definitely/does/not/exist.ndjson" }),
    );
    assert!(is_error(&r));
    assert!(text_of(&r).contains("exist.ndjson"), "{}", text_of(&r));
    assert!(r.structured_content.is_none());
}

#[test]
fn filter_log_tool_applies_every_filter() {
    let f = write_log(&format!("{INFO}\n{ERROR}\n{FATAL}\n"));
    let r = call(
        "filter_log",
        json!({
            "path": f.path().to_str().unwrap(),
            "min_level": "ERROR",
            "component": "db",
            "format": "JSON"
        }),
    );
    assert!(!is_error(&r));
    let text = text_of(&r);
    assert!(text.contains("boom") && text.contains("down"), "{text}");
    assert!(!text.contains("\"hi\""), "{text}");
    assert_eq!(r.structured_content.expect("structured")["count"], 2);
}

#[test]
fn filter_log_tool_accepts_any_case_for_the_level() {
    let f = write_log(&format!("{INFO}\n{ERROR}\n"));
    let r = call(
        "filter_log",
        json!({ "path": f.path().to_str().unwrap(), "min_level": "error" }),
    );
    assert!(!is_error(&r), "{r:?}");
    assert_eq!(r.structured_content.expect("structured")["count"], 1);
}

#[test]
fn filter_log_tool_rejects_a_bad_level_and_a_bad_format() {
    let f = write_log(INFO);
    let path = f.path().to_str().unwrap();
    let r = call(
        "filter_log",
        json!({ "path": path, "min_level": "NOT_A_LEVEL" }),
    );
    assert!(is_error(&r));
    assert!(text_of(&r).contains("NOT_A_LEVEL"), "{}", text_of(&r));
    assert!(text_of(&r).contains("CRITICAL"), "{}", text_of(&r));

    let r = call(
        "filter_log",
        json!({ "path": path, "format": "NotAFormat" }),
    );
    assert!(is_error(&r));
    assert!(text_of(&r).contains("NotAFormat"), "{}", text_of(&r));
}

#[test]
fn summarize_errors_tool_counts_per_component() {
    let f = write_log(&format!("{INFO}\n{ERROR}\n{FATAL}\n"));
    let r = call(
        "summarize_errors",
        json!({ "path": f.path().to_str().unwrap() }),
    );
    assert!(!is_error(&r));
    assert!(text_of(&r).contains("\"db\": 2"), "{}", text_of(&r));
    assert_eq!(
        r.structured_content,
        Some(json!({ "total": 2, "by_component": { "db": 2 } }))
    );
}

#[test]
fn tail_logs_glob_tool_filters_and_reports_bad_input() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("a.log"),
        format!("{INFO}\n{ERROR}\n"),
    )
    .unwrap();
    let pattern = format!("{}/*.log", dir.path().to_str().unwrap());
    let r = call(
        "tail_logs_glob",
        json!({ "glob_pattern": pattern, "level": "ERROR" }),
    );
    assert!(!is_error(&r), "{r:?}");
    assert!(text_of(&r).contains("boom"));
    assert_eq!(r.structured_content.expect("structured")["count"], 1);

    let r = call(
        "tail_logs_glob",
        json!({ "glob_pattern": pattern, "level": "NOPE" }),
    );
    assert!(is_error(&r));

    let r = call("tail_logs_glob", json!({ "glob_pattern": "a/**b[" }));
    assert!(is_error(&r));
    assert!(text_of(&r).contains("invalid glob pattern"));
}

#[test]
fn the_prompt_embeds_its_arguments_or_stays_generic() {
    let text = triage_error_spike(Some("/var/log/app.log"), Some(15));
    assert!(text.contains("/var/log/app.log"));
    assert!(text.contains("15 minutes"));
    assert!(text.contains("summarize_errors"));

    let generic = triage_error_spike(None, None);
    assert!(generic.contains("the rlg log"));
    assert!(!generic.contains("minutes"));

    // An empty path is the same as none.
    assert!(triage_error_spike(Some(""), None).contains("the rlg log"));
}

#[test]
fn the_server_describes_itself() {
    let info = LogServer::default().get_info();
    assert_eq!(info.server_info.name, "rlg-mcp");
    assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));
    assert!(info.capabilities.tools.is_some());
    assert!(info.capabilities.prompts.is_some());
    assert!(info.capabilities.resources.is_some());
    assert!(info.instructions.is_some());
}
