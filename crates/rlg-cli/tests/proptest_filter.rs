// proptest_filter.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Property tests for `rlg_cli::Filter`.
//
// See `docs/adr/0003-property-tested-formats.md`.

#![allow(missing_docs)]

use proptest::prelude::*;
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg_cli::Filter;
use std::borrow::Cow;
use std::collections::BTreeMap;

fn any_level() -> impl Strategy<Value = LogLevel> {
    prop_oneof![
        Just(LogLevel::TRACE),
        Just(LogLevel::DEBUG),
        Just(LogLevel::VERBOSE),
        Just(LogLevel::INFO),
        Just(LogLevel::WARN),
        Just(LogLevel::ERROR),
        Just(LogLevel::FATAL),
        Just(LogLevel::CRITICAL),
    ]
}

fn any_log() -> impl Strategy<Value = Log> {
    (
        any::<u64>(),
        any_level(),
        "[a-z]{1,10}",
        "[a-zA-Z0-9 ]{0,32}",
    )
        .prop_map(|(sid, level, component, description)| Log {
            session_id: sid,
            time: Cow::Borrowed("t"),
            level,
            component: component.into(),
            description,
            format: LogFormat::JSON,
            attributes: BTreeMap::new(),
        })
}

// ---------------------------------------------------------------------------
// Property 1 — The default filter matches every log.
//
// `Filter::new()` sets no criteria. It must accept every record;
// otherwise CLI usage without flags would silently drop lines.
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn default_filter_matches_every_log(log in any_log()) {
        prop_assert!(Filter::new().matches(&log));
    }
}

// ---------------------------------------------------------------------------
// Property 2 — `min_level` is monotone.
//
// If a filter with `min_level = HIGH` matches a record, then the same
// filter relaxed to `min_level = LOW <= HIGH` must also match. This
// is the monotonicity contract downstream code (rlg-mcp `filter_log`,
// rlg-report aggregation) relies on when combining level ranges.
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn min_level_is_monotone(
        log in any_log(),
        high in any_level(),
        low in any_level(),
    ) {
        // Only run the property when `low` is not stricter than `high`.
        prop_assume!(low.to_numeric() <= high.to_numeric());

        let strict = Filter::new().min_level(high);
        let relaxed = Filter::new().min_level(low);

        if strict.matches(&log) {
            prop_assert!(
                relaxed.matches(&log),
                "monotonicity violated: strict min_level={:?} matched but relaxed min_level={:?} rejected {:?}",
                high, low, log
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Property 3 — Component filter is a substring-of-none-else predicate.
//
// If a component filter accepts a record, the record's component
// equals the filter's component string.
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn component_filter_matches_only_exact(
        log in any_log(),
        needle in "[a-z]{1,10}",
    ) {
        let filter = Filter::new().component(needle.clone());
        if filter.matches(&log) {
            prop_assert_eq!(log.component.as_ref(), needle.as_str());
        }
    }
}
