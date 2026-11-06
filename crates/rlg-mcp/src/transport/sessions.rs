// sessions.rs
// Copyright © 2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A session manager that holds at most so many sessions at once.
//!
//! Every streamable HTTP `initialize` creates a session that lives
//! until the client closes it or it idles out (minutes). Without a
//! bound, a client could open sessions faster than they expire. This
//! wraps the SDK's in-memory manager and refuses to create one past
//! the limit; everything else is passed through.
//!
//! Ported from noyalib-mcp (commit 9279c28).

use std::fmt;
use std::sync::Arc;

use futures::Stream;
use rmcp::model::{ClientJsonRpcMessage, ServerJsonRpcMessage};
use rmcp::transport::streamable_http_server::session::local::{
    LocalSessionManager, LocalSessionManagerError,
};
use rmcp::transport::streamable_http_server::session::{
    EventStore, RestoreOutcome, ServerSseMessage, SessionId, SessionManager,
};

/// [`LocalSessionManager`] with a ceiling on live sessions.
#[derive(Debug, Default)]
pub(super) struct CappedSessions {
    inner: LocalSessionManager,
    max: usize,
}

impl CappedSessions {
    pub(super) fn new(max: usize) -> Self {
        Self {
            inner: LocalSessionManager::default(),
            max,
        }
    }
}

/// Why a session operation failed.
#[derive(Debug)]
pub(super) enum CappedError {
    /// Creating one more session would pass the limit.
    Full(usize),
    /// The wrapped manager failed.
    Inner(LocalSessionManagerError),
}

impl fmt::Display for CappedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full(max) => write!(
                f,
                "the server holds its session limit of {max}; close a session or retry later"
            ),
            Self::Inner(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for CappedError {}

type Transport = <LocalSessionManager as SessionManager>::Transport;
type Outcome<T> = Result<T, CappedError>;

impl SessionManager for CappedSessions {
    type Error = CappedError;
    type Transport = Transport;

    async fn create_session(&self) -> Outcome<(SessionId, Transport)> {
        if self.inner.sessions.read().await.len() >= self.max {
            return Err(CappedError::Full(self.max));
        }
        self.inner
            .create_session()
            .await
            .map_err(CappedError::Inner)
    }

    async fn initialize_session(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Outcome<ServerJsonRpcMessage> {
        let reply = self.inner.initialize_session(id, message).await;
        reply.map_err(CappedError::Inner)
    }

    async fn has_session(&self, id: &SessionId) -> Outcome<bool> {
        self.inner.has_session(id).await.map_err(CappedError::Inner)
    }

    async fn close_session(&self, id: &SessionId) -> Outcome<()> {
        self.inner
            .close_session(id)
            .await
            .map_err(CappedError::Inner)
    }

    async fn create_stream(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Outcome<impl Stream<Item = ServerSseMessage> + Send + Sync + 'static> {
        let stream = self.inner.create_stream(id, message).await;
        stream.map_err(CappedError::Inner)
    }

    async fn accept_message(&self, id: &SessionId, message: ClientJsonRpcMessage) -> Outcome<()> {
        let accepted = self.inner.accept_message(id, message).await;
        accepted.map_err(CappedError::Inner)
    }

    async fn create_standalone_stream(
        &self,
        id: &SessionId,
    ) -> Outcome<impl Stream<Item = ServerSseMessage> + Send + Sync + 'static> {
        let stream = self.inner.create_standalone_stream(id).await;
        stream.map_err(CappedError::Inner)
    }

    async fn resume(
        &self,
        id: &SessionId,
        last_event_id: String,
    ) -> Outcome<impl Stream<Item = ServerSseMessage> + Send + Sync + 'static> {
        let stream = self.inner.resume(id, last_event_id).await;
        stream.map_err(CappedError::Inner)
    }

    async fn restore_session(&self, id: SessionId) -> Outcome<RestoreOutcome<Transport>> {
        // A restore re-creates a session, so it counts against the cap
        // like `create_session`; one already in memory adds nothing.
        let present = self.inner.sessions.read().await.contains_key(&id);
        if !present && self.inner.sessions.read().await.len() >= self.max {
            return Err(CappedError::Full(self.max));
        }
        let restored = self.inner.restore_session(id).await;
        restored.map_err(CappedError::Inner)
    }

    fn event_store(&self) -> Option<Arc<dyn EventStore>> {
        self.inner.event_store()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_cap_refuses_one_session_too_many_until_one_closes() {
        let sessions = CappedSessions::new(1);
        let (id, _transport) = sessions.create_session().await.unwrap();
        assert!(sessions.has_session(&id).await.unwrap());
        let err = sessions.create_session().await.err().unwrap();
        assert!(matches!(err, CappedError::Full(1)), "{err}");
        assert!(err.to_string().contains("session limit of 1"), "{err}");
        sessions.close_session(&id).await.unwrap();
        assert!(!sessions.has_session(&id).await.unwrap());
        assert!(sessions.create_session().await.is_ok());
    }

    #[tokio::test]
    async fn an_unknown_session_is_the_inner_managers_error() {
        let sessions = CappedSessions::new(4);
        let id: SessionId = "no-such-session".into();
        let err = sessions.resume(&id, "0".to_owned()).await.err().unwrap();
        assert!(matches!(err, CappedError::Inner(_)), "{err}");
        assert!(err.to_string().contains("no-such-session"), "{err}");
        assert!(sessions.event_store().is_none());
    }

    #[tokio::test]
    async fn restore_is_passed_through() {
        // Only an HTTP service with a session store calls this, and
        // this server configures none.
        let sessions = CappedSessions::new(4);
        let id: SessionId = "restored".into();
        let first = sessions.restore_session(id.clone()).await.unwrap();
        assert!(matches!(first, RestoreOutcome::Restored(_)));
        let again = sessions.restore_session(id.clone()).await.unwrap();
        assert!(matches!(again, RestoreOutcome::AlreadyPresent));
        assert!(sessions.has_session(&id).await.unwrap());
    }

    #[tokio::test]
    async fn restore_counts_against_the_cap() {
        let sessions = CappedSessions::new(1);
        let id: SessionId = "restored".into();
        assert!(sessions.restore_session(id.clone()).await.is_ok());
        let err = sessions.restore_session("another".into()).await.err();
        assert!(matches!(err, Some(CappedError::Full(1))), "{err:?}");
        let again = sessions.restore_session(id).await.unwrap();
        assert!(matches!(again, RestoreOutcome::AlreadyPresent));
    }
}
