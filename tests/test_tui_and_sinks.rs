#![allow(missing_docs)]
use rlg::sink::PlatformSink;
use rlg::tui::{spawn_tui_thread, TuiMetrics};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[tokio::test]
async fn test_tui_thread_lifecycle() {
    let metrics = Arc::new(TuiMetrics::default());
    let shutdown = Arc::new(AtomicBool::new(false));

    metrics.inc_events();
    metrics.inc_errors();
    metrics.inc_spans();

    spawn_tui_thread(metrics.clone(), shutdown.clone());

    // Let it render a few frames
    thread::sleep(Duration::from_millis(50));

    shutdown.store(true, Ordering::SeqCst);
    thread::sleep(Duration::from_millis(20));
}

#[test]
fn test_platform_sink_stdout() {
    let mut sink = PlatformSink::Stdout;
    sink.emit("INFO", b"test stdout payload");
}

#[test]
fn test_platform_sink_native_creation() {
    let _sink = PlatformSink::native();
}

#[test]
fn test_platform_sink_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let log_file = temp_dir.path().join("test.log");
    let file = std::fs::File::create(&log_file).unwrap();
    let mut sink = PlatformSink::File(file);
    sink.emit("INFO", b"test file payload");
}

#[cfg(target_os = "macos")]
#[test]
fn test_platform_sink_oslog() {
    let mut sink = PlatformSink::OsLog;
    sink.emit("ERROR", b"test macos payload");
    sink.emit("INFO", b"test macos payload");
}

#[cfg(target_os = "linux")]
#[test]
fn test_platform_sink_journald_fallback() {
    let mut sink = PlatformSink::Journald(None);
    sink.emit("ERROR", b"test journald fallback");
}

#[cfg(target_os = "linux")]
#[test]
fn test_platform_sink_journald_valid() {
    use std::os::unix::net::UnixDatagram;
    if let Ok(socket) = UnixDatagram::unbound() {
        let mut sink = PlatformSink::Journald(Some(socket));
        sink.emit("WARN", b"test journald valid");
        sink.emit("DEBUG", b"test journald valid");
        sink.emit("FATAL", b"test journald valid");
        sink.emit("UNKNOWN", b"test journald valid");
    }
}
