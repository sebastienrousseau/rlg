// integration.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Drives a real `tower::Service` through the [`RlgLayer`] and
//! asserts the layered service responds correctly.
//!
//! The log emission is exercised indirectly — `emit()` calls the
//! global rlg engine, which records into its ring buffer regardless
//! of whether anyone is listening. What we *can* directly verify
//! here is that the layer doesn't break the service contract.

#![allow(missing_docs)]

use http::{Method, Request, Response, StatusCode};
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg_tower::RlgLayer;
use std::convert::Infallible;
use std::future::ready;
use tower::{Layer, ServiceExt};
use tower_service::Service;

#[derive(Clone, Copy)]
struct Echo;

impl Service<Request<()>> for Echo {
    type Response = Response<&'static str>;
    type Error = Infallible;
    type Future =
        std::future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Request<()>) -> Self::Future {
        ready(Ok(Response::builder()
            .status(StatusCode::OK)
            .body("ok")
            .unwrap()))
    }
}

#[tokio::test]
async fn layered_service_returns_inner_response() {
    let layer = RlgLayer::new()
        .level(LogLevel::INFO)
        .format(LogFormat::JSON)
        .component("test-svc");
    let mut svc = layer.layer(Echo);
    let req = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .body(())
        .unwrap();
    let res = svc.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.into_body(), "ok");
}

#[tokio::test]
async fn layered_service_extracts_trace_header() {
    let layer = RlgLayer::new().header("traceparent");
    let mut svc = layer.layer(Echo);
    let req = Request::builder()
        .method(Method::POST)
        .uri("/items")
        .header("traceparent", "00-abc-def-01")
        .body(())
        .unwrap();
    let res = svc.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn layered_service_handles_missing_trace_header() {
    // header() is configured but the request doesn't carry it.
    let layer = RlgLayer::new().header("x-b3-traceid");
    let mut svc = layer.layer(Echo);
    let req = Request::builder()
        .method(Method::DELETE)
        .uri("/things/1")
        .body(())
        .unwrap();
    let res = svc.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

/// Service that always returns an error. Exercises the
/// `Poll::Ready(Err(_))` branch in `RlgFuture::poll`.
#[derive(Clone, Copy)]
struct AlwaysErr;

impl Service<Request<()>> for AlwaysErr {
    type Response = Response<&'static str>;
    type Error = &'static str;
    type Future =
        std::future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Request<()>) -> Self::Future {
        std::future::ready(Err("service unavailable"))
    }
}

#[tokio::test]
async fn layered_service_propagates_inner_error() {
    let layer = RlgLayer::new().level(LogLevel::ERROR);
    let mut svc = layer.layer(AlwaysErr);
    let req = Request::builder()
        .method(Method::GET)
        .uri("/explode")
        .body(())
        .unwrap();
    let res = svc.ready().await.unwrap().call(req).await;
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), "service unavailable");
}
