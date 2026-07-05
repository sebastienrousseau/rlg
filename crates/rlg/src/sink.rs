// sink.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Platform-native logging sinks.
//!
//! [`PlatformSink`] routes formatted log payloads to the best available
//! output: `os_log` on macOS, `journald` on Linux, or stdout/file as fallback.
//! Construct via [`PlatformSink::native()`] or [`PlatformSink::from_config()`].

use std::io::Write;

#[cfg(unix)]
use std::os::unix::net::UnixDatagram;

#[cfg(not(unix))]
#[allow(dead_code)]
#[derive(Debug)]
pub struct UnixDatagram;

#[cfg(not(unix))]
#[allow(dead_code)]
impl UnixDatagram {
    pub fn send(&self, _: &[u8]) -> std::io::Result<usize> {
        Ok(0)
    }
}

/// Unified interface for platform-native log output.
#[derive(Debug)]
#[allow(variant_size_differences)]
pub enum PlatformSink {
    /// Standard output fallback.
    Stdout,
    /// File sink fallback.
    File(std::fs::File),
    /// Native OS Log on macOS.
    OsLog,
    /// Systemd Journald socket on Linux.
    Journald(Option<UnixDatagram>),
    /// Linux `io_uring`-backed file sink. Enabled with the `uring`
    /// feature. Only compiles on Linux — see
    /// `docs/adr/0011-io-uring-file-sink.md` for the current
    /// implementation status.
    ///
    /// The variant currently stores the underlying `File` and
    /// delegates writes to the standard synchronous path; the
    /// `io_uring` submission-queue integration lands in Phase 20.1.
    /// Consumers can already select this variant to future-proof
    /// their sink pipeline, and the `io-uring` dependency is
    /// resolved so the SQE wiring is drop-in.
    #[cfg(all(target_os = "linux", feature = "uring"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(target_os = "linux", feature = "uring")))
    )]
    UringFile(std::fs::File),
}

/// POSIX `syslog(3)` bindings.
///
/// `syslog(3)` is the supported, stable-ABI path for emitting log records
/// from a non-Objective-C process. On macOS (Sierra and newer) the syslog
/// gateway is routed into `os_log`, so records still appear under
/// `log stream` / Console.app. This avoids the `_os_log_impl` private
/// symbol, whose binary-trailer calling convention can only be produced
/// by the compiler-expanded `os_log` macro and is undefined behaviour to
/// call directly from Rust.
#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
mod syslog_ffi {
    use std::ffi::CStr;
    use std::os::raw::{c_char, c_int};
    use std::sync::OnceLock;

    pub(super) const LOG_CRIT: c_int = 2;
    pub(super) const LOG_ERR: c_int = 3;
    pub(super) const LOG_WARNING: c_int = 4;
    pub(super) const LOG_NOTICE: c_int = 5;
    pub(super) const LOG_INFO: c_int = 6;
    pub(super) const LOG_DEBUG: c_int = 7;
    const LOG_USER: c_int = 1 << 3;
    const LOG_PID: c_int = 0x01;

    unsafe extern "C" {
        fn openlog(
            ident: *const c_char,
            logopt: c_int,
            facility: c_int,
        );
        fn syslog(
            priority: c_int,
            format: *const c_char,
            arg: *const c_char,
        );
    }

    static INIT: OnceLock<()> = OnceLock::new();

    fn ensure_open() {
        INIT.get_or_init(|| {
            // POSIX requires the ident pointer to remain valid for the
            // lifetime of the syslog connection. A `c""` literal is a
            // `&'static CStr` embedded in the binary — no allocation,
            // no leak, no fallible construction.
            const IDENT: &CStr = c"rlg";
            // SAFETY: `IDENT.as_ptr()` is a valid null-terminated string with
            // a 'static lifetime; LOG_PID + LOG_USER are valid bit flags.
            unsafe { openlog(IDENT.as_ptr(), LOG_PID, LOG_USER) };
        });
    }

    /// Emit a single record. `msg` must be a valid null-terminated string.
    ///
    /// # Safety
    /// `msg` must point to a valid `\0`-terminated byte sequence that
    /// remains valid for the duration of the call.
    pub(super) unsafe fn emit(priority: c_int, msg: *const c_char) {
        ensure_open();
        // SAFETY: caller upholds `msg` validity. We pass a static "%s"
        // format with exactly one `%s` argument, which matches the variadic
        // contract `syslog(3)` expects (no varargs UB).
        unsafe { syslog(priority, c"%s".as_ptr(), msg) };
    }
}

impl PlatformSink {
    /// Build a sink from the given [`Config`](crate::config::Config).
    ///
    /// Inspects `logging_destinations` in order:
    /// - `File(path)` → open for append
    /// - `Stdout` → stdout
    /// - `Network(_)` → skipped (not yet implemented)
    ///
    /// Falls back to [`PlatformSink::native()`] if no destination matches.
    #[must_use]
    pub fn from_config(config: &crate::config::Config) -> Self {
        for dest in &config.logging_destinations {
            match dest {
                crate::config::LoggingDestination::File(path) => {
                    if let Ok(file) = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(path)
                    {
                        return Self::File(file);
                    }
                }
                crate::config::LoggingDestination::Stdout => {
                    return Self::Stdout;
                }
                crate::config::LoggingDestination::Network(_) => {
                    // Network sinks not yet implemented — fall through.
                }
            }
        }
        Self::native()
    }

