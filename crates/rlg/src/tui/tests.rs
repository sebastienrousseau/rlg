// tests.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Unit tests for `tui.rs`.

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
    let filled = info_bar.chars().filter(|&c| c == '\u{2588}').count();
    assert_eq!(filled, 8);
    // error_bar should have 2 filled blocks
    let filled = error_bar.chars().filter(|&c| c == '\u{2588}').count();
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

    let _ = render_tick(&m, &mut last_total, &mut ring, &mut cursor);

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

    let _ = render_tick(&m, &mut last_total, &mut ring, &mut cursor);
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
    let _ = render_tick(&m, &mut last_total, &mut ring, &mut cursor);
    assert_eq!(m.peak_throughput.load(Ordering::Relaxed), 6000);

    // Second tick: 50 more events => tps = 3000
    m.total_events.store(150, Ordering::Relaxed);
    let _ = render_tick(&m, &mut last_total, &mut ring, &mut cursor);
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
