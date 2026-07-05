# Contributing to RustLogs (RLG)

Thanks for your interest in `rlg`. This document covers the development setup, the verification commands every PR must pass, and the cryptographic signing policy that gates merges.

## Development Setup

```bash
git clone https://github.com/sebastienrousseau/rlg.git
cd rlg
cargo check --all-features
```

`rlg` targets Rust **1.88.0** (MSRV) and edition 2024. It runs on macOS, Linux, and WSL. Windows is supported on a best-effort basis.

## Verification — Required Before Pushing

Every PR must pass the following locally. CI re-runs them via the centralized [`sebastienrousseau/pipelines`](https://github.com/sebastienrousseau/pipelines) reusable workflows.

```bash
cargo fmt --check                                              # format
cargo clippy --all-features --tests --benches -- -D warnings   # lint
cargo test  --all-features                                     # unit + integration
cargo bench --bench competitive_bench                          # perf-sensitive changes only
```

On macOS, run integration tests with `RLG_FALLBACK_STDOUT=1` to bypass the `os_log` FFI dispatch when not needed.

### Miri (undefined-behaviour check)

The ring-buffer hot path in `crates/rlg/src/engine.rs` and the syslog FFI in `crates/rlg/src/sink.rs` are covered by [Miri](https://github.com/rust-lang/miri) on every PR that touches `crates/rlg/**` (via [`.github/workflows/miri.yml`](.github/workflows/miri.yml)). To run it locally:

```bash
rustup toolchain install nightly --component miri rust-src
cargo +nightly miri setup
MIRIFLAGS="-Zmiri-permissive-provenance" cargo +nightly miri test -p rlg --lib --all-features
```

Tests that legitimately cannot run under Miri (thread spawns, file I/O, FFI dispatch to `syslog(3)`) carry `#[cfg_attr(miri, ignore)]`. When adding such a test, apply the attribute rather than tightening the workflow.

### Loom (concurrency proofs)

The producer/flusher shutdown handshake and `session_id` monotonicity are exhaustively verified by Loom on every PR that touches `crates/rlg/src/engine.rs` or the proofs themselves (via [`.github/workflows/loom.yml`](.github/workflows/loom.yml)). To run the proofs locally:

```bash
RUSTFLAGS="--cfg loom" cargo test --release --test loom_engine -p rlg -- --nocapture --test-threads=1
```

See [`docs/adr/0001-loom-verified-ring-buffer.md`](docs/adr/0001-loom-verified-ring-buffer.md) for the model faithfulness argument and what is (and is not) covered.

### Fuzzing (untrusted input)

Four `cargo-fuzz` targets cover the deserialisation and scan entry points that accept untrusted input:

| Target | Exercises |
|---|---|
| `parse_record` | `rlg_cli::parse_record` |
| `log_format_from_str` | `<LogFormat as FromStr>::from_str` |
| `config_load` | `toml::from_str::<Config>` |
| `redact_scrub` | `Redactor::with_defaults().scrub` |

CI runs each for 30 s per PR ([`.github/workflows/fuzz-smoke.yml`](.github/workflows/fuzz-smoke.yml)); OSS-Fuzz runs them continuously post-onboarding (see [`docs/OSS-FUZZ.md`](docs/OSS-FUZZ.md)). To run locally:

```bash
cargo install cargo-fuzz --locked
cd fuzz
cargo +nightly fuzz run parse_record -- -max_total_time=30
```

Fuzz targets live under [`fuzz/`](fuzz/), which is deliberately excluded from the workspace so `libfuzzer-sys` and nightly-only build flags never leak into normal `cargo build` / `cargo test`. Strategy and corpus policy in [`docs/adr/0002-fuzz-strategy.md`](docs/adr/0002-fuzz-strategy.md).

### Property tests

Seven `proptest` properties cover the format `Display` impls, NDJSON single-line invariant, serde round-trip, and `Filter` monotonicity. Run with standard `cargo test`:

```bash
cargo test -p rlg --test proptest_round_trip
cargo test -p rlg-cli --test proptest_filter
```

Failures shrink to a minimal counter-example that appears directly in the test output. Strategy and coverage boundaries in [`docs/adr/0003-property-tested-formats.md`](docs/adr/0003-property-tested-formats.md).

### Kani (model-checked proofs)

Three Kani proofs formally verify the `LogLevel::from_numeric` ↔ `to_numeric` bijection and the session-counter fetch_add monotonicity. Kani runs on merges to `main` and weekly cron via [`.github/workflows/kani.yml`](.github/workflows/kani.yml), not per-PR. To run locally:

```bash
cargo install --locked kani-verifier
cargo kani setup
cd crates/rlg && cargo kani --tests
```

Coverage boundary and what Kani does NOT prove in [`docs/adr/0004-kani-verified-invariants.md`](docs/adr/0004-kani-verified-invariants.md).

## Cryptographic Signing — Mandatory

Every commit on every PR must be cryptographically verified. Unsigned commits are rejected at branch protection.

### One-time setup (SSH signing, recommended)

```bash
git config --global user.signingkey ~/.ssh/id_ed25519
git config --global gpg.format ssh
git config --global commit.gpgsign true
git config --global tag.gpgsign true
```

Add the same public key to your GitHub account under **Settings → SSH and GPG keys → New SSH key → Signing Key**.

### Per-commit

```bash
git commit -S -m "feat: …"
git tag -s v0.0.11 -m "release v0.0.11"
```

Verify a commit:

```bash
git log --show-signature -1
```

`git log` should report `Good "git" signature for <your-email>`. GitHub will display the `Verified` badge.

## Commit Conventions

`rlg` follows [Conventional Commits](https://www.conventionalcommits.org/) with the following type prefixes:

| Prefix       | Use for                                     |
| ------------ | ------------------------------------------- |
| `feat`       | New public API or capability                |
| `fix`        | Bug fix                                     |
| `perf`       | Performance improvement, no behaviour delta |
| `refactor`   | Internal restructuring                      |
| `docs`       | Documentation only                          |
| `test`       | Test additions / changes only               |
| `chore(deps)`| Dependency bumps                            |
| `ci`         | CI configuration                            |

## Pull Request Flow

1. Fork → branch from `main` (e.g. `feat/short-name`).
2. Make focused commits — one concern per commit, all signed.
3. Run the verification block above.
4. Open a PR. The PR description must reference any related issue and explain the *why*, not the *what* (the diff documents the what).
5. CI must pass green: `ci`, `security`, and (on `main`) `docs`.

## Security

Vulnerability reports go through the private channel documented in [`SECURITY.md`](SECURITY.md). Do not file public issues for security problems.

## License

By contributing you agree your work is dual-licensed under [Apache-2.0](LICENSE-APACHE) **or** [MIT](LICENSE-MIT) at the user's option, matching the project license.
