<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/rlg/images/logos/rlg.svg"
alt="RustLogs (RLG) logo" height="66" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# RustLogs (RLG)

A Rust library that implements application-level logging with a simple, readable output format.

[![Made With Love][made-with-rust]][05] [![Crates.io][crates-badge]][07] [![lib.rs][libs-badge]][03] [![Docs.rs][docs-badge]][08] [![Codecov][codecov-badge]][09] [![Build Status][build-badge]][10] [![GitHub][github-badge]][06]

## Overview

[`RustLogs (RLG)`][00] is a Rust library that provides application-level logging with a simple, readable output format. It offers logging APIs and various helper macros to simplify common logging tasks.

## Features

- Supports many log levels: `ALL`, `DEBUG`, `DISABLED`, `ERROR`, `FATAL`, `INFO`, `NONE`, `TRACE`, `VERBOSE`, and `WARN`
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

[00]: https://rustlogs.com
[03]: https://lib.rs/crates/rlg
[05]: https://github.com/sebastienrousseau/rlg/graphs/contributors
[06]: https://github.com/sebastienrousseau/rlg
[07]: https://crates.io/crates/rlg
[08]: https://docs.rs/rlg
[09]: https://codecov.io/gh/sebastienrousseau/rlg
[10]: https://github.com/sebastienrousseau/rlg/actions?query=branch%3Amain

[build-badge]: https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rlg/release.yml?branch=master&style=for-the-badge&logo=github "Build Status"
[codecov-badge]: https://img.shields.io/codecov/c/github/sebastienrousseau/rlg?style=for-the-badge&token=Q9KJ6XXL67&logo=codecov "Codecov"
[crates-badge]: https://img.shields.io/crates/v/rlg.svg?style=for-the-badge&color=fc8d62&logo=rust "Crates.io"
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.12-orange.svg?style=for-the-badge "View on lib.rs"
[docs-badge]: https://img.shields.io/badge/docs.rs-rlg-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs "Docs.rs"
[github-badge]: https://img.shields.io/badge/github-sebastienrousseau/rlg-8da0cb?style=for-the-badge&labelColor=555555&logo=github "GitHub"
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust'


## Changelog 📚
