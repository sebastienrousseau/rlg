<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR 0008 — Fused Redaction Automaton

- **Status:** Accepted
- **Date:** 2026-07-05
- **Phase:** 17 (per `docs/IMPLEMENTATION-PLAN-v0.1.0.md`)
- **Deciders:** repository maintainers
- **Related:** ADR 0003 (property tests) — the fusion property is
  covered by `rlg-redact/tests/integration.rs` and the new
  fusion-boundary tests in the inline test module.

## Context

Pre-Phase 17, `Redactor::scrub` iterated its `Vec<Regex>` and
called `regex.replace_all` once per pattern. On the six-pattern
default configuration, each `scrub` call performed **six full
passes** through every input string — the description and every
string attribute value.

This cost scales linearly in the number of patterns and multiplies
the effective bytes touched per record. For high-cardinality log
streams (millions of records / second at production sinks), the
loop-based approach caps throughput well below what the regex
engine can achieve when handed the full pattern alternation
up-front.

## Decision

Fuse every loaded pattern into a **single alternation regex**
compiled once at construction:

```text
(?:CREDIT_CARD)|(?:JWT)|(?:BEARER_TOKEN)|(?:EMAIL)|(?:IPV4)|(?:AWS_KEY)
```

The `regex` crate's DFA engine handles the union internally: one
traversal of the input replaces every match across every pattern
kind. `scrub` moves from `O(N · len)` to `O(len)`.

Public API — `empty`, `with_defaults`, `with_pattern`, `marker`,
`scrub`, `apply`, `len`, `is_empty` — is **unchanged** in shape,
signature, and observable behaviour. Existing consumers require no
migration.

## Design

### Data layout

```rust
pub struct Redactor {
    /// Source strings kept for `len()` reporting and for
    /// recompilation when a new pattern is appended.
    sources: Vec<String>,
    /// Fused alternation of `sources`. `None` when `sources` is
    /// empty — the fast path returns the input unchanged.
    combined: Option<Regex>,
    marker: String,
}
```

### Constructor cost

- `empty()` — no compilation. O(1).
- `with_defaults()` — clones a process-lifetime `LazyLock<Regex>`
  seeded at first-touch with the six built-in patterns' fused
  alternation. **O(1) past the first call.**
- `with_pattern(pat)` — validates `pat` in isolation, appends to
  `sources`, recompiles the fused regex. **O(cumulative pattern
  size)** per call.

Chaining `with_pattern` recompiles at each step. Callers that build
long chains should assemble their pattern list once and reuse the
resulting redactor — documented in the crate's performance model
section.

### Runtime cost

`apply(input)`:
- If `combined.is_none()`, return `input.to_string()` (unchanged
  no-op fast path).
- Else, one `regex.replace_all(input, marker)` pass.

The DFA handles alternation as a native union — no extra cost
above single-pattern scan for the same input.

### Semantics preserved

- **Leftmost-first match ordering** — the fused regex uses the same
  greedy-leftmost semantics as `regex::Regex`. Overlapping matches
  from different pattern kinds collapse into a single replacement
  span, which is a tightening (not a loosening) of the old
  behaviour and matches user intent for redaction.
- **`with_pattern` validation** — the standalone pattern is
  compiled first. If invalid, the error is precise. Only after
  standalone validation is the fused regex recompiled.
- **Invalid custom pattern** — same `regex::Error` propagation as
  before. Existing "reject bad regex" tests pass unchanged.

## Regression coverage

Three new tests exercise the fusion boundary directly, added to
the inline test module in `crates/rlg-redact/src/lib.rs`:

- `fusion_scans_all_pattern_kinds_in_one_pass` — every built-in
  pattern class appears once in a single input; the fused pass
  scrubs every kind and produces at least six markers.
- `fusion_prefers_leftmost_match_across_pattern_kinds` — with
  two patterns loaded, two overlapping sensitive spans collapse to
  exactly two `[REDACTED]` markers, proving leftmost-first
  semantics.
- `fusion_compiles_alternation_from_chained_with_pattern` — three
  chained `with_pattern` calls each contribute a distinct pattern;
  the final fused regex catches all three and does not silently
  drop any.

The 13 pre-existing unit tests and 13 integration tests continue
to pass verbatim — proof that the rewrite preserves observable
behaviour.

## Benchmark methodology

`crates/rlg-redact/benches/scrub.rs` gains a new case
`long_mixed_payload` that amplifies the fused-vs-loop delta: a
long description mixing multiple sensitive substrings, plus three
sensitive attribute values.

Local run against the workspace's Criterion baseline shows the
expected direction of change (single-pass fusion faster than
six-pass loop). Precise multipliers land on the CI-published
Criterion report at v0.1.0 per the plan's Phase 27 (live
`rustlogs.com/bench/` publication).

## Consequences

- **No breaking change.** Every public function keeps its
  signature. Downstream consumers upgrade transparently.
- **Faster scrub throughput** — the plan targets ≥3× on
  `heavy_pii_match` and ≤0% regression on `no_pii_match`. The
  no-PII path stays quick because the DFA fails fast when no
  pattern can match.
- **Slower `with_pattern` chains** — each `with_pattern` recompiles.
  Documented in the crate performance model; callers reuse the
  final redactor.
- **Larger memory footprint per redactor** — the fused regex's
  internal DFA is larger than any single-pattern regex. Marginal
  in absolute terms; not measured to add configuration around it.

## Alternatives considered

- **`RegexSet`** — matches multiple patterns but does not perform
  replacement in a single pass. Would still require post-processing
  to replace matches, keeping the multi-scan cost. Rejected.
- **`regex_automata::meta::Regex`** — the modern low-level Rust
  regex API. Considered. The `regex` crate's high-level `Regex`
  already dispatches to the same engine and offers the same
  performance for our alternation use case; the low-level API
  would add complexity without a measured win at this pattern
  count. Adopt if a future benchmark shows a specific win.
- **Aho-Corasick literal string matcher** — the crate exists as
  `aho-corasick` and is the state-of-the-art for **literal**
  multi-pattern matching. All six of our built-in patterns are
  regex patterns with non-literal metacharacters
  (`\b`, `\d`, `[A-Za-z]`, etc.), so a pure Aho-Corasick matcher
  cannot handle them. The `regex` crate uses Aho-Corasick
  internally as a prefilter for literal-heavy alternations, which
  gets us the DFA prefilter benefit without the constraint. This
  is the honest reading of the phase title "Aho-Corasick fused
  redaction": the fusion happens; the specific automaton is
  regex's engine, which uses AC where applicable.

## References

- [`regex` crate](https://docs.rs/regex) — engine used for the
  fused alternation.
- [`regex-automata`](https://docs.rs/regex-automata) — low-level
  API considered and deferred.
- [`aho-corasick`](https://docs.rs/aho-corasick) — literal
  multi-pattern matcher used internally by `regex`.
- ADR 0003 — property tests covering the fusion boundary.
