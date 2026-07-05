// backoff.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Retry policy, jitter, and a tokens-per-window circuit breaker
//! for the OTLP exporter.
//!
//! These primitives are transport-agnostic — the sync `HttpTransport`
//! in `lib.rs` uses them today, and the deferred async / gRPC
//! transports (see `docs/adr/0010-otlp-pluggable-transport.md`)
//! will use the same primitives without duplicating the reliability
//! logic.

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Retry configuration. Governs the exponential-backoff schedule
/// and the jitter distribution applied to each retry delay.
#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts after the initial request.
    /// `0` disables retries entirely.
    pub max_retries: u32,
    /// Base for the exponential backoff. Delay for attempt `n` is
    /// `base * 2^n`, capped at [`Self::max_delay`], then modulated by
    /// jitter.
    pub base: Duration,
    /// Absolute upper bound on the pre-jitter delay. Prevents runaway
    /// exponentials on high retry counts.
    pub max_delay: Duration,
    /// Fraction of the computed delay applied as full-jitter randomness.
    /// Value `0.0..=1.0`. `0.0` means no jitter; `1.0` means the delay
    /// can be anywhere in `[0, delay]`.
    pub jitter: f64,
}

impl RetryPolicy {
    /// AWS Architecture Blog's "full jitter" default: 200 ms base,
    /// 30 s cap, 100 % jitter, 3 retries.
    #[must_use]
    pub const fn default_full_jitter() -> Self {
        Self {
            max_retries: 3,
            base: Duration::from_millis(200),
            max_delay: Duration::from_secs(30),
            jitter: 1.0,
        }
    }

    /// Compute the delay for a given attempt (`0`-indexed).
    ///
    /// `attempt = 0` is the first *retry* (after the initial request
    /// failed). Delay grows as `base * 2^attempt`, capped at
    /// `max_delay`, then modulated by the jitter fraction using a
    /// caller-supplied random source in `[0.0, 1.0)`.
    #[must_use]
    pub fn delay(&self, attempt: u32, rng_0_to_1: f64) -> Duration {
        let shift = attempt.min(30); // 2^30 microseconds is already ~18 min
        let raw_multiplier = 1u64 << shift;
        let scaled = self.base.saturating_mul(
            u32::try_from(raw_multiplier.min(u64::from(u32::MAX)))
                .unwrap_or(u32::MAX),
        );
        let capped = scaled.min(self.max_delay);
        // full-jitter formula: sleep = rand(0, capped * jitter) +
        //                             capped * (1 - jitter)
        let jitter_frac = self.jitter.clamp(0.0, 1.0);
        let jittered_ns = f64::from(
            u32::try_from(capped.as_nanos().min(u128::from(u32::MAX)))
                .unwrap_or(u32::MAX),
        );
        let deterministic_ns = jittered_ns * (1.0 - jitter_frac);
        let random_ns =
            jittered_ns * jitter_frac * rng_0_to_1.clamp(0.0, 1.0);
        Duration::from_nanos((deterministic_ns + random_ns) as u64)
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::default_full_jitter()
    }
}

/// Cheap process-local RNG for jitter. Uses `Instant::now()` as the
/// entropy source — good enough for retry jitter, which does not
/// require cryptographic quality.
///
/// Not exported: consumers should not depend on the jitter source.
pub(crate) fn cheap_random_0_to_1() -> f64 {
    let now = Instant::now();
    // Coarse pseudo-random derived from monotonic clock nanoseconds.
    // Two calls in rapid succession will produce different values
    // because `Instant::now()` monotonically advances between them.
    let nanos = now.elapsed().as_nanos() as u64;
    let scrambled = nanos.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    (scrambled as f64) / (u64::MAX as f64)
}

// ---------------------------------------------------------------------------
// Circuit breaker
// ---------------------------------------------------------------------------

/// Tokens-per-window circuit breaker.
///
/// Every failed request consumes one token from a fixed budget. When
/// the budget hits zero, the breaker trips and rejects requests
/// without touching the network until the window elapses. On window
/// rollover, the budget refills.
///
/// This model matches the AWS-style "sliding-window" limiter used by
/// Envoy and many production OTLP collectors: cheap to compute, no
/// per-request coordination, and cleanly recovers after a failure
/// storm.
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Number of failure tokens available at window start.
    budget: u32,
    /// Duration of one window.
    window: Duration,
    inner: Mutex<CircuitBreakerState>,
}

#[derive(Debug)]
struct CircuitBreakerState {
    tokens: u32,
    window_start: Instant,
}

impl CircuitBreaker {
    /// New breaker with the given failure budget per window.
    #[must_use]
    pub fn new(budget: u32, window: Duration) -> Self {
        Self {
            budget,
            window,
            inner: Mutex::new(CircuitBreakerState {
                tokens: budget,
                window_start: Instant::now(),
            }),
        }
    }

    /// Should the caller attempt this request?
    ///
    /// Returns `true` if the breaker is closed (tokens available or
    /// window has rolled over) and `false` if the breaker is tripped
    /// for the current window.
    pub fn allow(&self) -> bool {
        let mut state = self.inner.lock().unwrap_or_else(|e| {
            // Poison recovery: reset window on lock poison to avoid
            // permanent trip. Poisoning is not a security-critical
            // event here.
            self.inner.clear_poison();
            e.into_inner()
        });
        if state.window_start.elapsed() >= self.window {
            state.tokens = self.budget;
            state.window_start = Instant::now();
        }
        state.tokens > 0
    }

