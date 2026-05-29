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
git tag -s v0.0.10 -m "release v0.0.10"
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
