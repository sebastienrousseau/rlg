<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR 0012 — eBPF Enricher (Phase 21: scaffold + portable enrichment)

- **Status:** Accepted
- **Date:** 2026-07-05
- **Phase:** 21 (per `docs/IMPLEMENTATION-PLAN-v0.1.0.md`)
- **Deciders:** repository maintainers
- **Related:** ADR 0010 (OTLP pluggable transport), ADR 0011
  (io_uring file sink) — same scaffold-then-fill pattern.

## Context

Enterprise deployments running multi-tenant workloads on a shared
host need to correlate log lines back to the specific process,
thread, or user that produced them. Traditional practice: join
against `/proc` off-line, or run each tenant in a separate
container to segment logs by hostname. Both are lossy and add
operational drag.

The plan's Phase 21 called for a new `rlg-ebpf` crate that
attaches this context via an eBPF program hooked into the kernel,
adding PID / TID / cgroup / UID / optional network 4-tuple to
every record.

The full eBPF path requires:

- A live `aya`-based BPF program compiled at build time.
- Kernel headers and BPF tooling in the build environment.
- `CAP_BPF` (Linux 5.8+) or `CAP_SYS_ADMIN` at runtime.
- CI infrastructure that supports privileged containers or
  BPF-capable runners.

None of that is portable. And the enrichment fields most
enterprises actually want first — PID, TID, UID — are readable
from userspace via `libc` on any Unix, no privileges required.

## Decision

Split Phase 21 into three sub-phases:

### Phase 21 (this commit) — portable enrichment + eBPF scaffold

- New crate `rlg-ebpf` (workspace member 11).
- Public `Enricher` trait with a single method
  `fn enrich(&self, log: Log) -> Log`.
- `ProcessEnricher` impl:
  - **PID** via `std::process::id()`. Portable.
  - **TID** via `libc::syscall(SYS_gettid)` on Linux,
    `libc::pthread_self()` cast to `u64` on other Unix targets.
    Absent on non-Unix.
  - **UID** via `libc::getuid()`. Absent on non-Unix.
- `EbpfEnricher` scaffold behind the `ebpf` feature. Its final
  implementation lands in Phase 21.1; the type delegates to
  `ProcessEnricher` today so consumers who select this type
  transparently get the extra kernel-side context when 21.1 lands.
- `Chain<A, B>` combinator for composing enrichers.
- 12 unit + integration tests, criterion bench, README, example.

### Phase 21.1 (follow-up) — `aya`-based BPF attach

- Add `aya 0.13` dep behind the existing `ebpf` feature.
- Compile a minimal BPF program that attaches to `sched_process_exec`
  and populates a BPF map with `(pid, cgroup_id, ambient_caps)`.
- `EbpfEnricher::enrich` reads from that map before delegating to
  `ProcessEnricher`.
- CI: privileged Linux runner via `--privileged` docker or
  `sudo -E` bpftool.

### Phase 21.2 (follow-up) — Windows enrichment

- `winapi` bindings for `GetCurrentThreadId`, `GetCurrentProcess`.
- `WindowsProcessEnricher` type.
- Feature gate to keep the Unix-only libc dep off Windows builds.

## What Phase 21 IS

- **A portable enricher trait shipping today.** Anyone on Linux,
  macOS, or FreeBSD gets PID/TID/UID enrichment without special
  privileges.
- **A scaffolded `EbpfEnricher` type** whose surface is stable.
  Phase 21.1 fills in the kernel-side attach without a breaking
  change.
- **A composition primitive (`Chain`)** so users can layer
  enrichers on top of each other — first application, then
  process context, then eBPF context.

## What Phase 21 is NOT

- **Not a kernel-side program.** The `ebpf` feature enables the
  type; the SEC() program lands in Phase 21.1.
- **Not privileged.** `ProcessEnricher` reads userspace state that
  every process has access to.
- **Not Windows-ready.** The Unix path uses libc unconditionally
  under `[target.'cfg(unix)']`. Windows enrichment is Phase 21.2.

## FFI safety

The workspace policy is `unsafe_code = "deny"` via
`[lints.rust]`. The `unix_ffi` module uses `#[allow(unsafe_code)]`
to wrap three `libc` calls:

- `libc::syscall(SYS_gettid)` — no arguments, returns `pid_t`.
- `libc::pthread_self()` — no arguments, returns thread handle.
- `libc::getuid()` — no arguments, returns `uid_t`.

Every call has a `// SAFETY:` comment justifying it. The FFI is
exclusively in one `#[allow(unsafe_code)]` sub-module; the rest
of the crate carries the workspace-default `deny`.

Note: the `unsafe_code` policy is applied via Cargo.toml
`[lints.rust]` (as `deny`, not `forbid`) so the sub-module allow
takes effect. `forbid` at the crate root is what would prevent
this pattern — the same trade-off `rlg::sink` makes for
`syslog(3)`.

## Consequences

- **New publishable crate.** Ships as `rlg-ebpf 0.0.11` to
  crates.io alongside the rest of the workspace at the next tag
  push.
- **Cross-Unix binary compatibility.** libc is the least-common-
  denominator dep; no build.rs, no BPF toolchain, no privilege
  escalation.
- **Deferred value.** Users who need the actual eBPF path today
  can't get it from this commit; Phase 21.1 delivers.
- **Bench target.** `<5 µs per record` is the plan's threshold.
  The current `ProcessEnricher` measurements will land on the
  live Criterion report at v0.1.0.

## Alternatives considered

- **Full Phase 21 in one commit.** Rejected on CI-risk grounds:
  the BPF toolchain, privileged runners, and cross-platform
  build-system dance would burn multiple CI iterations before
  landing green. Scaffold-then-fill matches the Phase 19c and
  Phase 20 pattern.
- **`libbpf-rs` instead of `aya`.** Considered for Phase 21.1.
  `aya` is pure Rust with no `libbpf` C build; `libbpf-rs`
  requires system `libbpf`. `aya` wins on build hygiene.
- **`procfs` crate instead of libc.** `procfs` is Linux-only and
  reads `/proc` filesystem. libc syscalls are faster, work on
  more Unix variants, and don't parse text. libc wins.
- **Skip `EbpfEnricher` scaffold, defer whole eBPF surface.**
  Rejected — establishing the type name now means Phase 21.1
  ships without a breaking change.

## References

- [`aya` book](https://aya-rs.dev/book/)
- [Linux BPF documentation](https://www.kernel.org/doc/html/latest/bpf/index.html)
- [libc's `getuid(2)` man page](https://man7.org/linux/man-pages/man2/getuid.2.html)
- ADR 0010 (OTLP transport) and ADR 0011 (io_uring sink) —
  companion scaffold-then-fill patterns.
