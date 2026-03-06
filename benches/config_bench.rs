// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Divan benchmark suite for the Config, LogRotation, and diff/override APIs.
//!
//! Proves zero performance regressions from the macro-driven refactoring by
//! measuring allocation counts, throughput for config operations, and the cost
//! of macro-generated `Config::diff()`.
//!
//! Run:  `cargo bench --bench config_bench`
//! Alloc report:  every benchmark row shows `alloc:` and `dealloc:` columns.

#![allow(missing_docs)]

use divan::AllocProfiler;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

// ===========================================================================
// Module: Config default construction — allocation tracking
// ===========================================================================

mod config_default {
    use divan::{black_box, Bencher};
    use rlg::config::Config;

    #[divan::bench]
    fn default_config(bencher: Bencher) {
        bencher.bench(|| {
            black_box(Config::default());
        });
    }

    #[divan::bench]
    fn clone_config(bencher: Bencher) {
        let config = Config::default();
        bencher.bench(|| {
            black_box(config.clone());
        });
    }
}

// ===========================================================================
// Module: Config::diff() — macro-generated field comparison
// ===========================================================================

mod config_diff {
    use divan::{black_box, Bencher};
    use rlg::config::{Config, LoggingDestination};
    use rlg::log_level::LogLevel;
    use std::collections::HashMap;
    use std::num::NonZeroU64;
    use std::path::PathBuf;

    #[divan::bench]
    fn diff_identical(bencher: Bencher) {
        let config1 = Config::default();
        let config2 = Config::default();
        bencher.bench(|| {
            black_box(Config::diff(black_box(&config1), black_box(&config2)));
        });
    }

    #[divan::bench]
    fn diff_one_field_changed(bencher: Bencher) {
        let config1 = Config::default();
        let config2 = Config {
            profile: "production".to_string(),
            ..Config::default()
        };
        bencher.bench(|| {
            black_box(Config::diff(black_box(&config1), black_box(&config2)));
        });
    }

    #[divan::bench]
    fn diff_all_fields_changed(bencher: Bencher) {
        let config1 = Config::default();
        let mut env_vars = HashMap::new();
        env_vars.insert("KEY".to_string(), "value".to_string());
        let config2 = Config {
            version: "2.0".to_string(),
            profile: "production".to_string(),
            log_file_path: PathBuf::from("/var/log/app.log"),
            log_level: LogLevel::ERROR,
            log_rotation: NonZeroU64::new(1024).map(rlg::config::LogRotation::Size),
            log_format: "%time - %level - %message".to_string(),
            logging_destinations: vec![
                LoggingDestination::Stdout,
                LoggingDestination::Network("tcp://localhost:514".to_string()),
            ],
            env_vars,
        };
        bencher.bench(|| {
            black_box(Config::diff(black_box(&config1), black_box(&config2)));
        });
    }
}

// ===========================================================================
// Module: Config::override_with() — field merging throughput
// ===========================================================================

mod config_override {
    use divan::{black_box, Bencher};
    use rlg::config::{Config, LoggingDestination};
    use rlg::log_level::LogLevel;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[divan::bench]
    fn override_empty(bencher: Bencher) {
        let base = Config::default();
        let overlay = Config::default();
        bencher.bench(|| {
            black_box(black_box(&base).override_with(black_box(&overlay)));
        });
    }

    #[divan::bench]
    fn override_with_env_vars(bencher: Bencher) {
        let base = Config::default();
        let mut env_vars = HashMap::new();
        for i in 0..10 {
            env_vars.insert(format!("VAR_{i}"), format!("value_{i}"));
        }
        let overlay = Config {
            env_vars,
            ..Config::default()
        };
        bencher.bench(|| {
            black_box(black_box(&base).override_with(black_box(&overlay)));
        });
    }

