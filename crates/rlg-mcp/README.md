<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/rlg/v1/logos/rlg.svg" alt="RLG logo" width="128" />
</p>

<h1 align="center">rlg-mcp</h1>

<p align="center">
  Model Context Protocol server exposing <code>rlg</code> log streams
  as tools to LLM agents (Claude Desktop, Cursor, mcp.run).
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/rlg/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rlg/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/rlg-mcp"><img src="https://img.shields.io/crates/v/rlg-mcp.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/rlg-mcp"><img src="https://img.shields.io/badge/docs.rs-rlg--mcp-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://lib.rs/crates/rlg-mcp"><img src="https://img.shields.io/badge/lib.rs-rlg--mcp-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/rlg"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/rlg?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" /></a>
</p>

---

## Install

```bash
cargo install rlg-mcp
```

Requires Rust **1.88.0** or newer (edition 2024).

## Tools

- `tail_log` — Inputs: `path`, optional `n` (default 100). Returns the last `n` parseable records, rendered in Logfmt.
- `filter_log` — Inputs: `path`, optional `min_level`, `component`, `format`. Returns records matching every supplied filter, in the chosen `LogFormat`.
- `summarize_errors` — Inputs: `path`. Returns a JSON map of `component → error_count` for ERROR-and-above records.

## Wire format

`rlg-mcp` speaks the MCP **stdio transport** — JSON-RPC 2.0,
one request per line on stdin, one response per line on stdout.
The protocol revision is **2025-06-18**.

### Quick smoke test

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | rlg-mcp
```

## Client configurations

### Claude Desktop

`~/Library/Application Support/Claude/claude_desktop_config.json` (macOS):

```jsonc
{
  "mcpServers": {
    "rlg": {
      "command": "rlg-mcp"
    }
  }
}
```

### Cursor

`.cursor/mcp.json` in the workspace root:

```jsonc
{
  "mcpServers": {
    "rlg": { "command": "rlg-mcp" }
  }
}
```

### mcp.run

Use the **stdio** server registration; supply `rlg-mcp` as
the executable and no arguments.

## Limitations

- v0.0.11 only parses the canonical `LogFormat::JSON` input
  shape. MCP / OTLP / ECS / GELF transport-envelope parsing
  is tracked under
  [`crates/rlg-cli/doc/INPUT-FORMATS.md`](../rlg-cli/doc/INPUT-FORMATS.md)
  and lands incrementally — when it does, `rlg-mcp` picks
  the new input formats up automatically through `rlg-cli`'s
  shared parser.
- No SSE / HTTP transport yet — stdio only.

## Related MCP Servers

Sibling developer-tools MCP servers by the same author — open-source, Apache-2.0 / MIT dual-licensed, targeting AI agents that need structured access to code, config, and observability data:

| Server | Purpose |
|---|---|
| [`noyalib-mcp`](https://github.com/sebastienrousseau/noyalib) | Lossless YAML 1.2 parsing, formatting & validation (Rust, 100% spec compliance) |
| [`pain001-mcp`](https://github.com/sebastienrousseau/pain001-mcp) | Generate & validate ISO 20022 pain.001 payment initiation files |
| [`bankstatementparser-mcp`](https://github.com/sebastienrousseau/bankstatementparser-mcp) | Parse bank statements (BAI2, MT940/MT942, CAMT.053, OFX, CSV) |
| [`camt053-mcp`](https://github.com/sebastienrousseau/camt053-mcp) | Parse & reconcile ISO 20022 camt.053 bank-to-customer statements |
| [`acmt001-mcp`](https://github.com/sebastienrousseau/acmt001-mcp) | Generate & validate ISO 20022 acmt.001 account management messages |

---

## MCP Registry

`mcp-name: io.github.sebastienrousseau/rlg-mcp`

---

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.
