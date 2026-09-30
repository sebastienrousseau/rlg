// lib.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `rlg-mcp` — a Model Context Protocol server for rlg log files.
//!
//! Four tools over log files on the server's filesystem — `tail_log`,
//! `filter_log`, `summarize_errors`, `tail_logs_glob` — plus one
//! prompt and two resources. The protocol is handled by [`rmcp`], the
//! official MCP SDK; this crate supplies the tools and the text a
//! model reads.
//!
//! The four operations are plain functions — [`tail_log`],
//! [`filter_log`], [`summarize_errors`], [`tail_logs_glob`] — so they
//! can be called and tested without a transport. [`LogServer`] is the
//! handler that exposes them as MCP tools; each answer is returned
//! twice, as text for the model and as a structured value for a client
//! that wants to read it without parsing prose.
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//! let recent = rlg_mcp::tail_log(Path::new("/var/log/app.ndjson"), 10).unwrap();
//! for line in recent { println!("{line}"); }
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

// The same file in every Rust MCP server of the suite, so it keeps
// its own formatting. Declared here rather than in `main.rs` so the
// transports can be served, and tested, in-process.
#[rustfmt::skip]
pub mod transport;

use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg_cli::{Filter, parse_record, render};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::{ToolCallContext, schema_for_output};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult,
    ContentBlock, ErrorData, GetPromptRequestParams, GetPromptResponse,
    GetPromptResult, Implementation, ListPromptsResult,
    ListResourceTemplatesResult, ListResourcesResult,
    PaginatedRequestParams, Prompt, PromptArgument, PromptMessage,
    ReadResourceRequestParams, ReadResourceResponse,
    ReadResourceResult, Resource, ResourceContents, ResourceTemplate,
    Role, ServerCapabilities, ServerConfig,
};
use rmcp::service::RequestContext;
use rmcp::{
    RoleServer, ServerHandler, tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

// ---------------------------------------------------------------------------
// Tool implementations — pure functions over a file path.
// ---------------------------------------------------------------------------

/// Return the last `n` parseable records from `path`, rendered in
/// `Logfmt`. Unparseable lines are skipped.
///
/// # Errors
/// Returns `io::Error` if the file cannot be opened or read.
pub fn tail_log(path: &Path, n: usize) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let mut all = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if let Ok(record) = parse_record(&line) {
            all.push(render(record, LogFormat::Logfmt));
        }
    }
    let start = all.len().saturating_sub(n);
    Ok(all.split_off(start))
}

/// Apply `filter` to every record in `path` and return matches
/// rendered in `format`.
///
/// # Errors
/// Returns `io::Error` if the file cannot be opened or read.
pub fn filter_log(
    path: &Path,
    filter: &Filter,
    format: LogFormat,
) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let mut out = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if let Ok(record) = parse_record(&line)
            && filter.matches(&record)
        {
            out.push(render(record, format));
        }
    }
    Ok(out)
}

/// Count error+ records grouped by component.
///
/// # Errors
/// Returns `io::Error` if the file cannot be opened or read.
pub fn summarize_errors(
    path: &Path,
) -> std::io::Result<BTreeMap<String, u64>> {
    let file = File::open(path)?;
    let mut buckets: BTreeMap<String, u64> = BTreeMap::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if let Ok(record) = parse_record(&line)
            && record.level.to_numeric() >= LogLevel::ERROR.to_numeric()
        {
            *buckets
                .entry(record.component.to_string())
                .or_insert(0) += 1;
        }
    }
    Ok(buckets)
}

/// Tail the last `n` records across every file matching a glob
/// pattern (e.g. `/var/log/**/*.log`), newest last, optionally keeping
/// only records at or above `level`. Matched files are read in sorted
/// path order and their records concatenated before the final `n` are
/// taken; unparseable lines are skipped.
///
/// # Errors
/// Returns an error string if the glob pattern is invalid, or if a
/// matched path cannot be read (e.g. it resolves to a directory).
pub fn tail_logs_glob(
    pattern: &str,
    n: usize,
    level: Option<LogLevel>,
) -> Result<Vec<String>, String> {
    let mut paths: Vec<std::path::PathBuf> = glob::glob(pattern)
        .map_err(|e| format!("invalid glob pattern: {e}"))?
        .filter_map(Result::ok)
        .collect();
    paths.sort();
    let mut all = Vec::new();
    for path in paths {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("read {}: {e}", path.display()))?;
        for line in content.lines() {
            if let Ok(record) = parse_record(line) {
                if let Some(min) = level
                    && record.level.to_numeric() < min.to_numeric()
                {
                    continue;
                }
                all.push(render(record, LogFormat::Logfmt));
            }
        }
    }
    let start = all.len().saturating_sub(n);
    Ok(all.split_off(start))
}

