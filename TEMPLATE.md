<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/rlg/images/logos/rlg.svg"
alt="RustLogs (RLG) logo" height="261" width="261" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# RustLogs (RLG)

A Rust library that implements application-level logging with a simple, readable output format.

[![Made With Love][made-with-rust]][05]
[![Crates.io][crates-badge]][07]
[![Lib.rs][libs-badge]][09]
[![Docs.rs][docs-badge]][08]
[![License][license-badge]][02]

![divider][divider]

## Overview

RustLogs (RLG) is a Rust library that provides application-level logging with a simple, readable output format. It offers logging APIs and various helper macros to simplify common logging tasks.

## Features

- Supports many log levels: `ALL`, `DEBUG`, `DISABLED`, `ERROR`, `FATAL`, `INFO`, `NONE`, `TRACE`, `VERBOSE`, and `WARNING`
- Provides structured log formats that are easy to parse and filter
- Compatible with multiple output formats including:
  - Common Event Format (CEF)
  - Extended Log Format (ELF)
  - Graylog Extended Log Format (GELF)
  - JavaScript Object Notation (JSON)
  - NCSA Common Log Format (CLF)
  - W3C Extended Log File Format (W3C)
  - Syslog Format
  - Apache Access Log Format
  - Logstash Format
  - Log4j XML Format
  - NDJSON (Newline Delimited JSON)
  - and more

[02]: http://opensource.org/licenses/MIT
[05]: https://github.com/sebastienrousseau/rlg/graphs/contributors
[07]: https://crates.io/crates/rlg
[08]: https://docs.rs/rlg
[09]: https://lib.rs/crates/rlg

[crates-badge]: https://img.shields.io/crates/v/rlg.svg?style=for-the-badge 'Crates.io'
[divider]: https://kura.pro/common/images/elements/divider.svg "divider"
[docs-badge]: https://img.shields.io/docsrs/rlg.svg?style=for-the-badge 'Docs.rs'
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.4-orange.svg?style=for-the-badge 'Lib.rs'
[license-badge]: https://img.shields.io/crates/l/rlg.svg?style=for-the-badge 'License'
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust'

## Changelog 📚
