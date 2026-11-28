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

Requires Rust **1.89.0** or newer (edition 2024).

## Tools

Every tool is read-only and idempotent, and returns its answer twice:
as text for the model and as structured content.

- `tail_log`: inputs `path`, optional `n` (default 100). Returns the
  last `n` parseable records, rendered in Logfmt.
- `filter_log`: inputs `path`, optional `min_level`, `component`,
  `format`. Returns records matching every supplied filter, in the
  chosen `LogFormat` (default Logfmt).
- `summarize_errors`: input `path`. Returns a JSON map of
  `component → error_count` for ERROR-and-above records.
- `tail_logs_glob`: inputs `glob_pattern`, optional `lines` (default
  100), `level`. Returns the last `lines` parseable records across
  every file the glob matches, newest last.

## Prompt and resources

- Prompt `triage_error_spike`: a guided workflow for investigating a
  spike of errors with the tools above.
- Resource `rlg://log-levels`: the ordered rlg levels, as JSON.
- Resource template `rlg://tail/{path}`: the last 100 parseable
  records of a log file, as text.

## Transports

```bash
rlg-mcp --root /var/log/app                                  # stdio (default)
rlg-mcp --root /var/log/app --transport streamable-http      # http://127.0.0.1:8000/mcp
rlg-mcp --root /var/log/app --transport sse --port 8001      # GET /sse, POST /messages/
```

- **stdio**: JSON-RPC 2.0, one message per line; a line may not pass
  16 MiB.
- **Streamable HTTP** (`/mcp`): MCP revisions 2025-11-25 and
  2026-07-28 on one endpoint.
- **HTTP+SSE** (`/sse` and `/messages/`): the 2024-11-05 transport,
  for clients that still expect it.

The protocol is handled by [`rmcp`](https://crates.io/crates/rmcp),
the official MCP SDK, which negotiates the revision with the client.
`--host` (default `127.0.0.1`), `--port` (default 8000) and
`--max-sessions` (default 64) apply to the HTTP transports. Both HTTP
transports refuse a `Host` the server does not answer to and a
non-local `Origin`; the SSE transport refuses a posted message over
1 MiB. `rlg-mcp --help` lists every option.

### Quick smoke test

Over stdio a client initialises the session first, then lists the
tools:

```bash
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"smoke","version":"0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' |
  rlg-mcp --root .
```

## Where the tools may read

The tools read only regular files under one directory, the **log
root**: `--root <dir>`, else the `RLG_MCP_ROOT` environment variable,
else the current directory, with a warning. Running without a root
was deprecated in 0.0.21 and keeps working until `--root` becomes
required in the first release on or after 2027-02-26; see
[`DEPRECATIONS.md`](../../DEPRECATIONS.md). A path or glob that
resolves outside the root, through `..`, an absolute path or a
symlink, is refused. A file is read up to 64 MiB, lines over 1 MiB
are skipped, an answer holds at most 10,000 records, and a glob may
match at most 1,000 files.

```bash
rlg-mcp --root /var/log/app
```

In a container, mount the logs read-only and point the root at them:

```bash
docker run --rm -i -v /var/log/app:/logs:ro \
  ghcr.io/sebastienrousseau/rlg-mcp:0.0.22 --root /logs
```

The HTTP transports have no authentication: keep `--host` on
loopback (the default). A non-loopback `--host` prints a warning.

## Client configurations

### Claude Desktop

`~/Library/Application Support/Claude/claude_desktop_config.json` (macOS):

```jsonc
{
  "mcpServers": {
    "rlg": {
      "command": "rlg-mcp",
      "args": ["--root", "/var/log/app"]
    }
  }
}
```

### Cursor

`.cursor/mcp.json` in the workspace root:

```jsonc
{
  "mcpServers": {
    "rlg": { "command": "rlg-mcp", "args": ["--root", "/var/log/app"] }
  }
}
```

### mcp.run

Use the **stdio** server registration; supply `rlg-mcp` as
the executable and `--root <your log directory>` as its arguments.

## Limitations

- Input is the canonical `LogFormat::JSON` record shape, one record
  per line, through `rlg-cli`'s shared parser; other lines are
  skipped. Reading the other rlg formats (MCP, OTLP, ECS envelopes)
  back is planned under
  [WP-X2 in the implementation plan](../../docs/IMPLEMENTATION-PLAN.md#wp-x2-rlg-mcp-for-agents),
  and `rlg-mcp` picks each one up through that parser when it lands.
- The HTTP transports have no authentication; see above.

## Related MCP Servers

Sibling developer-tools MCP servers by the same author — open-source, Apache-2.0 / MIT dual-licensed, targeting AI agents that need structured access to code, config, and observability data:

| Server | Purpose |
| --- | --- |
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
