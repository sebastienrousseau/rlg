<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Development

The single entry point for working on rlg: toolchain, the gates CI runs
and how to run each locally, where the tests live, and how a release is
made. The repository's rules are in [`AGENTS.md`](AGENTS.md); commit and
signing conventions in [`CONTRIBUTING.md`](CONTRIBUTING.md).

## Toolchain

- **Rust**: the minimum supported version is **1.88.0** (edition 2024),
  set as `rust-version` in each crate. Development uses the version
  pinned in `mise.toml` (`mise install`).
- **Nightly** is needed only for Miri and fuzzing.
- **Tools** the gates call: `cargo-deny`, `cargo-vet`,
  `cargo-semver-checks`, `cargo-tarpaulin`, `cargo-kani`,
  `cargo-fuzz`, `rust-code-analysis-cli` 0.0.25, `mdbook`, `lychee`.

## The one command

```bash
make verify
```

Formatting, clippy with warnings denied, the full test suite,
cargo-semver-checks, cargo-deny, cargo-vet, the complexity gate and the
version checks. Run it before opening a pull request.

## Every CI gate, locally

| Gate | Command | Workflow |
| :--- | :--- | :--- |
| Format | `cargo fmt --all --check` | `ci.yml` |
| Lint | `cargo clippy --workspace --all-features --tests --benches --examples -- -D warnings` | `ci.yml` |
| Tests | `cargo test --workspace --all-features` | `ci.yml` |
| Coverage (95% floor) | `cargo tarpaulin` (reads `tarpaulin.toml`) | `ci.yml` |
| Dependency policy | `cargo deny --all-features check` | `ci.yml` |
| Dependency audit trail | `cargo vet --locked` | `cargo-vet.yml` |
| API breakage | `cargo semver-checks check-release --workspace --exclude rlg-ebpf` | `semver-checks.yml` |
| Complexity ceilings | `scripts/complexity-gate.py` | `ci.yml` |
| Install snippets | `scripts/check-doc-versions.sh` | `ci.yml` |
| MCP manifests | `scripts/check-mcp-manifests.sh` | `ci.yml` |
| Markdown lint | `npx markdownlint-cli2` | `ci.yml` |
| Manual and links | `mdbook build && lychee --offline target/book` | `ci.yml` |
| Undefined behaviour | `cargo +nightly miri test -p rlg --lib --all-features` | `miri.yml` |
| Concurrency proofs | `RUSTFLAGS="--cfg loom" cargo test --release --test loom_engine -p rlg` | `loom.yml` |
| Model checking | `cd crates/rlg && cargo kani --all-features` | `kani.yml` |
| Fuzzing (smoke) | `cd fuzz && cargo +nightly fuzz run <target> -- -max_total_time=60` | `fuzz-smoke.yml` |
| Examples | `cargo run -p <crate> --example <name>` | `examples-smoke.yml` |
| MCP conformance | `cargo build --release -p rlg-mcp`, then the MCP Inspector | `mcp-inspect.yml` |

## Where the tests live

- **Unit tests**: next to the code, in `src/**/tests.rs` or a
  `#[cfg(test)] mod tests` block.
- **Integration tests**: `crates/<crate>/tests/`, one file per concern.
- **Property tests**: `proptest` suites in `crates/rlg/tests/` and
  `crates/rlg-cli/tests/`.
- **Proofs**: Loom in `crates/rlg/tests/loom_engine.rs`, Kani harnesses
  in `crates/rlg/src/kani_proofs.rs`.
- **Fuzz targets**: `fuzz/fuzz_targets/`.
- **Benchmarks**: `crates/<crate>/benches/`, run with
  `cargo bench -p <crate>`.

Thread-spawning tests carry `#[cfg_attr(miri, ignore)]`.

## Documentation

- The user manual is built with mdBook from [`docs/`](docs/) (`mdbook
  build`, output in `target/book`) and published with the API reference
  to <https://doc.rustlogs.com/> (manual under `/manual/`).
- API documentation: `cargo doc --workspace --all-features --no-deps`.
  Public items must be documented (`missing_docs`).

## Release model

1. Work for the next version happens on `feat/v<next-version>` and
   reaches `main` through that branch's single release pull request.
2. All ten publishable crates share one version; the release pull
   request bumps every manifest, install snippet and MCP manifest
   together and moves the CHANGELOG entries under the new heading.
3. The maintainer merges, then pushes a signed annotated tag
   `v<VERSION>`. `release.yml` publishes to crates.io through Trusted
   Publishing and attaches sigstore-signed SBOMs; see
   [`pkg/VERIFY.md`](pkg/VERIFY.md) to verify a release.
