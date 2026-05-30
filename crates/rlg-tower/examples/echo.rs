// echo.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Demonstrates wrapping a hand-rolled `tower::Service` with
// `RlgLayer`. Logs one record per request without needing an HTTP
// server to be running.
//
// Run with: cargo run -p rlg-tower --example echo

#![allow(missing_docs)]

use http::{Method, Request, Response, StatusCode};
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg_tower::RlgLayer;
use std::convert::Infallible;
use std::future::{Ready, ready};
use std::task::{Context, Poll};
use tower::{Layer, ServiceExt};
use tower_service::Service;

#[derive(Clone, Copy)]
struct Echo;

impl Service<Request<()>> for Echo {
    type Response = Response<&'static str>;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _: &mut Context<'_>,
    ) -> Poll<Result<(), Infallible>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Request<()>) -> Self::Future {
        ready(Ok(Response::builder()
            .status(StatusCode::OK)
            .body("hello world")
            .unwrap()))
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let _guard = rlg::init().expect("rlg::init");

    let mut svc = RlgLayer::new()
        .level(LogLevel::INFO)
        .format(LogFormat::Logfmt)
        .component("echo")
        .header("traceparent")
        .layer(Echo);

    let req = Request::builder()
        .method(Method::GET)
        .uri("/hello")
        .header("traceparent", "00-abc-def-01")
        .body(())
        .unwrap();
    let res = svc.ready().await.unwrap().call(req).await.unwrap();
    println!("response: {} {}", res.status(), res.into_body());
}
