<!-- mdBook part titles are level-1 headings by design. -->
<!-- markdownlint-disable-file MD025 -->

# Summary

[Introduction](introduction.md)

# Using rlg

- [Getting started](tutorials/getting-started.md)
- [The fluent API](how-to/fluent-api.md)
- [Migrating from `log`](migration/from-log.md)
- [Migrating from `slog`](migration/from-slog.md)
- [Migrating from `tracing`](migration/from-tracing.md)

# How it works

- [Architecture](ARCHITECTURE.md)
- [Engine design](explanation/engine-design.md)
- [Safety: Miri and FFI](explanation/safety.md)
- [Logs as MCP tools](whitepapers/01-logs-as-mcp-tools.md)

# Decisions

- [Architecture decision records](adr/README.md)
  - [ADR 0001 — Loom-Verified Shutdown Handshake](adr/0001-loom-verified-ring-buffer.md)
  - [ADR 0002 — Fuzz Strategy](adr/0002-fuzz-strategy.md)
  - [ADR 0003 — Property-Tested Formats & Filter](adr/0003-property-tested-formats.md)
  - [ADR 0004 — Kani-Verified Invariants](adr/0004-kani-verified-invariants.md)
  - [ADR 0005 — Sigstore + SBOM on every release](adr/0005-sigstore-and-sbom.md)
  - [ADR 0006 — cargo-vet Audit Chain](adr/0006-cargo-vet-adoption.md)
  - [ADR 0007 — cargo-deny Hardened](adr/0007-cargo-deny-hardened.md)
  - [ADR 0008 — Fused Redaction Automaton](adr/0008-fused-redaction-automaton.md)
  - [ADR 0009 — Sharded Producer Queue](adr/0009-sharded-producer-queue.md)
  - [ADR 0010 — OTLP Pluggable Transport (Phase 19a: reliability primitives)](adr/0010-otlp-pluggable-transport.md)
  - [ADR 0011 — io_uring File Sink (Phase 20: scaffold)](adr/0011-io-uring-file-sink.md)
  - [ADR 0012 — eBPF Enricher (Phase 21: scaffold + portable enrichment)](adr/0012-ebpf-enricher.md)
  - [ADR 0013 — WASI 0.2 Component Model for rlg-wasm (Phase 22: scaffold)](adr/0013-wasi-0.2-component.md)
  - [ADR 0014 — `no_std` Core (Phase 23: strategy + gate)](adr/0014-no-std-core.md)
  - [ADR 0015 — OTLP Through a Local Collector (no TLS in-process)](adr/0015-otlp-local-collector-transport.md)

# Project

- [Policies](POLICIES.md)
- [Release 0.0.13 highlights](releases/v0.0.13.md)
- [Fuzzing and OSS-Fuzz](OSS-FUZZ.md)
- [Implementation plan to 0.1.0](IMPLEMENTATION-PLAN-v0.1.0.md)
