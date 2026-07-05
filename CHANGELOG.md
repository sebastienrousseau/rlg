# Changelog

All notable changes to this project are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

(Nothing yet — `[v0.0.11]` is the cut.)

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
