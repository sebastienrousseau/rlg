// tui.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Terminal UI dashboard for real-time metrics during local development.

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

/// Number of throughput samples in the sparkline ring buffer (one per tick).
const SPARKLINE_RING_SIZE: usize = 60;

/// TUI render interval in milliseconds (~60 FPS).
const TUI_TICK_INTERVAL_MS: u64 = 16;

/// Default terminal height when detection fails.
const DEFAULT_TERMINAL_HEIGHT: u16 = 24;

/// Default terminal width when detection fails.
const DEFAULT_TERMINAL_WIDTH: u16 = 80;

/// Width of level-distribution bar charts (in block characters).
const LEVEL_BAR_WIDTH: usize = 10;

#[cfg(not(windows))]
/// Returns the terminal height for the given handle, or 24 as fallback.
///
/// # Panics
///
/// This function does not panic.
#[must_use]
pub fn get_terminal_height_of(handle: &impl std::os::fd::AsFd) -> u16 {
    terminal_size::terminal_size_of(handle).map_or(
        DEFAULT_TERMINAL_HEIGHT,
        |(_, terminal_size::Height(h))| h,
    )
}

fn get_terminal_height() -> u16 {
    terminal_size::terminal_size().map_or(
        DEFAULT_TERMINAL_HEIGHT,
        |(_, terminal_size::Height(h))| h,
    )
}

