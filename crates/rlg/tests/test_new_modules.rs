#![allow(missing_docs)]
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Integration tests for the new modules added in v0.0.7:
//! - `logger.rs` (RlgLogger, log facade bridge)
//! - `init.rs` (zero-config init API)
//! - `tracing.rs` (RlgLayer behind `tracing-layer` feature)
//! - `engine.rs` (Debug impl, inc_format)

#[cfg(test)]
mod tests {
    use rlg::engine::{ENGINE, LockFreeEngine, LogEvent};
    use rlg::log::Log;
    use rlg::log_format::LogFormat;
    use rlg::log_level::LogLevel;

    // =========================================================================
    // engine.rs: Debug impl coverage
    // =========================================================================

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_engine_debug_impl() {
        // Force LazyLock initialization by accessing the engine
        ENGINE.set_filter(0);
        // LazyLock<T> Debug shows the inner T once initialized
        let dbg = format!("{ENGINE:?}");
        assert!(dbg.contains("LockFreeEngine"));
        assert!(dbg.contains("queue"));
        assert!(dbg.contains("shutdown_flag"));
        assert!(dbg.contains("metrics"));
        assert!(dbg.contains("filter_level"));
        assert!(dbg.contains("flusher_thread"));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_engine_debug_custom_instance() {
        let engine = LockFreeEngine::new(16);
        let dbg = format!("{engine:?}");
        assert!(dbg.contains("LockFreeEngine"));
        engine.shutdown();
    }

    // =========================================================================
    // engine.rs: inc_format coverage
    // =========================================================================

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_engine_inc_format() {
        ENGINE.inc_format(LogFormat::JSON);
        ENGINE.inc_format(LogFormat::MCP);
        ENGINE.inc_format(LogFormat::OTLP);
    }

    // =========================================================================
    // logger.rs: RlgLogger integration
    // =========================================================================

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_rlg_logger_via_log_trait() {
        use rlg::logger::RlgLogger;

        let logger = RlgLogger::new(LogFormat::JSON);

        // Test enabled check
        let meta = log::MetadataBuilder::new()
            .level(log::Level::Info)
            .target("test")
            .build();
        assert!(log::Log::enabled(&logger, &meta));

        // Test log with all optional fields
        let record = log::RecordBuilder::new()
            .args(format_args!("integration test msg"))
            .level(log::Level::Error)
            .target("integration::test")
            .file(Some("test.rs"))
            .line(Some(100))
            .module_path(Some("integration::test"))
            .build();
        log::Log::log(&logger, &record);

        // Test flush (no-op)
        log::Log::flush(&logger);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_rlg_logger_log_without_optional_fields() {
        use rlg::logger::RlgLogger;

        let logger = RlgLogger::new(LogFormat::MCP);

        // Record with no file/line/module
        let record = log::RecordBuilder::new()
            .args(format_args!("no optional fields"))
            .level(log::Level::Warn)
            .target("bare_target")
            .build();
        log::Log::log(&logger, &record);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_rlg_logger_all_log_levels() {
        use rlg::logger::RlgLogger;

        let logger = RlgLogger::new(LogFormat::JSON);
        for level in &[
            log::Level::Error,
            log::Level::Warn,
            log::Level::Info,
            log::Level::Debug,
            log::Level::Trace,
        ] {
            let record = log::RecordBuilder::new()
                .args(format_args!("level test"))
                .level(*level)
                .target("level_test")
                .build();
            log::Log::log(&logger, &record);
        }
    }

    #[test]
    fn test_map_log_level() {
        use rlg::logger::map_log_level;
        assert_eq!(map_log_level(log::Level::Error), LogLevel::ERROR);
        assert_eq!(map_log_level(log::Level::Warn), LogLevel::WARN);
        assert_eq!(map_log_level(log::Level::Info), LogLevel::INFO);
        assert_eq!(map_log_level(log::Level::Debug), LogLevel::DEBUG);
        assert_eq!(map_log_level(log::Level::Trace), LogLevel::TRACE);
    }

    #[test]
    fn test_to_log_level_filter() {
        use rlg::logger::to_log_level_filter;
        assert_eq!(
            to_log_level_filter(LogLevel::ALL),
            log::LevelFilter::Trace
        );
        assert_eq!(
            to_log_level_filter(LogLevel::TRACE),
            log::LevelFilter::Trace
        );
        assert_eq!(
            to_log_level_filter(LogLevel::DEBUG),
            log::LevelFilter::Debug
        );
        assert_eq!(
            to_log_level_filter(LogLevel::VERBOSE),
            log::LevelFilter::Info
        );
        assert_eq!(
            to_log_level_filter(LogLevel::INFO),
            log::LevelFilter::Info
        );
        assert_eq!(
            to_log_level_filter(LogLevel::WARN),
            log::LevelFilter::Warn
        );
        assert_eq!(
            to_log_level_filter(LogLevel::ERROR),
            log::LevelFilter::Error
        );
        assert_eq!(
            to_log_level_filter(LogLevel::FATAL),
            log::LevelFilter::Error
        );
        assert_eq!(
            to_log_level_filter(LogLevel::CRITICAL),
            log::LevelFilter::Error
        );
        assert_eq!(
            to_log_level_filter(LogLevel::NONE),
            log::LevelFilter::Off
        );
        assert_eq!(
            to_log_level_filter(LogLevel::DISABLED),
            log::LevelFilter::Off
        );
    }

    // =========================================================================
    // init.rs: Builder pattern and InitError
    // =========================================================================

    #[test]
    fn test_init_error_display_all() {
        use rlg::InitError;
        assert_eq!(
            InitError::LoggerAlreadySet.to_string(),
            "a log crate logger was already set"
        );
        assert_eq!(
            InitError::SubscriberAlreadySet.to_string(),
            "a tracing subscriber was already set"
        );
        assert_eq!(
            InitError::AlreadyInitialized.to_string(),
            "rlg was already initialized"
        );
    }

    #[test]
    fn test_init_error_debug() {
        use rlg::InitError;
        assert_eq!(
            format!("{:?}", InitError::LoggerAlreadySet),
            "LoggerAlreadySet"
        );
        assert_eq!(
            format!("{:?}", InitError::SubscriberAlreadySet),
            "SubscriberAlreadySet"
        );
        assert_eq!(
            format!("{:?}", InitError::AlreadyInitialized),
            "AlreadyInitialized"
        );
    }

    #[test]
    fn test_init_error_is_std_error() {
        use rlg::InitError;
        let err = InitError::LoggerAlreadySet;
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_init_error_clone_copy() {
        use rlg::InitError;
        let err = InitError::AlreadyInitialized;
        let copy = err;
        assert_eq!(format!("{err:?}"), format!("{copy:?}"));
    }

    #[test]
    fn test_rlg_builder_defaults() {
        let b = rlg::builder();
        let dbg = format!("{b:?}");
        assert!(dbg.contains("INFO"));
        // Format is auto-detected (Logfmt for TTY, JSON for pipe/CI)
        assert!(dbg.contains("JSON") || dbg.contains("Logfmt"));
        assert!(dbg.contains("install_log: true"));
        assert!(dbg.contains("install_tracing: true"));
    }

    #[test]
    fn test_rlg_builder_level() {
        let b = rlg::builder().level(LogLevel::DEBUG);
        assert!(format!("{b:?}").contains("DEBUG"));
    }

    #[test]
    fn test_rlg_builder_format() {
        let b = rlg::builder().format(LogFormat::JSON);
        assert!(format!("{b:?}").contains("JSON"));
    }

    #[test]
    fn test_rlg_builder_without_log() {
        let b = rlg::builder().without_log();
        assert!(format!("{b:?}").contains("install_log: false"));
    }

    #[test]
    fn test_rlg_builder_without_tracing() {
        let b = rlg::builder().without_tracing();
        assert!(format!("{b:?}").contains("install_tracing: false"));
    }

    #[test]
    fn test_rlg_builder_full_chain() {
        let b = rlg::builder()
            .level(LogLevel::TRACE)
            .format(LogFormat::ECS)
            .without_log()
            .without_tracing();
        let dbg = format!("{b:?}");
        assert!(dbg.contains("TRACE"));
        assert!(dbg.contains("ECS"));
        assert!(dbg.contains("install_log: false"));
        assert!(dbg.contains("install_tracing: false"));
    }

    #[test]
    fn test_rlg_builder_copy() {
        let b = rlg::builder().level(LogLevel::WARN);
        let b2 = b;
        assert_eq!(format!("{b:?}"), format!("{b2:?}"));
    }

    // =========================================================================
    // tracing.rs: RlgSubscriber event routing for all levels
    // =========================================================================

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_tracing_subscriber_all_levels_via_dispatch() {
        use rlg::tracing::RlgSubscriber;
        use tracing_core::dispatcher::Dispatch;

        let sub = RlgSubscriber::new();
        let dispatch = Dispatch::new(sub);

        tracing_core::dispatcher::with_default(&dispatch, || {
            tracing::error!("error event");
            tracing::warn!("warn event");
            tracing::info!("info event");
            tracing::debug!("debug event");
            tracing::trace!("trace event");
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_tracing_subscriber_structured_fields() {
        use rlg::tracing::RlgSubscriber;
        use tracing_core::dispatcher::Dispatch;

        let sub = RlgSubscriber::new();
        let dispatch = Dispatch::new(sub);

        tracing_core::dispatcher::with_default(&dispatch, || {
            tracing::info!(
                user_id = 42_u64,
                latency = 12.5_f64,
                ok = true,
                "structured event"
            );
        });
    }

    // =========================================================================
    // tracing.rs: RlgLayer (behind tracing-layer feature)
    // =========================================================================

    #[cfg(feature = "tracing-layer")]
    mod tracing_layer_tests {
        use rlg::log_format::LogFormat;
        use rlg::tracing::RlgLayer;
        use tracing_subscriber::prelude::*;

        #[test]
        fn test_rlg_layer_new() {
            let layer = RlgLayer::new();
            assert!(format!("{layer:?}").contains("MCP"));
        }

        #[test]
        fn test_rlg_layer_with_format() {
            let layer = RlgLayer::new().with_format(LogFormat::JSON);
            assert!(format!("{layer:?}").contains("JSON"));
        }

        #[test]
        fn test_rlg_layer_default() {
            let layer = RlgLayer::default();
            assert!(format!("{layer:?}").contains("MCP"));
        }

        #[test]
        fn test_rlg_layer_clone_copy() {
            let layer = RlgLayer::new();
            let cloned = layer;
            let _ = format!("{layer:?}");
            let _ = format!("{cloned:?}");
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn test_rlg_layer_event_routing() {
            let layer = RlgLayer::new().with_format(LogFormat::JSON);
            let subscriber = tracing_subscriber::registry().with(layer);
            let dispatch =
                tracing_core::dispatcher::Dispatch::new(subscriber);

            tracing_core::dispatcher::with_default(&dispatch, || {
                tracing::info!("layer event test");
                tracing::warn!(key = "value", "structured layer event");
                tracing::error!("error layer event");
                tracing::debug!("debug layer event");
                tracing::trace!("trace layer event");
            });
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn test_rlg_layer_span_tracking() {
            let layer = RlgLayer::new();
            let subscriber = tracing_subscriber::registry().with(layer);
            let dispatch =
                tracing_core::dispatcher::Dispatch::new(subscriber);

            tracing_core::dispatcher::with_default(&dispatch, || {
                let _span =
                    tracing::info_span!("test_span", key = "val")
                        .entered();
                tracing::info!("inside span");
            });
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        fn test_rlg_layer_multiple_formats() {
            for format in &[
                LogFormat::JSON,
                LogFormat::MCP,
                LogFormat::OTLP,
                LogFormat::ECS,
                LogFormat::Logfmt,
            ] {
                let layer = RlgLayer::new().with_format(*format);
                let subscriber =
                    tracing_subscriber::registry().with(layer);
                let dispatch =
                    tracing_core::dispatcher::Dispatch::new(subscriber);
                tracing_core::dispatcher::with_default(
                    &dispatch,
                    || {
                        tracing::info!("format test");
                    },
                );
            }
        }
    }

    // =========================================================================
    // LogEvent: new struct shape
    // =========================================================================

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_log_event_with_log_field() {
        let log = Log::info("test event");
        let event = LogEvent {
            level: LogLevel::INFO,
            level_num: 6,
            log: log.clone(),
        };
        assert_eq!(event.level, LogLevel::INFO);
        assert_eq!(event.level_num, 6);
        assert_eq!(event.log.description, "test event");

        let cloned = event.clone();
        assert_eq!(cloned.log.description, "test event");

        let dbg = format!("{event:?}");
        assert!(dbg.contains("LogEvent"));
    }

    // =========================================================================
    // Log: deferred fire() and log() coverage
    // =========================================================================

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_log_fire_deferred() {
        Log::info("deferred fire test")
            .component("test")
            .with("key", "value")
            .format(LogFormat::JSON)
            .fire();
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_log_log_deferred() {
        let log = Log::info("deferred log test")
            .component("test")
            .with("key", "value")
            .format(LogFormat::MCP);
        log.log();
    }
}
