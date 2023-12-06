<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/rlg/images/logos/rlg.svg"
alt="RustLogs (RLG) logo" height="261" width="261" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# RLG

A Rust library that implements application-level logging with a simple, readable output format.

[![Made With Love][made-with-rust]][05]
[![Crates.io][crates-badge]][07]
[![Lib.rs][libs-badge]][09]
[![Docs.rs][docs-badge]][08]
[![License][license-badge]][02]

![divider][divider]

## Welcome to RustLogs (RLG) 👋

![RLG Banner][banner]

<!-- markdownlint-disable MD033 -->
<center>

**[Website][00]
• [Documentation][08]
• [Report Bug][03]
• [Request Feature][03]
• [Contributing Guidelines][04]**

</center>

<!-- markdownlint-enable MD033 -->

## Overview 📖

`RustLogs (RLG)` is a Rust library that provides application-level logging with
a simple, readable output format. It offers logging APIs and various helper
macros to simplify common logging tasks.

## Features ✨

- Supports many log levels: `ALL`, `DEBUG`, `DISABLED`, `ERROR`,
  `FATAL`, `INFO`, `NONE`, `TRACE`, `VERBOSE` and `WARNING`,
- Provides structured log formats that are easy to parse and filter,
- Compatible with multiple output formats including:
  - Common Event Format (CEF),
  - Extended Log Format (ELF),
  - Graylog Extended Log Format (GELF),
  - JavaScript Object Notation (JSON)
  - NCSA Common Log Format (CLF),
  - W3C Extended Log File Format (W3C),
  - and more

## Installation 📦

To use `rlg` in your Rust project, add the following line to your `Cargo.toml` file:

```toml
[dependencies]
rlg = "0.0.2"
```

### Requirements

`rlg` requires Rust **1.67.0** or later.

### Documentation

> ℹ️ **Info:** Please check out our [website][00] for more information
and find our documentation on [docs.rs][08], [lib.rs][09] and
[crates.io][07].

## Usage 📖

## Macros 🔨

The `rlg` library provides the following macros for logging:

- `macro_log!`: Main macro for logging messages with customizable format.
- `macro_print_log!`: Print log message to stdout.
- `macro_log_to_file!`: Asynchronously log message to a file.
- `macro_warn_log!`: Macro for warning log messages.
- `macro_error_log!`: Macro for error log messages.
- `macro_set_log_format_clf!`: Set log format to Common Log Format (CLF).
- `macro_debug_log!`: Conditional debug logging.

### macro_log!

The `macro_log!` macro is the main macro for logging messages. It takes the following arguments:

- **session_id**: A unique identifier for the log session.
- **time**: The timestamp of the log message.
- **level**: The log level, such as DEBUG, INFO, WARN, or ERROR.
- **component**: The name of the component that generated the log message.
- **description**: The description of the log message.
- **format**: The log format, such as CLF, JSON, or GELF.


Example usage of the `macro_log!` macro:

```rust
use rlg::{Log, LogLevel, LogFormat};
use vrd::Random;

let log = macro_log!(
    &Random::default().int(0, 1_000_000_000).to_string(),
    "2023-04-18T12:38:55Z",
    &LogLevel::INFO,
    "ComponentName",
    "Log message description",
    &LogFormat::CLF,
);

macro_log!(log);
```

### Examples

`RLG` comes with a set of examples that you can use to get started. The
examples are located in the `examples` directory of the project. To run
the examples, clone the repository and run the following command in your
terminal from the project root directory.

```shell
cargo run --example rlg
```

## Semantic Versioning Policy 🚥

For transparency into our release cycle and in striving to maintain
backward compatibility, `QRC` follows [semantic versioning][06].

## License 📝

The project is licensed under the terms of both the MIT license and the
Apache License (Version 2.0).

- [Apache License, Version 2.0][01]
- [MIT license][02]

## Contribution 🤝

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms
or conditions.

![divider][divider]

## Acknowledgements 💙

A big thank you to all the awesome contributors of [Mini Functions][05]
for their help and support. A special thank you goes to the
[Rust Reddit](https://www.reddit.com/r/rust/) community for providing a
lot of useful suggestions on how to improve this project.

[00]: https://rustlogs.com
[01]: http://www.apache.org/licenses/LICENSE-2.0
[02]: http://opensource.org/licenses/MIT
[03]: https://github.com/sebastienrousseau/rlg/issues
[04]: https://raw.githubusercontent.com/sebastienrousseau/rlg/main/.github/CONTRIBUTING.md
[05]: https://github.com/sebastienrousseau/rlg/graphs/contributors
[06]: http://semver.org/
[07]: https://crates.io/crates/rlg
[08]: https://docs.rs/rlg
[09]: https://lib.rs/crates/rlg
[10]: https://github.com/sebastienrousseau/rlg

[banner]: https://kura.pro/rlg/images/titles/title-rlg.svg "RLG Banner"
[crates-badge]: https://img.shields.io/crates/v/rlg.svg?style=for-the-badge 'Crates.io'
[divider]: https://raw.githubusercontent.com/sebastienrousseau/vault/main/assets/elements/divider.svg "divider"
[docs-badge]: https://img.shields.io/docsrs/rlg.svg?style=for-the-badge 'Docs.rs'
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.2-orange.svg?style=for-the-badge 'Lib.rs'
[license-badge]: https://img.shields.io/crates/l/rlg.svg?style=for-the-badge 'License'
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust'
