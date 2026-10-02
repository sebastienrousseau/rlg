// sse.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The 2024-11-05 HTTP+SSE transport.
//!
//! The SDK dropped the server side of this transport in 3.x. It is a
//! small thing: one event stream per session, and a POST endpoint that
//! feeds messages into it. The SDK's service runs over an in-memory
//! pair of channels, exactly as it would over a socket.

use std::collections::HashMap;
use std::fmt;
use std::io;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use axum::Router;
use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use futures::channel::mpsc;
use futures::{SinkExt, Stream, StreamExt};
use rmcp::model::{ClientJsonRpcMessage, ServerJsonRpcMessage};
use rmcp::{ServerHandler, ServiceExt};
use serde::Deserialize;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

use super::{MESSAGE_PATH, SSE_PATH, interrupted};

/// One session's inbox: the channel its posted messages go down.
type Inbox = mpsc::Sender<ClientJsonRpcMessage>;

/// The live sessions, by id.
type Sessions = Arc<Mutex<HashMap<String, Inbox>>>;

/// What the two SSE handlers share.
struct SseState<H> {
    factory: Box<dyn Fn() -> H + Send + Sync>,
    sessions: Sessions,
    /// Cancelled when the server stops, ending every session.
    shutdown: CancellationToken,
}

/// `?sessionId=...` on the message endpoint.
#[derive(Debug, Deserialize)]
struct SessionQuery {
    #[serde(rename = "sessionId")]
    session_id: String,
}

pub(super) async fn serve_sse<H, F>(listener: TcpListener, factory: F) -> io::Result<()>
where
    H: ServerHandler,
    F: Fn() -> H + Send + Sync + 'static,
{
    let shutdown = CancellationToken::new();
    let state = Arc::new(SseState {
        factory: Box::new(factory),
        sessions: Sessions::default(),
        shutdown: shutdown.clone(),
    });
    let router = Router::new()
        .route(SSE_PATH, get(open_stream::<H>))
        .route(MESSAGE_PATH, post(post_message::<H>))
        // Without the trailing slash too: clients differ on whether
        // they keep it, and a 404 over a slash is a poor way to fail.
        .route(MESSAGE_PATH.trim_end_matches('/'), post(post_message::<H>))
        .with_state(state);
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            interrupted().await;
            shutdown.cancel();
        })
        .await
}

/// `GET /sse`: start a session and stream its events.
///
/// The first event is `endpoint`, naming where this session's
/// messages are posted. Every message the server sends after that is a
/// `message` event carrying one JSON-RPC message.
async fn open_stream<H: ServerHandler>(
    State(state): State<Arc<SseState<H>>>,
) -> Response {
    let id = uuid::Uuid::new_v4().simple().to_string();
    let (inbox, from_client) = mpsc::channel::<ClientJsonRpcMessage>(32);
    let (to_client, outbox) = mpsc::channel::<ServerJsonRpcMessage>(32);
    let ct = state.shutdown.child_token();
    let _ = state
        .sessions
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(id.clone(), inbox);

    let handler = (state.factory)();
    let session_ct = ct.clone();
    drop(tokio::spawn(async move {
        // The service ends when its client stream closes -- the POST
        // side dropped -- or when the token is cancelled. Either way
        // there is nobody to report to.
        if let Ok(running) = handler
            .serve_with_ct((to_client, from_client), session_ct)
            .await
        {
            let _ = running.waiting().await;
        }
    }));

    let endpoint = Event::default()
        .event("endpoint")
        .data(format!("{MESSAGE_PATH}?sessionId={id}"));
    let messages = outbox.map(|message| {
        serde_json::to_string(&message)
            .map(|json| Event::default().event("message").data(json))
    });
    let events = futures::stream::once(async { Ok(endpoint) }).chain(messages);
    let stream = SessionStream {
        events: Box::pin(events),
        guard: SessionGuard {
            id,
            sessions: Arc::clone(&state.sessions),
            ct,
        },
    };
    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}

/// `POST /messages/?sessionId=...`: one JSON-RPC message in, `202` out.
///
/// The reply, if any, goes down the session's event stream, which is
/// what makes this the older transport: the HTTP response carries
/// nothing.
async fn post_message<H: ServerHandler>(
    State(state): State<Arc<SseState<H>>>,
    Query(query): Query<SessionQuery>,
    body: Bytes,
) -> Response {
    let message: ClientJsonRpcMessage = match serde_json::from_slice(&body) {
        Ok(message) => message,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("invalid JSON-RPC: {e}"))
                .into_response();
        }
    };
    let inbox = state
        .sessions
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&query.session_id)
        .cloned();
    let Some(mut inbox) = inbox else {
        return (StatusCode::NOT_FOUND, "no such session").into_response();
    };
    match inbox.send(message).await {
        Ok(()) => StatusCode::ACCEPTED.into_response(),
        // The service has gone but the stream has not yet been torn
        // down: the session is over.
        Err(_) => (StatusCode::GONE, "session closed").into_response(),
    }
}

/// Ends the session when the event stream is dropped -- which is how
/// a client hangs up.
struct SessionGuard {
    id: String,
    sessions: Sessions,
    ct: CancellationToken,
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        let _ = self
            .sessions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&self.id);
        self.ct.cancel();
    }
}

/// The event stream of one session, with its guard attached.
struct SessionStream {
    events:
        Pin<Box<dyn Stream<Item = Result<Event, serde_json::Error>> + Send>>,
    #[allow(dead_code, reason = "held for its Drop")]
    guard: SessionGuard,
}

impl Stream for SessionStream {
    type Item = Result<Event, serde_json::Error>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.events.as_mut().poll_next(cx)
    }
}

impl<H> fmt::Debug for SseState<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SseState").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_output_hides_the_handler_factory() {
        let state = SseState::<()> {
            factory: Box::new(|| ()),
            sessions: Sessions::default(),
            shutdown: CancellationToken::new(),
        };
        assert_eq!(format!("{state:?}"), "SseState { .. }");
    }
}
