# Changelog

All notable changes to this project are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.12] — 2026-08-26

The **portability and supply-chain** cut. Fixes a build failure on
targets without 64-bit atomics, moves publishing to crates.io Trusted
Publishing, and folds in eight weeks of dependency and MCP work.

Workspace-lockstep versioning: all 10 publishable crates are at
`0.0.12` (`rlg`, `rlg-cli`, `rlg-ebpf`, `rlg-mcp`, `rlg-otlp`,
`rlg-redact`, `rlg-report`, `rlg-test`, `rlg-tower`, `rlg-wasm`).
`xtask` stays at `0.0.0` per workspace convention.

**A note on version numbering.** The manifests briefly carried `0.0.13`
and `0.0.14` while dependency batches were consolidated under those
names, but neither was ever tagged or published — crates.io went
straight from `0.0.11` to here. Rather than publish two versions that
predate the portability fix below and would be permanently broken on
32-bit targets, the manifests were renumbered back to `0.0.12`, which
is the version this CHANGELOG and the README already advertised. There
is no `0.0.13` or `0.0.14`, and there never was one to install.

### Fixed

- **Builds on targets without 64-bit atomics.** `SPAN_ID_COUNTER` in
  `tracing.rs` and `SESSION_COUNTER` in `log.rs` used `AtomicU64`,
  which does not exist on `powerpc-unknown-linux-gnu` and similar
  targets — the import failed outright with `E0432`. Both now use
  `euxis_commons::counter::Counter`, which selects a lock-free or
  mutex-backed implementation behind
  `#[cfg(target_has_atomic = "64")]`. Verified with
  `cargo check -p rlg --target powerpc-unknown-linux-gnu`.
- **RUSTSEC-2026-0204** patched, alongside clippy 1.97 lints and TUI
  test feature gates.
- **`rustdoc::redundant_explicit_links`** failures that were breaking
  the GitHub Pages documentation build.
- **`publish-mcp`** no longer injects `packages[0].version` at publish
  time.
- **`docsrs` `doc(cfg)`** failure in the documentation build.

### Changed

- **Publishing uses crates.io Trusted Publishing.** Releases
  authenticate over OIDC rather than a stored `CARGO_REGISTRY_TOKEN`,
  so there is no long-lived registry credential in the repository. The
  secret has been removed.
- **Releases can be re-run without moving a tag.** The release
  workflow accepts a `workflow_dispatch` with the tag to publish, after
  a GitHub Actions incident left a tag-triggered run unrecoverable —
  it reported as queued, completed and running at once and refused both
  cancel and rerun. All three jobs pin their checkout to the supplied
  tag, so a dispatch cannot package the default branch under a tag's
  version.
- **Dependabot** groups GitHub Actions updates into a single pull
  request.

### Added — MCP

- **Prompts and resources** for MCP Trinity parity.
- **`tail_logs_glob`** — a multi-file glob log tailer.
- **`glama.json`**, tool titles, MCP annotations and usage guidance.
- **Dockerfile** so Glama can build a release image.
- **CNAME written into the Pages artifact**, so the custom
  documentation domain survives each deploy.

### Dependencies

Eight weeks of consolidated updates, including `serial_test` 3.5 → 4.0,
`actions/setup-node` 6 → 7, `actions/configure-pages` 5 → 6, several
`minor-and-patch` group batches, and `euxis-commons` 0.0.2 → 0.0.4
(which carries the portable counter above). cargo-vet exemptions were
moved to match.

## [v0.0.11] - 2026-07-02

The **MCP-discoverability** cut for `rlg-mcp`. Registers `rlg-mcp`
with the official Model Context Protocol Registry (via OCI
packaging), adds MCP-spec conformance CI, ships a Glama directory
manifest, and cross-links sibling developer-tools MCP servers.

Workspace-lockstep versioning: all 9 publishable crates bump from
`0.0.10` → `0.0.11` (`rlg`, `rlg-cli`, `rlg-mcp`, `rlg-otlp`,
`rlg-redact`, `rlg-report`, `rlg-test`, `rlg-tower`, `rlg-wasm`).
`xtask` stays at `0.0.0` per workspace convention. This matches the
release workflow's "tag matches every publishable crate" check.
Only `rlg-mcp` has substantive changes in this cut; the other 8
crates ship no code changes, so existing consumers can upgrade
without any migration.

### Added — MCP registry work (rlg-mcp)

- **Official MCP Registry integration.** `rlg-mcp` is now registered
  with the official Model Context Protocol Registry
  (`registry.modelcontextprotocol.io`) as
  `io.github.sebastienrousseau/rlg-mcp`. A new `server.json` at the
  repo root provides the registry metadata using `registryType: oci`
  (the OCI image at `ghcr.io/sebastienrousseau/rlg-mcp` is the
  package artefact — crates.io is not a registry-supported
  `registryType`). `crates/rlg-mcp/README.md` carries an
  `mcp-name: io.github.sebastienrousseau/rlg-mcp` marker used by the
  registry for OCI ownership verification.
- **Auto-publish workflow** (`.github/workflows/publish-mcp.yml`) —
  on every `v*.*.*` tag push:
  1. Builds and pushes the OCI image (via the new
     `pkg/docker/Dockerfile.mcp` — Rust 1.88 builder, distroless-cc
     runtime, non-root user) to GHCR.
  2. Authenticates to the MCP Registry via GitHub OIDC (no secrets
     required), syncs the tag version into `server.json`, and runs
     `mcp-publisher publish`.
- **Protocol conformance CI** (`.github/workflows/mcp-inspect.yml`) —
  builds `rlg-mcp` release binary, then runs
  `@modelcontextprotocol/inspector --cli` against `tools/list`.
  Path-filtered to `crates/rlg-mcp/**`, `crates/rlg/**`, and
  `crates/rlg-cli/**` to keep the CI budget bounded.
- **Docker packaging** (`pkg/docker/Dockerfile.mcp`) — multi-stage
  build, distroless runtime, non-root user, reproducible via
  `SOURCE_DATE_EPOCH` and `--remap-path-prefix`.
- **Glama directory manifest** (`glama.json`) — Glama listing under
  the `developer-tools` category with OCI runtime spec.
- **Suite discoverability.** `crates/rlg-mcp/README.md` now cross-
  links sibling MCP servers — `noyalib-mcp` as a fellow developer-
  tools server, and the four ISO 20022 banking MCP servers
  (`pain001-mcp`, `bankstatementparser-mcp`, `camt053-mcp`,
  `acmt001-mcp`) as author-portfolio siblings.

### Changed

- GitHub repository description and topics — description will be
  refreshed to mention the MCP server; topics will gain `mcp-server`,
  `mcp`, `model-context-protocol`, `observability`, `sre`,
  `claude`, `claude-desktop`, and `ai-agents` (previously empty).

### No functional / API changes to non-MCP crates

- Only `rlg-mcp` has substantive changes (the MCP registry work
  above). The other 8 publishable crates (`rlg`, `rlg-cli`,
  `rlg-otlp`, `rlg-redact`, `rlg-report`, `rlg-test`, `rlg-tower`,
  `rlg-wasm`) bump to `0.0.11` as part of the workspace-lockstep
  cut but ship no code changes — existing consumers can upgrade
  without any migration.
