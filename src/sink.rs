// sink.rs
// Platform-native zero-overhead logging sinks.

use std::io::Write;

#[cfg(any(target_os = "linux", test))]
use std::os::unix::net::UnixDatagram;

/// A unified interface for platform-native logging.
#[derive(Debug)]
pub enum PlatformSink {
    /// Standard output fallback.
    Stdout,
    /// File sink fallback.
    File(std::fs::File),
    /// Native OS Log on macOS.
    OsLog,
    /// Systemd Journald socket on Linux.
    Journald(Option<UnixDatagram>),
}

#[cfg(any(target_os = "macos", test))]
mod macos_ffi {
    use std::os::raw::{c_char, c_void};
    #[allow(dead_code)]
    pub(crate) type os_log_t = *mut c_void;
    #[repr(transparent)]
    #[allow(dead_code)]
    pub(crate) struct os_log_type_t(pub(crate) u8);

    #[allow(dead_code)]
    pub(crate) const OS_LOG_TYPE_DEFAULT: os_log_type_t =
        os_log_type_t(0x00);
    #[allow(dead_code)]
    pub(crate) const OS_LOG_TYPE_INFO: os_log_type_t =
        os_log_type_t(0x01);
    #[allow(dead_code)]
    pub(crate) const OS_LOG_TYPE_DEBUG: os_log_type_t =
        os_log_type_t(0x02);
    #[allow(dead_code)]
    pub(crate) const OS_LOG_TYPE_ERROR: os_log_type_t =
        os_log_type_t(0x10);
    #[allow(dead_code)]
    pub(crate) const OS_LOG_TYPE_FAULT: os_log_type_t =
        os_log_type_t(0x11);

    extern "C" {
        #[allow(dead_code)]
        pub(crate) fn os_log_create(
            subsystem: *const c_char,
            category: *const c_char,
        ) -> os_log_t;
        #[allow(dead_code)]
        pub(crate) fn _os_log_impl(
            dso: *mut c_void,
            log: os_log_t,
            log_type: os_log_type_t,
            format: *const c_char,
            buf: *const u8,
            size: u32,
        );
    }
}

impl PlatformSink {
    /// Creates a native sink based on the OS.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn native() -> Self {
        // IBM-Standard Rigor: Provide an explicit escape hatch for high-compliance environments.
        if std::env::var("RLG_FALLBACK_STDOUT").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
        {
            return Self::Stdout;
        }

        #[cfg(target_os = "macos")]
        {
            Self::OsLog
        }
        #[cfg(all(target_os = "linux", not(test)))]
        {
            if let Ok(socket) = UnixDatagram::unbound() {
                if socket.connect("/run/systemd/journal/socket").is_ok()
                {
                    return Self::Journald(Some(socket));
                }
            }
            Self::Journald(None)
        }
        #[cfg(all(target_os = "linux", test))]
        {
            Self::Journald(None)
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Self::Stdout
        }
    }

    /// Emits a log payload via the native sink mechanism.
    #[allow(unused_variables)]
    pub fn emit(&mut self, level: &str, payload: &[u8]) {
        match self {
            Self::Stdout => {
                let _ = std::io::stdout().write_all(payload);
                let _ = std::io::stdout().write_all(b"\n");
            }
            Self::File(ref mut f) => {
                let _ = f.write_all(payload);
                let _ = f.write_all(b"\n");
            }
            Self::OsLog => {
                #[cfg(all(target_os = "macos", not(any(test, miri))))]
                {
                    use macos_ffi::*;
                    use std::ffi::CString;

                    let subsystem =
                        CString::new("com.rlg.logger").unwrap();
                    let category = CString::new("default").unwrap();

                    // SAFETY: The pointers passed to `os_log_create` and `_os_log_impl` are derived from
                    // valid, null-terminated `CString`s. The `buf` pointer is valid for `size` bytes.
                    unsafe {
                        let log_handle = os_log_create(
                            subsystem.as_ptr(),
                            category.as_ptr(),
                        );
                        let log_type = match level {
                            "ERROR" | "FATAL" => OS_LOG_TYPE_ERROR,
                            "CRITICAL" => OS_LOG_TYPE_FAULT,
                            "WARN" => OS_LOG_TYPE_DEFAULT,
                            "INFO" => OS_LOG_TYPE_INFO,
                            "DEBUG" | "TRACE" | "VERBOSE" => {
                                OS_LOG_TYPE_DEBUG
                            }
                            _ => OS_LOG_TYPE_DEFAULT,
                        };

                        let format =
                            CString::new("%{public}s").unwrap();
                        let msg =
                            CString::new(payload).unwrap_or_default();

                        _os_log_impl(
                            std::ptr::null_mut(),
                            log_handle,
                            log_type,
                            format.as_ptr(),
                            msg.as_ptr() as *const u8,
                            payload.len() as u32,
                        );
                    }
                }
                #[cfg(any(not(target_os = "macos"), test, miri))]
                {
                    let _ = (level, payload);
                }
            }
            Self::Journald(socket_opt) => {
                if let Some(socket) = socket_opt {
                    #[cfg(any(test, miri))]
                    let _ = socket;
                    let priority = match level {
                        "ERROR" | "FATAL" | "CRITICAL" => "3",
                        "WARN" => "4",
                        "INFO" => "6",
                        "DEBUG" | "TRACE" | "VERBOSE" => "7",
                        _ => "5",
                    };

                    // Journald expects newline-separated key-value pairs
                    let mut journal_payload =
                        Vec::with_capacity(payload.len() + 32);
                    journal_payload.extend_from_slice(b"PRIORITY=");
                    journal_payload
                        .extend_from_slice(priority.as_bytes());
                    journal_payload.extend_from_slice(b"\nMESSAGE=");
                    journal_payload.extend_from_slice(payload);
                    journal_payload.extend_from_slice(b"\n");

                    #[cfg(all(target_os = "linux", not(any(test, miri))))]
                    let _ = socket.send(&journal_payload);
                    #[cfg(any(not(target_os = "linux"), test, miri))]
                    {
                        let _ = journal_payload;
                    }
                } else {
                    let _ = std::io::stdout().write_all(payload);
                    let _ = std::io::stdout().write_all(b"\n");
                }
            }
        }
    }
}

#[cfg(all(test, not(miri)))]
mod tests {
    use super::*;
    #[cfg(target_os = "linux")]
    use std::os::unix::net::UnixDatagram;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_platform_sink_stdout() {
        let mut sink = PlatformSink::Stdout;
        sink.emit("INFO", b"test stdout");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_platform_sink_fallback_env_var() {
        std::env::set_var("RLG_FALLBACK_STDOUT", "1");
        let sink = PlatformSink::native();
        assert!(matches!(sink, PlatformSink::Stdout));
        std::env::remove_var("RLG_FALLBACK_STDOUT");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_platform_sink_journald_coverage() {
        #[cfg(target_os = "linux")]
        {
            let (sock1, _sock2) = UnixDatagram::pair().unwrap();
            let mut sink = PlatformSink::Journald(Some(sock1));
            sink.emit("INFO", b"test journald");

            let mut sink_none = PlatformSink::Journald(None);
            sink_none.emit("INFO", b"test journald fallback");
        }
    }
}
