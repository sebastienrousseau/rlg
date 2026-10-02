<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# AGENTS.md — rlg

Rules for anyone, human or agent, changing this repository. The
portfolio-wide standard (`~/Code/AGENTS.md`) applies too; where this file
is more specific, it wins.

## What rlg is

rlg (RustLogs) is a near-lock-free structured logging library for Rust:
an application thread hands a record to a 65k-slot ring buffer with
atomic operations only, and a background flusher formats and writes it
to the platform sink (`os_log`, `journald`, a file or stdout). Around the
core crate sit satellites that share its version: a CLI (`rlg-cli`),
reports (`rlg-report`), an MCP server (`rlg-mcp`), an OTLP exporter
(`rlg-otlp`), and redaction, tower, test, WASM and eBPF crates.

The core value is the hot path: `ingest()` never takes a lock and never
formats. Anything that adds a lock, an allocation or formatting work to
it is a regression, whatever else it improves.

## Rules

1. **Branch.** Work for the next release goes on `feat/v<next-version>`,
   one `0.0.1` above the current version, and reaches `main` only through
   that branch's single release pull request. Dependabot and topic work
   land on it as commits, never as pull requests into it.
2. **Versions move in lockstep.** All ten publishable crates share one
   version; `xtask` stays at `0.0.0`. Install snippets in the docs must
   name it (`scripts/check-doc-versions.sh`), and so must the MCP
   manifests (`scripts/check-mcp-manifests.sh`).
3. **Commits** are signed, `<type>: <subject>` in at most 50 characters,
   with an `Assisted-by:` trailer when an agent helped. Agents never add
   `Signed-off-by:`.
4. **No new suppressions.** `clippy::pedantic` and `clippy::nursery` are
   denied; fix the finding instead of allowing it.
5. **Complexity ceilings** (cyclomatic 10, cognitive 15, Halstead
   difficulty 30, 60 lines per function, 500 per file) hold for code you
   write or touch. `scripts/complexity-baseline.json` lists existing
   offenders; it only ever shrinks, via `scripts/complexity-gate.py
   --update` after an improvement.
6. **Dependencies** must pass `cargo deny --all-features check` and
   `cargo vet --locked`. Never widen `deny.toml` or add a vet exemption
   to make a check pass without saying why in the commit.
7. **`unsafe` is denied** except for the documented platform FFI in
   `crates/rlg/src/sink.rs`. Thread-spawning tests carry
   `#[cfg_attr(miri, ignore)]`.
8. **Decisions that will be questioned later** get an ADR in
   `docs/adr/`; reversing one means a new ADR that supersedes it.

## The one command

```bash
make verify
```

It runs formatting, clippy with warnings denied, the full test suite,
cargo-semver-checks, cargo-deny, cargo-vet, the complexity gate and the
version checks: what CI runs on every pull request.

## Leave alone

- `crates/rlg-mcp/src/transport.rs` and `transport/` are shared with the
  suite's other MCP servers (oxml-mcp, noyalib-mcp); only the file
  headers and line wrapping differ. Change them in all of them
  together, never here alone.
- `supply-chain/imports.lock` is written by `cargo vet`, not by hand.
- `LICENSE-*`, `KEYS.asc` and signing configuration change only when the
  task is explicitly about them.
- Published tags and releases are immutable.
