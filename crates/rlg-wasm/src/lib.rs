// lib.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! WebAssembly bindings for `rlg`.
//!
//! Targets browser, Deno, Cloudflare Workers, Bun, and any host
//! that loads `wasm-bindgen`-produced modules. The 14 rlg
//! `LogFormat` variants are all available; records are rendered to
//! a UTF-8 `String` and (under `wasm32`) dispatched to `console.log`
//! / `console.warn` / `console.error` via the JavaScript bridge.
//!
//! On non-wasm targets the bindings are still compilable — the JS
//! dispatch is replaced by `eprintln!`, so the same code can be
//! exercised in host-side unit tests.
//!
//! # JavaScript usage
//!
//! ```js,ignore
//! import init, { RlgWasm } from "rlg-wasm";
//! await init();
//! const rlg = new RlgWasm("worker", "JSON");
//! rlg.info("worker booted", '{"region":"eu-west-1"}');
//! rlg.error("db timeout", null);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use std::str::FromStr;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// ---------------------------------------------------------------------------
// Host bridge — `console.*` on wasm32, `eprintln!` everywhere else.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
unsafe extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    fn console_warn(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(s: &str);
}

fn dispatch(level: LogLevel, rendered: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        match level {
            LogLevel::WARN => console_warn(rendered),
            LogLevel::ERROR
            | LogLevel::FATAL
            | LogLevel::CRITICAL => console_error(rendered),
            _ => console_log(rendered),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = level;
        eprintln!("{rendered}");
    }
}

// ---------------------------------------------------------------------------
// Public surface — exposed to JS via `wasm-bindgen` on wasm32.
// ---------------------------------------------------------------------------

/// Logger handle. Construct once per "channel" (logical service or
/// component) and emit records through the level shortcuts.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct RlgWasm {
    component: String,
    format: LogFormat,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl RlgWasm {
    /// Construct a new logger.
    ///
    /// `format` is the string name of a [`LogFormat`] variant
    /// (`"JSON"`, `"MCP"`, `"OTLP"`, `"Logfmt"`, etc.). Defaults to
    /// `"JSON"` if the input is unknown.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    #[must_use]
    pub fn new(component: &str, format: &str) -> Self {
        Self {
            component: component.to_string(),
            format: LogFormat::from_str(format).unwrap_or(LogFormat::JSON),
        }
    }

    /// Emit an INFO record. `attributes_json` is an optional JSON
    /// object string; keys become structured attributes on the log
    /// entry. Pass `null` / `undefined` (or `None` from Rust) to skip.
    pub fn info(&self, message: &str, attributes_json: Option<String>) {
        self.emit(LogLevel::INFO, message, attributes_json.as_deref());
    }

    /// Emit a WARN record.
    pub fn warn(&self, message: &str, attributes_json: Option<String>) {
        self.emit(LogLevel::WARN, message, attributes_json.as_deref());
    }

    /// Emit an ERROR record.
    pub fn error(&self, message: &str, attributes_json: Option<String>) {
        self.emit(LogLevel::ERROR, message, attributes_json.as_deref());
    }

    /// Emit a DEBUG record.
    pub fn debug(&self, message: &str, attributes_json: Option<String>) {
        self.emit(LogLevel::DEBUG, message, attributes_json.as_deref());
    }

    fn emit(
        &self,
        level: LogLevel,
        message: &str,
        attributes_json: Option<&str>,
    ) {
        let mut log = Log::build(level, message)
            .component(&self.component)
            .format(self.format);
        if let Some(s) = attributes_json
            && let Ok(serde_json::Value::Object(map)) =
                serde_json::from_str::<serde_json::Value>(s)
        {
            for (k, v) in map {
                log = log.with(&k, v);
            }
        }
        let rendered = format!("{log}");
        dispatch(level, &rendered);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_unknown_format_falls_back_to_json() {
        let r = RlgWasm::new("worker", "NotAFormat");
        assert_eq!(r.format, LogFormat::JSON);
        assert_eq!(r.component, "worker");
    }

    #[test]
    fn new_accepts_each_format_variant() {
        for name in [
            "CLF",
            "CEF",
            "ELF",
            "W3C",
            "JSON",
            "GELF",
            "Logstash",
            "NDJSON",
            "MCP",
            "OTLP",
            "Logfmt",
            "ECS",
        ] {
            let r = RlgWasm::new("svc", name);
            assert_eq!(format!("{:?}", r.format), name);
        }
    }

    #[test]
    fn info_emits_without_panic() {
        let r = RlgWasm::new("worker", "Logfmt");
        r.info("hello", None);
        r.info(
            "with attrs",
            Some(r#"{"user_id":42,"region":"eu-west-1"}"#.to_string()),
        );
    }

    #[test]
    fn invalid_attributes_json_is_silently_dropped() {
        let r = RlgWasm::new("worker", "JSON");
        // Invalid JSON: the closure short-circuits to None and the
        // record is still emitted (no panic, no garbage attributes).
        r.warn("malformed input", Some("not json".to_string()));
    }

    #[test]
    fn level_shortcuts_compose() {
        let r = RlgWasm::new("svc", "JSON");
        r.debug("d", None);
        r.info("i", None);
        r.warn("w", None);
        r.error("e", None);
    }

    #[test]
    fn rlg_wasm_is_clone() {
        let a = RlgWasm::new("svc", "JSON");
        let b = a.clone();
        assert_eq!(a.component, b.component);
        assert_eq!(a.format, b.format);
    }
}
