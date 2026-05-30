<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/rlg/v1/logos/rlg.svg" alt="RLG logo" width="128" />
</p>

<h1 align="center">rlg-cli</h1>

<p align="center">
  <code>jq</code> for structured logs. Tail, filter, and convert log
  streams across all 14 <code>rlg</code> formats from the command line.
</p>

<p align="center">
  <a href="https://crates.io/crates/rlg-cli"><img src="https://img.shields.io/crates/v/rlg-cli.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/rlg-cli"><img src="https://img.shields.io/badge/docs.rs-rlg--cli-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
</p>

---

## Install

```bash
cargo install rlg-cli
```

The binary is named `rlg`.

## Usage

```bash
# Default: read JSON records from stdin, emit Logfmt to stdout.
my-service | rlg

# Tail a file and convert to MCP-shaped JSON-RPC notifications.
rlg /var/log/app.ndjson --format mcp

# Drop everything below ERROR.
my-service | rlg --min-level error

# Show only records from the `db` component.
my-service | rlg --component db

# Filter by a single attribute (value parsed as JSON).
my-service | rlg --attr user_id=42
my-service | rlg --attr 'region="eu-west-1"'

# Combine filters and pick an output format.
rlg /var/log/app.ndjson \
    --min-level warn \
    --component api \
    --format ecs
```

### Supported output formats

`clf`, `cef`, `elf`, `w3c`, `apache`, `log4j-xml`, `json`,
`gelf`, `logstash`, `ndjson`, `mcp`, `otlp`, `logfmt`, `ecs`.

### Input

For `0.0.9`, the CLI accepts the **canonical `LogFormat::JSON`
shape** — one record per line. Unparseable lines pass through
verbatim so you can pipe non-rlg log noise through `rlg`
without losing entries.

Parsers for the other input formats (MCP transport envelope,
OTLP, ECS, GELF, Logstash) are tracked under
[`crates/rlg-cli/doc/INPUT-FORMATS.md`](doc/INPUT-FORMATS.md)
and land incrementally.

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.
