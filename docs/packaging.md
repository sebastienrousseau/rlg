<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Packaging rlg

For distribution maintainers. Everything here comes from the repository;
if something you need is missing, open an issue.

## What there is to package

| Crate | Ships | Notes |
| :--- | :--- | :--- |
| `rlg-cli` | the `rlg` binary | filter and convert log files |
| `rlg-report` | the `rlg-report` binary | summaries of a log file |
| `rlg-mcp` | the `rlg-mcp` binary | MCP server; also an OCI image, `pkg/docker/Dockerfile.mcp` |
| `rlg`, `rlg-otlp`, `rlg-redact`, `rlg-tower`, `rlg-test`, `rlg-wasm`, `rlg-ebpf` | libraries | for distributions that package Rust crates (Debian, Fedora) |

All ten publishable crates share one version.

## License

Every crate is dual-licensed **Apache-2.0 OR MIT**; the texts are
`LICENSE-APACHE` and `LICENSE-MIT` at the repository root, and each
crate's `Cargo.toml` carries `license = "MIT OR Apache-2.0"`. The
dependency tree is limited to the licences allowed in `deny.toml`
(MIT, Apache-2.0, Unicode-3.0, BSL-1.0, Unlicense, BSD-3-Clause),
enforced by `cargo deny --all-features check` in CI.

## Toolchain

The minimum supported Rust is **1.88.0** for building the libraries and
binaries; the test suite may need newer (see
[POLICIES.md](POLICIES.md)). A rise in the floor is a CHANGELOG entry.

## Dependencies

- `Cargo.lock` is committed and CI builds with `--locked`. Build with
  `--locked` or `--frozen` to get the tree CI tested.
- Every dependency is recorded in cargo-vet (`supply-chain/`) and passes
  cargo-deny: no duplicate versions, no git or non-crates.io sources.
- Optional features pull in optional dependencies only; the default
  build has none of `tokio`, `terminal_size` or `tracing-subscriber`.

## Building and testing offline

```bash
cargo vendor --locked vendor > .cargo-vendor.toml   # once, with network
cargo build --frozen --release -p rlg-cli -p rlg-report -p rlg-mcp \
  --config .cargo-vendor.toml
cargo test --frozen --workspace --config .cargo-vendor.toml
```

The tests need no network: the ones that exercise HTTP bind loopback
sockets (`127.0.0.1`) and talk to themselves.

## Installing, manpages and completions

`make DESTDIR="$pkgdir" PREFIX=/usr install` builds the release
binaries and installs them, their manpages (section 1) and bash, zsh
and fish completions into an FHS tree; CI checks the staged tree on a
clean runner. To do it by hand, generate everything from the binaries;
never ship copies from elsewhere:

```bash
rlg --completions bash        > rlg.bash
rlg-report --completions zsh  > _rlg-report
rlg --manpage                 > rlg.1
```

Supported shells: bash, zsh, fish, elvish, PowerShell. `make completions`
writes all of them for both binaries into `target/completions/`.

## Verifying a release

Releases are signed tags `v<VERSION>`. Each GitHub release carries SPDX
and CycloneDX SBOMs signed keyless with sigstore; the certificate
identity is pinned to this repository's `release.yml` on a tag.
[`pkg/VERIFY.md`](https://github.com/sebastienrousseau/rlg/blob/main/pkg/VERIFY.md)
is the step-by-step runbook.

## Recipes in this repository

| Format | Path | State |
| :--- | :--- | :--- |
| Arch (AUR) | `PKGBUILD` | builds `rlg`, installs completions; `pkgver` is CI-checked against the workspace |
| Debian (debcargo) | `debian/debcargo.toml` | overlay for the `rlg` library crate |
| Nix | `flake.nix` | builds with the 1.88.0 toolchain |
| OCI | `pkg/docker/Dockerfile.mcp` | the `rlg-mcp` image published to `ghcr.io` |

## Where rlg is packaged

[Repology](https://repology.org/project/rlg/versions) tracks which
distributions ship rlg. The README gains a Repology badge once two
distributions do.
