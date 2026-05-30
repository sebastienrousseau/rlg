// lib.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tower middleware that emits structured rlg records for every
//! HTTP request that passes through it.
//!
//! Compatible with any service that satisfies
//! `tower::Service<http::Request<B>>` — axum, tonic, hyper services,
//! `lambda_runtime`, custom stacks.
//!
//! # Example (axum)
//!
//! ```ignore
//! use axum::{Router, routing::get};
//! use rlg::log_format::LogFormat;
//! use rlg::log_level::LogLevel;
//! use rlg_tower::RlgLayer;
//!
//! let app: Router = Router::new()
//!     .route("/", get(|| async { "hello" }))
//!     .layer(
//!         RlgLayer::new()
//!             .level(LogLevel::INFO)
//!             .format(LogFormat::JSON)
//!             .header("x-request-id"),
//!     );
//! ```
//!
//! One record is emitted **per response**, with the following fields:
//!
//! | Attribute | Source |
//! | --- | --- |
//! | `component` | `"rlg-tower"` (override with `.component(...)`) |
//! | `description` | `"<METHOD> <path> -> <status>"` |
//! | `http.method` | request method |
//! | `http.path` | request URI path |
//! | `http.status` | response status code |
//! | `http.latency_ms` | wall-clock milliseconds from poll-ready to ready response |
//! | `trace_id`, `span_id` | extracted from configured request headers (W3C / b3) if present |

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use http::Request;
use http::Response;
use pin_project_lite::pin_project;
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;
use tower_layer::Layer;
use tower_service::Service;

/// Tower [`Layer`] that wraps a service with rlg request logging.
///
/// Build with [`RlgLayer::new`] and the fluent setters. Cheap to
/// clone — internal state is a couple of small fields.
#[derive(Debug, Clone)]
pub struct RlgLayer {
    config: Config,
}

#[derive(Debug, Clone)]
struct Config {
    level: LogLevel,
    format: LogFormat,
    component: &'static str,
    trace_header: Option<&'static str>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: LogLevel::INFO,
            format: LogFormat::Logfmt,
            component: "rlg-tower",
            trace_header: None,
        }
    }
}

impl Default for RlgLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl RlgLayer {
    /// Construct a new layer with the default configuration:
    /// level INFO, format Logfmt, component `"rlg-tower"`, no
    /// trace-header extraction.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    /// Override the level emitted for every record.
    #[must_use]
    pub const fn level(mut self, level: LogLevel) -> Self {
        self.config.level = level;
        self
    }

    /// Override the wire format.
    #[must_use]
    pub const fn format(mut self, format: LogFormat) -> Self {
        self.config.format = format;
        self
    }

    /// Override the `component` attribute. Useful when multiple
    /// services share a process and you want to distinguish their
    /// access logs.
    #[must_use]
    pub const fn component(mut self, component: &'static str) -> Self {
        self.config.component = component;
        self
    }

    /// Configure a request header to extract as the `trace_id`
    /// attribute. The common picks are `traceparent` (W3C
    /// trace-context) or `x-b3-traceid` (Zipkin B3).
    #[must_use]
    pub const fn header(mut self, name: &'static str) -> Self {
        self.config.trace_header = Some(name);
        self
    }
}

impl<S> Layer<S> for RlgLayer {
    type Service = RlgService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RlgService {
            inner,
            config: self.config.clone(),
        }
    }
}

/// Service wrapper produced by [`RlgLayer`].
#[derive(Debug, Clone)]
pub struct RlgService<S> {
    inner: S,
    config: Config,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for RlgService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = RlgFuture<S::Future>;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let trace_id = self
            .config
            .trace_header
            .and_then(|name| req.headers().get(name))
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let cfg = self.config.clone();
        let start = Instant::now();
        let fut = self.inner.call(req);
        RlgFuture {
            inner: fut,
            method,
            path,
            trace_id,
            cfg,
            start,
        }
    }
}

pin_project! {
    /// Future returned by [`RlgService::call`].
    #[derive(Debug)]
    pub struct RlgFuture<F> {
        #[pin]
        inner: F,
        method: http::Method,
        path: String,
        trace_id: Option<String>,
        cfg: Config,
        start: Instant,
    }
}

impl<F, ResBody, E> Future for RlgFuture<F>
where
    F: Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(response)) => {
                emit(
                    this.cfg,
                    this.method,
                    this.path,
                    this.trace_id.as_deref(),
                    Some(response.status().as_u16()),
                    this.start.elapsed().as_millis() as u64,
                );
                Poll::Ready(Ok(response))
            }
            Poll::Ready(Err(err)) => {
                emit(
                    this.cfg,
                    this.method,
                    this.path,
                    this.trace_id.as_deref(),
                    None,
                    this.start.elapsed().as_millis() as u64,
                );
                Poll::Ready(Err(err))
            }
        }
    }
}

fn emit(
    cfg: &Config,
    method: &http::Method,
    path: &str,
    trace_id: Option<&str>,
    status: Option<u16>,
    latency_ms: u64,
) {
    let status_str =
        status.map_or_else(|| "ERR".to_string(), |s| s.to_string());
    let description = format!("{method} {path} -> {status_str}");
    let mut log = Log::build(cfg.level, &description)
        .component(cfg.component)
        .format(cfg.format)
        .with("http.method", method.as_str())
        .with("http.path", path)
        .with("http.latency_ms", latency_ms);
    if let Some(s) = status {
        log = log.with("http.status", s);
    }
    if let Some(t) = trace_id {
        log = log.with("trace_id", t);
    }
    log.fire();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sensible() {
        let cfg = Config::default();
        assert_eq!(cfg.level, LogLevel::INFO);
        assert_eq!(cfg.format, LogFormat::Logfmt);
        assert_eq!(cfg.component, "rlg-tower");
        assert!(cfg.trace_header.is_none());
    }

    #[test]
    fn fluent_setters_chain() {
        let layer = RlgLayer::new()
            .level(LogLevel::WARN)
            .format(LogFormat::JSON)
            .component("api")
            .header("traceparent");
        assert_eq!(layer.config.level, LogLevel::WARN);
        assert_eq!(layer.config.format, LogFormat::JSON);
        assert_eq!(layer.config.component, "api");
        assert_eq!(layer.config.trace_header, Some("traceparent"));
    }

    #[test]
    fn default_constructor_matches_new() {
        let a = RlgLayer::default();
        let b = RlgLayer::new();
        assert_eq!(a.config.level, b.config.level);
        assert_eq!(a.config.format, b.config.format);
    }

    #[test]
    fn layer_is_clone() {
        let l1 = RlgLayer::new().component("svc");
        let l2 = l1.clone();
        assert_eq!(l1.config.component, l2.config.component);
    }
}
