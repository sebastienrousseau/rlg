// lib.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Kernel-context enrichment for `rlg` records.
//!
//! Attaches process-level context (PID, TID, UID) to every log
//! record before it hits the sink. Enterprise deployments running
//! multi-tenant workloads on a shared host use these fields to
//! correlate log lines back to the specific process, thread, or
//! user that produced them, without an off-line join against
//! `/proc`.
//!
//! # Enricher trait
//!
//! The [`Enricher`] trait is the extension point:
//!
//! ```
//! use rlg::log::Log;
//! use rlg_ebpf::{Enricher, ProcessEnricher};
//!
//! let enricher = ProcessEnricher::new();
//! let log = Log::info("checkout completed");
//! let enriched = enricher.enrich(log);
//!
//! assert!(enriched.attributes.contains_key("pid"));
//! # #[cfg(unix)]
//! # {
//! assert!(enriched.attributes.contains_key("uid"));
//! # }
//! ```
//!
//! # `EbpfEnricher` (Phase 21.1 scaffold)
//!
//! Enable the `ebpf` feature to expose a placeholder
//! [`EbpfEnricher`] type. Its final implementation will attach an
//! eBPF program via `aya` to hook into the kernel and pull network
//! 4-tuple context from the socket layer. Landing in Phase 21.1.

// Note: `unsafe_code = "deny"` is applied via `[lints.rust]` in
// Cargo.toml. `deny` (not `forbid`) so the crate-private
// `unix_ffi` module can locally allow the libc FFI calls with
// `#[allow(unsafe_code)]`, following the same pattern as
// `rlg::sink` for `syslog(3)`.
#![deny(missing_docs)]

use rlg::log::Log;
use serde_json::Value;

/// Attaches per-record context. Implementations are stateless from
/// the caller's perspective — they read process / kernel state on
/// each call.
pub trait Enricher {
    /// Return the given log with additional context attached.
    #[must_use]
    fn enrich(&self, log: Log) -> Log;
}

// ---------------------------------------------------------------------------
// ProcessEnricher — pid/tid/uid via std + libc
// ---------------------------------------------------------------------------

/// Attaches process-level identity (PID, TID, UID) to every log.
///
/// - **PID** is `std::process::id()`. Works everywhere.
/// - **TID** is `libc::syscall(SYS_gettid)` on Linux,
///   `libc::pthread_self()` cast to `u64` on other Unix targets.
///   Absent on non-Unix.
/// - **UID** is `libc::getuid()`. Absent on non-Unix.
///
/// The Windows path lands in Phase 21.2 alongside `GetCurrentThreadId`
/// and `GetCurrentProcess` bindings.
#[derive(Debug, Default, Clone, Copy)]
pub struct ProcessEnricher;

impl ProcessEnricher {
    /// New process enricher. Cheap to construct — no state.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Enricher for ProcessEnricher {
    fn enrich(&self, mut log: Log) -> Log {
        log.attributes.insert(
            "pid".into(),
            Value::from(u64::from(std::process::id())),
        );
        #[cfg(unix)]
        {
            log.attributes
                .insert("tid".into(), Value::from(current_tid()));
            log.attributes
                .insert("uid".into(), Value::from(current_uid()));
        }
        log
    }
}

// ---------------------------------------------------------------------------
// Unix TID / UID helpers.
// ---------------------------------------------------------------------------
//
// `libc` calls into an FFI boundary; the workspace policy is
// `unsafe_code = "deny"` at the crate level, so wrap the FFI in
// crate-private helpers marked with the `#[allow(unsafe_code)]`
// attribute — the same escape hatch `rlg::sink` uses for `syslog(3)`.

#[cfg(unix)]
#[allow(unsafe_code)]
mod unix_ffi {
    /// Current thread ID. Linux uses `SYS_gettid`; other Unix
    /// targets return `pthread_self()` cast to a `u64` (best
    /// available portable substitute).
    pub(super) fn current_tid() -> u64 {
        #[cfg(target_os = "linux")]
        {
            // SAFETY: `SYS_gettid` takes no arguments and returns a
            // pid_t. The call cannot fail and has no memory-safety
            // implications.
            unsafe { libc::syscall(libc::SYS_gettid) as u64 }
        }
        #[cfg(not(target_os = "linux"))]
        {
            // SAFETY: `pthread_self` takes no arguments and returns
            // an opaque handle. Cast to u64 for uniform attribute
            // typing.
            unsafe { libc::pthread_self() as u64 }
        }
    }

    /// Current user ID via `libc::getuid()`.
    pub(super) fn current_uid() -> u64 {
        // SAFETY: `getuid()` takes no arguments and cannot fail.
        u64::from(unsafe { libc::getuid() })
    }
}

#[cfg(unix)]
use unix_ffi::{current_tid, current_uid};

// ---------------------------------------------------------------------------
// EbpfEnricher scaffold (Phase 21.1)
// ---------------------------------------------------------------------------

