// outcome.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! What a tool call returns: the shared reply shape, the wording of
//! failures, and the two tools whose work takes more than one call.

use crate::Root;
use crate::model::{FilterLogArgs, Records, TailLogsGlobArgs};
use crate::ops::{self, filter_log};
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg_cli::Filter;
use rmcp::model::{CallToolResult, ContentBlock, ErrorData};
use serde::Serialize;
use std::fmt;
use std::path::Path;

/// A failure to read `path`, worded for a model: the path is what
/// makes the message actionable.
pub(crate) fn read_error(path: &str, e: &std::io::Error) -> String {
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
pub(crate) fn reply<T: Serialize + fmt::Display>(
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

pub(crate) fn filter_log_outcome(
    path: &Path,
    args: &FilterLogArgs,
) -> Result<Records, String> {
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
    filter_log(path, &filter, format)
        .map(Records::from)
        .map_err(|e| read_error(&args.path, &e))
}

pub(crate) fn tail_logs_glob_outcome(
    root: &Root,
    args: &TailLogsGlobArgs,
) -> Result<Records, String> {
    let level = args.level.as_deref().map(parse_level).transpose()?;
    let paths = root.glob(&args.glob_pattern)?;
    ops::tail_files(&paths, args.lines, level).map(Records::from)
}