    #[divan::bench]
    fn override_full(bencher: Bencher) {
        let base = Config::default();
        let mut env_vars = HashMap::new();
        env_vars.insert("APP_ENV".to_string(), "production".to_string());
        let overlay = Config {
            version: "2.0".to_string(),
            profile: "production".to_string(),
            log_file_path: PathBuf::from("/var/log/prod.log"),
            log_level: LogLevel::WARN,
            log_rotation: None,
            log_format: "json".to_string(),
            logging_destinations: vec![LoggingDestination::Stdout],
            env_vars,
        };
        bencher.bench(|| {
            black_box(black_box(&base).override_with(black_box(&overlay)));
        });
    }
}

// ===========================================================================
// Module: Config::validate() — validation throughput
// ===========================================================================

mod config_validate {
    use divan::{black_box, Bencher};
    use rlg::config::Config;

    #[divan::bench]
    fn validate_default(bencher: Bencher) {
        let config = Config::default();
        bencher.bench(|| {
            black_box(black_box(&config).validate()).unwrap();
        });
    }

    #[divan::bench]
    fn validate_with_env_vars(bencher: Bencher) {
        let mut config = Config::default();
        for i in 0..20 {
            config
                .env_vars
                .insert(format!("VAR_{i}"), format!("value_{i}"));
        }
        bencher.bench(|| {
            black_box(black_box(&config).validate()).unwrap();
        });
    }
}

// ===========================================================================
// Module: Config::set() — dynamic field assignment
// ===========================================================================

mod config_set {
    use divan::{black_box, Bencher};
    use rlg::config::Config;

    const KEYS: &[&str] = &["version", "profile", "log_format"];

    #[divan::bench(args = KEYS)]
    fn set_string_field(bencher: Bencher, key: &&str) {
        bencher.bench(|| {
            let mut config = Config::default();
            config.set(black_box(key), black_box("new_value")).unwrap();
            black_box(&config);
        });
    }

    #[divan::bench]
    fn set_log_level(bencher: Bencher) {
        bencher.bench(|| {
            let mut config = Config::default();
            config
                .set(black_box("log_level"), black_box("ERROR"))
                .unwrap();
            black_box(&config);
        });
    }
}

// ===========================================================================
// Module: Config::expand_env_vars() — environment expansion
// ===========================================================================

mod config_expand {
    use divan::{black_box, Bencher};
    use rlg::config::Config;

    #[divan::bench]
    fn expand_empty_env(bencher: Bencher) {
        let config = Config::default();
        bencher.bench(|| {
            black_box(black_box(&config).expand_env_vars());
        });
    }

    #[divan::bench]
    fn expand_with_vars(bencher: Bencher) {
        let mut config = Config::default();
        for i in 0..5 {
            config
                .env_vars
                .insert(format!("BENCH_VAR_{i}"), format!("val_{i}"));
        }
        bencher.bench(|| {
            black_box(black_box(&config).expand_env_vars());
        });
    }
}

// ===========================================================================
// Module: LogRotation — FromStr parsing throughput
// ===========================================================================

mod log_rotation_parse {
    use divan::{black_box, Bencher};
    use rlg::config::LogRotation;
    use std::str::FromStr;

    const ROTATION_STRINGS: &[&str] =
        &["size:1048576", "time:3600", "date", "count:10"];

    #[divan::bench(args = ROTATION_STRINGS)]
    fn from_str(bencher: Bencher, s: &&str) {
        bencher.bench(|| {
            black_box(LogRotation::from_str(black_box(s)).unwrap());
        });
    }

    const ALL_ROTATIONS: &[&str] = &[
        "size:1024",
        "size:10485760",
        "time:60",
        "time:86400",
        "date",
        "count:5",
        "count:100",
    ];

    #[divan::bench(args = ALL_ROTATIONS)]
    fn display(bencher: Bencher, s: &&str) {
        let rotation = LogRotation::from_str(s).unwrap();
        bencher.bench(|| {
            black_box(format!("{}", black_box(&rotation)));
        });
    }
}