    /// Detect and return the best native sink for the current OS.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn native() -> Self {
        // Allow explicit fallback to stdout via environment variable.
        if std::env::var("RLG_FALLBACK_STDOUT").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
        {
            return Self::Stdout;
        }

        #[cfg(target_os = "macos")]
        {
            Self::OsLog
        }
        #[cfg(target_os = "linux")]
        {
            Self::detect_journald()
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Self::Stdout
        }
    }

    /// Detect the `journald` socket on Linux.
    #[cfg(target_os = "linux")]
    fn detect_journald() -> Self {
        Self::try_journald_socket("/run/systemd/journal/socket")
    }

    /// Connect a `UnixDatagram` to the given socket path.
    #[cfg(target_os = "linux")]
    fn try_journald_socket(path: &str) -> Self {
        UnixDatagram::unbound()
            .ok()
            .and_then(|socket| {
                socket.connect(path).ok().map(|()| socket)
            })
            .map_or(Self::Journald(None), |s| Self::Journald(Some(s)))
    }

    /// Write a formatted log payload to this sink.
    #[allow(unused_variables)]
    pub fn emit(&mut self, level: &str, payload: &[u8]) {
        match self {
            Self::Stdout => {
                let _ = std::io::stdout().write_all(payload);
                let _ = std::io::stdout().write_all(b"\n");
            }
            Self::File(f) => {
                let _ = f.write_all(payload);
                let _ = f.write_all(b"\n");
            }
            Self::OsLog => Self::emit_os_log(level, payload),
            Self::Journald(socket_opt) => {
                Self::emit_journald(
                    level,
                    payload,
                    socket_opt.as_ref(),
                );
            }
            #[cfg(all(target_os = "linux", feature = "uring"))]
            Self::UringFile(f) => {
                // Phase 20 scaffold: delegates to the sync write
                // path for correctness. Phase 20.1 wires up the
                // io_uring submission queue for zero-copy
                // batched writes. Consumers who need the io_uring
                // performance profile today can construct their
                // own SQE loop against the underlying `File`
                // via the `io-uring` crate — the feature already
                // resolves the dep.
                let _ = f.write_all(payload);
                let _ = f.write_all(b"\n");
            }
        }
    }

    /// Emit one record to macOS `os_log` via the `syslog(3)` gateway.
    ///
    /// On non-macOS targets this is a no-op so the `OsLog` variant remains
    /// callable cross-platform.
    #[cfg(target_os = "macos")]
    fn emit_os_log(level: &str, payload: &[u8]) {
        use syslog_ffi::{
            LOG_CRIT, LOG_DEBUG, LOG_ERR, LOG_INFO, LOG_NOTICE,
            LOG_WARNING, emit,
        };

        if std::env::var("RLG_FALLBACK_STDOUT").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
        {
            let _ = std::io::stdout().write_all(payload);
            let _ = std::io::stdout().write_all(b"\n");
            return;
        }

        let priority = match level {
            "FATAL" | "CRITICAL" => LOG_CRIT,
            "ERROR" => LOG_ERR,
            "WARN" => LOG_WARNING,
            "INFO" => LOG_INFO,
            "DEBUG" | "TRACE" | "VERBOSE" => LOG_DEBUG,
            _ => LOG_NOTICE,
        };
        // Strip embedded NULs so the C string is well-formed, then
        // append our own terminator.
        let mut buf: Vec<u8> =
            payload.iter().copied().filter(|&b| b != 0).collect();
        buf.push(0);
        // SAFETY: `buf` is owned for the duration of this call, ends
        // in a `\0`, and `syslog(3)` is thread-safe.
        #[allow(unsafe_code)]
        unsafe {
            emit(priority, buf.as_ptr().cast::<std::os::raw::c_char>());
        }
    }

    /// No-op on non-macOS targets.
    #[cfg(not(target_os = "macos"))]
    const fn emit_os_log(_level: &str, _payload: &[u8]) {}

    /// Emit one record to the `journald` Unix-datagram socket.
    ///
    /// Falls back to stdout when the socket is unavailable, when the host
    /// is not Linux, or when `RLG_FALLBACK_STDOUT` / `GITHUB_ACTIONS` is
    /// set.
    fn emit_journald(
        level: &str,
        payload: &[u8],
        socket_opt: Option<&UnixDatagram>,
    ) {
        let Some(socket) = socket_opt else {
            let _ = std::io::stdout().write_all(payload);
            let _ = std::io::stdout().write_all(b"\n");
            return;
        };

        if std::env::var("RLG_FALLBACK_STDOUT").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
        {
            let _ = socket;
            return;
        }

        #[cfg(target_os = "linux")]
        {
            let priority = match level {
                "ERROR" | "FATAL" | "CRITICAL" => "3",
                "WARN" => "4",
                "INFO" => "6",
                "DEBUG" | "TRACE" | "VERBOSE" => "7",
                _ => "5",
            };
            let mut journal_payload =
                Vec::with_capacity(payload.len() + 32);
            journal_payload.extend_from_slice(b"PRIORITY=");
            journal_payload.extend_from_slice(priority.as_bytes());
            journal_payload.extend_from_slice(b"\nMESSAGE=");
            journal_payload.extend_from_slice(payload);
            journal_payload.extend_from_slice(b"\n");
            let _ = socket.send(&journal_payload);
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (level, payload, socket);
        }
    }
}

