// hot_reload.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Polling hot-reload for [`Config`], behind the `tokio` feature.

use super::{Config, ConfigError};
use parking_lot::RwLock;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;

/// How often [`Config::hot_reload_async`] checks the watched file.
pub const HOT_RELOAD_POLL_INTERVAL: std::time::Duration =
    std::time::Duration::from_millis(250);

/// A file's modification time and length, compared by hot-reload.
type Fingerprint = (Option<std::time::SystemTime>, u64);

fn file_fingerprint(path: &Path) -> std::io::Result<Fingerprint> {
    Ok(fingerprint_of(&fs::metadata(path)?))
}

/// The same, without blocking the runtime: for the polling task.
async fn file_fingerprint_async(
    path: &Path,
) -> std::io::Result<Fingerprint> {
    Ok(fingerprint_of(&tokio::fs::metadata(path).await?))
}

fn fingerprint_of(meta: &fs::Metadata) -> Fingerprint {
    (meta.modified().ok(), meta.len())
}

impl Config {
    /// Hot-reloads configuration on file change.
    ///
    /// Polls the file every [`HOT_RELOAD_POLL_INTERVAL`] and reloads
    /// it when its modification time or size changes, including
    /// when an editor replaces it by renaming a new file over it.
    /// A file that fails to load leaves the current configuration in
    /// place. Send `()` on the returned channel, or drop it, to stop.
    ///
    /// Requires the `tokio` feature and a running Tokio runtime.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::WatcherError`] if the file cannot be
    /// read when watching starts.
    pub fn hot_reload_async(
        config_path: &str,
        config: &Arc<RwLock<Self>>,
    ) -> Result<mpsc::Sender<()>, ConfigError> {
        let mut seen = file_fingerprint(Path::new(config_path))
            .map_err(ConfigError::WatcherError)?;
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);
        let config = Arc::clone(config);
        let path = config_path.to_string();
        tokio::spawn(async move {
            let mut ticks =
                tokio::time::interval(HOT_RELOAD_POLL_INTERVAL);
            ticks.set_missed_tick_behavior(
                tokio::time::MissedTickBehavior::Delay,
            );
            loop {
                tokio::select! {
                    _ = ticks.tick() => {
                        Self::reload_if_changed(&path, &mut seen, &config).await;
                    }
                    _ = stop_rx.recv() => break,
                }
            }
        });
        Ok(stop_tx)
    }

    /// Reload `path` into `config` if its fingerprint moved on from
    /// `seen`. A missing file is skipped until it reappears.
    async fn reload_if_changed(
        path: &str,
        seen: &mut Fingerprint,
        config: &Arc<RwLock<Self>>,
    ) {
        let Ok(now) = file_fingerprint_async(Path::new(path)).await
        else {
            return;
        };
        if now == *seen {
            return;
        }
        *seen = now;
        if let Ok(new_config) = Self::load_async(Some(path)).await {
            let fresh = new_config.read().clone();
            *config.write() = fresh;
        }
    }
}