// ---------------------------------------------------------------------------
// Structured outputs.
// ---------------------------------------------------------------------------

/// A slice of rendered log records: what `tail_log`, `filter_log` and
/// `tail_logs_glob` return.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Records {
    /// How many records are returned.
    pub count: usize,
    /// The records, oldest first, one rendered record each.
    pub records: Vec<String>,
}

impl From<Vec<String>> for Records {
    fn from(records: Vec<String>) -> Self {
        Self {
            count: records.len(),
            records,
        }
    }
}

impl fmt::Display for Records {
    /// One record per line. When nothing matched, say so: an empty
    /// string would read to a model as a file with nothing in it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.records.is_empty() {
            return f.write_str("No parseable rlg records matched.");
        }
        f.write_str(&self.records.join("\n"))
    }
}

/// ERROR-and-above records counted per component: what
/// `summarize_errors` returns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ErrorSummary {
    /// How many ERROR-and-above records the file holds.
    pub total: u64,
    /// The count per component, sorted by component name.
    pub by_component: BTreeMap<String, u64>,
}

impl From<BTreeMap<String, u64>> for ErrorSummary {
    fn from(by_component: BTreeMap<String, u64>) -> Self {
        Self {
            total: by_component.values().sum(),
            by_component,
        }
    }
}

impl fmt::Display for ErrorSummary {
    /// The `component → count` map as pretty-printed JSON, which is
    /// the text this tool has always returned.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = serde_json::to_string_pretty(&self.by_component)
            .unwrap_or_default();
        f.write_str(&text)
    }
}

// ---------------------------------------------------------------------------
// Tool arguments.
// ---------------------------------------------------------------------------
//
// The doc comments on the fields are the descriptions a client shows
// the model, kept word for word from the previous release. The
// examples are what an auditor or a client with no log of its own
// sends: a path a log file can be written to on any Unix host.

fn default_count() -> usize {
    100
}

fn default_format() -> String {
    "Logfmt".to_owned()
}

/// Arguments of `tail_log`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TailLogArgs {
    /// Filesystem path to an rlg log file (Logfmt/JSON records, one per line).
    #[schemars(example = &"/tmp/rlg/app.ndjson")]
    pub path: String,
    /// How many of the most recent parseable records to return (default 100).
    #[serde(default = "default_count")]
    #[schemars(range(min = 1), example = &10)]
    pub n: usize,
}

/// Arguments of `filter_log`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FilterLogArgs {
    /// Filesystem path to an rlg log file to read.
    #[schemars(example = &"/tmp/rlg/app.ndjson")]
    pub path: String,
    /// Keep only records at or above this severity. Omit to keep all levels.
    #[serde(default)]
    #[schemars(
        with = "String",
        extend("enum" = ["TRACE", "DEBUG", "VERBOSE", "INFO", "WARN", "ERROR", "FATAL", "CRITICAL"]),
        example = &"WARN"
    )]
    pub min_level: Option<String>,
    /// Keep only records whose component matches this exact value. Omit to keep all components.
    #[serde(default)]
    #[schemars(with = "String", example = &"db")]
    pub component: Option<String>,
    /// rlg LogFormat name to render matched records in (e.g. Logfmt, JSON). Defaults to Logfmt.
    #[serde(default = "default_format")]
    #[schemars(example = &"JSON")]
    pub format: String,
}

/// Arguments of `summarize_errors`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SummarizeErrorsArgs {
    /// Filesystem path to an rlg log file to scan for ERROR-and-above records.
    #[schemars(example = &"/tmp/rlg/app.ndjson")]
    pub path: String,
}

