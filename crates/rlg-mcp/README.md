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
  <a href="https://crates.io/crates/rlg-mcp"><img src="https://img.shields.io/crates/v/rlg-mcp.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/rlg-mcp"><img src="https://img.shields.io/badge/docs.rs-rlg--mcp-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
</p>

---

## Install

```bash
cargo install rlg-mcp
```

## Tools

| Tool | Inputs | Returns |
| --- | --- | --- |
| **`tail_log`** | `path`, optional `n` (default 100) | Last `n` parseable records, rendered in Logfmt. |
| **`filter_log`** | `path`, optional `min_level`, `component`, `format` | Records matching every supplied filter, in the chosen `LogFormat`. |
| **`summarize_errors`** | `path` | JSON map of `component → error_count` for ERROR-and-above records. |

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

- v0.0.1 only parses the canonical `LogFormat::JSON` input
  shape. MCP / OTLP / ECS / GELF transport-envelope parsing
  is tracked under
  [`crates/rlg-cli/doc/INPUT-FORMATS.md`](../rlg-cli/doc/INPUT-FORMATS.md)
  and lands incrementally — when it does, `rlg-mcp` picks
  the new input formats up automatically through `rlg-cli`'s
  shared parser.
- No SSE / HTTP transport yet — stdio only.

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.