/// Kernel-side eBPF-backed enricher.
///
/// # Status: scaffold
///
/// Phase 21 ships the type layout and feature flag. The `aya`-based
/// program attach lands in Phase 21.1 — see
/// `docs/adr/0012-ebpf-enricher.md`.
///
/// The current implementation delegates to [`ProcessEnricher`] so
/// consumers who select this type today still get useful
/// enrichment; Phase 21.1 adds cgroup ID, capability set, and
/// (optional) network 4-tuple attributes on top.
#[cfg(feature = "ebpf")]
#[cfg_attr(docsrs, doc(cfg(feature = "ebpf")))]
#[derive(Debug, Default, Clone, Copy)]
pub struct EbpfEnricher {
    _reserved: (),
}

#[cfg(feature = "ebpf")]
impl EbpfEnricher {
    /// Construct a new eBPF-backed enricher.
    #[must_use]
    pub const fn new() -> Self {
        Self { _reserved: () }
    }
}

#[cfg(feature = "ebpf")]
impl Enricher for EbpfEnricher {
    fn enrich(&self, log: Log) -> Log {
        // Phase 21 scaffold: delegate to ProcessEnricher. Phase 21.1
        // adds kernel-side attributes (cgroup, capabilities,
        // network 4-tuple).
        ProcessEnricher::new().enrich(log)
    }
}

// ---------------------------------------------------------------------------
// Composition
// ---------------------------------------------------------------------------

/// Chain two enrichers. Applied left-to-right.
#[derive(Debug, Clone, Copy)]
pub struct Chain<A, B> {
    /// First enricher applied.
    pub first: A,
    /// Second enricher applied.
    pub second: B,
}

impl<A, B> Chain<A, B> {
    /// Construct a chain of two enrichers.
    #[must_use]
    pub const fn new(first: A, second: B) -> Self {
        Self { first, second }
    }
}

impl<A: Enricher, B: Enricher> Enricher for Chain<A, B> {
    fn enrich(&self, log: Log) -> Log {
        self.second.enrich(self.first.enrich(log))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_enricher_adds_pid() {
        let e = ProcessEnricher::new();
        let out = e.enrich(Log::info("test"));
        let pid = out.attributes.get("pid").unwrap();
        assert!(pid.is_u64());
        assert!(pid.as_u64().unwrap() > 0);
    }

    #[cfg(unix)]
    #[test]
    fn process_enricher_adds_tid_and_uid_on_unix() {
        let e = ProcessEnricher::new();
        let out = e.enrich(Log::info("test"));
        let tid = out.attributes.get("tid").unwrap();
        assert!(tid.is_u64());
        assert!(tid.as_u64().unwrap() > 0);
        // UID may be 0 in a container; assert the field exists.
        assert!(out.attributes.contains_key("uid"));
    }

    #[test]
    fn process_enricher_preserves_original_attributes() {
        let e = ProcessEnricher::new();
        let log = Log::info("test").with("order_id", 42_u64);
        let out = e.enrich(log);
        assert_eq!(
            out.attributes.get("order_id").unwrap().as_u64(),
            Some(42)
        );
        assert!(out.attributes.contains_key("pid"));
    }

    #[test]
    fn process_enricher_new_and_default_are_both_constructible() {
        // `Default` is derived on this unit struct; explicit `new()`
        // exists for readability.
        let _new = ProcessEnricher::new();
        // `#[allow]` here because clippy nursery flags
        // `Default::default()` on a unit struct as redundant, but
        // we test the Default derive contract intentionally.
        #[allow(clippy::default_constructed_unit_structs)]
        let _default: ProcessEnricher = ProcessEnricher::default();
    }

    #[test]
    fn chain_applies_both_enrichers() {
        struct AddFoo;
        impl Enricher for AddFoo {
            fn enrich(&self, mut log: Log) -> Log {
                log.attributes.insert("foo".into(), Value::from("bar"));
                log
            }
        }

        struct AddBaz;
        impl Enricher for AddBaz {
            fn enrich(&self, mut log: Log) -> Log {
                log.attributes.insert("baz".into(), Value::from("qux"));
                log
            }
        }

        let chain = Chain::new(AddFoo, AddBaz);
        let out = chain.enrich(Log::info("test"));
        assert_eq!(
            out.attributes.get("foo").unwrap().as_str(),
            Some("bar")
        );
        assert_eq!(
            out.attributes.get("baz").unwrap().as_str(),
            Some("qux")
        );
    }

    #[test]
    fn chain_composes_with_process_enricher() {
        struct AddTag;
        impl Enricher for AddTag {
            fn enrich(&self, mut log: Log) -> Log {
                log.attributes
                    .insert("service".into(), Value::from("api"));
                log
            }
        }

        let chain = Chain::new(ProcessEnricher::new(), AddTag);
        let out = chain.enrich(Log::info("test"));
        assert!(out.attributes.contains_key("pid"));
        assert_eq!(
            out.attributes.get("service").unwrap().as_str(),
            Some("api")
        );
    }

    #[cfg(feature = "ebpf")]
    #[test]
    fn ebpf_enricher_scaffold_delegates_to_process() {
        let e = EbpfEnricher::new();
        let out = e.enrich(Log::info("test"));
        assert!(out.attributes.contains_key("pid"));
    }
}