/// Arguments of `tail_logs_glob`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TailLogsGlobArgs {
    /// Glob pattern matching one or more rlg log files, e.g. `/var/log/**/*.log`.
    #[schemars(example = &"/tmp/rlg/*.ndjson")]
    pub glob_pattern: String,
    /// How many of the most recent parseable records to return across all matched files (default 100).
    #[serde(default = "default_count")]
    #[schemars(range(min = 1), example = &10)]
    pub lines: usize,
    /// Keep only records at or above this severity. Omit to keep all levels.
    #[serde(default)]
    #[schemars(
        with = "String",
        extend("enum" = ["TRACE", "DEBUG", "VERBOSE", "INFO", "WARN", "ERROR", "FATAL", "CRITICAL"]),
        example = &"ERROR"
    )]
    pub level: Option<String>,
}

/// A failure to read `path`, worded for a model: the path is what
/// makes the message actionable.
fn read_error(path: &str, e: &std::io::Error) -> String {
    format!("Cannot read `{path}`: {e}")
}

fn parse_level(name: &str) -> Result<LogLevel, String> {
    name.parse::<LogLevel>().map_err(|e| {
        format!(
            "{e}. The levels are TRACE, DEBUG, VERBOSE, INFO, WARN, \
             ERROR, FATAL and CRITICAL."
        )
    })
}

/// A tool result carrying the same answer twice: as text for the
/// model and as a structured value for the client.
///
/// A failure keeps the text only. The structured schema describes a
/// result, and an error is not one.
fn reply<T: Serialize + fmt::Display>(
    outcome: Result<T, String>,
) -> Result<CallToolResult, ErrorData> {
    match outcome {
        Ok(value) => {
            let structured =
                serde_json::to_value(&value).map_err(|e| {
                    ErrorData::internal_error(e.to_string(), None)
                })?;
            let mut result =
                CallToolResult::success(vec![ContentBlock::text(
                    value.to_string(),
                )]);
            result.structured_content = Some(structured);
            Ok(result)
        }
        // A tool that ran and could not do the job: a *successful*
        // JSON-RPC response carrying `isError`, so the model sees the
        // text and can react to it. A JSON-RPC error would be handled
        // by the client and never shown.
        Err(message) => {
            Ok(CallToolResult::error(vec![ContentBlock::text(message)]))
        }
    }
}

// ---------------------------------------------------------------------------
// The server.
// ---------------------------------------------------------------------------

/// The MCP server: the four tools, one prompt and two resources over
/// [`rmcp`].
///
/// Cheap to create and to clone; the HTTP transports create one per
/// session. It holds nothing between calls.
#[derive(Debug, Clone)]
pub struct LogServer {
    tool_router: ToolRouter<Self>,
}

impl Default for LogServer {
    fn default() -> Self {
        Self::new()
    }
}

// Every tool here only reads a caller-supplied log file from disk:
// read-only, idempotent, never destructive, and open-world (it touches
// the local filesystem). These MCP annotations let clients reason
// about safety without executing the tool.
#[tool_router]
#[allow(
    clippy::unused_self,
    reason = "the SDK's tool router calls tools as methods"
)]
impl LogServer {
    /// A server with all four tools registered.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        name = "tail_log",
        description = "Return the last N parseable rlg (RustLogs) records from a log file, newest last. Use this to glance at the most recent activity in a log; use `filter_log` when you need to select records by level or component, and `summarize_errors` for an aggregated error count.",
        annotations(
            title = "Tail rlg log file",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        ),
        output_schema = schema_for_output::<Records>()
    )]
    fn tail_log_tool(
        &self,
        Parameters(args): Parameters<TailLogArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        reply(
            tail_log(Path::new(&args.path), args.n)
                .map(Records::from)
                .map_err(|e| read_error(&args.path, &e)),
        )
    }

    #[tool(
        name = "filter_log",
        description = "Select rlg records by minimum severity and/or component and render them in any rlg LogFormat. Use this to narrow a log to what matters (e.g. WARN-and-above for one service); use `tail_log` for a raw recent slice and `summarize_errors` when you only need per-component error totals.",
        annotations(
            title = "Filter rlg log records",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        ),
        output_schema = schema_for_output::<Records>()
    )]
    fn filter_log_tool(
        &self,
        Parameters(args): Parameters<FilterLogArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        reply(filter_log_outcome(&args))
    }

    #[tool(
        name = "summarize_errors",
        description = "Group ERROR-and-above rlg records by component and count them, giving a quick error taxonomy for triage. Use this for an at-a-glance failure breakdown; use `filter_log` when you need the underlying records rather than counts.",
        annotations(
            title = "Summarize rlg errors by component",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        ),
        output_schema = schema_for_output::<ErrorSummary>()
    )]
    fn summarize_errors_tool(
        &self,
        Parameters(args): Parameters<SummarizeErrorsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        reply(
            summarize_errors(Path::new(&args.path))
                .map(ErrorSummary::from)
                .map_err(|e| read_error(&args.path, &e)),
        )
    }

    #[tool(
        name = "tail_logs_glob",
        description = "Return the last N parseable rlg records across every file matching a glob pattern (e.g. `/var/log/**/*.log`), newest last, optionally filtered to a minimum level. Use this to tail many rotated or per-service log files at once; use `tail_log` for a single file and `summarize_errors` for per-component error totals.",
        annotations(
            title = "Tail rlg logs across a glob",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        ),
        output_schema = schema_for_output::<Records>()
    )]
    fn tail_logs_glob_tool(
        &self,
        Parameters(args): Parameters<TailLogsGlobArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        reply(tail_logs_glob_outcome(&args))
    }
}

