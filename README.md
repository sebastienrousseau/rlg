<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/rlg/images/logos/rlg.svg"
alt="RustLogs (RLG) logo" height="66" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# RustLogs (RLG)

A robust and flexible Rust library for application-level logging with support for multiple formats and asynchronous operations.

[![Made With Love][made-with-rust]][00] [![Crates.io][crates-badge]][07] [![lib.rs][libs-badge]][03] [![Docs.rs][docs-badge]][08] [![Codecov][codecov-badge]][09] [![Build Status][build-badge]][10] [![GitHub][github-badge]][06]

## Overview

[`RustLogs (RLG)`][00] is a comprehensive Rust library that provides advanced application-level logging capabilities. It offers a wide range of logging APIs, helper macros, and flexible configuration options to simplify common logging tasks and meet diverse logging requirements.

## Features

- Multiple log levels: `ALL`, `DEBUG`, `DISABLED`, `ERROR`, `FATAL`, `INFO`, `NONE`, `TRACE`, `VERBOSE`, and `WARN`
- Structured log formats for easy parsing and filtering
- Support for multiple output formats including:
  - Common Log Format (CLF)
  - JavaScript Object Notation (JSON)
  - Common Event Format (CEF)
  - Extended Log Format (ELF)
  - W3C Extended Log File Format
  - Graylog Extended Log Format (GELF)
  - Apache Access Log Format
  - Logstash Format
  - Log4j XML Format
  - NDJSON (Newline Delimited JSON)
- Configurable logging destinations (file, stdout, network)
- Asynchronous logging for improved performance
- Log rotation support (size-based, time-based, date-based, count-based)
- Environment variable expansion in configuration
- Hot-reloading of configuration
- Comprehensive error handling and custom error types

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rlg = "0.0.6"
```

## Documentation

For full API documentation, please visit [https://doc.rustlogs.com/][04] or [https://docs.rs/rlg][08].

## Rust Version Compatibility

Compiler support: requires rustc 1.56.0+

## Usage

### Basic Logging

```rust
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg::utils::generate_timestamp;

// Create a new log entry
let log_entry = Log::new(
    &vrd::random::Random::default().int(0, 1_000_000_000).to_string(),
    &generate_timestamp(),
    &LogLevel::INFO,
    "MyComponent",
    "This is a sample log message",
    &LogFormat::JSON,
);

// Log the entry asynchronously
tokio::runtime::Runtime::new().unwrap().block_on(async {
    log_entry.log().await.unwrap();
});
```

### Custom Log Configuration

```rust
use rlg::config::{Config, LogRotation, LoggingDestination};
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use std::path::PathBuf;
use std::num::NonZeroU64;

// Create a custom configuration
let mut config = Config::default();
config.log_file_path = PathBuf::from("/path/to/log/file.log");
config.log_level = LogLevel::DEBUG;
config.log_rotation = Some(LogRotation::Size(NonZeroU64::new(10_000_000).unwrap())); // 10 MB
config.logging_destinations = vec![
    LoggingDestination::File(PathBuf::from("/path/to/log/file.log")),
    LoggingDestination::Stdout,
];

// Create a new log entry with custom configuration
let log_entry = Log::new(
    &vrd::random::Random::default().int(0, 1_000_000_000).to_string(),
    &generate_timestamp(),
    &LogLevel::INFO,
    "MyComponent",
    "This is a sample log message",
    &LogFormat::JSON,
);

// Log the entry asynchronously
tokio::runtime::Runtime::new().unwrap().block_on(async {
    log_entry.log().await.unwrap();
});
```

## Configuration

RustLogs (RLG) provides a flexible configuration system. You can customize various aspects of logging behavior, including:

- Log file path
- Log level
- Log rotation settings
- Log format
- Logging destinations
- Environment variables

You can load configuration from a file or environment variables using the `Config::load_async` method.

## Error Handling

RustLogs (RLG) provides comprehensive error handling through the `RlgError` type. This allows for more specific error handling in your application:

```rust
use rlg::log::Log;
use rlg::log_format::LogFormat;
use rlg::log_level::LogLevel;
use rlg::error::RlgError;

// Create a new log entry
let log_entry = Log::new(
    &vrd::random::Random::default().int(0, 1_000_000_000).to_string(),
    &generate_timestamp(),
    &LogLevel::INFO,
    "MyComponent",
    "This is a sample log message",
    &LogFormat::JSON,
);

// Log the entry asynchronously and handle potential errors
tokio::runtime::Runtime::new().unwrap().block_on(async {
    match log_entry.log().await {
        Ok(_) => println!("Log entry successfully written"),
        Err(RlgError::IoError(err)) => eprintln!("I/O error when logging: {}", err),
        Err(RlgError::FormattingError(err)) => eprintln!("Formatting error: {}", err),
        Err(err) => eprintln!("Error logging entry: {}", err),
    }
});
```

## Macros

RustLogs (RLG) provides a set of convenient macros to simplify logging tasks:

- `macro_log!`: Creates a new log entry with specified parameters.
- `macro_info_log!`: Creates an info log with default session ID and format.
- `macro_warn_log!`: Creates a warning log.
- `macro_error_log!`: Creates an error log.
- `macro_trace_log!`: Creates a trace log.
- `macro_fatal_log!`: Creates a fatal log.
- `macro_log_to_file!`: Asynchronously logs a message to a file.
- `macro_print_log!`: Prints a log to stdout.
- `macro_set_log_format_clf!`: Sets the log format to CLF if not already defined.
- `macro_log_if!`: Conditionally logs a message based on a predicate.
- `macro_debug_log!`: Conditionally logs a debug message based on the `debug_enabled` feature flag.
- `macro_log_with_metadata!`: Logs a message with additional metadata.

Refer to the [documentation][08] for more details on how to use these macros.

## Examples

To run the examples, clone the repository and use the following command:

```shell
cargo run --example example_name
```

Replace `example_name` with the name of the example you want to run.

## License

The project is dual-licensed under the terms of both the MIT license and the Apache License (Version 2.0).

- [Apache License, Version 2.0][01]
- [MIT license][02]

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Acknowledgements

A special thank you goes to the [Rust Reddit](https://www.reddit.com/r/rust/) community for providing a lot of useful knowledge and expertise.

[00]: https://rustlogs.com
[01]: http://www.apache.org/licenses/LICENSE-2.0
[02]: http://opensource.org/licenses/MIT
[03]: https://lib.rs/crates/rlg
[04]: https://doc.rustlogs.com/
[06]: https://github.com/sebastienrousseau/rlg
[07]: https://crates.io/crates/rlg
[08]: https://docs.rs/rlg
[09]: https://codecov.io/gh/sebastienrousseau/rlg
[10]: https://github.com/sebastienrousseau/rlg/actions?query=branch%3Amaster

[build-badge]: https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rlg/release.yml?branch=master&style=for-the-badge&logo=github "Build Status"
[codecov-badge]: https://img.shields.io/codecov/c/github/sebastienrousseau/rlg?style=for-the-badge&token=Q9KJ6XXL67&logo=codecov "Codecov"
[crates-badge]: https://img.shields.io/crates/v/rlg.svg?style=for-the-badge&color=fc8d62&logo=rust "Crates.io"
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.6-orange.svg?style=for-the-badge "View on lib.rs"
[docs-badge]: https://img.shields.io/badge/docs.rs-rlg-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs "Docs.rs"
[github-badge]: https://img.shields.io/badge/github-sebastienrousseau/rlg-8da0cb?style=for-the-badge&labelColor=555555&logo=github "GitHub"
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust'
