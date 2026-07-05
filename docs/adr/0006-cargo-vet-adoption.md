<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR 0006 — cargo-vet Audit Chain

- **Status:** Accepted
- **Date:** 2026-07-05
- **Phase:** 15 (per `docs/IMPLEMENTATION-PLAN-v0.1.0.md`)
- **Deciders:** repository maintainers
- **Related:** ADR 0005 (sigstore + SBOM) — orthogonal provenance
  layer covering the release artefact.
  ADR 0007 (cargo-deny hardening) — the licence + duplicate-version
  gate.

## Context

`cargo audit` catches *known* advisories. `cargo deny` catches
*policy* violations (licences, bans, duplicate versions). Neither
answers the question a security-conscious enterprise adopter asks
first:

> "Who — a human, not a bot — has actually read the source of the
> 400 transitive crates my product will pull in?"

`cargo-vet` fills that gap. It maintains a per-workspace audit
chain that either:

1. **Trusts an external auditor** — a project like Google, Mozilla,
   the Bytecode Alliance, or Zcash publishes a `supply-chain/
   audits.toml` naming crates its own engineers have reviewed at a
   given criteria level. Consuming projects import that file and
   inherit the trust.
2. **Adds a local audit** — the maintainer writes an entry in
   `supply-chain/audits.toml` stating they read the crate at a
   given version and confirm it meets a criteria level
   (`safe-to-run`, `safe-to-deploy`, `does-not-implement-crypto`,
   etc.).
3. **Exempts the crate** — a documented "we haven't audited this
   yet, but we accept the risk." Bootstrap exemptions are the
   compromise that makes cargo-vet adoption tractable for a
   workspace that starts with 200+ transitive deps.

Every dep must be covered by one of these three states. Anything
outside them fails `cargo vet --locked` and blocks the merge.

## Decision

Adopt cargo-vet (0.10) as the third supply-chain gate alongside
`cargo audit` and `cargo deny check`. Author the audit chain under
`supply-chain/`:

- `supply-chain/config.toml` — imports + exemptions.
- `supply-chain/audits.toml` — this workspace's own audits (empty
  at bootstrap; grows as reviews land).
- `supply-chain/imports.lock` — machine-generated pin of the
  imported audit sets.

CI workflow `.github/workflows/cargo-vet.yml` runs `cargo vet
--locked` on every PR that touches `crates/**`, `Cargo.toml`,
`Cargo.lock`, or the `supply-chain/` directory itself.

## Trusted imports

Four upstream audit sets are imported at bootstrap:

- **Bytecode Alliance** — the wasmtime project's audit set. Deep
  coverage of the `no_std` and low-level ecosystem crates.
- **Google (`google/rust-crate-audits`)** — Fuchsia + Chromium
  auditors. Broad coverage of proc-macro, serde, tokio adjacencies.
- **Mozilla (`mozilla/supply-chain`)** — Firefox's audit set. Deep
  coverage of the async runtime + crypto ecosystem.
- **Zcash** — Zebra chain's audit set. Excellent crypto and
  networking coverage.

These four project imports cover **81 crates fully + 2 partially**
of the workspace's 331-crate transitive tree at Phase 15 bootstrap,
so 248 exemptions remain.

## Bootstrap exemptions policy

Exemptions carry the criteria level `safe-to-deploy` (production
dep) or `safe-to-run` (dev-dep only). They are **not** guarantees
— they are IOUs that the maintainer intends to either:

- Audit locally in a subsequent PR and remove the exemption; or
- Wait for a trusted upstream to publish an audit and re-run
  `cargo vet prune` to inherit it.

The bootstrap set is a snapshot of the tree as-of the Phase 15
merge. Anything added post-bootstrap **must** be audited or
imported before the introducing PR merges. That is the value the
CI gate delivers — no silent additions.

## What cargo-vet does NOT check

- **Compile-time correctness.** That is `cargo check`'s job.
- **Runtime behaviour.** That is Miri, Loom, Kani, proptest.
- **Version drift.** That is `cargo-outdated` and Renovate/
  Dependabot.
- **Licence policy.** That is `cargo deny` (ADR 0007).
- **Known CVEs.** That is `cargo audit`.

cargo-vet's unique role is the human-in-the-loop attestation. It
cannot compensate for the other tools; it stacks with them.

## Consequences

- **CI cost.** ~30 s per PR (dominated by fetching the import
  audit sets). Negligible.
- **Contributor cost.** New dependencies now block CI. The fix is
  either:
  1. Wait for a trusted upstream to publish an audit and run
     `cargo vet prune`; or
  2. Audit locally with `cargo vet certify <crate> <version>
     safe-to-deploy` after reading the source; or
  3. Add a documented exemption with the justification in the PR
     description.
- **Ongoing maintenance.** Exemptions age. A follow-up phase will
  reduce the bootstrap 248 via targeted local audits of the most
  critical deps (regex, serde_json, tokio-adjacent).

## Alternatives considered

- **Skip cargo-vet.** Rejected — leaves the "who has read this?"
  question unanswered, which is a hard gate on enterprise
  procurement RFPs from 2026 onwards.
- **Local audits only, no imports.** Rejected — 331 audits from
  scratch is uneconomical and duplicates work Google / Mozilla /
  Bytecode Alliance / Zcash have already done publicly.
- **`cargo-crev`** (Distributed Web of Trust for Cargo). Considered.
  Rejected: broader trust model but weaker tooling integration and
  smaller adopter base. cargo-vet is the pragmatic 2026 default.

## References

- [cargo-vet book](https://mozilla.github.io/cargo-vet/)
- [Google rust-crate-audits](https://github.com/google/rust-crate-audits)
- [Mozilla supply-chain audits](https://github.com/mozilla/supply-chain)
- [Bytecode Alliance wasmtime audits](https://github.com/bytecodealliance/wasmtime/tree/main/supply-chain)
- [Zcash Zebra audits](https://github.com/zcash/zcash/tree/master/qa/supply-chain)
