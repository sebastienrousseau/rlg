<p align="center">
  <img src="https://cloudcdn.pro/rlg/v1/logos/rlg.svg" alt="RLG logo" width="128" />
</p>

<h1 align="center">RLG</h1>

<p align="center">
  <strong>A Rust library for application-level logging with log rotation, network logging, and structured formats.</strong>
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/rlg/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rlg/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/rlg"><img src="https://img.shields.io/crates/v/rlg.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/rlg"><img src="https://img.shields.io/badge/docs.rs-rlg-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://codecov.io/gh/sebastienrousseau/rlg"><img src="https://img.shields.io/codecov/c/github/sebastienrousseau/rlg?style=for-the-badge&logo=codecov" alt="Coverage" /></a>
  <a href="https://lib.rs/crates/rlg"><img src="https://img.shields.io/badge/lib.rs-v0.0.9-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
</p>

---

## Install

```bash
cargo add rlg
```

Or add to `Cargo.toml`:

```toml
[dependencies]
rlg = "0.0.9"
```

You need [Rust](https://rustup.rs/) 1.88.0 or later. Works on macOS, Linux, and Windows.

---

## Overview

RLG implements application-level logging with a simple, readable output format. It supports log rotation, network logging, and multiple structured formats.

- **Standard log levels** from TRACE to CRITICAL
- **File rotation** by size or time interval
- **Network logging** to remote collectors
- **Structured formats** — JSON, CEF, CLF, GELF, and more

---

## Features

| | |
| :--- | :--- |
| **Log levels** | Standard log levels (TRACE through CRITICAL) |
| **Log rotation** | File-based rotation by size or time |
| **Network logging** | Send logs to remote collectors |
| **Structured formats** | JSON, CEF, CLF, GELF, and more |
| **Async support** | Non-blocking log writing |
| **Macros** | Convenient logging macros |

---

## Usage

```rust
use rlg::log::Log;
use rlg::log_level::LogLevel;

fn main() {
    let log = Log::new(
        "session-id",
        "2024-01-15T10:30:00Z",
        &LogLevel::INFO,
        "app",
        "Server started on port 8080",
        "AppLog",
    );
    println!("{}", log);
}
```

---

## Development

```bash
cargo build        # Build the project
cargo test         # Run all tests
cargo clippy       # Lint with Clippy
cargo fmt          # Format with rustfmt
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, signed commits, and PR guidelines.

---

**THE ARCHITECT** \u00b7 [Sebastien Rousseau](https://sebastienrousseau.com)
**THE ENGINE** \u00b7 [EUXIS](https://euxis.co) \u00b7 Enterprise Unified Execution Intelligence System

---

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.

<p align="right"><a href="#rlg">Back to Top</a></p>