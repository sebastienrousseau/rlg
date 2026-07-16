#![allow(missing_docs)]
#![cfg(not(miri))]

//! Tests specifically targeting uncovered lines identified by cargo-llvm-cov.

#[cfg(test)]
mod tests {
    // =========================================================================
    // tracing.rs coverage: DEBUG/TRACE events, record_str, record, record_follows_from
    // =========================================================================

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_tracing_debug_and_trace_events() {
        use rlg::tracing::RlgSubscriber;
        use tracing::{debug, trace};
        use tracing_core::dispatcher::{self, Dispatch};

        let subscriber = RlgSubscriber::new();
        let dispatch = Dispatch::new(subscriber);

        // Set filter to ALL to let DEBUG and TRACE through
        rlg::engine::ENGINE.set_filter(rlg::LogLevel::ALL.to_numeric());

        dispatcher::with_default(&dispatch, || {
            debug!("debug event for coverage");
            trace!("trace event for coverage");
            debug!(key = 42u64, "debug with field");
            trace!(key = true, "trace with field");
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_tracing_record_and_follows_from() {
        use rlg::tracing::RlgSubscriber;
        use tracing::{Level, span};
        use tracing_core::dispatcher::{self, Dispatch};

        let subscriber = RlgSubscriber::new();
        let dispatch = Dispatch::new(subscriber);

        dispatcher::with_default(&dispatch, || {
            let span1 = span!(
                Level::INFO,
                "span1",
                field1 = tracing::field::Empty
            );
            let span2 = span!(Level::INFO, "span2");

            // Trigger Subscriber::record()
            span1.record("field1", "recorded_value");

            // Trigger Subscriber::record_follows_from()
            span2.follows_from(&span1);
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_tracing_record_str_visitor() {
        use rlg::tracing::RlgSubscriber;
        use tracing::info;
        use tracing_core::dispatcher::{self, Dispatch};

        let subscriber = RlgSubscriber::new();
        let dispatch = Dispatch::new(subscriber);

        dispatcher::with_default(&dispatch, || {
            // Pass &str fields to trigger record_str visitor method
            let s: &str = "string_value";
            info!(str_field = s, "test record_str");

            // Also pass a String via Display to cover other paths
            let owned = String::from("owned_string");
            info!(owned_field = owned.as_str(), "test owned str");

            // Test message field through record_str (using % display hint)
            info!(msg_field = %"display_str", "test display");

            // Trigger record_str with field named "message" (the special branch)
            // In tracing, passing message = "value" explicitly as a &str field
            // should dispatch through record_str
            info!(message = "explicit message via record_str");
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_tracing_warn_event() {
        use rlg::tracing::RlgSubscriber;
        use tracing::warn;
        use tracing_core::dispatcher::{self, Dispatch};

        let subscriber = RlgSubscriber::new();
        let dispatch = Dispatch::new(subscriber);

        dispatcher::with_default(&dispatch, || {
            warn!(key = "val", "warn event with field for coverage");
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_tracing_span_enter_exit() {
        use rlg::tracing::RlgSubscriber;
        use tracing::{Level, span};
        use tracing_core::dispatcher::{self, Dispatch};

        let subscriber = RlgSubscriber::new();
        let dispatch = Dispatch::new(subscriber);

        dispatcher::with_default(&dispatch, || {
            let span = span!(Level::INFO, "enter_exit_span");
            // enter() triggers Subscriber::enter, drop of guard triggers exit
            let _guard = span.enter();
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_tracing_record_error_visitor() {
        use rlg::tracing::RlgSubscriber;
        use tracing::info;
        use tracing_core::dispatcher::{self, Dispatch};

        let subscriber = RlgSubscriber::new();
        let dispatch = Dispatch::new(subscriber);

        dispatcher::with_default(&dispatch, || {
            let err = std::io::Error::other("test error for coverage");
            let err_ref: &(dyn std::error::Error + 'static) = &err;
            info!(error = err_ref, "event with error field");
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_tracing_record_u128_i128() {
        use rlg::tracing::RlgSubscriber;
        use tracing::info;
        use tracing_core::dispatcher::{self, Dispatch};

        let subscriber = RlgSubscriber::new();
        let dispatch = Dispatch::new(subscriber);

        dispatcher::with_default(&dispatch, || {
            info!(big_u = 42u128, big_i = -42i128, "u128/i128 fields");
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_tracing_all_numeric_types() {
        use rlg::tracing::RlgSubscriber;
        use tracing::info;
        use tracing_core::dispatcher::{self, Dispatch};

        let subscriber = RlgSubscriber::new();
        let dispatch = Dispatch::new(subscriber);

        dispatcher::with_default(&dispatch, || {
            info!(
                u64_val = 100u64,
                i64_val = -100i64,
                bool_val = true,
                f64_val = 1.5_f64,
                "all numeric field types"
            );
        });
    }

    // =========================================================================
    // utils.rs coverage: generate_trace_id
    // =========================================================================

    #[test]
    fn test_generate_trace_id() {
        let trace_id = rlg::utils::generate_trace_id();
        assert_eq!(
            trace_id.len(),
            32,
            "Trace ID should be 32 hex chars"
        );
        assert!(
            trace_id.chars().all(|c| c.is_ascii_hexdigit()),
            "Trace ID should be valid hex"
        );

        // Two consecutive IDs should differ
        let trace_id2 = rlg::utils::generate_trace_id();
        // They might be the same if RNG is identical, but likely different
        let _ = trace_id2;
    }

    #[test]
    fn test_generate_span_id() {
        let span_id = rlg::utils::generate_span_id();
        assert_eq!(span_id.len(), 16, "Span ID should be 16 hex chars");
        assert!(
            span_id.chars().all(|c| c.is_ascii_hexdigit()),
            "Span ID should be valid hex"
        );
    }

    // =========================================================================
    // log_level.rs coverage: as_str_lowercase for ALL, NONE, DISABLED, VERBOSE, CRITICAL
    // =========================================================================

    #[test]
    fn test_log_level_as_str_lowercase_all_variants() {
        use rlg::log_level::LogLevel;

        assert_eq!(LogLevel::ALL.as_str_lowercase(), "all");
        assert_eq!(LogLevel::NONE.as_str_lowercase(), "none");
        assert_eq!(LogLevel::DISABLED.as_str_lowercase(), "disabled");
        assert_eq!(LogLevel::VERBOSE.as_str_lowercase(), "verbose");
        assert_eq!(LogLevel::CRITICAL.as_str_lowercase(), "critical");
        // Already covered but verify for completeness
        assert_eq!(LogLevel::TRACE.as_str_lowercase(), "trace");
        assert_eq!(LogLevel::DEBUG.as_str_lowercase(), "debug");
        assert_eq!(LogLevel::INFO.as_str_lowercase(), "info");
        assert_eq!(LogLevel::WARN.as_str_lowercase(), "warn");
        assert_eq!(LogLevel::ERROR.as_str_lowercase(), "error");
        assert_eq!(LogLevel::FATAL.as_str_lowercase(), "fatal");
    }

    // =========================================================================
    // log_format.rs coverage: ELF/ApacheAccessLog validate, JSON format_log error
    // =========================================================================

    #[test]
    fn test_log_format_validate_elf_and_apache() {
        use rlg::log_format::LogFormat;

        // ELF and ApacheAccessLog always return true for non-empty strings
        assert!(LogFormat::ELF.validate("any string here"));
        assert!(LogFormat::ApacheAccessLog.validate("any string here"));

        // But empty string should return false (handled by the early check)
        assert!(!LogFormat::ELF.validate(""));
        assert!(!LogFormat::ApacheAccessLog.validate(""));
    }

    // =========================================================================
    // config.rs coverage: VersionError, ensure_paths parent dir creation
    // =========================================================================

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_config_load_async_version_mismatch() {
        use rlg::config::Config;

        let temp_dir = tempfile::tempdir().unwrap();
        let config_content = r#"
            version = "99.99"
            log_file_path = "RLG.log"
            log_format = "%level - %message"
        "#;

        let config_file_path = temp_dir.path().join("config.toml");
        tokio::fs::write(&config_file_path, config_content)
            .await
            .unwrap();

        let result = Config::load_async(Some(&config_file_path)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{err}");
        assert!(
            err_msg.contains("version"),
            "Error should mention version: {err_msg}"
        );
    }

    #[test]
    fn test_config_ensure_paths_with_subdirectory() {
        use rlg::config::{Config, LoggingDestination};

        let temp_dir = tempfile::tempdir().unwrap();
        let nested_path =
            temp_dir.path().join("sub").join("dir").join("test.log");

        let config = Config {
            logging_destinations: vec![LoggingDestination::File(
                nested_path.clone(),
            )],
            ..Config::default()
        };

        // Should create parent directories and succeed
        assert!(config.ensure_paths().is_ok());
        assert!(nested_path.parent().unwrap().exists());
    }

    #[test]
    fn test_config_ensure_paths_no_file_destination() {
        use rlg::config::{Config, LoggingDestination};

        let config = Config {
            logging_destinations: vec![LoggingDestination::Stdout],
            ..Config::default()
        };

        // Should succeed without creating any files
        assert!(config.ensure_paths().is_ok());
    }

    // =========================================================================
    // log.rs coverage: write_json_str escaping for special characters
    // =========================================================================

    #[test]
    fn test_log_json_str_escaping() {
        use rlg::log::Log;
        use rlg::log_format::LogFormat;
        use rlg::log_level::LogLevel;

        // Create a log with description containing special chars that need escaping
        let log = Log::build(LogLevel::INFO, "desc with \"quotes\" and \\backslash and \nnewline and \rcarriage and \ttab")
            .session_id(1)
            .time("ts")
            .component("comp")
            .format(LogFormat::JSON);

        let output = format!("{log}");

        // Verify JSON escaping works
        assert!(output.contains("\\\""), "Should escape double quotes");
        assert!(output.contains("\\\\"), "Should escape backslashes");
        assert!(output.contains("\\n"), "Should escape newlines");
        assert!(
            output.contains("\\r"),
            "Should escape carriage returns"
        );
        assert!(output.contains("\\t"), "Should escape tabs");

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output)
            .unwrap_or_else(|e| {
                panic!(
                    "Output should be valid JSON: {e}\nOutput: {output}"
                )
            });
        assert!(parsed.is_object());
    }

    #[test]
    fn test_log_json_str_escaping_in_component() {
        use rlg::log::Log;
        use rlg::log_format::LogFormat;
        use rlg::log_level::LogLevel;

        // Test escaping in the component field too
        let log = Log::build(LogLevel::INFO, "normal desc")
            .session_id(1)
            .time("ts")
            .component("comp\twith\ttabs")
            .format(LogFormat::JSON);

        let output = format!("{log}");
        assert!(
            output.contains("\\t"),
            "Should escape tabs in component"
        );

        // Also test session_id field (now u64, no escaping needed)
        let log2 = Log::build(LogLevel::INFO, "desc")
            .session_id(42)
            .time("ts")
            .component("comp")
            .format(LogFormat::JSON);
        let output2 = format!("{log2}");
        assert!(
            output2.contains("\"SessionID\":42"),
            "Should contain numeric session_id"
        );
    }

    #[test]
    fn test_log_json_str_escaping_via_fluent_api() {
        use rlg::log::Log;
        use rlg::log_format::LogFormat;

        // Test with special chars via fluent API
        let log = Log::info("test\nwith\nnewlines")
            .format(LogFormat::GELF)
            .component("comp\\with\\backslash");

        let output = format!("{log}");
        assert!(
            output.contains("\\n"),
            "Should escape newlines in GELF format"
        );
    }

    // =========================================================================
    // sink.rs coverage: journald env var fallback path
    // =========================================================================

    #[test]
    #[allow(unsafe_code)]
    fn test_platform_sink_journald_with_env_fallback() {
        // Test the journald emit path where RLG_FALLBACK_STDOUT is set
        // This covers the env var check inside the Journald emit branch
        // SAFETY: Test-only; no other threads depend on this env var.
        unsafe { std::env::set_var("RLG_FALLBACK_STDOUT", "1") };

        #[cfg(unix)]
        {
            use rlg::sink::PlatformSink;
            use std::os::unix::net::UnixDatagram;
            if let Ok(socket) = UnixDatagram::unbound() {
                let mut sink = PlatformSink::Journald(Some(socket));
                sink.emit("INFO", b"test with fallback env");
            }
        }

        // SAFETY: Test-only cleanup.
        unsafe { std::env::remove_var("RLG_FALLBACK_STDOUT") };
    }

    // =========================================================================
    // engine.rs coverage: filter level return, queue management
    // =========================================================================

    #[test]
    fn test_engine_filter_drops_below_level() {
        use rlg::LogLevel;
        use rlg::engine::{LockFreeEngine, LogEvent};

        let engine = LockFreeEngine::new(10);
        // Set filter to ERROR level
        engine.set_filter(LogLevel::ERROR.to_numeric());

        // This event should be filtered out (level < filter)
        let event = LogEvent {
            level: LogLevel::DEBUG,
            level_num: LogLevel::DEBUG.to_numeric(),
            log: rlg::log::Log::debug("should be dropped"),
        };
        engine.ingest(event);

        engine.shutdown();
    }

    // =========================================================================
    // log.rs: write_logfmt attribute branch that produces "?" for other values
    // =========================================================================

    #[test]
    fn test_log_write_logfmt_quote_in_description() {
        use rlg::log::Log;
        use rlg::log_format::LogFormat;
        use rlg::log_level::LogLevel;

        let log = Log::build(LogLevel::INFO, "desc with \"quotes\"")
            .session_id(1)
            .time("ts")
            .component("comp")
            .format(LogFormat::Logfmt);
        let output = format!("{log}");
        assert!(output.contains(r#"msg="desc with \"quotes\"""#));
    }

    // =========================================================================
    // config.rs: validate with env vars that have a valid key but empty value
    // (line 439 is the closing brace after error return - need a non-empty
    // key with non-empty value to pass validation successfully in the loop)
    // =========================================================================

    #[test]
    fn test_config_validate_env_vars_valid_pair() {
        use rlg::config::Config;
        use std::collections::HashMap;

        // This exercises the loop body where both key and value are non-empty
        let mut env_vars = HashMap::new();
        env_vars
            .insert("VALID_KEY".to_string(), "valid_value".to_string());
        let config = Config {
            env_vars,
            ..Config::default()
        };
        assert!(config.validate().is_ok());
    }

    // =========================================================================
    // config.rs: hot_reload_async trigger modify event
    // =========================================================================

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_config_hot_reload_modify_event() {
        use parking_lot::RwLock;
        use rlg::config::Config;
        use std::sync::Arc;
        use tokio::time::{Duration, sleep};

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Write initial valid config
        let initial = r#"
            version = "1.0"
            profile = "default"
            log_file_path = "RLG.log"
            log_format = "%level - %message"
        "#;
        tokio::fs::write(&config_path, initial).await.unwrap();

        let shared_config = Arc::new(RwLock::new(Config::default()));
        let stop_tx = Config::hot_reload_async(
            config_path.to_str().unwrap(),
            &shared_config,
        )
        .unwrap();

        // Modify the config file to trigger a Modify event
        let modified = r#"
            version = "1.0"
            profile = "hot_reloaded"
            log_file_path = "RLG.log"
            log_format = "%level - %message"
        "#;
        tokio::fs::write(&config_path, modified).await.unwrap();

        // Give the watcher time to detect and process the change
        sleep(Duration::from_millis(500)).await;

        let _ = stop_tx.send(()).await;
    }

    // =========================================================================
    // log_format.rs: MCP/OTLP/Logfmt from_str
    // =========================================================================

    #[test]
    fn test_log_format_from_str_remaining_variants() {
        use rlg::log_format::LogFormat;
        use std::str::FromStr;

        assert_eq!(LogFormat::from_str("mcp").unwrap(), LogFormat::MCP);
        assert_eq!(
            LogFormat::from_str("otlp").unwrap(),
            LogFormat::OTLP
        );
        assert_eq!(
            LogFormat::from_str("logfmt").unwrap(),
            LogFormat::Logfmt
        );
        assert_eq!(LogFormat::from_str("ecs").unwrap(), LogFormat::ECS);
        assert_eq!(
            LogFormat::from_str("apache").unwrap(),
            LogFormat::ApacheAccessLog
        );
    }

    // =========================================================================
    // log.rs: Display for formats with special characters
    // =========================================================================

    #[test]
    fn test_log_display_all_json_formats_with_escaping() {
        use rlg::log::Log;
        use rlg::log_format::LogFormat;

        let special_desc = "line1\nline2\ttab\\backslash\"quote";

        // Test each JSON format
        for format in [
            LogFormat::JSON,
            LogFormat::GELF,
            LogFormat::Logstash,
            LogFormat::NDJSON,
            LogFormat::MCP,
            LogFormat::ECS,
        ] {
            let log = Log::info(special_desc)
                .format(format)
                .component("test");
            let output = format!("{log}");
            // All JSON formats should produce valid JSON with proper escaping
            assert!(
                serde_json::from_str::<serde_json::Value>(&output)
                    .is_ok(),
                "Format {:?} should produce valid JSON. Got: {}",
                format,
                output
            );
        }

        // OTLP format (has specific structure)
        let log = Log::info(special_desc)
            .format(LogFormat::OTLP)
            .component("test");
        let output = format!("{log}");
        // OTLP has extra }} at end, verify it still contains escaped chars
        assert!(output.contains("\\n"));
    }

    // =========================================================================
    // log.rs: write_json_map with empty map (already covered by default attributes)
    // and with content
    // =========================================================================

    #[test]
    fn test_log_json_map_with_multiple_attributes() {
        use rlg::log::Log;
        use rlg::log_format::LogFormat;

        let log = Log::info("test attributes")
            .format(LogFormat::JSON)
            .with("key1", "value1")
            .with("key2", 42)
            .with("key3", true);

        let output = format!("{log}");
        let parsed: serde_json::Value =
            serde_json::from_str(&output).unwrap();
        let attrs = parsed.get("Attributes").unwrap();
        assert_eq!(attrs.get("key1").unwrap(), "value1");
        assert_eq!(attrs.get("key2").unwrap(), 42);
        assert_eq!(attrs.get("key3").unwrap(), true);
    }

    // =========================================================================
    // engine.rs: queue full path (reliable fill without flusher race)
    // =========================================================================

    #[test]
    fn test_engine_queue_full_retry() {
        use rlg::LogLevel;
        use rlg::engine::{LockFreeEngine, LogEvent};

        // Create a tiny queue
        let engine = LockFreeEngine::new(1);

        // Rapidly fill beyond capacity to trigger the retry loop
        for _ in 0..5 {
            let event = LogEvent {
                level: LogLevel::INFO,
                level_num: LogLevel::INFO.to_numeric(),
                log: rlg::log::Log::info("fill"),
            };
            engine.ingest(event);
        }

        engine.shutdown();
    }

    // =========================================================================
    // sink.rs: native() without env vars (covers the cfg(test) Journald path)
    // =========================================================================

    #[test]
    fn test_platform_sink_native_without_env() {
        use rlg::sink::PlatformSink;

        // Call native() directly - in test cfg on Linux, this hits the
        // #[cfg(all(target_os = "linux", test))] block returning Journald(None)
        // unless RLG_FALLBACK_STDOUT or GITHUB_ACTIONS is set
        let _sink = PlatformSink::native();
    }

    // =========================================================================
    // config.rs: hot_reload_async with invalid path (watcher error)
    // =========================================================================

    #[cfg(feature = "tokio")]
    #[test]
    fn test_config_hot_reload_invalid_path() {
        use parking_lot::RwLock;
        use rlg::config::Config;
        use std::sync::Arc;

        let shared_config = Arc::new(RwLock::new(Config::default()));
        let result = Config::hot_reload_async(
            "/nonexistent/path/that/does/not/exist.toml",
            &shared_config,
        );
        // The watcher.watch() call should fail for non-existent paths
        assert!(result.is_err());
    }

    // =========================================================================
    // config.rs: ensure_paths with root path (parent() returns None)
    // =========================================================================

    #[test]
    fn test_config_ensure_paths_root_path_no_parent() {
        use rlg::config::{Config, LoggingDestination};
        use std::path::PathBuf;

        // PathBuf::from("/") has parent() == None on Unix.
        // This exercises the implicit else branch of `if let Some(parent_dir)`.
        let config = Config {
            logging_destinations: vec![LoggingDestination::File(
                PathBuf::from("/"),
            )],
            ..Config::default()
        };

        // Opening "/" for append fails (it's a directory), but that's OK —
        // we're testing that the parent-dir-creation block is skipped.
        let result = config.ensure_paths();
        assert!(result.is_err());
    }

    // =========================================================================
    // config.rs: hot_reload_async spawn completes cleanly
    // =========================================================================

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_config_hot_reload_spawn_completes() {
        use parking_lot::RwLock;
        use rlg::config::Config;
        use std::sync::Arc;
        use tokio::time::{Duration, sleep};

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let initial = r#"
            version = "1.0"
            profile = "default"
            log_file_path = "RLG.log"
            log_format = "%level - %message"
        "#;
        tokio::fs::write(&config_path, initial).await.unwrap();

        let shared_config = Arc::new(RwLock::new(Config::default()));
        let stop_tx = Config::hot_reload_async(
            config_path.to_str().unwrap(),
            &shared_config,
        )
        .unwrap();

        // Send stop signal and wait for the spawned task to complete
        let _ = stop_tx.send(()).await;
        sleep(Duration::from_millis(100)).await;
    }

    // =========================================================================
    // log.rs: write_logfmt with attributes (all value branches)
    // =========================================================================

    #[test]
    fn test_log_write_logfmt_all_attribute_types() {
        use rlg::log::Log;
        use rlg::log_format::LogFormat;
        use rlg::log_level::LogLevel;

        // Test logfmt with all attribute value types to cover every branch
        let log = Log::build(LogLevel::INFO, "logfmt test")
            .session_id(1)
            .time("ts")
            .component("comp")
            .with("simple_str", "nospaces") // unquoted string
            .with("spaced_str", "has spaces") // quoted string (contains space)
            .with("quoted_str", "has\"quotes") // quoted string (contains quote)
            .with("empty_str", "") // quoted string (empty)
            .with("number", 42) // non-string value
            .with("flag", true) // non-string value
            .format(LogFormat::Logfmt);

        let output = format!("{log}");

        // Verify base fields
        assert!(
            output.contains("level=info"),
            "Should contain level=info"
        );
        assert!(
            output.contains("msg=\"logfmt test\""),
            "Should contain quoted msg"
        );
        assert!(
            output.contains("session_id=1"),
            "Should contain session_id"
        );
        assert!(
            output.contains("component=\"comp\""),
            "Should contain quoted component"
        );

        // Verify attribute formatting
        assert!(
            output.contains("simple_str=nospaces"),
            "Simple string unquoted"
        );
        assert!(
            output.contains("spaced_str=\"has spaces\""),
            "Spaced string quoted"
        );
        assert!(output.contains("number=42"), "Number without quotes");
        assert!(output.contains("flag=true"), "Bool without quotes");
    }

    // =========================================================================
    // log_format.rs: format_log for all JSON-based formats (success path)
    // =========================================================================

    #[test]
    fn test_log_format_format_log_json_success() {
        use rlg::log_format::LogFormat;

        let json_input = r#"{"key": "value", "count": 42}"#;
        for format in [
            LogFormat::JSON,
            LogFormat::Logstash,
            LogFormat::NDJSON,
            LogFormat::GELF,
            LogFormat::MCP,
            LogFormat::OTLP,
            LogFormat::ECS,
        ] {
            let result = format.format_log(json_input);
            assert!(
                result.is_ok(),
                "format_log should succeed for {:?}: {:?}",
                format,
                result
            );
            let formatted = result.unwrap();
            assert!(
                formatted.contains("key"),
                "Formatted output should contain key for {:?}",
                format
            );
        }
    }

    // =========================================================================
    // tui.rs: get_terminal_height_fd with invalid fd (covers fallback path)
    // and with a real PTY (covers success path)
    // =========================================================================

    #[test]
    #[cfg(all(not(windows), feature = "tui"))]
    fn test_get_terminal_height_of_non_terminal() {
        // A regular file is not a terminal, so terminal_size_of returns None → fallback 24
        let file = std::fs::File::open("/dev/null").unwrap();
        let height = rlg::tui::get_terminal_height_of(&file);
        assert_eq!(
            height, 24,
            "Non-terminal fd should return fallback height"
        );
    }

    #[test]
    #[cfg(all(not(windows), feature = "tui"))]
    fn test_get_terminal_height_of_with_pty() {
        // Open a PTY master to get a real terminal fd
        let result = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/ptmx");
        if let Ok(ptmx) = result {
            let height = rlg::tui::get_terminal_height_of(&ptmx);
            // PTY might report 0 rows (no real terminal size),
            // but terminal_size_of should succeed, covering the success branch.
            let _ = height;
        }
        // If /dev/ptmx is not available, skip silently
    }

    // sink.rs: native() coverage is tested in src/sink.rs inline tests
    // using #[serial] to prevent env var race conditions.

    // =========================================================================
    // engine.rs: queue full retry with merged break condition
    // =========================================================================

    #[test]
    fn test_engine_queue_full_merged_break() {
        use rlg::LogLevel;
        use rlg::engine::{LockFreeEngine, LogEvent};

        // Create the smallest possible queue
        let engine = LockFreeEngine::new(1);

        // Shut down the flusher thread first, then wait for it to exit.
        // This guarantees no one is draining the queue.
        engine.shutdown();
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Now fill the queue with guaranteed overflow.
        // First event fills the capacity-1 queue; second triggers the while loop.
        let event1 = LogEvent {
            level: LogLevel::INFO,
            level_num: LogLevel::INFO.to_numeric(),
            log: rlg::log::Log::info("fill1"),
        };
        engine.ingest(event1); // succeeds, queue now full

        let event2 = LogEvent {
            level: LogLevel::INFO,
            level_num: LogLevel::INFO.to_numeric(),
            log: rlg::log::Log::info("overflow"),
        };
        engine.ingest(event2); // queue full → enters while loop → break
    }

    // =========================================================================
    // log.rs: Display for Logfmt format (exercises write_logfmt success path)
    // =========================================================================

    #[test]
    fn test_log_display_logfmt_basic() {
        use rlg::log::Log;
        use rlg::log_format::LogFormat;
        use rlg::log_level::LogLevel;

        let log = Log::build(LogLevel::WARN, "simple warning")
            .session_id(1)
            .time("2025-01-01")
            .component("api")
            .format(LogFormat::Logfmt);

        let output = format!("{log}");
        assert!(output.starts_with("level=warn "));
        assert!(output.contains("msg=\"simple warning\""));
        assert!(output.contains("session_id=1"));
        assert!(output.contains("component=\"api\""));
    }

    // =========================================================================
    // log_format.rs: format_log for non-JSON formats (CLF, Logfmt, etc.)
    // =========================================================================

    #[test]
    fn test_log_format_format_log_non_json() {
        use rlg::log_format::LogFormat;

        let plain_input =
            "127.0.0.1 - user [2025-01-01] \"GET / HTTP/1.1\" 200 1234";

        for format in [
            LogFormat::CLF,
            LogFormat::ApacheAccessLog,
            LogFormat::CEF,
            LogFormat::ELF,
            LogFormat::W3C,
            LogFormat::Log4jXML,
            LogFormat::Logfmt,
        ] {
            let result = format.format_log(plain_input);
            assert!(
                result.is_ok(),
                "format_log should succeed for {:?}",
                format
            );
        }
    }
}