    /// Report a failed request. Consumes one token if any are left.
    pub fn record_failure(&self) {
        let mut state = self.inner.lock().unwrap_or_else(|e| {
            self.inner.clear_poison();
            e.into_inner()
        });
        if state.window_start.elapsed() >= self.window {
            state.tokens = self.budget;
            state.window_start = Instant::now();
        }
        state.tokens = state.tokens.saturating_sub(1);
    }

    /// Report a successful request. Optional but improves accuracy:
    /// a successful call restores the failure budget by one token,
    /// modelling the fact that the collector is healthy.
    pub fn record_success(&self) {
        let mut state = self.inner.lock().unwrap_or_else(|e| {
            self.inner.clear_poison();
            e.into_inner()
        });
        if state.tokens < self.budget {
            state.tokens += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- RetryPolicy ----------

    #[test]
    fn delay_zero_attempt_is_base() {
        let p = RetryPolicy {
            max_retries: 3,
            base: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            jitter: 0.0,
        };
        // No jitter, attempt 0 → base
        assert_eq!(p.delay(0, 0.5), Duration::from_millis(100));
    }

    #[test]
    fn delay_doubles_per_attempt_no_jitter() {
        let p = RetryPolicy {
            max_retries: 3,
            base: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            jitter: 0.0,
        };
        assert_eq!(p.delay(0, 0.0), Duration::from_millis(100));
        assert_eq!(p.delay(1, 0.0), Duration::from_millis(200));
        assert_eq!(p.delay(2, 0.0), Duration::from_millis(400));
        assert_eq!(p.delay(3, 0.0), Duration::from_millis(800));
    }

    #[test]
    fn delay_caps_at_max() {
        let p = RetryPolicy {
            max_retries: 20,
            base: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
            jitter: 0.0,
        };
        // 100ms * 2^20 = 100 000 s pre-cap. Post-cap: 1 s.
        assert_eq!(p.delay(20, 0.0), Duration::from_secs(1));
    }

    #[test]
    fn delay_with_full_jitter_stays_within_bound() {
        let p = RetryPolicy {
            max_retries: 3,
            base: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            jitter: 1.0,
        };
        // Full jitter with rng=0 → 0 delay.
        assert_eq!(p.delay(2, 0.0), Duration::ZERO);
        // Full jitter with rng=1 → capped delay (400ms for attempt 2).
        let max = p.delay(2, 1.0);
        assert!(max <= Duration::from_millis(400), "got {max:?}");
        assert!(max >= Duration::from_millis(399));
    }

    #[test]
    fn delay_high_attempt_does_not_panic() {
        // Attempt 100 with default base would overflow if not
        // saturated. Test the saturation path.
        let p = RetryPolicy::default_full_jitter();
        let _ = p.delay(100, 0.5);
        let _ = p.delay(u32::MAX, 0.5);
    }

    #[test]
    fn cheap_random_is_in_range() {
        for _ in 0..64 {
            let r = cheap_random_0_to_1();
            assert!((0.0..=1.0).contains(&r), "out of range: {r}");
        }
    }

    // ---------- CircuitBreaker ----------

    #[test]
    fn breaker_closed_when_budget_untouched() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        assert!(cb.allow());
    }

    #[test]
    fn breaker_trips_after_budget_exhausted() {
        let cb = CircuitBreaker::new(2, Duration::from_secs(60));
        cb.record_failure();
        assert!(cb.allow());
        cb.record_failure();
        // Two failures consumed the budget of 2 → tripped.
        assert!(!cb.allow());
    }

    #[test]
    fn breaker_resets_after_window() {
        let cb = CircuitBreaker::new(1, Duration::from_millis(50));
        cb.record_failure();
        assert!(!cb.allow());
        std::thread::sleep(Duration::from_millis(75));
        // Window elapsed → budget refilled → allow again.
        assert!(cb.allow());
    }

    #[test]
    fn breaker_success_refills_budget() {
        let cb = CircuitBreaker::new(2, Duration::from_secs(60));
        cb.record_failure();
        cb.record_failure();
        assert!(!cb.allow());
        cb.record_success();
        assert!(cb.allow());
    }

    #[test]
    fn breaker_success_capped_at_budget() {
        // Recording more successes than the budget can hold does not
        // overflow the token count.
        let cb = CircuitBreaker::new(2, Duration::from_secs(60));
        for _ in 0..100 {
            cb.record_success();
        }
        assert!(cb.allow());
        cb.record_failure();
        assert!(cb.allow());
        cb.record_failure();
        assert!(!cb.allow());
    }

    #[test]
    fn breaker_survives_lock_poison() {
        use std::sync::Arc;
        use std::thread;

        let cb =
            Arc::new(CircuitBreaker::new(3, Duration::from_secs(60)));
        let cb_clone = cb.clone();
        let _ = thread::spawn(move || {
            let _guard = cb_clone.inner.lock().unwrap();
            panic!("poison the lock");
        })
        .join();
        // After poison, `allow` still returns a valid answer.
        assert!(cb.allow());
    }
}
