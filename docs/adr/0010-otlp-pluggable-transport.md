<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR 0010 — OTLP Pluggable Transport (Phase 19a: reliability primitives)

- **Status:** Accepted
- **Date:** 2026-07-05
- **Phase:** 19a (per `docs/IMPLEMENTATION-PLAN-v0.1.0.md`)
- **Deciders:** repository maintainers
- **Related:** ADR 0009 (sharded producer queue) — same "extract
  what varies, keep public API stable" pattern.

## Context

Phase 19 in the v0.1.0 plan calls for a pluggable transport
abstraction over `rlg-otlp`: sync `ureq`, async `reqwest`, and
gRPC via `tonic` + `opentelemetry-proto`. The full delivery is
estimated at ~1 500 LOC across 15 files with new dependency
graphs, wiremock-based integration tests, and per-transport
benchmarks.

Landing that in one commit against a workspace running fmt +
clippy (workspace) + clippy (pedantic + nursery via pipelines) +
Miri + Loom + Kani + cargo-vet + cargo-deny gates is high-risk
for merge-conflict-driven CI iteration cost. The correctness
value is real; the delivery risk is not proportional.

## Decision

Split Phase 19 into three sub-phases and land the highest-value,
lowest-risk slice first:

### Phase 19a (this commit) — reliability primitives

Extract retry / jitter / circuit breaker into
`crates/rlg-otlp/src/backoff.rs` as transport-agnostic primitives.
The sync HTTP path in `lib.rs` uses them today; every future
transport reuses them without duplicating the reliability logic.

**Delivered here:**

- **`RetryPolicy`** — configurable `max_retries`, `base`,
  `max_delay`, and `jitter` fraction. `delay(attempt, rng_0_to_1)`
  implements AWS-style "full jitter" backoff:
  `sleep = base * 2^attempt` capped at `max_delay`, then
  `[0, delay]` uniform on the jitter fraction.
- **`CircuitBreaker`** — tokens-per-window model. Failure consumes
  a token; success refunds one. Window rollover refills to full
  budget. Breaker is `Arc<Mutex<State>>` so clones share state;
  lock poisoning is recovered from silently (poison isn't security
  in this context).
- **`OtlpError::CircuitOpen`** — new error variant surfaced when
  the breaker rejects a request without touching the network.
- **`OtlpExporterBuilder::circuit(Arc<CircuitBreaker>)`** — opt-in
  breaker per exporter. Existing consumers who don't call it get
  identical behaviour to the pre-Phase-19 exporter.

**Test coverage (12 new backoff tests):**

- `RetryPolicy` — base delay, doubling, cap, jitter bound at
  `rng=0` and `rng=1`, high-attempt no-panic.
- `CircuitBreaker` — closed by default, trips after budget
  exhausted, resets after window, success refill, success cap,
  survives lock poison.
- `cheap_random_0_to_1` — range assertion over many samples.

### Phase 19b (follow-up) — async HTTP transport

Add `async` feature: `reqwest` (rustls-tls default) +
runtime-agnostic `Transport` trait. `AsyncOtlpExporter::export_one`
and `export_batch` returning `impl Future`. Reuses
`RetryPolicy` / `CircuitBreaker` from Phase 19a.

**Scope estimate:** ~500 LOC + wiremock integration tests + one
new example.

### Phase 19c (follow-up) — gRPC transport

Add `grpc` feature: `tonic` + `opentelemetry-proto`. New
`GrpcOtlpExporter` against the OTLP/gRPC protocol. Same
reliability primitives.

**Scope estimate:** ~700 LOC + tonic mock server tests + one new
example demonstrating a real `otelcol` gRPC endpoint.

## Why this split

- **Correctness value stacks.** Phase 19a's retry-with-jitter is
  the reliability improvement enterprise adopters actually need
  first — a poorly-jittered fleet can synchronise retries and
  DDoS the collector. Circuit-breaking prevents cascading failure
  storms.
- **Transport work depends on the primitives.** Every future
  transport reuses `RetryPolicy` and `CircuitBreaker`. Landing
  them first removes duplication from Phases 19b and 19c.
- **CI risk is proportional to diff size.** A ~250 LOC commit
  lands green faster than a ~1 500 LOC commit; the plan's
  discipline of "must always be green" makes staged delivery
  strictly cheaper.

## What is intentionally NOT delivered here

- The `Transport` trait. Introducing it now with only one impl
  (`ureq`) is a speculative abstraction. Phase 19b adds it
  alongside the second impl, where the trait's boundary can be
  designed against two concrete uses.
- Async or gRPC transports.
- Wiremock-based integration tests. Follow-up phases.

## Consequences

- **Public API unchanged in shape.** The only additions are the
  new `OtlpError::CircuitOpen` variant and the
  `OtlpExporterBuilder::circuit` builder method. `export_one`,
  `export_batch`, `serialise_batch`, and the existing builder
  methods keep their signatures.
- **SemVer.** Additive-only change to the public enum
  (`CircuitOpen` is a new variant). Downstream `match` statements
  on `OtlpError` without a wildcard arm will need to add one —
  documented in the `CHANGELOG.md` at v0.1.0.
- **Default behaviour preserved.** Consumers who don't call
  `.circuit(...)` get identical retry-then-error semantics to the
  pre-Phase-19a exporter, with the improvement that the retry
  delay now includes full jitter instead of a deterministic
  `base * 2^attempt` sequence.
- **Test count grows from 22 → 34** in `rlg-otlp`. Every new test
  targets the reliability primitives directly.

## Alternatives considered

- **Land the full Phase 19 in one commit.** Rejected on CI-risk
  grounds — see §"Why this split".
- **Skip Phase 19a and jump to async.** Rejected — the async
  transport would ship without jitter or circuit-breaking, or
  would duplicate the reliability logic that a shared primitive
  now removes.
- **Drop circuit-breaking as speculative.** Rejected — enterprise
  adopters running rlg-otlp against a shared collector need the
  breaker to survive collector outages without saturating the
  fleet's retry paths. It is table stakes at the sizes rlg
  targets.

## References

- [AWS Architecture Blog: Exponential Backoff and Jitter](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/)
- [Envoy's HTTP circuit breaker docs](https://www.envoyproxy.io/docs/envoy/latest/intro/arch_overview/upstream/circuit_breaking)
- ADR 0009 — sharded producer queue (companion "extract what
  varies, keep public API stable" refactor).
