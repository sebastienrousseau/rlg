// blocking_http.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The blocking exporter's I/O: the crate's own HTTP/1.1 exchange
//! (`src/http.rs`) over a `std::net::TcpStream`, with one deadline
//! covering connect, send and the response head. There is no TLS;
//! see `docs/adr/0015-otlp-local-collector-transport.md`.

use crate::http::{self, Endpoint};
use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

/// Connect to `target`, write `request`, and read until the final
/// status line and headers have arrived, all within `timeout`.
///
/// # Errors
/// [`io::ErrorKind::TimedOut`] when the deadline passes, the
/// connect or I/O error otherwise, and
/// [`io::ErrorKind::InvalidData`] for an answer that is not HTTP/1.x.
pub(crate) fn exchange(
    target: &Endpoint,
    request: &[u8],
    timeout: Duration,
) -> io::Result<u16> {
    let deadline = Instant::now() + timeout;
    let mut stream = connect(target, deadline)?;
    stream.set_write_timeout(Some(remaining(deadline)?))?;
    stream.write_all(request).map_err(timed_out)?;
    read_status(&mut stream, deadline)
}

/// Read until the final status line and headers have arrived.
fn read_status(
    stream: &mut TcpStream,
    deadline: Instant,
) -> io::Result<u16> {
    let mut buf = Vec::with_capacity(512);
    let mut chunk = [0_u8; 1024];
    loop {
        stream.set_read_timeout(Some(remaining(deadline)?))?;
        let n = stream.read(&mut chunk).map_err(timed_out)?;
        if let Some(status) = http::accept_chunk(&mut buf, &chunk[..n])?
        {
            return Ok(status);
        }
    }
}

/// Try each address `target` resolves to until one accepts.
fn connect(
    target: &Endpoint,
    deadline: Instant,
) -> io::Result<TcpStream> {
    let mut last = io::Error::new(
        io::ErrorKind::NotFound,
        "collector host resolved to no addresses",
    );
    for addr in (target.host.as_str(), target.port).to_socket_addrs()? {
        match TcpStream::connect_timeout(&addr, remaining(deadline)?) {
            Ok(stream) => return Ok(stream),
            Err(e) => last = timed_out(e),
        }
    }
    Err(last)
}

/// Time left before `deadline`, or a timeout error once it passes.
/// Never zero: the socket timeouts reject a zero duration.
fn remaining(deadline: Instant) -> io::Result<Duration> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|d| !d.is_zero())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::TimedOut,
                "collector timed out",
            )
        })
}

/// A socket timeout surfaces as `WouldBlock` on Unix and `TimedOut`
/// on Windows; report both as `TimedOut`.
fn timed_out(e: io::Error) -> io::Error {
    if e.kind() == io::ErrorKind::WouldBlock {
        io::Error::new(io::ErrorKind::TimedOut, "collector timed out")
    } else {
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remaining_refuses_a_passed_deadline() {
        let err = remaining(Instant::now()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::TimedOut);
        assert!(
            remaining(Instant::now() + Duration::from_secs(5)).is_ok()
        );
    }

    #[test]
    fn timed_out_maps_would_block_only() {
        let e = timed_out(io::ErrorKind::WouldBlock.into());
        assert_eq!(e.kind(), io::ErrorKind::TimedOut);
        let e = timed_out(io::ErrorKind::ConnectionRefused.into());
        assert_eq!(e.kind(), io::ErrorKind::ConnectionRefused);
    }
}
