// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Criterion benchmark suite for the Config, LogRotation, and diff/override APIs.
//!
//! Run:  `cargo bench --bench config_bench`

#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use rlg::config::{Config, LogRotation, LoggingDestination};
use rlg::log_level::LogLevel;
use std::collections::HashMap;
use std::hint::black_box;
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::str::FromStr;

// ===========================================================================
// Config default construction
// ===========================================================================

fn bench_config_default(c: &mut Criterion) {
    let mut g = c.benchmark_group("config_default");

    g.bench_function("default_config", |b| {
        b.iter(|| {
            black_box(Config::default());
        });
    });

    g.bench_function("clone_config", |b| {
        let config = Config::default();
        b.iter(|| {
            black_box(config.clone());
        });
    });

    g.finish();
}

// ===========================================================================
// Config::diff() — macro-generated field comparison
// ===========================================================================

fn bench_config_diff(c: &mut Criterion) {
    let mut g = c.benchmark_group("config_diff");

    g.bench_function("diff_identical", |b| {
        let config1 = Config::default();
        let config2 = Config::default();
        b.iter(|| {
            black_box(Config::diff(
                black_box(&config1),
                black_box(&config2),
            ));
        });
    });

    g.bench_function("diff_one_field_changed", |b| {
        let config1 = Config::default();
        let config2 = Config {
            profile: "production".to_string(),
            ..Config::default()
        };
        b.iter(|| {
            black_box(Config::diff(
                black_box(&config1),
                black_box(&config2),
            ));
        });
    });

    g.bench_function("diff_all_fields_changed", |b| {
        let config1 = Config::default();
        let mut env_vars = HashMap::new();
        env_vars.insert("KEY".to_string(), "value".to_string());
        let config2 = Config {
            version: "2.0".to_string(),
            profile: "production".to_string(),
            log_file_path: PathBuf::from("/var/log/app.log"),
            log_level: LogLevel::ERROR,
            log_rotation: NonZeroU64::new(1024).map(LogRotation::Size),
            log_format: "%time - %level - %message".to_string(),
            logging_destinations: vec![
                LoggingDestination::Stdout,
                LoggingDestination::Network(
                    "tcp://localhost:514".to_string(),
                ),
            ],
            env_vars,
        };
        b.iter(|| {
            black_box(Config::diff(
                black_box(&config1),
                black_box(&config2),
            ));
        });
    });

    g.finish();
}

// ===========================================================================
// Config::override_with() — field merging throughput
// ===========================================================================

fn bench_config_override(c: &mut Criterion) {
    let mut g = c.benchmark_group("config_override");

    g.bench_function("override_empty", |b| {
        let base = Config::default();
        let overlay = Config::default();
        b.iter(|| {
            black_box(
                black_box(&base).override_with(black_box(&overlay)),
            );
        });
    });

    g.bench_function("override_with_env_vars", |b| {
        let base = Config::default();
        let mut env_vars = HashMap::new();
        for i in 0..10 {
            env_vars.insert(format!("VAR_{i}"), format!("value_{i}"));
        }
        let overlay = Config {
            env_vars,
            ..Config::default()
        };
        b.iter(|| {
            black_box(
                black_box(&base).override_with(black_box(&overlay)),
            );
        });
    });

    g.bench_function("override_full", |b| {
        let base = Config::default();
        let mut env_vars = HashMap::new();
        env_vars
            .insert("APP_ENV".to_string(), "production".to_string());
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
        b.iter(|| {
            black_box(
                black_box(&base).override_with(black_box(&overlay)),
            );
        });
    });

    g.finish();
}

// ===========================================================================
// Config::validate() — validation throughput
// ===========================================================================

fn bench_config_validate(c: &mut Criterion) {
    let mut g = c.benchmark_group("config_validate");

    g.bench_function("validate_default", |b| {
        let config = Config::default();
        b.iter(|| {
            black_box(black_box(&config).validate()).unwrap();
        });
    });

    g.bench_function("validate_with_env_vars", |b| {
        let mut config = Config::default();
        for i in 0..20 {
            config
                .env_vars
                .insert(format!("VAR_{i}"), format!("value_{i}"));
        }
        b.iter(|| {
            black_box(black_box(&config).validate()).unwrap();
        });
    });

    g.finish();
}

// ===========================================================================
// Config::set() — dynamic field assignment
// ===========================================================================

fn bench_config_set(c: &mut Criterion) {
    let mut g = c.benchmark_group("config_set");

    for key in &["version", "profile", "log_format"] {
        g.bench_function(format!("set_{key}"), |b| {
            b.iter(|| {
                let mut config = Config::default();
                config
                    .set(black_box(key), black_box("new_value"))
                    .unwrap();
                black_box(&config);
            });
        });
    }

    g.bench_function("set_log_level", |b| {
        b.iter(|| {
            let mut config = Config::default();
            config
                .set(black_box("log_level"), black_box("ERROR"))
                .unwrap();
            black_box(&config);
        });
    });

    g.finish();
}

// ===========================================================================
// Config::expand_env_vars() — environment expansion
// ===========================================================================

fn bench_config_expand(c: &mut Criterion) {
    let mut g = c.benchmark_group("config_expand");

    g.bench_function("expand_empty_env", |b| {
        let config = Config::default();
        b.iter(|| {
            black_box(black_box(&config).expand_env_vars());
        });
    });

    g.bench_function("expand_with_vars", |b| {
        let mut config = Config::default();
        for i in 0..5 {
            config
                .env_vars
                .insert(format!("BENCH_VAR_{i}"), format!("val_{i}"));
        }
        b.iter(|| {
            black_box(black_box(&config).expand_env_vars());
        });
    });

    g.finish();
}

// ===========================================================================
// LogRotation — FromStr parsing throughput
// ===========================================================================

fn bench_log_rotation_parse(c: &mut Criterion) {
    let mut g = c.benchmark_group("log_rotation_parse");

    for s in &["size:1048576", "time:3600", "date", "count:10"] {
        g.bench_function(format!("from_str_{s}"), |b| {
            b.iter(|| {
                black_box(LogRotation::from_str(black_box(s)).unwrap());
            });
        });
    }

    g.finish();
}

fn bench_log_rotation_display(c: &mut Criterion) {
    let mut g = c.benchmark_group("log_rotation_display");

    for s in &[
        "size:1024",
        "size:10485760",
        "time:60",
        "time:86400",
        "date",
        "count:5",
        "count:100",
    ] {
        let rotation = LogRotation::from_str(s).unwrap();
        g.bench_function(format!("display_{s}"), |b| {
            b.iter(|| {
                black_box(format!("{}", black_box(&rotation)));
            });
        });
    }

    g.finish();
}

criterion_group!(
    benches,
    bench_config_default,
    bench_config_diff,
    bench_config_override,
    bench_config_validate,
    bench_config_set,
    bench_config_expand,
    bench_log_rotation_parse,
    bench_log_rotation_display,
);
criterion_main!(benches);
