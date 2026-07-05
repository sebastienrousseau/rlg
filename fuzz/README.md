<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# rlg fuzz targets

This directory holds the `cargo-fuzz` targets for the workspace. It
is deliberately **excluded from the workspace** (see the root
`Cargo.toml` `[workspace] exclude` entry) so `libfuzzer-sys` and the
nightly-only build flags do not leak into normal `cargo build` /
`cargo test`.

See [`docs/adr/0002-fuzz-strategy.md`](../docs/adr/0002-fuzz-strategy.md)
for the strategy and [`docs/OSS-FUZZ.md`](../docs/OSS-FUZZ.md) for
the OSS-Fuzz onboarding runbook.

## Targets

| Target | Exercises |
|---|---|
| `parse_record` | `rlg_cli::parse_record` — one JSON-shape record per line |
| `log_format_from_str` | `<LogFormat as FromStr>::from_str` — the 14 variants |
| `config_load` | `toml::from_str::<Config>` — the config file parser |
| `redact_scrub` | `Redactor::with_defaults().scrub` — the six built-in patterns |

## Local run

Nightly toolchain is required (`libfuzzer-sys` needs `-Zbuild-std`
under the hood). Install `cargo-fuzz` once:

```bash
cargo install cargo-fuzz --locked
```

Run a single target for 30 seconds (matches the CI smoke budget):

```bash
cd fuzz
cargo +nightly fuzz run parse_record -- -max_total_time=30
```

Run indefinitely (Ctrl-C to stop):

```bash
cargo +nightly fuzz run parse_record
```

Reproduce a specific crash artefact:

```bash
cargo +nightly fuzz run parse_record fuzz/artifacts/parse_record/crash-<hash>
```

Minimise the corpus:

```bash
cargo +nightly fuzz cmin parse_record
```

## When a target crashes

1. Reproduce locally with the artefact path.
2. Diagnose the panic; the fix lives in the crate that owns the
   API (not in `fuzz/`).
3. Add a regression test in the owning crate's `tests/` that
   exercises the crashing input.
4. Land the fix as a normal signed commit.
5. Re-run the fuzz smoke CI to confirm the corpus no longer
   reproduces.

## Contract with `[lints]`

The workspace-wide `missing_docs = "deny"` lint does not inherit
here because `fuzz/` is excluded. The lints block in
`fuzz/Cargo.toml` sets `unsafe_code = "forbid"` and
`missing_docs = "allow"` — fuzz targets are essentially `main.rs`
entrypoints and do not carry public API.
