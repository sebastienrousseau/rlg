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

## Installation

To use `rlg` in your Rust project, add the following to your `Cargo.toml` file:

```toml
[dependencies]
rlg = "0.0.4"
```

### Requirements

`rlg` requires Rust **1.60** or later.

### Documentation

> ℹ️ **Info:** Please check out our [website][00] for more information and find our documentation on [docs.rs][08], [lib.rs][09], and [crates.io][07].

## Usage

### Basic Logging

```rust
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;

// Create a new log entry
let log_entry = Log::new(
    "12345",
    "2023-01-01T12:00:00Z",
    &LogLevel::INFO,
    "MyComponent",
    "This is a sample log message",
    &LogFormat::JSON, // Choose from various formats like JSON, Syslog, NDJSON, etc.
);

// Log the entry asynchronously
tokio::runtime::Runtime::new().unwrap().block_on(async {
    log_entry.log().await.unwrap();
});
```

### Custom Log Configuration

```rust
use rlg::config::Config;
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;

// Customize log file path
std::env::set_var("LOG_FILE_PATH", "/path/to/log/file.log");

// Load custom configuration
let config = Config::load();

// Create a new log entry with custom configuration
let log_entry = Log::new(
    "12345",
    "2023-01-01T12:00:00Z",
    &LogLevel::INFO,
    "MyComponent",
    "This is a sample log message",
    &LogFormat::ApacheAccessLog
);

// Log the entry asynchronously
tokio::runtime::Runtime::new().unwrap().block_on(async {
    log_entry.log().await.unwrap();
});
```

## Configuration

By default, RustLogs (RLG) logs to a file named "RLG.log" in the current directory. You can customize the log file path by setting the `LOG_FILE_PATH` environment variable.

## Error Handling

Errors can occur during logging operations, such as file I/O errors or formatting errors. The `log()` method returns a `Result<(), io::Error>` that indicates the outcome of the logging operation. You should handle potential errors appropriately in your code.

```rust
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;

// Create a new log entry
let log_entry = Log::new(
    "12345",
    "2023-01-01T12:00:00Z",
    &LogLevel::INFO,
    "MyComponent",
    "This is a sample log message",
    &LogFormat::NDJSON, // Using NDJSON format for this example
);

// Log the entry asynchronously and handle potential errors
tokio::runtime::Runtime::new().unwrap().block_on(async {
    match log_entry.log().await {
        Ok(_) => println!("Log entry successfully written"),
        Err(err) => eprintln!("Error logging entry: {}", err),
    }
});
```

## Macros

RustLogs (RLG) provides a set of useful macros to simplify logging tasks:

- `macro_log!`: Creates a new log entry with specified parameters.
- `macro_info_log!`: Creates an info log with default session ID and format.
- `macro_print_log!`: Prints a log to stdout.
- `macro_log_to_file!`: Asynchronously logs a message to a file.
- `macro_warn_log!`: Creates a warning log.
- `macro_error_log!`: Creates an error log with default format.
- `macro_set_log_format_clf!`: Sets the log format to CLF if not already defined.
- `macro_debug_log!`: Conditionally logs a message based on the `debug_enabled` feature flag.
- `macro_trace_log!`: Creates a trace log.
- `macro_fatal_log!`: Creates a fatal log.
- `macro_log_if!`: Conditionally logs a message based on a predicate.
- `macro_log_with_metadata!`: Logs a message with additional metadata.

Refer to the [documentation][08] for more details on how to use these macros.

## Examples

`RLG` comes with a set of examples that you can use to get started. The examples are located in the `examples` directory of the project. To run the examples, clone the repository and run the following command in your terminal from the project root directory:

```shell
cargo run --example rlg
```

## Semantic Versioning Policy

For transparency into our release cycle and in striving to maintain backward compatibility, `RLG` follows [semantic versioning][06].

## License

The project is licensed under the terms of both the MIT license and the Apache License (Version 2.0).

- [Apache License, Version 2.0][01]
- [MIT license][02]

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Acknowledgements

A special thank you goes to the [Rust Reddit](https://www.reddit.com/r/rust/) community for providing a lot of useful suggestions on how to improve this project.

[00]: https://rustlogs.com
[01]: http://www.apache.org/licenses/LICENSE-2.0
[02]: http://opensource.org/licenses/MIT
[05]: https://github.com/sebastienrousseau/rlg/graphs/contributors
[06]: http://semver.org/
[07]: https://crates.io/crates/rlg
[08]: https://docs.rs/rlg
[09]: https://lib.rs/crates/rlg

[crates-badge]: https://img.shields.io/crates/v/rlg.svg?style=for-the-badge 'Crates.io'
[divider]: https://kura.pro/common/images/elements/divider.svg "divider"
[docs-badge]: https://img.shields.io/docsrs/rlg.svg?style=for-the-badge 'Docs.rs'
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.4-orange.svg?style=for-the-badge 'Lib.rs'
[license-badge]: https://img.shields.io/crates/l/rlg.svg?style=for-the-badge 'License'
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust'
