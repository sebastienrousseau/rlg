// tests.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Unit tests for `lib.rs`.

use super::*;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg_cli::Filter;
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

/// A server whose root is the system temp directory, where every
/// test file lives.
fn server() -> LogServer {
    LogServer::with_root(Root::new(std::env::temp_dir()).unwrap())
}

fn call(tool: &str, args: Value) -> CallToolResult {
    call_on(&server(), tool, args)
}

fn call_on(
    server: &LogServer,
    tool: &str,
    args: Value,
) -> CallToolResult {
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

// ---------------------------------------------------------------------------
// The log root and the read limits.
// ---------------------------------------------------------------------------

/// A root holding `app.ndjson` and `sub/`, and a file outside it.
fn rooted() -> (tempfile::TempDir, tempfile::TempDir, Root) {
    let inside = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(
        inside.path().join("app.ndjson"),
        format!("{INFO}\n{ERROR}\n"),
    )
    .unwrap();
    std::fs::create_dir(inside.path().join("sub")).unwrap();
    std::fs::write(
        outside.path().join("secret.ndjson"),
        format!("{FATAL}\n"),
    )
    .unwrap();
    let root = Root::new(inside.path()).unwrap();
    (inside, outside, root)
}

#[test]
fn the_root_refuses_anything_outside_it_with_one_message() {
    let (inside, outside, root) = rooted();
    assert!(root.resolve("app.ndjson").is_ok());
    let abs_inside = inside.path().join("app.ndjson");
    assert!(root.resolve(abs_inside.to_str().unwrap()).is_ok());
    let secret = outside.path().join("secret.ndjson");
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        &secret,
        inside.path().join("link.ndjson"),
    )
    .unwrap();
    let refused = [
        secret.to_str().unwrap().to_string(),
        "../secret.ndjson".to_string(),
        format!(
            "sub/../../{}/secret.ndjson",
            outside.path().file_name().unwrap().to_str().unwrap()
        ),
        "sub".to_string(),
        "missing.ndjson".to_string(),
        "/dev/zero".to_string(),
        #[cfg(unix)]
        "link.ndjson".to_string(),
    ];
    let messages: Vec<String> = refused
        .iter()
        .map(|p| root.resolve(p).unwrap_err())
        .collect();
    for (p, m) in refused.iter().zip(&messages) {
        assert!(
            m.contains("not a log file under the log root"),
            "{p}: {m}"
        );
        assert!(
            !m.contains("No such file") && !m.contains("denied"),
            "{p}: {m}"
        );
    }
}

#[test]
fn tools_read_only_under_the_root() {
    let (inside, outside, root) = rooted();
    let server = LogServer::with_root(root);
    let ok =
        call_on(&server, "tail_log", json!({"path": "app.ndjson"}));
    assert!(!is_error(&ok), "{}", text_of(&ok));
    let secret = outside.path().join("secret.ndjson");
    for tool in ["tail_log", "filter_log", "summarize_errors"] {
        let r = call_on(&server, tool, json!({"path": secret}));
        assert!(is_error(&r), "{tool}");
        assert!(
            !text_of(&r).contains("down"),
            "{tool}: {}",
            text_of(&r)
        );
    }
    let r = call_on(
        &server,
        "tail_logs_glob",
        json!({"glob_pattern": format!("{}/*", outside.path().display())}),
    );
    assert!(
        is_error(&r) && !text_of(&r).contains("down"),
        "{}",
        text_of(&r)
    );
    for walk_everything in ["/**/*", "../**/*"] {
        let r = call_on(
            &server,
            "tail_logs_glob",
            json!({"glob_pattern": walk_everything}),
        );
        assert!(is_error(&r), "{walk_everything}: {}", text_of(&r));
    }
    let r = call_on(
        &server,
        "tail_logs_glob",
        json!({"glob_pattern": "**/*.ndjson"}),
    );
    assert!(text_of(&r).contains("boom"), "{}", text_of(&r));
    drop(inside);
}

#[test]
fn a_glob_may_not_match_too_many_files() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..=MAX_GLOB_FILES {
        std::fs::write(dir.path().join(format!("{i}.ndjson")), "")
            .unwrap();
    }
    let server = LogServer::with_root(Root::new(dir.path()).unwrap());
    let r = call_on(
        &server,
        "tail_logs_glob",
        json!({"glob_pattern": "*.ndjson"}),
    );
    assert!(
        is_error(&r) && text_of(&r).contains("files"),
        "{}",
        text_of(&r)
    );
}

#[test]
fn reads_are_bounded() {
    let f = write_log(&format!(
        "{}\n{ERROR}\n",
        "x".repeat(MAX_LINE_BYTES + 1)
    ));
    let out = tail_log(f.path(), usize::MAX).unwrap();
    assert_eq!(out.len(), 1, "the oversized line is skipped");
    let many: String =
        std::iter::repeat_n(format!("{INFO}\n"), MAX_RECORDS + 5)
            .collect();
    let f = write_log(&many);
    assert_eq!(
        tail_log(f.path(), usize::MAX).unwrap().len(),
        MAX_RECORDS
    );
    let filter = Filter::new();
    assert_eq!(
        filter_log(f.path(), &filter, LogFormat::Logfmt)
            .unwrap()
            .len(),
        MAX_RECORDS
    );
}

#[test]
fn root_and_host_arguments_are_read() {
    let args = |a: &[&str]| {
        a.iter().map(|s| (*s).to_string()).collect::<Vec<_>>()
    };
    let (root, rest) =
        take_root_arg(args(&["--root", "/logs", "--port", "1"]))
            .unwrap();
    assert_eq!(root.as_deref(), Some("/logs"));
    assert_eq!(rest, args(&["--port", "1"]));
    let (root, _) = take_root_arg(args(&["--root=/a"])).unwrap();
    assert_eq!(root.as_deref(), Some("/a"));
    assert!(take_root_arg(args(&["--root"])).is_err());
    assert!(binds_beyond_loopback(&args(&["--host", "0.0.0.0"])));
    assert!(binds_beyond_loopback(&args(&["--host=10.0.0.5"])));
    assert!(!binds_beyond_loopback(&args(&["--host", "127.0.0.1"])));
    assert!(!binds_beyond_loopback(&args(&["--port", "9"])));
}
