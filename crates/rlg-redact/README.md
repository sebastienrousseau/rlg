<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/rlg/v1/logos/rlg.svg" alt="RLG logo" width="128" />
</p>

<h1 align="center">rlg-redact</h1>

<p align="center">
  PII / secret redaction for <code>rlg</code> records. Scrub credit
  cards, JWTs, OAuth bearers, emails, IPv4 addresses, and AWS keys
  before they reach disk, OTLP, or syslog.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/rlg/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rlg/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/rlg-redact"><img src="https://img.shields.io/crates/v/rlg-redact.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/rlg-redact"><img src="https://img.shields.io/badge/docs.rs-rlg--redact-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://lib.rs/crates/rlg-redact"><img src="https://img.shields.io/badge/lib.rs-rlg--redact-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/rlg"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/rlg?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" /></a>
</p>

---

## Install

```toml
[dependencies]
rlg-redact = "0.0.11"
```

Requires Rust **1.88.0** or newer (edition 2024).

## Usage

```rust
use rlg::log::Log;
use rlg_redact::Redactor;

let redactor = Redactor::with_defaults();

let log = Log::info("card 4111-1111-1111-1111 declined")
    .with("email", "user@example.com")
    .with("client_ip", "10.0.1.42");

let safe = redactor.scrub(log);
//   description: "card [REDACTED] declined"
//   email:       "[REDACTED]"
//   client_ip:   "[REDACTED]"
safe.fire();
```

## Built-in patterns

| Constant | Matches |
| --- | --- |
| `CREDIT_CARD` | 13–19-digit PANs (Visa, MC, Amex, Discover) with optional spaces/hyphens |
| `JWT` | three-segment base64url JWTs |
| `BEARER_TOKEN` | `Authorization: Bearer <token>` headers |
| `EMAIL` | RFC 5321 addresses (good-enough subset) |
| `IPV4` | dotted-quad IPv4 addresses |
| `AWS_ACCESS_KEY` | AWS key IDs (AKIA / ASIA / AGPA / ANPA / ANVA / AROA / AIPA) |

`Redactor::with_defaults()` loads all six. Build a minimal redactor
with `Redactor::empty()` and add patterns one at a time with
`.with_pattern(...)`.

## Custom patterns

```rust
use rlg_redact::Redactor;

let redactor = Redactor::with_defaults()
    .with_pattern(r"(?i)password=\S+")?
    .with_pattern(r"\bsk_live_[A-Za-z0-9]{24}\b")?     // Stripe live keys
    .marker("***");
```

## What gets scrubbed

- `Log::description`
- Every string attribute value
- String values inside JSON arrays + objects (recursive)

Non-string attributes (`u64`, `i64`, `bool`, `f64`) are passed
through unchanged — numeric IDs are not redacted.

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.
