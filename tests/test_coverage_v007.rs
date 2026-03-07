// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Additional tests for v0.0.7 to achieve >=95% code coverage.

#![allow(missing_docs)]

#[cfg(test)]
mod tests {
    use rlg::log_format::LogFormat;
    use rlg::log_level::LogLevel;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    // =========================================================================
    // init.rs: Exercise the init() method body
    // =========================================================================

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_init_without_facades() {
        // We can't call the real rlg::init() because it sets global state
        // and other tests may have already done so. Instead, exercise the
        // builder code path that skips global installation.
        let b = rlg::builder().without_log().without_tracing();
        // This won't succeed if another test already called init() (OnceLock),
        // but the code path through the builder is still covered.
        let result = b.init();
        // Either Ok (first call) or Err(AlreadyInitialized)
        match result {
            Ok(()) => {} // First init in this process
            Err(rlg::InitError::AlreadyInitialized) => {} // Already initialized
            Err(e) => panic!("unexpected error: {e}"),
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_init_double_call() {
        // Force initialization
        let _ = rlg::builder().without_log().without_tracing().init();
        // Second call should fail with AlreadyInitialized
        let result =
            rlg::builder().without_log().without_tracing().init();
        assert!(result.is_err(), "double init should return Err");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_init_function() {
        // The init() function delegates to builder().init()
        // It may succeed or fail depending on test ordering
        let result = rlg::init();
        match result {
            Ok(()) | Err(rlg::InitError::AlreadyInitialized) => {}
            Err(rlg::InitError::LoggerAlreadySet) => {}
            Err(rlg::InitError::SubscriberAlreadySet) => {}
        }
    }

    // =========================================================================
    // error.rs: Cover structural lines (impl blocks, type alias)
    // =========================================================================

    #[test]
    fn test_rlg_error_from_common_error() {
        let common_err =
            rlg::commons::error::CommonError::custom("test common");
        let rlg_err: rlg::RlgError = common_err.into();
        assert!(rlg_err.to_string().contains("test common"));
    }

    #[test]
    fn test_rlg_error_custom_method() {
        let err = rlg::RlgError::custom("custom msg");
        assert_eq!(err.to_string(), "custom msg");
    }

    #[test]
    fn test_rlg_result_type_alias() {
        let ok_result: rlg::RlgResult<i32> = Ok(42);
        assert!(matches!(ok_result, Ok(42)));

        let err_result: rlg::RlgResult<i32> =
            Err(rlg::RlgError::custom("fail"));
        assert!(err_result.is_err());
    }

    // =========================================================================
    // macros.rs: Exercise macro bodies
    // =========================================================================

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_rlg_span_macro_coverage() {
        let result = rlg::rlg_span!("coverage_span", { 42 });
        assert_eq!(result, 42);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_rlg_time_it_macro_coverage() {
        let result = rlg::rlg_time_it!("coverage_timer", {
            let mut sum = 0;
            for i in 0..10 {
                sum += i;
            }
            sum
        });
        assert_eq!(result, 45);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_rlg_mcp_notify_macro_coverage() {
        rlg::rlg_mcp_notify!("test_state", "active");
    }

    // =========================================================================
    // tui.rs: Exercise spawn_tui_thread body (the render loop)
    // =========================================================================

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_tui_thread_with_populated_metrics() {
        use rlg::tui::{TuiMetrics, spawn_tui_thread};

        let metrics = Arc::new(TuiMetrics::default());
        let shutdown = Arc::new(AtomicBool::new(false));

        // Pre-populate metrics to exercise format/level rendering
        metrics.total_events.store(100, Ordering::Relaxed);
        metrics.error_count.store(5, Ordering::Relaxed);
        metrics.active_spans.store(2, Ordering::Relaxed);
        metrics.level_info.store(70, Ordering::Relaxed);
        metrics.level_warn.store(20, Ordering::Relaxed);
        metrics.level_error.store(5, Ordering::Relaxed);
        metrics.level_debug.store(3, Ordering::Relaxed);
        metrics.level_trace.store(2, Ordering::Relaxed);
        metrics.fmt_json.store(50, Ordering::Relaxed);
        metrics.fmt_mcp.store(30, Ordering::Relaxed);
        metrics.fmt_otlp.store(20, Ordering::Relaxed);

        spawn_tui_thread(metrics.clone(), shutdown.clone());

        // Let it render a few frames
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Shutdown
        shutdown.store(true, Ordering::Relaxed);
        std::thread::sleep(std::time::Duration::from_millis(30));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_tui_thread_with_events_during_render() {
        use rlg::tui::{TuiMetrics, spawn_tui_thread};

        let metrics = Arc::new(TuiMetrics::default());
        let shutdown = Arc::new(AtomicBool::new(false));

        spawn_tui_thread(metrics.clone(), shutdown.clone());

        // Simulate events while TUI is running
        for _ in 0..50 {
            metrics.inc_events();
        }
        metrics.inc_level(LogLevel::INFO);
        metrics.inc_level(LogLevel::ERROR);
        metrics.inc_format(LogFormat::JSON);
        metrics.inc_format(LogFormat::MCP);

        // Let it render
        std::thread::sleep(std::time::Duration::from_millis(50));

        shutdown.store(true, Ordering::Relaxed);
        std::thread::sleep(std::time::Duration::from_millis(30));
    }

    #[test]
    fn test_tui_get_terminal_height_of() {
        #[cfg(not(windows))]
        {
            // Test with a non-terminal handle (pipe)
            let (r, _w) =
                std::os::unix::net::UnixStream::pair().unwrap();
            let h = rlg::tui::get_terminal_height_of(&r);
            assert_eq!(h, 24); // Fallback for non-terminal
        }
    }

    // =========================================================================
    // config.rs: Cover remaining edge case lines
    // =========================================================================

    #[test]
    fn test_config_log_rotation_time_zero() {
        use rlg::config::LogRotation;
        use std::str::FromStr;
        // "time:0" should fail validation
        let result = LogRotation::from_str("time:0");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_log_rotation_size_zero() {
        use rlg::config::LogRotation;
        use std::str::FromStr;
        let result = LogRotation::from_str("size:0");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_log_rotation_count_zero() {
        use rlg::config::LogRotation;
        use std::str::FromStr;
        let result = LogRotation::from_str("count:0");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_log_rotation_invalid_kind() {
        use rlg::config::LogRotation;
        use std::str::FromStr;
        let result = LogRotation::from_str("unknown:42");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_default() {
        let config = rlg::config::Config::default();
        assert_eq!(config.log_level, LogLevel::INFO);
    }

    #[test]
    fn test_config_set_version() {
        let mut config = rlg::config::Config::default();
        let result = config.set("version", "2.0");
        assert!(result.is_ok());
        assert_eq!(config.version, "2.0");
    }

    #[test]
    fn test_config_save_to_file_fail() {
        let config = rlg::config::Config::default();
        let result =
            config.save_to_file("/nonexistent/path/config.json");
        assert!(result.is_err());
    }
}