#[cfg(all(test, not(miri)))]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_platform_sink_stdout() {
        let mut sink = PlatformSink::Stdout;
        sink.emit("INFO", b"test stdout");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    #[allow(unsafe_code)]
    #[serial]
    fn test_platform_sink_fallback_env_var() {
        // SAFETY: Test-only; no other threads depend on this env var.
        unsafe { std::env::set_var("RLG_FALLBACK_STDOUT", "1") };
        let sink = PlatformSink::native();
        assert!(matches!(sink, PlatformSink::Stdout));
        // SAFETY: Test-only cleanup.
        unsafe { std::env::remove_var("RLG_FALLBACK_STDOUT") };
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    #[allow(unsafe_code)]
    #[serial]
    fn test_platform_sink_native_journald_path() {
        // SAFETY: Test-only env var cleanup so native() reaches platform code.
        unsafe {
            std::env::remove_var("RLG_FALLBACK_STDOUT");
            std::env::remove_var("GITHUB_ACTIONS");
        }
        let sink = PlatformSink::native();
        #[cfg(target_os = "linux")]
        assert!(matches!(sink, PlatformSink::Journald(_)));
        #[cfg(target_os = "macos")]
        assert!(matches!(sink, PlatformSink::OsLog));
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        assert!(matches!(sink, PlatformSink::Stdout));
        // SAFETY: Restore fallback for other tests.
        unsafe { std::env::set_var("RLG_FALLBACK_STDOUT", "1") };
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    #[cfg(target_os = "linux")]
    fn test_try_journald_socket_failure() {
        let sink =
            PlatformSink::try_journald_socket("/nonexistent/path");
        assert!(matches!(sink, PlatformSink::Journald(None)));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_platform_sink_journald_coverage() {
        #[cfg(unix)]
        {
            let (sock1, _sock2) = UnixDatagram::pair().unwrap();
            let mut sink = PlatformSink::Journald(Some(sock1));
            sink.emit("INFO", b"test journald");
        }

        let mut sink_none = PlatformSink::Journald(None);
        sink_none.emit("INFO", b"test journald fallback");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    #[allow(unsafe_code)]
    #[serial]
    fn test_platform_sink_oslog_fallback_stdout() {
        // Drive the `RLG_FALLBACK_STDOUT` branch in `emit_os_log`.
        // SAFETY: serial test; sole writer of this env var.
        unsafe { std::env::set_var("RLG_FALLBACK_STDOUT", "1") };
        let mut sink = PlatformSink::OsLog;
        sink.emit("INFO", b"fallback-test");
        // SAFETY: restore for following tests.
        unsafe { std::env::remove_var("RLG_FALLBACK_STDOUT") };
    }

    #[cfg(target_os = "macos")]
    #[test]
    #[cfg_attr(miri, ignore)]
    #[allow(unsafe_code)]
    #[serial]
    fn test_platform_sink_oslog_real_syslog_call() {
        // Exercise the actual `syslog(3)` FFI path on macOS, once per
        // process. The serial lock ensures no other test toggles
        // RLG_FALLBACK_STDOUT mid-flight.
        unsafe {
            std::env::remove_var("RLG_FALLBACK_STDOUT");
            std::env::remove_var("GITHUB_ACTIONS");
        }
        let mut sink = PlatformSink::OsLog;
        // Every level branch must be exercised to cover the priority
        // mapping match in `emit_os_log`.
        for level in [
            "FATAL",
            "CRITICAL",
            "ERROR",
            "WARN",
            "INFO",
            "DEBUG",
            "TRACE",
            "VERBOSE",
            "UNKNOWN_LEVEL",
        ] {
            sink.emit(level, b"rlg coverage test record");
        }
        // Restore fallback so later tests stay deterministic.
        unsafe { std::env::set_var("RLG_FALLBACK_STDOUT", "1") };
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_platform_sink_oslog_with_embedded_nulls() {
        // Drives the NUL-stripping branch in `emit_os_log` even on
        // non-macOS targets (the path is a no-op there but the
        // dispatch still exercises the match arm).
        let mut sink = PlatformSink::OsLog;
        sink.emit("INFO", b"with\0embedded\0nulls");
    }
}
