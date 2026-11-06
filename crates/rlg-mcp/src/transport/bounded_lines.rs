// bounded_lines.rs
// Copyright © 2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A cap on the length of one stdio line.
//!
//! The SDK frames stdio messages by newline and buffers a line until
//! its newline arrives, however long that takes. A client that never
//! ends a line would grow that buffer until the pipe closes. This
//! reader sits under the SDK's framing, passes bytes through
//! unchanged, and fails the read once a single line passes 16 MiB,
//! which ends the session. Ported from oxml-mcp.

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll, ready};
use tokio::io::{AsyncRead, ReadBuf};

/// The longest line, newline excluded, a client may send.
const MAX_LINE_BYTES: usize = 16 * 1024 * 1024;

/// `reader` with every line held to [`MAX_LINE_BYTES`].
#[derive(Debug)]
pub(super) struct BoundedLines<R> {
    reader: R,
    line_bytes: usize,
}

impl<R> BoundedLines<R> {
    pub(super) fn new(reader: R) -> Self {
        Self {
            reader,
            line_bytes: 0,
        }
    }

    /// Count `bytes` against the current line; a newline starts a new
    /// one.
    fn charge(&mut self, bytes: &[u8]) -> io::Result<()> {
        for byte in bytes {
            self.line_bytes = if *byte == b'\n' {
                0
            } else {
                self.line_bytes.saturating_add(1)
            };
            if self.line_bytes > MAX_LINE_BYTES {
                return Err(line_limit());
            }
        }
        Ok(())
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for BoundedLines<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        output: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if self.line_bytes > MAX_LINE_BYTES {
            return Poll::Ready(Err(line_limit()));
        }
        if output.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }
        let mut bytes = [0_u8; 8192];
        let capacity = bytes.len().min(output.remaining());
        let mut chunk = ReadBuf::new(&mut bytes[..capacity]);
        ready!(Pin::new(&mut self.reader).poll_read(cx, &mut chunk))?;
        self.charge(chunk.filled())?;
        output.put_slice(chunk.filled());
        Poll::Ready(Ok(()))
    }
}

fn line_limit() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "stdio line exceeds the 16 MiB limit",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt as _;

    #[tokio::test]
    async fn lines_under_the_cap_pass_through_unchanged() {
        let input = b"{\"a\":1}\n{\"b\":2}\n".to_vec();
        let mut reader = BoundedLines::new(input.as_slice());
        let mut output = Vec::new();
        let _ = reader.read_to_end(&mut output).await.unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn a_newline_resets_the_count() {
        let mut lines = BoundedLines::new(&b""[..]);
        let almost = vec![b'x'; MAX_LINE_BYTES];
        lines.charge(&almost).unwrap();
        lines.charge(b"\n").unwrap();
        lines.charge(&almost).unwrap();
        assert!(lines.charge(b"x").is_err());
    }

    #[tokio::test]
    async fn one_line_past_the_cap_fails_the_read_and_stays_failed() {
        let input = vec![b' '; MAX_LINE_BYTES + 1];
        let mut reader = BoundedLines::new(input.as_slice());
        let mut output = Vec::new();
        let err = reader.read_to_end(&mut output).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("16 MiB"), "{err}");
        let mut more = [0_u8; 1];
        assert!(reader.read(&mut more).await.is_err());
    }
}
