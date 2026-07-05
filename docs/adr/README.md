<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Architectural Decision Records

Every non-trivial architectural decision on the rlg workspace
lands as an ADR under this directory. The convention:

- One file per decision.
- Filename `NNNN-short-slug.md`.
- Frontmatter: Status (Proposed / Accepted / Superseded by NNNN /
  Deprecated), Date, Phase, Deciders, Related.
- Body: Context, Decision, Consequences, Alternatives considered,
  References.

## Index

| ADR | Title | Phase | Status |
|---|---|---|---|
| [0001](0001-loom-verified-ring-buffer.md) | Loom-Verified Shutdown Handshake | 10 | Accepted |
| [0002](0002-fuzz-strategy.md) | Fuzz Strategy | 11 | Accepted |
| [0003](0003-property-tested-formats.md) | Property-Tested Formats & Filter | 12 | Accepted |
| [0004](0004-kani-verified-invariants.md) | Kani-Verified Invariants | 13 | Accepted |
| [0005](0005-sigstore-and-sbom.md) | Sigstore + SBOM on every release | 14 | Accepted |
| [0006](0006-cargo-vet-adoption.md) | cargo-vet Audit Chain | 15 | Accepted |
| [0007](0007-cargo-deny-hardened.md) | cargo-deny Hardened | 16 | Accepted |
| [0008](0008-fused-redaction-automaton.md) | Fused Redaction Automaton | 17 | Accepted |
| [0009](0009-sharded-producer-queue.md) | Sharded Producer Queue | 18 | Accepted |
| [0010](0010-otlp-pluggable-transport.md) | OTLP Pluggable Transport | 19a/b/c | Accepted |
| [0011](0011-io-uring-file-sink.md) | io_uring File Sink | 20 | Accepted |
| [0012](0012-ebpf-enricher.md) | eBPF Enricher | 21 | Accepted |
| [0013](0013-wasi-0.2-component.md) | WASI 0.2 Component Model | 22 | Accepted |
| [0014](0014-no-std-core.md) | `no_std` Core | 23 | Accepted |

## Reading order for a new maintainer

1. **0009** (sharded queue) + **0001** (Loom-verified handshake)
   — how the ingest hot path is shaped and proved.
2. **0008** (fused redaction) + **0017** (Aho-Corasick) — same
   pattern applied to the redactor.
3. **0010** (OTLP transport) + **0011** (io_uring) + **0012**
   (eBPF) + **0013** (WASI 0.2) + **0014** (`no_std`) — the
   scaffold-then-fill pattern that unifies Wave 2 and Wave 3.
4. **0005** (sigstore/SBOM) + **0006** (cargo-vet) + **0007**
   (cargo-deny hardened) — the supply-chain moat.
5. **0002** (fuzz) + **0003** (proptest) + **0004** (Kani) —
   correctness proofs stacked with Loom.

## Authoring a new ADR

Copy `0014-no-std-core.md` as a template. It's the newest and
uses the current header shape. Update the number, slug, phase,
and content.

Register the new ADR in the index above.
