<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/rlg/v1/logos/rlg.svg" alt="RLG logo" width="128" />
</p>

<h1 align="center">rlg — RustLogs</h1>

<p align="center">
  Near-lock-free structured logging for Rust. Sub-microsecond ingestion
  via a 65k-slot ring buffer, deferred formatting, and native OS sinks.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/rlg/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rlg/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/rlg"><img src="https://img.shields.io/crates/v/rlg.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/rlg"><img src="https://img.shields.io/badge/docs.rs-rlg-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://codecov.io/gh/sebastienrousseau/rlg"><img src="https://img.shields.io/codecov/c/github/sebastienrousseau/rlg?style=for-the-badge&logo=codecov" alt="Coverage" /></a>
  <a href="https://lib.rs/crates/rlg"><img src="https://img.shields.io/badge/lib.rs-rlg-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
</p>

---

This is the Cargo workspace root. The library crate lives at
[`crates/rlg`](crates/rlg) — see [`crates/rlg/README.md`](crates/rlg/README.md)
for installation, the fluent API, the 14 supported output
formats, platform-sink details, configuration, and the security
posture.

## Workspace layout

```text
.
├── Cargo.toml            # workspace manifest (profiles, members)
├── CHANGELOG.md
├── CONTRIBUTING.md
├── SECURITY.md
├── LICENSE-APACHE
├── LICENSE-MIT
├── README.md             # ← you are here
├── crates/
│   └── rlg/              # the library crate
│       ├── Cargo.toml
│       ├── README.md     # crate-focused documentation (docs.rs root)
│       ├── LICENSE-APACHE
│       ├── LICENSE-MIT
│       ├── build.rs
│       ├── src/
│       ├── tests/
│       ├── benches/
│       ├── examples/
│       └── doc/          # mdBook-style long-form docs
└── .github/
    └── workflows/
        └── ci.yml        # delegates to sebastienrousseau/pipelines
```

## Quick links

- **Crate documentation:** [`crates/rlg/README.md`](crates/rlg/README.md) · [docs.rs/rlg](https://docs.rs/rlg)
- **Contributing & signing policy:** [`CONTRIBUTING.md`](CONTRIBUTING.md)
- **Security policy:** [`SECURITY.md`](SECURITY.md)
- **Release notes:** [`CHANGELOG.md`](CHANGELOG.md)

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.
