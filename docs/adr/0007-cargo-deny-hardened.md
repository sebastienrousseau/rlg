<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR 0007 — cargo-deny Hardened

- **Status:** Accepted
- **Date:** 2026-07-05
- **Phase:** 16 (per `docs/IMPLEMENTATION-PLAN-v0.1.0.md`)
- **Deciders:** repository maintainers
- **Related:** ADR 0005 (sigstore + SBOM), ADR 0006 (cargo-vet
  audit chain). Together these three form the workspace's
  supply-chain moat.

## Context

`deny.toml` was previously an *advisory* configuration: it emitted
warnings for duplicate versions and had empty ban / source lists.
CI ran `cargo deny check` and moved on regardless of warnings.

Advisory-mode dependency policy is a stated policy that isn't
enforced. Every enterprise adopter's supply-chain reviewer treats
it as a false claim. Wave 1 closes with the pragmatic tightening.

## Decision

Flip every advisory to enforced. Concretely:

### `[bans]`

- **`multiple-versions = "deny"`** — was `"warn"`. Duplicate
  versions of the same crate now fail CI unless explicitly
  skipped with a documented reason.
- **`wildcards = "deny"`** — new. `Cargo.toml` may not declare a
  workspace dep with a wildcard version range. All existing
  workspace deps already pin to concrete ranges.
- **Documented skips** for five known duplicate-version cases the
  ecosystem forces on us:

  | Skip | Cause |
  |---|---|
  | `toml 0.8.*` | `config` crate depends on old toml |
  | `toml_datetime 0.6.*` | (same) |
  | `serde_spanned 0.6.*` | (same) |
  | `winnow 0.7.*` | (same) |
  | `hashbrown 0.14.*` | Ubiquitous transitive; `indexmap` / `criterion` / `config` haven't converged on 0.16 |

  Each entry cites the upstream that pulls in the older version.
  As those crates upgrade, we remove the corresponding skip.

- **New `deny` list** — preventive bans on three crates *not
  currently in the tree*. Their transitive introduction through a
  careless dep bump would fail CI and force a discussion:

  | Deny | Reason |
  |---|---|
  | `openssl-sys` | rlg-otlp uses `rustls`; libssl on the target host is a supply-chain footgun |
  | `native-tls` | Same reason as openssl-sys |
  | `chrono` | rlg uses `jiff` and in-house datetime helpers; chrono has a history of breakage and a large-attack-surface C locale path |

### `[sources]`

- **`unknown-registry = "deny"`** — was default. Every dep must
  come from crates.io (or workspace-local path deps, which
  cargo-deny allows automatically).
- **`unknown-git = "deny"`** — no git deps allowed. If we ever
  need one, it enters the whitelist explicitly.
- **`allow-registry = ["https://github.com/rust-lang/crates.io-index"]`** —
  crates.io is the sole registry.

### `[licenses]` unchanged

The existing licence allowlist (MIT, Apache-2.0, Unicode-3.0,
Unicode-DFS-2016, ISC, CC0-1.0, BSL-1.0, Zlib, Unlicense,
BSD-3-Clause) already covers the tree. No changes.

## Blockers surfaced by the flip

Two required immediate resolution before the tightening could
merge green:

1. **`rlg-cli` wildcard dev-dep.** `crates/rlg/Cargo.toml` added
   `rlg-cli = { path = "../rlg-cli" }` in Phase 12 without a
   version constraint. The path dep alone is a wildcard from
   cargo-deny's perspective. Fixed by adding
   `version = "0.0.11"`.
2. **`hashbrown 0.15` orphaned skip.** The initial skip list
   included 0.15.* speculatively; the actual tree only uses
   0.14 + 0.16. Pruned to the version we actually see.

Both fixes are in the same commit as the deny.toml tightening so
CI stays green on the introducing PR.

## Consequences

- **No CI cost.** `cargo deny check` already runs via
  `sebastienrousseau/pipelines/security.yml`. This ADR strengthens
  the policy the existing job enforces — same job, tighter gate.
- **Contributor cost.** New dep must now be added to a whitelisted
  registry (crates.io) with a pinned version. A transitively-added
  chrono / openssl-sys / native-tls fails CI with a clear error
  message pointing at this ADR.
- **Ongoing maintenance.** The five documented skips get pruned as
  the upstream crates converge on newer versions. `cargo deny
  check` surfaces stale skips as `unmatched-skip` warnings.

## What cargo-deny does NOT check

- **CVEs.** That is `cargo audit` (already in CI via
  `security.yml`).
- **Human review of source.** That is `cargo vet` (ADR 0006).
- **Correctness / behavioural bugs.** That is Miri / Loom / Kani /
  proptest.
- **Reproducible builds.** Out of scope for the workspace.

## References

- [cargo-deny book](https://embarkstudios.github.io/cargo-deny/)
- [`deny.toml`](../../deny.toml)
- Prior ADRs in this series: 0005 (sigstore + SBOM), 0006
  (cargo-vet).
