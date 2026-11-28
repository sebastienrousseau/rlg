// lib.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # rlg-core — the `no_std` core of RLG
//!
//! The pure part of [`rlg`](https://docs.rs/rlg): severity levels,
//! a borrowed [`Record`] view, and the rendering of a record into each
//! of the 14 output formats, written through [`core::fmt::Write`]. It
//! needs `alloc` (attributes are a `BTreeMap` of `serde_json` values)
//! and nothing from `std`, so it builds for bare-metal targets such as
//! `thumbv7em-none-eabihf`.
//!
//! `rlg` depends on this crate and re-exports [`LogLevel`] and
//! [`ParseLogLevelError`] at their old paths; `rlg::Log`'s `Display`
//! renders through [`Record`], so the bytes are the same either way.
//! The engine, sinks, config and `rlg::LogFormat` (whose parsing and
//! validation need `std`) stay in `rlg`.
//!
//! ```
//! use core::fmt::Write;
//! use rlg_core::{Format, LogLevel, Record};
//!
//! let mut out = String::new();
//! let record = Record::new(LogLevel::WARN, "battery low")
//!     .component("power")
//!     .session_id(7)
//!     .format(Format::Logfmt);
//! write!(out, "{record}").unwrap();
//! assert_eq!(
//!     out,
//!     "level=warn msg=\"battery low\" session_id=7 component=\"power\""
//! );
//! ```

#![no_std]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

extern crate alloc;

/// The 14 output formats a [`Record`] can be written in.
pub mod format;
/// Severity levels: `ALL` through `CRITICAL`, with `FromStr` parsing.
pub mod level;
/// The borrowed record view and its rendering.
pub mod record;

pub use crate::format::Format;
pub use crate::level::{LogLevel, ParseLogLevelError};
pub use crate::record::{Attributes, Record};

/// The README's code blocks, compiled and run as doctests so its
/// Quick Start cannot drift from the API.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
