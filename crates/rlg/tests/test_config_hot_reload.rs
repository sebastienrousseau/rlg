// test_config_hot_reload.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `Config::hot_reload_async` applies edits, including editor-style
//! atomic replaces, and keeps the last good config on a bad write.

#![cfg(all(feature = "tokio", not(miri)))]

use parking_lot::RwLock;
use rlg::config::Config;
use std::fs;
use std::sync::Arc;

/// Poll `shared` until its profile is `want`, for up to two
/// seconds.
async fn wait_for_profile(
    shared: &Arc<RwLock<Config>>,
    want: &str,
) -> bool {
    for _ in 0..40 {
        if shared.read().profile == want {
            return true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    false
}

#[tokio::test]
async fn test_hot_reload_applies_edit_and_atomic_replace() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    Config::default().save_to_file(&path).unwrap();
    let shared = Arc::new(RwLock::new(Config::default()));
    let stop =
        Config::hot_reload_async(path.to_str().unwrap(), &shared)
            .unwrap();

    // An in-place edit is picked up.
    let edited = Config {
        profile: "edited".into(),
        ..Config::default()
    };
    edited.save_to_file(&path).unwrap();
    assert!(wait_for_profile(&shared, "edited").await);

    // So is an editor-style save: write a sibling, rename over.
    let replaced = Config {
        profile: "replaced".into(),
        ..Config::default()
    };
    let tmp = dir.path().join("config.toml.tmp");
    replaced.save_to_file(&tmp).unwrap();
    fs::rename(&tmp, &path).unwrap();
    assert!(wait_for_profile(&shared, "replaced").await);

    // Invalid TOML keeps the last good config.
    fs::write(&path, "invalid = [toml").unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    assert_eq!(shared.read().profile, "replaced");

    stop.send(()).await.unwrap();
}
