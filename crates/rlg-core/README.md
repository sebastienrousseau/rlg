<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/rlg/v1/logos/rlg.svg" alt="RLG logo" width="128" />
</p>

<h1 align="center">rlg-core</h1>

<p align="center">
  The <code>no_std</code> core of <code>rlg</code>: log levels, a borrowed
  record view, and the 14 output formats, written through
  <code>core::fmt::Write</code>.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/rlg/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rlg/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/rlg-core"><img src="https://img.shields.io/crates/v/rlg-core.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/rlg-core"><img src="https://img.shields.io/badge/docs.rs-rlg--core-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://lib.rs/crates/rlg-core"><img src="https://img.shields.io/badge/lib.rs-rlg--core-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/rlg"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/rlg?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" /></a>
</p>

---

## Install

```toml
[dependencies]
rlg-core = "0.0.23"
```

Requires Rust **1.89.0** or newer (edition 2024). The crate is
`#![no_std]` and needs `alloc`; CI builds it for
`thumbv7em-none-eabihf`.

## Quick Start

Render a record into a fixed-size buffer. Everything here is `core`;
the same code runs in a `#![no_std]` firmware crate:

```rust
use core::fmt::{self, Write};
use rlg_core::{Format, LogLevel, Record};

/// A fixed-capacity sink, such as a UART frame buffer.
struct Frame {
    buf: [u8; 128],
    len: usize,
}

impl Write for Frame {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len + s.len();
        let dst = self.buf.get_mut(self.len..end).ok_or(fmt::Error)?;
        dst.copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

let record = Record::new(LogLevel::ERROR, "brown-out")
    .component("power")
    .session_id(42)
    .format(Format::Logfmt);

let mut frame = Frame { buf: [0; 128], len: 0 };
record.write_to(&mut frame).unwrap();
assert_eq!(
    &frame.buf[..frame.len],
    b"level=error msg=\"brown-out\" session_id=42 component=\"power\""
);
```

Attributes are a `BTreeMap<String, serde_json::Value>`
([`Attributes`](https://docs.rs/rlg-core/latest/rlg_core/record/type.Attributes.html)),
passed by reference with `.attributes(&map)`.

## What is in it

| Item | Purpose |
| --- | --- |
| `LogLevel`, `ParseLogLevelError` | Severity levels and their parsing. `rlg::LogLevel` is this type. |
| `Record<'a>` | A borrowed view of one record: the fields of `rlg::Log` plus the Apache host name. |
| `Format` | The 14 formats: CLF, JSON, CEF, ELF, W3C, GELF, Apache, Logstash, Log4j XML, NDJSON, MCP, OTLP, logfmt, ECS. |
| `Record::write_to` / `Display` | Writes the record in its format, byte for byte as `rlg::Log`'s `Display`. |

## What stays in `rlg`

The ring buffer and flusher thread, the platform sinks (`os_log`,
`journald`, file, stdout), rotation, config loading and hot-reload,
the `log` and `tracing` bridges, the clock and session counter, and
`rlg::LogFormat`: its `FromStr` error is `rlg::RlgError` and its
`validate` uses regular expressions, both of which need `std`. Use
`rlg` on a hosted target; use `rlg-core` where there is no `std`.

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.
