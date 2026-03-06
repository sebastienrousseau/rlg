// tui.rs
// Generative Terminal UI (TUI) Dashboard for local development.

use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(not(windows))]
fn get_terminal_height() -> u16 {
    unsafe {
        let mut winsize = libc::winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        // SAFETY: 1 is the file descriptor for stdout. winsize is a valid pointer to a winsize struct.
        if libc::ioctl(1, libc::TIOCGWINSZ, &mut winsize) == 0 {
            winsize.ws_row
        } else {
            24 // Fallback
        }
    }
}

#[cfg(windows)]
fn get_terminal_height() -> u16 {
    24 // Fallback for windows if not using a virtual terminal
}

/// Live metrics tracked by the lock-free engine.
#[derive(Debug, Default)]
pub struct TuiMetrics {
    /// Total number of log events ingested.
    pub total_events: AtomicUsize,
    /// Number of error/fatal events.
    pub error_count: AtomicUsize,
    /// Number of active spans (OpenTelemetry style).
    pub active_spans: AtomicUsize,
    /// Calculated events per second.
    pub throughput: AtomicUsize,
}

impl TuiMetrics {
    /// Increments the total event count.
    pub fn inc_events(&self) {
        self.total_events.fetch_add(1, Ordering::Relaxed);
    }

    /// Increments the error count.
    pub fn inc_errors(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Increments active spans.
    pub fn inc_spans(&self) {
        self.active_spans.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrements active spans.
    pub fn dec_spans(&self) {
        self.active_spans.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Spawns the background "Liquid Glass" TUI renderer.
///
/// # Panics
///
/// This function panics if the TUI background thread fails to spawn.
pub fn spawn_tui_thread(
    metrics: Arc<TuiMetrics>,
    shutdown_flag: Arc<AtomicBool>,
) {
    thread::Builder::new()
        .name("rlg-tui".into())
        .spawn(move || {
            // Reserve bottom 4 lines for the dashboard using scroll region
            // ANSI: \x1b[s (save cursor) \x1b[1;{H-4}r (set scroll region)

            // For a robust cross-platform implementation, we assume a standard 24 line height
            // or rely on the terminal handling \x1b[;r to reset.
            // In a production Apple-standard system, we'd use libc::ioctl for TIOCGWINSZ.

            let mut last_total = 0;

            loop {
                if shutdown_flag.load(Ordering::Relaxed) {
                    break;
                }

                thread::sleep(Duration::from_millis(16)); // ~60 FPS

                let total = metrics.total_events.load(Ordering::Relaxed);
                let errors = metrics.error_count.load(Ordering::Relaxed);
                let spans = metrics.active_spans.load(Ordering::Relaxed);

                // Calculate throughput (events per ~16ms tick -> scale to second)
                let diff = total.saturating_sub(last_total);
                last_total = total;
                metrics.throughput.store(diff * 60, Ordering::Relaxed);
                let tps = metrics.throughput.load(Ordering::Relaxed);

                // Render "Liquid Glass" Dashboard at the bottom of the screen.
                // We use ANSI save/restore cursor to ensure stdout flows cleanly above.
                let mut stdout = std::io::stdout();
                let height = get_terminal_height();

                // ANSI escape sequence sequence:
                // \x1b7 : Save cursor position
                // \x1b[{height};1H : Move cursor to bottom-most row
                // \x1b[3A : Move up 3 lines
                // \x1b[J : Clear below
                let _ = write!(
                    stdout,
                    "\x1b7\x1b[{height};1H\x1b[3A\x1b[J
\x1b[38;5;33m[ \x1b[1;37mRLG Liquid Glass Dashboard \x1b[0;38;5;33m]\x1b[0m
\x1b[1mErrors:\x1b[0m {errors} | \x1b[1mActive Spans:\x1b[0m {spans} | \x1b[1mThroughput:\x1b[0m {tps} ev/s
\x1b[38;5;239m-------------------------------------------------\x1b[0m\x1b8"
                );
                let _ = stdout.flush();
            }

            // Clean up terminal state on exit
            let _ = write!(std::io::stdout(), "\x1b[r\x1b[J"); // Reset scroll region
        })
        .expect("Failed to spawn TUI thread");
}
