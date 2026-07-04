<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/rlg/v1/logos/rlg.svg" alt="RLG logo" width="128" />
</p>

<h1 align="center">rlg-test</h1>

<p align="center">
  Test utilities for downstream crates that depend on <code>rlg</code>.
  Capture records and assert on them inside <code>#[test]</code> scopes.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/rlg/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rlg/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/rlg-test"><img src="https://img.shields.io/crates/v/rlg-test.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/rlg-test"><img src="https://img.shields.io/badge/docs.rs-rlg--test-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://lib.rs/crates/rlg-test"><img src="https://img.shields.io/badge/lib.rs-rlg--test-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/rlg"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/rlg?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" /></a>
</p>

---

## Install

```toml
[dev-dependencies]
rlg-test = "0.0.11"
```

Requires Rust **1.88.0** or newer (edition 2024).

## Usage

```rust
use rlg::log::Log;
use rlg::log_level::LogLevel;
use rlg_test::{capture, LogExt, assert_logged};

#[test]
fn emits_auth_record() {
    let capture = capture();

    // Route the record into the capture handle instead of the engine.
    Log::info("user authenticated")
        .component("auth")
        .with("user_id", 42_u64)
        .log_to(&capture);

    assert_logged!(capture, level == LogLevel::INFO);
    assert_logged!(capture, contains "authenticated");
    assert_logged!(capture, component "auth");
    assert_logged!(capture, attribute "user_id" => 42_u64);
    assert_logged!(capture, len == 1);
}
```

## Predicates

| Form | Predicate |
| --- | --- |
| `level == LogLevel::INFO` | record at the given level exists |
| `contains "needle"` | record description contains the substring |
| `attribute "k" => v` | record's attribute equals the value |
| `component "name"` | record's component equals the string |
| `len == n` | exactly `n` records were captured |

Each is also exposed as a free function (`has_level`,
`description_contains`, `attribute_eq`, `has_component`) so you can
compose them or use them outside the macro.

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.
