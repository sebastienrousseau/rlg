<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR 0003 — Property-Tested Formats & Filter

- **Status:** Accepted
- **Date:** 2026-07-05
- **Phase:** 12 (per `docs/IMPLEMENTATION-PLAN-v0.1.0.md`)
- **Deciders:** repository maintainers
- **Related:** ADR 0002 (fuzz strategy) — same invariant surface,
  different exploration.

## Context

The 14 `LogFormat` variants each carry an implicit contract:

- **Never panic** on any legal `Log`.
- **NDJSON is single-line** by definition — one record per line.
- **JSON, NDJSON, MCP, ECS** produce valid UTF-8 that downstream
  parsers can consume.
- **The serde canonical form round-trips** — `serde_json::to_string`
  → `parse_record` → `Log` is a fixed point.

`rlg_cli::Filter` carries three more:

- **The default filter accepts every record** — CLI usage without
  flags never silently drops lines.
- **`min_level` is monotone** — if a stricter filter accepts a
  record, a relaxed one must too. Downstream aggregation
  (`rlg-mcp::filter_log`, `rlg-report`) relies on this to combine
  level ranges.
- **Component filter is exact-match only** — no substring surprises.

Unit tests exercise these on hand-picked inputs. Nothing exhaustively
explores the input space.

## Decision

Adopt `proptest` (1.5) as the structured input generator for these
seven invariants. Author the proofs under two integration test
files:

- `crates/rlg/tests/proptest_round_trip.rs` — four properties on
  `Log` and its `Display` impls.
- `crates/rlg-cli/tests/proptest_filter.rs` — three properties on
  `Filter`.

Each property runs the proptest default of 256 cases per CI
execution. Failures shrink to a minimal counter-example that lands
directly in the CI log for actionable triage.

## Model

Strategies are **restricted intentionally** in the string domain
(`[a-zA-Z0-9 _\-./:]{0,32}`) to focus proptest on the shape /
combination axes rather than on the escape-heavy corner of the
UTF-8 space. Escape correctness is a fuzz-target concern (see
ADR 0002); property tests should not fight with it.

`session_id`, `level`, `format`, and the numeric attribute values
use the full unrestricted `any::<T>()` strategies.

## Findings surfaced by this ADR

`Log::fmt` for `LogFormat::JSON` produces PascalCase field names
(`SessionID`, `Component`, `Description`, `Format`, `Level`,
`Timestamp`, `Attributes`), while `rlg_cli::parse_record` expects
the serde-default snake_case shape (`session_id`, `component`, …).

The two shapes are not interchangeable. `parse_record(format!("{log}"))`
does **not** round-trip when `log.format == JSON` — even though
downstream consumers reasonably assume it should.

The property is retained in the form
`parse_record(serde_json::to_string(&log)) == log`, which proves the
serde canonical form does round-trip.

The Display/serde asymmetry is queued as a **v0.1.0 API-alignment
task**: unify `Log::fmt` for `LogFormat::JSON` onto the serde shape.
This is a breaking change for any consumer parsing the current
PascalCase output; landing it will carry an ADR of its own and a
one-release deprecation window.

## What is not proven

- **Escape correctness for exotic UTF-8** — fuzz targets (ADR 0002)
  do that.
- **Format-specific validation** — CLF / CEF / W3C / Apache /
  Log4jXML shapes have precise byte-level requirements verified by
  targeted unit tests in `log_format.rs`, not by property tests.
- **Filter attribute matching** — the attribute-based Filter branch
  is exercised only by the integration tests today. A follow-up
  proptest can extend coverage once the shape stabilises.

## Consequences

- **CI cost.** Negligible: ~200 ms per proptest suite at 256 cases.
- **Contributor cost.** Local reproducer:
  ```bash
  cargo test -p rlg --test proptest_round_trip
  cargo test -p rlg-cli --test proptest_filter
  ```
- **Shrinking output.** Proptest counter-examples appear directly in
  test failure output. No extra tooling required.
- **v0.1.0 breaking-change ticket.** The Display/serde asymmetry
  finding above enters the v0.1.0 backlog. It is not fixed in this
  phase.

## Alternatives considered

- **`quickcheck`** — simpler API but weaker shrinking. Proptest's
  shrinking makes minimum-repro cases trivial to inspect. Rejected.
- **Hand-rolled generators** — reproducibility is a non-goal at this
  layer, and proptest's macro handles shrinking automatically.
  Rejected.

## References

- [`proptest` book](https://proptest-rs.github.io/proptest/)
- Contract-based testing precedent in the wider Rust ecosystem:
  serde's own proptest suite, tokio's runtime tests.