fn filter_log_outcome(args: &FilterLogArgs) -> Result<Records, String> {
    let mut filter = Filter::new();
    if let Some(level) = args.min_level.as_deref() {
        filter = filter.min_level(parse_level(level)?);
    }
    if let Some(component) = args.component.as_deref() {
        filter = filter.component(component);
    }
    let format = args.format.parse::<LogFormat>().map_err(|e| {
        format!(
            "{e}. Use an rlg LogFormat name such as Logfmt or JSON."
        )
    })?;
    filter_log(Path::new(&args.path), &filter, format)
        .map(Records::from)
        .map_err(|e| read_error(&args.path, &e))
}

fn tail_logs_glob_outcome(
    args: &TailLogsGlobArgs,
) -> Result<Records, String> {
    let level = args.level.as_deref().map(parse_level).transpose()?;
    tail_logs_glob(&args.glob_pattern, args.lines, level)
        .map(Records::from)
}

/// The name of the one prompt.
const TRIAGE_PROMPT: &str = "triage_error_spike";
/// The static resource: the severity ladder.
const LEVELS_URI: &str = "rlg://log-levels";
/// The templated resource: a log's recent tail.
const TAIL_URI_PREFIX: &str = "rlg://tail/";

/// Build the `triage_error_spike` prompt text, embedding the
/// caller-supplied `path` and `window_minutes` arguments when present.
fn triage_error_spike(
    path: Option<&str>,
    window_minutes: Option<u64>,
) -> String {
    let target = match path {
        Some(p) if !p.is_empty() => format!("`{p}`"),
        _ => "the rlg log".to_owned(),
    };
    let window_clause = match window_minutes {
        Some(m) => format!(" Focus on roughly the last {m} minutes."),
        None => String::new(),
    };
    format!(
        "Help me triage an error spike in {target}.{window_clause} Start with \
         summarize_errors to get the ERROR-and-above count per component and \
         spot which component spiked. Then call filter_log with \
         min_level=ERROR (and component set to the worst offender) to read the \
         actual failing records, and tail_log for the surrounding recent \
         context. From the messages, group the errors by likely root cause and \
         propose the most probable trigger (a recent deploy, a dependency \
         outage, a config change) with the evidence for each."
    )
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for LogServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .build(),
        )
        .with_server_info(
            Implementation::new("rlg-mcp", env!("CARGO_PKG_VERSION"))
                .with_title("RustLogs MCP")
                .with_website_url(env!("CARGO_PKG_HOMEPAGE")),
        )
        .with_instructions(
            "Tools over rlg (RustLogs) log files on the server's \
             filesystem, given by path. tail_log returns the most recent \
             records of one file and tail_logs_glob the same across every \
             file matching a glob; filter_log selects records by minimum \
             level and component; summarize_errors counts ERROR-and-above \
             records per component. The triage_error_spike prompt walks \
             through an error-spike investigation with those tools.",
        )
    }

    /// A tool the server does not have is reported as a tool result,
    /// not a protocol error.
    ///
    /// The SDK's default is `-32602`, which the stateless HTTP revision
    /// carries as an HTTP 400 — a transport fault to the client, and
    /// nothing a model gets to read. A model that misspelt a tool name
    /// is better served by text saying so.
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        if !self.tool_router.has_route(&request.name) {
            return Ok(CallToolResult::error(vec![ContentBlock::text(
                format!(
                    "Unknown tool: {}. The tools are tail_log, filter_log, \
                     summarize_errors and tail_logs_glob.",
                    request.name
                ),
            )])
            .into());
        }
        let call = ToolCallContext::new(self, request, context);
        self.tool_router.call(call).await
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, ErrorData> {
        let prompt = Prompt::new(
            TRIAGE_PROMPT,
            Some(
                "Guided SRE workflow for investigating a spike of errors in \
                 an rlg log using tail_log, filter_log and summarize_errors.",
            ),
            Some(vec![
                PromptArgument::new("path")
                    .with_description(
                        "Filesystem path to the rlg log file to triage.",
                    )
                    .with_required(false),
                PromptArgument::new("window_minutes")
                    .with_description(
                        "Recent time window to focus on, in minutes.",
                    )
                    .with_required(false),
            ]),
        )
        .with_title("Triage an rlg error spike");
        Ok(ListPromptsResult::with_all_items(vec![prompt]))
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResponse, ErrorData> {
        if request.name != TRIAGE_PROMPT {
            return Err(ErrorData::invalid_params(
                format!(
                    "unknown prompt: {}. The one prompt is {TRIAGE_PROMPT}.",
                    request.name
                ),
                None,
            ));
        }
        let args = request.arguments.unwrap_or_default();
        let path = args.get("path").and_then(serde_json::Value::as_str);
        // A client sends prompt arguments as strings; a number is
        // accepted too.
        let window = args.get("window_minutes").and_then(|v| {
            v.as_u64().or_else(|| {
                v.as_str().and_then(|s| s.trim().parse().ok())
            })
        });
        let text = triage_error_spike(path, window);
        Ok(GetPromptResult::new(vec![PromptMessage::new_text(
            Role::User,
            text,
        )])
        .with_description("Guided rlg error-spike SRE triage workflow.")
        .into())
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        let levels = Resource::new(LEVELS_URI, "log-levels")
            .with_title("rlg severity ladder")
            .with_description(
                "The ordered rlg log levels, lowest to highest severity, as \
                 accepted by filter_log's min_level.",
            )
            .with_mime_type("application/json");
        Ok(ListResourcesResult::with_all_items(vec![levels]))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        let tail = ResourceTemplate::new(
            format!("{TAIL_URI_PREFIX}{{path}}"),
            "tail",
        )
        .with_title("Recent rlg log tail")
        .with_description(
            "The last 100 parseable rlg records from the log file at \
             {path}, newest last, as a read-only resource.",
        )
        .with_mime_type("text/plain");
        Ok(ListResourceTemplatesResult::with_all_items(vec![tail]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        let uri = request.uri;
        if uri == LEVELS_URI {
            let text = serde_json::json!([
                "TRACE", "DEBUG", "VERBOSE", "INFO", "WARN", "ERROR",
                "FATAL", "CRITICAL"
            ])
            .to_string();
            let contents = ResourceContents::text(text, uri)
                .with_mime_type("application/json");
            return Ok(ReadResourceResult::new(vec![contents]).into());
        }
        if let Some(path) = uri.strip_prefix(TAIL_URI_PREFIX) {
            let lines =
                tail_log(Path::new(path), 100).map_err(|e| {
                    ErrorData::resource_not_found(
                        read_error(path, &e),
                        None,
                    )
                })?;
            let contents =
                ResourceContents::text(lines.join("\n"), uri)
                    .with_mime_type("text/plain");
            return Ok(ReadResourceResult::new(vec![contents]).into());
        }
        Err(ErrorData::resource_not_found(
            format!(
                "resource not found: {uri}. The resources are {LEVELS_URI} \
                 and {TAIL_URI_PREFIX}{{path}}."
            ),
            None,
        ))
    }
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests;
