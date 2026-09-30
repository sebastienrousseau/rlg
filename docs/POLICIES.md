<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Policies

## Minimum supported Rust version

**The floor is Rust 1.88.0** (edition 2024), declared as `rust-version`
in every crate.

- **What it covers**: building every library and binary in the
  workspace, with all features, from the committed `Cargo.lock`. The CI
  job `MSRV (1.88.0) builds` runs `cargo +1.88.0 check --locked
  --workspace --all-features --lib --bins` on every pull request.
- **What it does not cover**: the test suite. Dev-dependencies may need a
  newer toolchain (today `serial_test` 4 needs 1.93.1); contributors use
  the version pinned in `mise.toml`.
- **When it may rise**: in any release, when a dependency or a language
  feature needs it. A rise is its own commit, lists the new floor and the
  reason under `Changed` in [`CHANGELOG.md`](https://github.com/sebastienrousseau/rlg/blob/main/CHANGELOG.md), and moves the
  CI job and this page in the same change.
- **Distributions**: rlg makes no claim about the Rust shipped by any
  Linux distribution's long-term release.

## Versioning

- All ten publishable crates share one version and are released
  together.
- Releases go `0.0.1` at a time (`0.0.12` → `0.0.13`); `0.1.0` follows
  `0.0.999`.
- Under Cargo's SemVer rules every `0.0.x` release may be breaking. Each
  release's CHANGELOG section marks breaking changes as **Breaking**, and
  `cargo-semver-checks` runs on every pull request so none is
  accidental.
- There is no deprecation window before `0.1.0`: a removed item is gone
  in the release that removes it, with the replacement named in the
  CHANGELOG.

## Output stability

rlg's formats are an interface: other programs parse them. A change to
the bytes a format produces for the same record (a renamed key, a
reordered field, different escaping) is a breaking change even when no
Rust signature moves. It is marked **Breaking** in the CHANGELOG, and the
format tests that pin the output are updated in the same change.

## Security fixes

Only the latest release receives fixes; see
[`SECURITY.md`](https://github.com/sebastienrousseau/rlg/blob/main/SECURITY.md).
