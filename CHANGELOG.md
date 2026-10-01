# Changelog

All notable changes to this project are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.13] — unreleased

The **smaller-tree** cut. rlg-mcp moves onto the official MCP SDK and
serves stdio, streamable HTTP and the older HTTP+SSE transport; the
optional dependencies that failed `cargo deny --all-features` (miette,
notify, reqwest, tonic) are replaced by in-house code or removed, taking
107 crates out of the lockfile; and the repository gains the gates that
keep it that way: an enforcing cargo-deny job over all features, a
complexity baseline, and a check that install snippets name the shipped
version.

Workspace-lockstep versioning: all 10 publishable crates are at
`0.0.13`. `xtask` stays at `0.0.0`.

This is the first `0.0.13` on crates.io. The number was used briefly
for an internal dependency batch before 0.0.12 (see the note there),
but nothing was ever tagged or published under it.

### Added

- `rlg --completions <SHELL>` and `rlg-report --completions <SHELL>`
  print shell completions (bash, zsh, fish, elvish, PowerShell)
  generated from the CLI definition; `make completions` writes them all.
- `--manpage` on both binaries prints a section-1 manual page generated
  from the CLI definition, and a `GNUmakefile` adds `make install` /
  `make uninstall` (honouring `PREFIX` and `DESTDIR`) for the binaries,
  manpages and completions.
- A user manual built with mdBook from `docs/` and published at
  <https://doc.rustlogs.com/manual/>, with `ARCHITECTURE.md`,
  `POLICIES.md`, `packaging.md`, `COMPARISON.md` and `BENCHMARKS.md`.
- CI gates: the 1.88.0 MSRV build, markdownlint, an offline link check
  of the manual and README, the README template, and OpenSSF Scorecard.
- `rlg-mcp` runs on the official MCP SDK (`rmcp`) and serves stdio (the
  default), streamable HTTP (`--transport streamable-http`) or the older
  HTTP+SSE transport (`--transport sse`), covering protocol revisions
  2024-11-05 through 2026-07-28.
- `RlgError::code`, `RlgError::help` and `RlgError::report`: stable error
  codes (`rlg::io_error`, …), a resolution hint, and a multi-line report,
  with no extra dependency.
- `rlg_otlp::DEFAULT_ENDPOINT` (`http://localhost:4318/v1/logs`), and
  `OtlpError::InvalidEndpoint` / `OtlpError::InvalidHeader`.
- CI runs `cargo deny --all-features check` as a failing gate. The shared
  security workflow's run is `continue-on-error`, so policy violations
  had gone unreported.

### Changed

- `fire()` is about 40% cheaper on the calling thread: the timestamp and
  the `caller` attribute are no longer formatted through `format!`.
  Output is byte-identical. In CI it went from 2.4x to about 1.0x the
  cost of `tracing::info!` measured in the same run.
- `rlg-otlp` sends plain OTLP/HTTP to a local OpenTelemetry Collector,
  which owns TLS and credentials towards the backend
  ([ADR 0015](docs/adr/0015-otlp-local-collector-transport.md)).
  `AsyncOtlpExporter` now runs on an in-house HTTP/1.1 client over Tokio
  instead of `reqwest`, and rejects `https://` endpoints and unsafe
  headers at `build()`. Both builders default to `DEFAULT_ENDPOINT`
  instead of panicking when no endpoint is set. **Breaking:**
  `OtlpError::AsyncTransport` wraps `std::io::Error`. Deployments that
  exported straight to a SaaS `https://` endpoint must add a Collector;
  the crate docs carry a minimal configuration.
- `Config::hot_reload_async` polls the file (`HOT_RELOAD_POLL_INTERVAL`,
  250 ms) instead of using `notify`, and now also picks up editor-style
  saves that rename a new file over the old one. **Breaking:**
  `ConfigError::WatcherError` wraps `std::io::Error`.
- rlg builds `config` with the TOML format only; YAML, JSON5, RON and INI
  parsers it never used are no longer compiled.

### Removed

- The `rlg-otlp` `grpc` feature, `GrpcOtlpExporter` and its error
  variants. It was a scaffold whose send path returned
  `GrpcNotImplemented`; collectors accept OTLP/HTTP on port 4318.
- The `rlg` `miette` feature; use `RlgError::code`, `help` and `report`.
- `reqwest`, `tonic`, `prost`, `rustls`, `ring`, `webpki-roots`,
  `miette` and `notify` from the dependency tree (82 crates in all).

### Fixed

- The configuration example in the `rlg` README did not load
  (`LogRotation` and `LoggingDestination` use `{ Size = N }` and
  `{ type = "File", value = ... }`); it does now, and a test loads it.
- `bench-publish.yml` never ran a benchmark: its output directory did
  not exist and the error was swallowed. Release benchmarks now run and
  publish.
- The `rlg` README and crate docs claimed `fire()` takes ~1.4 µs against
  ~20 µs for mainstream loggers. Measured in CI it is ~0.85 µs, and
  `tracing` formatting to a discarding writer is ~0.35 µs; the claims
  are replaced by the published numbers.
- `.github/SECURITY.md`, the copy GitHub shows first, was a template
  with no reporting channel; the real policy is now the one shown.
- `PKGBUILD` said 0.0.7 and `debian/debcargo.toml` named a feature that
  does not exist; both are fixed and the `PKGBUILD` version is CI-checked.
- Install snippets in thirteen places (every crate README, the getting
  started tutorial, the introduction and the tracing migration guide)
  named 0.0.11 or 0.0.7. They name 0.0.12, and
  `scripts/check-doc-versions.sh` fails CI when a snippet drifts from
  `crates/rlg/Cargo.toml` again.
- The blocking OTLP exporter retried 4xx responses and reported every
  failing status as a transport error: ureq returned non-2xx as errors,
  so the status handling never ran. A 4xx is now final and statuses are
  reported as `OtlpError::BadStatus`.
- `glama.json` and `server.json` named 0.0.11 while the workspace shipped
  0.0.12, so the Glama listing and the registry's install command pointed
  at the previous image. Both are stamped, the README's lockstep line with
  them, and `scripts/check-mcp-manifests.sh` now fails CI when they drift
  from `crates/rlg-mcp/Cargo.toml` again.

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
