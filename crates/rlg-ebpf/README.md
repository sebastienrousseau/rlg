<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/rlg/v1/logos/rlg.svg" alt="RLG logo" width="128" />
</p>

<h1 align="center">rlg-ebpf</h1>

<p align="center">
  Kernel-context enrichment for <code>rlg</code> records. Attaches
  PID, TID, UID, and (future) network 4-tuple to every log record.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/rlg/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rlg/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/rlg-ebpf"><img src="https://img.shields.io/crates/v/rlg-ebpf.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/rlg-ebpf"><img src="https://img.shields.io/badge/docs.rs-rlg--ebpf-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://lib.rs/crates/rlg-ebpf"><img src="https://img.shields.io/badge/lib.rs-rlg--ebpf-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/rlg"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/rlg?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" /></a>
</p>

---

## Install

```toml
[dependencies]
rlg-ebpf = "0.0.11"
```

Requires Rust **1.88.0** or newer (edition 2024).

## Usage

```rust
use rlg::log::Log;
use rlg_ebpf::{Enricher, ProcessEnricher};

let enricher = ProcessEnricher::new();
let log = Log::info("checkout completed");
let enriched = enricher.enrich(log);

// enriched.attributes now contains "pid", plus "tid" and "uid"
// on Unix targets.
```

Chain multiple enrichers with `Chain`:

```rust
use rlg_ebpf::{Chain, Enricher, ProcessEnricher};

struct AddService;
impl Enricher for AddService { /* … attach service name … */ }

let pipeline = Chain::new(ProcessEnricher::new(), AddService);
```

## Enrichers

| Enricher | Fields attached | Platform | Feature |
|---|---|---|---|
| `ProcessEnricher` | `pid`, `tid` (Unix), `uid` (Unix) | Portable | (default) |
| `EbpfEnricher` | delegates to `ProcessEnricher` (Phase 21.1: adds `cgroup`, `caps`, network 4-tuple) | Linux | `ebpf` |
| `Chain<A, B>` | union of the two enrichers, applied left-to-right | Portable | (default) |

## `EbpfEnricher` status: scaffold

The `ebpf` feature enables an `EbpfEnricher` type whose live
kernel-side program attach lands in Phase 21.1. Today the type
delegates to `ProcessEnricher` for correctness — future consumers
of the type get the extra kernel context transparently once
Phase 21.1 lands. See [`docs/adr/0012-ebpf-enricher.md`](../../docs/adr/0012-ebpf-enricher.md).

## Capability requirements

Attaching the future eBPF program will require either `CAP_BPF`
(Linux 5.8+) or `CAP_SYS_ADMIN` (older kernels). The current
`ProcessEnricher` has no capability requirements — pid/tid/uid are
readable from any process.

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.
