// log_level.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// The level types live in the `no_std` crate `rlg-core` and are
// re-exported here, so `rlg::log_level::LogLevel` and
// `rlg_core::LogLevel` are one type.

pub use rlg_core::level::{LogLevel, ParseLogLevelError};
