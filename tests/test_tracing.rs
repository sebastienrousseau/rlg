#![cfg(not(miri))]
#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    use rlg::tracing::RlgSubscriber;
    use tracing::{error, info, span, warn, Level};
    use tracing_core::dispatcher::{self, Dispatch};

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_tracing_integration() {
        let subscriber = RlgSubscriber::new();
        let dispatch = Dispatch::new(subscriber);

        dispatcher::with_default(&dispatch, || {
            info!(target: "my_comp", key = "val", "This is an info message from tracing");
            warn!("This is a warning");
            error!("This is an error");
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_tracing_all_types() {
        let subscriber = RlgSubscriber::new();
        let dispatch = Dispatch::new(subscriber);

        dispatcher::with_default(&dispatch, || {
            let error = std::io::Error::other("test error");
            let err_ref: &dyn std::error::Error = &error;
            info!(
                u64_val = 1u64,
                i64_val = -1i64,
                f64_val = 1.5f64,
                bool_val = true,
                err_val = err_ref,
                u128_val = 1u128,
                i128_val = -1i128,
                debug_val = ?vec![1, 2, 3],
                "Testing all types"
            );
            // Specifically trigger i64 and other variants if they weren't hit
            info!(field = -42i64, "i64");
            info!(field = 42u64, "u64");
            info!(field = true, "bool");
            info!(field = 1.23f64, "f64");
            info!(field = 1u128, "u128");
            info!(field = -1i128, "i128");
            // Test record_debug specifically for the "message" branch
            info!(message = ? "debug message", "ignored");
            // Test record_debug for other fields
            info!(debug_field = ? vec![1], "debug field");
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_tracing_enabled() {
        use rlg::engine::ENGINE;
        let subscriber = RlgSubscriber::new();
        let dispatch = Dispatch::new(subscriber);

        ENGINE.set_filter(rlg::LogLevel::ERROR.to_numeric());

        dispatcher::with_default(&dispatch, || {
            // These should be filtered out by the engine (though tracing might still call enabled)
            info!("This should be filtered");
            error!("This should NOT be filtered");
        });

        ENGINE.set_filter(rlg::LogLevel::ALL.to_numeric());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_subscriber_noop_methods() {
        let subscriber = RlgSubscriber::new();
        let dispatch = Dispatch::new(subscriber);

        dispatcher::with_default(&dispatch, || {
            let s = span!(Level::INFO, "my_span");
            let _enter = s.enter();
            // record on span
            s.record("key", "val");
        });
    }
}