/// Live metrics tracked by the lock-free engine.
#[repr(align(64))]
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
    /// Peak throughput (events per second).
    pub peak_throughput: AtomicUsize,
    /// Engine start time (epoch seconds).
    pub start_epoch_secs: AtomicUsize,

    // Per-level counters
    /// TRACE-level event count.
    pub level_trace: AtomicUsize,
    /// DEBUG-level event count.
    pub level_debug: AtomicUsize,
    /// INFO-level event count.
    pub level_info: AtomicUsize,
    /// WARN-level event count.
    pub level_warn: AtomicUsize,
    /// ERROR-level event count.
    pub level_error: AtomicUsize,
    /// FATAL-level event count.
    pub level_fatal: AtomicUsize,
    /// CRITICAL-level event count.
    pub level_critical: AtomicUsize,
    /// Number of events dropped due to full ring buffer.
    pub dropped_events: AtomicUsize,

    // Per-format counters
    /// CLF format count.
    pub fmt_clf: AtomicUsize,
    /// JSON format count.
    pub fmt_json: AtomicUsize,
    /// CEF format count.
    pub fmt_cef: AtomicUsize,
    /// ELF format count.
    pub fmt_elf: AtomicUsize,
    /// W3C format count.
    pub fmt_w3c: AtomicUsize,
    /// GELF format count.
    pub fmt_gelf: AtomicUsize,
    /// Apache Access Log format count.
    pub fmt_apache: AtomicUsize,
    /// Logstash format count.
    pub fmt_logstash: AtomicUsize,
    /// Log4j XML format count.
    pub fmt_log4j: AtomicUsize,
    /// NDJSON format count.
    pub fmt_ndjson: AtomicUsize,
    /// MCP format count.
    pub fmt_mcp: AtomicUsize,
    /// OTLP format count.
    pub fmt_otlp: AtomicUsize,
    /// Logfmt format count.
    pub fmt_logfmt: AtomicUsize,
    /// ECS format count.
    pub fmt_ecs: AtomicUsize,
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

    /// Increments the dropped event count.
    pub fn inc_dropped(&self) {
        self.dropped_events.fetch_add(1, Ordering::Relaxed);
    }

    /// Increments the counter for the given log level.
    pub fn inc_level(&self, level: crate::log_level::LogLevel) {
        use crate::log_level::LogLevel;
        match level {
            LogLevel::TRACE => {
                self.level_trace.fetch_add(1, Ordering::Relaxed);
            }
            LogLevel::DEBUG => {
                self.level_debug.fetch_add(1, Ordering::Relaxed);
            }
            LogLevel::INFO => {
                self.level_info.fetch_add(1, Ordering::Relaxed);
            }
            LogLevel::WARN => {
                self.level_warn.fetch_add(1, Ordering::Relaxed);
            }
            LogLevel::ERROR => {
                self.level_error.fetch_add(1, Ordering::Relaxed);
            }
            LogLevel::FATAL => {
                self.level_fatal.fetch_add(1, Ordering::Relaxed);
            }
            LogLevel::CRITICAL => {
                self.level_critical.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    /// Increments the counter for the given log format.
    pub fn inc_format(&self, format: crate::log_format::LogFormat) {
        use crate::log_format::LogFormat;
        match format {
            LogFormat::CLF => {
                self.fmt_clf.fetch_add(1, Ordering::Relaxed);
            }
            LogFormat::JSON => {
                self.fmt_json.fetch_add(1, Ordering::Relaxed);
            }
            LogFormat::CEF => {
                self.fmt_cef.fetch_add(1, Ordering::Relaxed);
            }
            LogFormat::ELF => {
                self.fmt_elf.fetch_add(1, Ordering::Relaxed);
            }
            LogFormat::W3C => {
                self.fmt_w3c.fetch_add(1, Ordering::Relaxed);
            }
            LogFormat::GELF => {
                self.fmt_gelf.fetch_add(1, Ordering::Relaxed);
            }
            LogFormat::ApacheAccessLog => {
                self.fmt_apache.fetch_add(1, Ordering::Relaxed);
            }
            LogFormat::Logstash => {
                self.fmt_logstash.fetch_add(1, Ordering::Relaxed);
            }
            LogFormat::Log4jXML => {
                self.fmt_log4j.fetch_add(1, Ordering::Relaxed);
            }
            LogFormat::NDJSON => {
                self.fmt_ndjson.fetch_add(1, Ordering::Relaxed);
            }
            LogFormat::MCP => {
                self.fmt_mcp.fetch_add(1, Ordering::Relaxed);
            }
            LogFormat::OTLP => {
                self.fmt_otlp.fetch_add(1, Ordering::Relaxed);
            }
            LogFormat::Logfmt => {
                self.fmt_logfmt.fetch_add(1, Ordering::Relaxed);
            }
            LogFormat::ECS => {
                self.fmt_ecs.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// Returns the terminal width, or `DEFAULT_TERMINAL_WIDTH` as fallback.
fn get_terminal_width() -> u16 {
    terminal_size::terminal_size().map_or(
        DEFAULT_TERMINAL_WIDTH,
        |(terminal_size::Width(w), _)| w,
    )
}

/// Sparkline characters indexed by intensity (0..=7).
const SPARK_CHARS: [char; 8] = [
    '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}',
    '\u{2586}', '\u{2587}', '\u{2588}',
];

/// Renders a sparkline string from a circular buffer of throughput samples.
fn render_sparkline(
    ring: &[usize; SPARKLINE_RING_SIZE],
    cursor: usize,
) -> String {
    let max_val = ring.iter().copied().max().unwrap_or(1).max(1);
    let mut out = String::with_capacity(SPARKLINE_RING_SIZE);
    for i in 0..SPARKLINE_RING_SIZE {
        let idx = (cursor + i) % SPARKLINE_RING_SIZE;
        let scaled = (ring[idx] * 7) / max_val;
        out.push(SPARK_CHARS[scaled.min(7)]);
    }
    out
}

/// Renders a level bar: filled blocks + empty blocks, `LEVEL_BAR_WIDTH` chars wide.
fn render_level_bar(count: usize, total: usize) -> String {
    if total == 0 {
        return "\u{2591}".repeat(LEVEL_BAR_WIDTH);
    }
    let filled =
        ((count * LEVEL_BAR_WIDTH) / total).min(LEVEL_BAR_WIDTH);
    let mut bar = String::with_capacity(LEVEL_BAR_WIDTH * 3);
    for _ in 0..filled {
        bar.push('\u{2588}');
    }
    for _ in filled..LEVEL_BAR_WIDTH {
        bar.push('\u{2591}');
    }
    bar
}

/// Formats an uptime duration as HH:MM:SS.
fn format_uptime(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

/// Builds the format counts line from metrics.
pub fn build_fmt_line(metrics: &TuiMetrics) -> String {
    let fmt_counts: Vec<(&str, usize)> = [
        ("CLF", metrics.fmt_clf.load(Ordering::Relaxed)),
        ("JSON", metrics.fmt_json.load(Ordering::Relaxed)),
        ("CEF", metrics.fmt_cef.load(Ordering::Relaxed)),
        ("ELF", metrics.fmt_elf.load(Ordering::Relaxed)),
        ("W3C", metrics.fmt_w3c.load(Ordering::Relaxed)),
        ("GELF", metrics.fmt_gelf.load(Ordering::Relaxed)),
        ("Apache", metrics.fmt_apache.load(Ordering::Relaxed)),
        ("Logstash", metrics.fmt_logstash.load(Ordering::Relaxed)),
        ("Log4j", metrics.fmt_log4j.load(Ordering::Relaxed)),
        ("NDJSON", metrics.fmt_ndjson.load(Ordering::Relaxed)),
        ("MCP", metrics.fmt_mcp.load(Ordering::Relaxed)),
        ("OTLP", metrics.fmt_otlp.load(Ordering::Relaxed)),
        ("Logfmt", metrics.fmt_logfmt.load(Ordering::Relaxed)),
        ("ECS", metrics.fmt_ecs.load(Ordering::Relaxed)),
    ]
    .into_iter()
    .filter(|(_, c)| *c > 0)
    .collect();

    let mut fmt_line = String::new();
    for (i, (name, count)) in fmt_counts.iter().enumerate() {
        use std::fmt::Write as _;
        if i > 0 {
            fmt_line.push_str(" | ");
        }
        let _ = write!(fmt_line, "{name}: {count}");
    }
    if fmt_line.is_empty() {
        fmt_line.push_str("(none)");
    }
    fmt_line
}

/// Computes level bar rendering from metrics.
///
/// Returns `(info_bar, info_pct, error_bar, error_pct)`.
pub fn compute_level_bars(
    metrics: &TuiMetrics,
) -> (String, usize, String, usize) {
    let info_c = metrics.level_info.load(Ordering::Relaxed);
    let warn_c = metrics.level_warn.load(Ordering::Relaxed);
    let error_c = metrics.level_error.load(Ordering::Relaxed);
    let debug_c = metrics.level_debug.load(Ordering::Relaxed);
    let trace_c = metrics.level_trace.load(Ordering::Relaxed);

    let level_total = info_c + warn_c + error_c + debug_c + trace_c;

    let info_bar = render_level_bar(info_c, level_total.max(1));
    let info_pct = if level_total > 0 {
        (info_c * 100) / level_total
    } else {
        0
    };
    let error_bar = render_level_bar(error_c, level_total.max(1));
    let error_pct = if level_total > 0 {
        (error_c * 100) / level_total
    } else {
        0
    };

    (info_bar, info_pct, error_bar, error_pct)
}

/// Performs one tick of the TUI dashboard, returning the ANSI-formatted frame.
///
/// This function is extracted from the render loop for testability.
#[allow(clippy::cast_possible_truncation)]
pub fn render_tick(
    metrics: &TuiMetrics,
    last_total: &mut usize,
    sparkline_ring: &mut [usize; SPARKLINE_RING_SIZE],
    spark_cursor: &mut usize,
) -> String {
    let total = metrics.total_events.load(Ordering::Relaxed);
    let errors = metrics.error_count.load(Ordering::Relaxed);
    let spans = metrics.active_spans.load(Ordering::Relaxed);
    let dropped = metrics.dropped_events.load(Ordering::Relaxed);

    // Calculate throughput (events per ~16ms tick -> scale to second)
    let diff = total.saturating_sub(*last_total);
    *last_total = total;
    let tps = diff * 60;
    metrics.throughput.store(tps, Ordering::Relaxed);

    // Track peak throughput
    let _ = metrics.peak_throughput.fetch_max(tps, Ordering::Relaxed);
    let peak = metrics.peak_throughput.load(Ordering::Relaxed);

    // Update sparkline ring buffer
    sparkline_ring[*spark_cursor % SPARKLINE_RING_SIZE] = tps;
    *spark_cursor = spark_cursor.wrapping_add(1);

    // Uptime
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as usize;
    let uptime_secs = now_secs.saturating_sub(
        metrics.start_epoch_secs.load(Ordering::Relaxed),
    );
    let uptime = format_uptime(uptime_secs as u64);

    // Level bars
    let (info_bar, info_pct, error_bar, error_pct) =
        compute_level_bars(metrics);

    // Format counts line
    let fmt_line = build_fmt_line(metrics);

    let sparkline = render_sparkline(
        sparkline_ring,
        *spark_cursor % SPARKLINE_RING_SIZE,
    );

    let total_fmt = format_with_commas(total);

    let width = get_terminal_width() as usize;
    let separator: String =
        "\u{2500}".repeat(width.min(SPARKLINE_RING_SIZE));

    let height = get_terminal_height();

    format!(
        "\x1b7\x1b[{height};1H\x1b[7A\x1b[J\
\x1b[38;5;33m[ \x1b[1;37mRLG Liquid Glass Dashboard \x1b[0;38;5;33m]\x1b[0m\n\
\x1b[1mErrors:\x1b[0m {errors} | \x1b[1mDropped:\x1b[0m {dropped} | \x1b[1mActive Spans:\x1b[0m {spans} | \x1b[1mThroughput:\x1b[0m {tps} ev/s\n\
\x1b[1mPeak:\x1b[0m {peak} ev/s | \x1b[1mUptime:\x1b[0m {uptime} | \x1b[1mTotal:\x1b[0m {total_fmt}\n\
\x1b[1mThroughput\x1b[0m {sparkline}\n\
\x1b[1mLevels:\x1b[0m {info_bar} {info_pct}% INFO  {error_bar} {error_pct}% ERROR\n\
\x1b[1mFormats:\x1b[0m {fmt_line}\n\
\x1b[38;5;239m{separator}\x1b[0m\x1b8"
    )
}

/// Spawns the background TUI renderer thread.
///
/// # Panics
///
/// This function panics if the TUI background thread fails to spawn.
pub fn spawn_tui_thread(
    metrics: Arc<TuiMetrics>,
    shutdown_flag: Arc<AtomicBool>,
) {
    // Record engine start time
    #[allow(clippy::cast_possible_truncation)]
    let start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as usize;
    metrics.start_epoch_secs.store(start, Ordering::Relaxed);

    thread::Builder::new()
        .name("rlg-tui".into())
        .spawn(move || {
            let mut last_total: usize = 0;
            let mut sparkline_ring = [0_usize; SPARKLINE_RING_SIZE];
            let mut spark_cursor: usize = 0;

            loop {
                if shutdown_flag.load(Ordering::Relaxed) {
                    break;
                }

                thread::sleep(Duration::from_millis(
                    TUI_TICK_INTERVAL_MS,
                ));

                let frame = render_tick(
                    &metrics,
                    &mut last_total,
                    &mut sparkline_ring,
                    &mut spark_cursor,
                );

                let mut stdout = std::io::stdout();
                let _ = write!(stdout, "{frame}");
                let _ = stdout.flush();
            }

            // Clean up terminal state on exit
            let _ = write!(std::io::stdout(), "\x1b[r\x1b[J");
        })
        .expect("Failed to spawn TUI thread");
}

/// Formats an integer with comma separators.
fn format_with_commas(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(ch);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_with_commas_zero() {
        assert_eq!(format_with_commas(0), "0");
    }

    #[test]
    fn test_format_with_commas_small() {
        assert_eq!(format_with_commas(42), "42");
        assert_eq!(format_with_commas(999), "999");
    }

    #[test]
    fn test_format_with_commas_thousands() {
        assert_eq!(format_with_commas(1000), "1,000");
        assert_eq!(format_with_commas(1_234), "1,234");
        assert_eq!(format_with_commas(999_999), "999,999");
    }

    #[test]
    fn test_format_with_commas_millions() {
        assert_eq!(format_with_commas(1_000_000), "1,000,000");
        assert_eq!(format_with_commas(1_234_567), "1,234,567");
    }

    #[test]
    fn test_format_uptime_zero() {
        assert_eq!(format_uptime(0), "00:00:00");
    }

    #[test]
    fn test_format_uptime_seconds() {
        assert_eq!(format_uptime(45), "00:00:45");
    }

    #[test]
    fn test_format_uptime_minutes() {
        assert_eq!(format_uptime(125), "00:02:05");
    }

    #[test]
    fn test_format_uptime_hours() {
        assert_eq!(format_uptime(3661), "01:01:01");
        assert_eq!(format_uptime(86399), "23:59:59");
    }

    #[test]
    fn test_render_level_bar_zero_total() {
        let bar = render_level_bar(0, 0);
        assert_eq!(bar.chars().count(), 10);
        // All empty blocks
        assert!(bar.chars().all(|c| c == '\u{2591}'));
    }

    #[test]
    fn test_render_level_bar_full() {
        let bar = render_level_bar(100, 100);
        assert_eq!(bar.chars().count(), 10);
        assert!(bar.chars().all(|c| c == '\u{2588}'));
    }

    #[test]
    fn test_render_level_bar_half() {
        let bar = render_level_bar(50, 100);
        assert_eq!(bar.chars().count(), 10);
        let filled = bar.chars().filter(|&c| c == '\u{2588}').count();
        assert_eq!(filled, 5);
    }

    #[test]
    fn test_render_level_bar_empty() {
        let bar = render_level_bar(0, 100);
        assert_eq!(bar.chars().count(), 10);
        assert!(bar.chars().all(|c| c == '\u{2591}'));
    }

    #[test]
    fn test_render_sparkline_empty() {
        let ring = [0_usize; SPARKLINE_RING_SIZE];
        let sparkline = render_sparkline(&ring, 0);
        assert_eq!(sparkline.chars().count(), SPARKLINE_RING_SIZE);
        // All minimum bars since all values are 0
        assert!(sparkline.chars().all(|c| c == '\u{2581}'));
    }

    #[test]
    fn test_render_sparkline_uniform() {
        let ring = [100_usize; SPARKLINE_RING_SIZE];
        let sparkline = render_sparkline(&ring, 0);
        assert_eq!(sparkline.chars().count(), SPARKLINE_RING_SIZE);
    }

    #[test]
    fn test_render_sparkline_varied() {
        let mut ring = [0_usize; SPARKLINE_RING_SIZE];
        ring[0] = 100;
        ring[30] = 50;
        ring[59] = 25;
        let sparkline = render_sparkline(&ring, 0);
        assert_eq!(sparkline.chars().count(), SPARKLINE_RING_SIZE);
    }

    #[test]
    fn test_render_sparkline_cursor_wrap() {
        let mut ring = [0_usize; SPARKLINE_RING_SIZE];
        ring[55] = 10;
        ring[5] = 20;
        let sparkline = render_sparkline(&ring, 50);
        assert_eq!(sparkline.chars().count(), SPARKLINE_RING_SIZE);
    }

    #[test]
    fn test_get_terminal_width() {
        // In test/CI environments this typically returns 80 fallback
        let w = get_terminal_width();
        assert!(w > 0);
    }

    #[test]
    fn test_tui_metrics_inc_level_all_variants() {
        let m = TuiMetrics::default();

        m.inc_level(crate::log_level::LogLevel::TRACE);
        assert_eq!(m.level_trace.load(Ordering::Relaxed), 1);

        m.inc_level(crate::log_level::LogLevel::DEBUG);
        assert_eq!(m.level_debug.load(Ordering::Relaxed), 1);

        m.inc_level(crate::log_level::LogLevel::INFO);
        assert_eq!(m.level_info.load(Ordering::Relaxed), 1);

        m.inc_level(crate::log_level::LogLevel::WARN);
        assert_eq!(m.level_warn.load(Ordering::Relaxed), 1);

        m.inc_level(crate::log_level::LogLevel::ERROR);
        assert_eq!(m.level_error.load(Ordering::Relaxed), 1);

        m.inc_level(crate::log_level::LogLevel::FATAL);
        assert_eq!(m.level_fatal.load(Ordering::Relaxed), 1);

        m.inc_level(crate::log_level::LogLevel::CRITICAL);
        assert_eq!(m.level_critical.load(Ordering::Relaxed), 1);

        // Non-tracked levels should not panic
        m.inc_level(crate::log_level::LogLevel::ALL);
        m.inc_level(crate::log_level::LogLevel::NONE);
    }

    #[test]
    fn test_tui_metrics_inc_format_all_variants() {
        let m = TuiMetrics::default();

        m.inc_format(crate::log_format::LogFormat::CLF);
        assert_eq!(m.fmt_clf.load(Ordering::Relaxed), 1);

        m.inc_format(crate::log_format::LogFormat::JSON);
        assert_eq!(m.fmt_json.load(Ordering::Relaxed), 1);

        m.inc_format(crate::log_format::LogFormat::CEF);
        assert_eq!(m.fmt_cef.load(Ordering::Relaxed), 1);

        m.inc_format(crate::log_format::LogFormat::ELF);
        assert_eq!(m.fmt_elf.load(Ordering::Relaxed), 1);

        m.inc_format(crate::log_format::LogFormat::W3C);
        assert_eq!(m.fmt_w3c.load(Ordering::Relaxed), 1);

        m.inc_format(crate::log_format::LogFormat::GELF);
        assert_eq!(m.fmt_gelf.load(Ordering::Relaxed), 1);

        m.inc_format(crate::log_format::LogFormat::ApacheAccessLog);
        assert_eq!(m.fmt_apache.load(Ordering::Relaxed), 1);

        m.inc_format(crate::log_format::LogFormat::Logstash);
        assert_eq!(m.fmt_logstash.load(Ordering::Relaxed), 1);

        m.inc_format(crate::log_format::LogFormat::Log4jXML);
        assert_eq!(m.fmt_log4j.load(Ordering::Relaxed), 1);

        m.inc_format(crate::log_format::LogFormat::NDJSON);
        assert_eq!(m.fmt_ndjson.load(Ordering::Relaxed), 1);

        m.inc_format(crate::log_format::LogFormat::MCP);
        assert_eq!(m.fmt_mcp.load(Ordering::Relaxed), 1);

        m.inc_format(crate::log_format::LogFormat::OTLP);
        assert_eq!(m.fmt_otlp.load(Ordering::Relaxed), 1);

        m.inc_format(crate::log_format::LogFormat::Logfmt);
        assert_eq!(m.fmt_logfmt.load(Ordering::Relaxed), 1);

        m.inc_format(crate::log_format::LogFormat::ECS);
        assert_eq!(m.fmt_ecs.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_tui_metrics_peak_throughput() {
        let m = TuiMetrics::default();
        m.peak_throughput.store(100, Ordering::Relaxed);
        assert_eq!(m.peak_throughput.load(Ordering::Relaxed), 100);
    }

    #[test]
    fn test_tui_metrics_start_epoch() {
        let m = TuiMetrics::default();
        m.start_epoch_secs.store(1_234_567_890, Ordering::Relaxed);
        assert_eq!(
            m.start_epoch_secs.load(Ordering::Relaxed),
            1_234_567_890
        );
    }

    #[test]
    fn test_spark_chars_length() {
        assert_eq!(SPARK_CHARS.len(), 8);
    }

    #[test]
    fn test_build_fmt_line_empty() {
        let m = TuiMetrics::default();
        let line = build_fmt_line(&m);
        assert_eq!(line, "(none)");
    }

    #[test]
    fn test_build_fmt_line_single() {
        let m = TuiMetrics::default();
        m.fmt_json.store(42, Ordering::Relaxed);
        let line = build_fmt_line(&m);
        assert_eq!(line, "JSON: 42");
    }

    #[test]
    fn test_build_fmt_line_multiple() {
        let m = TuiMetrics::default();
        m.fmt_json.store(10, Ordering::Relaxed);
        m.fmt_mcp.store(20, Ordering::Relaxed);
        m.fmt_otlp.store(5, Ordering::Relaxed);
        let line = build_fmt_line(&m);
        assert!(line.contains("JSON: 10"));
        assert!(line.contains("MCP: 20"));
        assert!(line.contains("OTLP: 5"));
        assert!(line.contains(" | "));
    }

    #[test]
    fn test_build_fmt_line_all_formats() {
        let m = TuiMetrics::default();
        m.fmt_clf.store(1, Ordering::Relaxed);
        m.fmt_json.store(2, Ordering::Relaxed);
        m.fmt_cef.store(3, Ordering::Relaxed);
        m.fmt_elf.store(4, Ordering::Relaxed);
        m.fmt_w3c.store(5, Ordering::Relaxed);
        m.fmt_gelf.store(6, Ordering::Relaxed);
        m.fmt_apache.store(7, Ordering::Relaxed);
        m.fmt_logstash.store(8, Ordering::Relaxed);
        m.fmt_log4j.store(9, Ordering::Relaxed);
        m.fmt_ndjson.store(10, Ordering::Relaxed);
        m.fmt_mcp.store(11, Ordering::Relaxed);
        m.fmt_otlp.store(12, Ordering::Relaxed);
        m.fmt_logfmt.store(13, Ordering::Relaxed);
        m.fmt_ecs.store(14, Ordering::Relaxed);
        let line = build_fmt_line(&m);
        assert!(line.contains("CLF: 1"));
        assert!(line.contains("ECS: 14"));
    }

    #[test]
    fn test_compute_level_bars_empty() {
        let m = TuiMetrics::default();
        let (info_bar, info_pct, error_bar, error_pct) =
            compute_level_bars(&m);
        assert_eq!(info_pct, 0);
        assert_eq!(error_pct, 0);
        assert_eq!(info_bar.chars().count(), 10);
        assert_eq!(error_bar.chars().count(), 10);
    }

    #[test]
    fn test_compute_level_bars_with_data() {
        let m = TuiMetrics::default();
        m.level_info.store(80, Ordering::Relaxed);
        m.level_error.store(20, Ordering::Relaxed);
        let (info_bar, info_pct, error_bar, error_pct) =
            compute_level_bars(&m);
        assert_eq!(info_pct, 80);
        assert_eq!(error_pct, 20);
        // info_bar should have 8 filled blocks
        let filled =
            info_bar.chars().filter(|&c| c == '\u{2588}').count();
        assert_eq!(filled, 8);
        // error_bar should have 2 filled blocks
        let filled =
            error_bar.chars().filter(|&c| c == '\u{2588}').count();
        assert_eq!(filled, 2);
    }

    #[test]
    fn test_compute_level_bars_all_levels() {
        let m = TuiMetrics::default();
        m.level_info.store(50, Ordering::Relaxed);
        m.level_warn.store(20, Ordering::Relaxed);
        m.level_error.store(10, Ordering::Relaxed);
        m.level_debug.store(15, Ordering::Relaxed);
        m.level_trace.store(5, Ordering::Relaxed);
        let (_info_bar, info_pct, _error_bar, error_pct) =
            compute_level_bars(&m);
        assert_eq!(info_pct, 50);
        assert_eq!(error_pct, 10);
    }

    #[test]
    fn test_render_tick_basic() {
        let m = TuiMetrics::default();
        m.total_events.store(100, Ordering::Relaxed);
        m.error_count.store(5, Ordering::Relaxed);
        m.active_spans.store(2, Ordering::Relaxed);
        m.level_info.store(80, Ordering::Relaxed);
        m.level_error.store(20, Ordering::Relaxed);
        m.fmt_json.store(50, Ordering::Relaxed);
        m.fmt_mcp.store(30, Ordering::Relaxed);

        #[allow(clippy::cast_possible_truncation)]
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as usize;
        m.start_epoch_secs.store(now, Ordering::Relaxed);

        let mut last_total = 0_usize;
        let mut ring = [0_usize; SPARKLINE_RING_SIZE];
        let mut cursor = 0_usize;

        let frame =
            render_tick(&m, &mut last_total, &mut ring, &mut cursor);
        assert!(frame.contains("RLG Liquid Glass Dashboard"));
        assert!(frame.contains("Errors:"));
        assert!(frame.contains("Active Spans:"));
        assert!(frame.contains("Throughput"));
        assert!(frame.contains("Peak:"));
        assert!(frame.contains("Uptime:"));
        assert!(frame.contains("Levels:"));
        assert!(frame.contains("Formats:"));
        assert!(frame.contains("JSON: 50"));
        assert!(frame.contains("MCP: 30"));
    }

    #[test]
    fn test_render_tick_updates_state() {
        let m = TuiMetrics::default();
        m.total_events.store(100, Ordering::Relaxed);
        m.start_epoch_secs.store(0, Ordering::Relaxed);

        let mut last_total = 0_usize;
        let mut ring = [0_usize; SPARKLINE_RING_SIZE];
        let mut cursor = 0_usize;

        let _ =
            render_tick(&m, &mut last_total, &mut ring, &mut cursor);

        // last_total should be updated
        assert_eq!(last_total, 100);
        // cursor should be incremented
        assert_eq!(cursor, 1);
        // ring[0] should have the throughput value
        assert_eq!(ring[0], 100 * 60); // diff * 60
        // throughput metric should be stored
        assert_eq!(m.throughput.load(Ordering::Relaxed), 100 * 60);
    }

    #[test]
    fn test_render_tick_no_diff() {
        let m = TuiMetrics::default();
        m.total_events.store(50, Ordering::Relaxed);
        m.start_epoch_secs.store(0, Ordering::Relaxed);

        let mut last_total = 50_usize;
        let mut ring = [0_usize; SPARKLINE_RING_SIZE];
        let mut cursor = 0_usize;

        let _ =
            render_tick(&m, &mut last_total, &mut ring, &mut cursor);
        assert_eq!(m.throughput.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_render_tick_peak_tracking() {
        let m = TuiMetrics::default();
        m.start_epoch_secs.store(0, Ordering::Relaxed);

        let mut last_total = 0_usize;
        let mut ring = [0_usize; SPARKLINE_RING_SIZE];
        let mut cursor = 0_usize;

        // First tick: 100 events => tps = 6000
        m.total_events.store(100, Ordering::Relaxed);
        let _ =
            render_tick(&m, &mut last_total, &mut ring, &mut cursor);
        assert_eq!(m.peak_throughput.load(Ordering::Relaxed), 6000);

        // Second tick: 50 more events => tps = 3000
        m.total_events.store(150, Ordering::Relaxed);
        let _ =
            render_tick(&m, &mut last_total, &mut ring, &mut cursor);
        // Peak should still be 6000
        assert_eq!(m.peak_throughput.load(Ordering::Relaxed), 6000);
    }

    #[test]
    fn test_render_tick_no_formats() {
        let m = TuiMetrics::default();
        m.start_epoch_secs.store(0, Ordering::Relaxed);

        let mut last_total = 0_usize;
        let mut ring = [0_usize; SPARKLINE_RING_SIZE];
        let mut cursor = 0_usize;

        let frame =
            render_tick(&m, &mut last_total, &mut ring, &mut cursor);
        assert!(frame.contains("(none)"));
    }

    #[test]
    fn test_get_terminal_height() {
        let h = get_terminal_height();
        assert!(h > 0);
    }

    #[test]
    fn test_tui_metrics_dropped_events() {
        let m = TuiMetrics::default();
        assert_eq!(m.dropped_events.load(Ordering::Relaxed), 0);
        m.inc_dropped();
        m.inc_dropped();
        assert_eq!(m.dropped_events.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_render_tick_shows_dropped() {
        let m = TuiMetrics::default();
        m.start_epoch_secs.store(0, Ordering::Relaxed);
        m.dropped_events.store(42, Ordering::Relaxed);

        let mut last_total = 0_usize;
        let mut ring = [0_usize; SPARKLINE_RING_SIZE];
        let mut cursor = 0_usize;

        let frame =
            render_tick(&m, &mut last_total, &mut ring, &mut cursor);
        assert!(frame.contains("Dropped:"));
        assert!(frame.contains("42"));
    }
}
