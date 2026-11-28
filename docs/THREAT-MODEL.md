<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Threat model

What rlg trusts, what it does not, and what protects each boundary.
Written for 0.0.21; update it with every change to a boundary.
Report a weakness privately, as [`SECURITY.md`](https://github.com/sebastienrousseau/rlg/blob/main/SECURITY.md)
describes.

## Assets

- **Log integrity:** a record says what the application logged, and
  no input can forge a record, a field or a journal entry.
- **Confidentiality of log content:** secrets and personal data that
  reach a log stay where the operator sends them.
- **Confidentiality of the host:** tools that read logs (`rlg-mcp`,
  `rlg-cli`, `rlg-report`) do not become a way to read other files.
- **Availability of the application:** logging never blocks or
  crashes the caller; overload drops records and counts them.
- **Release integrity:** what users install is what this repository
  built and signed.

## Trust boundaries

| # | Boundary | Untrusted input | Protection | Status |
| --- | --- | --- | --- | --- |
| 1 | Application to rlg | Message text, attribute keys and values, component, time | Every text format escapes controls and its delimiters (`log/escape.rs`); JSON formats escape every control character; fuzzed by `render_one_line` | 0.0.21 |
| 2 | rlg to journald | The formatted record | Binary field framing for values with newlines (`sink/journal.rs`); fixed `PRIORITY` and `SYSLOG_IDENTIFIER` | 0.0.21 |
| 3 | rlg to files | Paths from config | Files created `0640`, directories `0750`; rotation never replaces an archive. Symlink refusal and retention limits: 0.0.22 | Partial |
| 4 | Config and environment to rlg | TOML files, `RLG_*` and `OTEL_*` variables | Config validation; `env_vars` values hidden from `Debug` and `diff`; OTLP endpoints refused unless printable ASCII; `OTEL_EXPORTER_OTLP_HEADERS` decoded and checked for line breaks | 0.0.21 |
| 5 | rlg-otlp to a collector | Collector responses | Bounded response head (16 KiB), status line only, per-request deadline covering DNS, connect and read; plain `http://` only, to a local collector (ADR 0015) | 0.0.21; TLS in WP-X1 |
| 6 | MCP clients to rlg-mcp | Tool arguments (paths, globs, counts), HTTP requests | Reads only regular files under the log root (`--root`), with one refusal message; 64 MiB per file, 1 MiB per line, 10,000 records, 1,000 glob matches; Host and Origin checks, loopback bind by default (warning otherwise), session cap, 1 MiB message cap, stdin line cap. Required `--root` and a token for remote binds: 0.0.22 | 0.0.21 |
| 7 | HTTP clients to rlg-tower | Request path, trace header | Trace header kept only when it is a `traceparent` or a short plain id; path logged without the query | 0.0.21 |
| 8 | Log files to rlg-cli and rlg-report | Records read from disk | Parsed with `serde_json` into typed records; terminal escaping of output is planned (plan WP-D2) | Partial |
| 9 | Dependencies and CI to releases | Crates, actions, tools, base images | `cargo deny` (advisories, yanked, bans, sources), `cargo vet`, SHA-pinned actions, pinned and checksummed tools, digest-pinned distroless images, signed tags, Sigstore signatures, SLSA provenance, signed container image with SBOM | 0.0.21 |

## Out of scope

- An attacker who can run code as the application's user: they can
  write any log line and read any file the application can.
- Confidentiality on the network between rlg-otlp and a non-local
  collector: the exporter has no TLS; run a collector on the same
  host or pod (ADR 0015) until WP-X1 adds TLS.
- Denial of service by the application itself logging faster than the
  sink can write: records are dropped and counted, by design.

## Assumptions

- The OpenTelemetry Collector rlg-otlp sends to is operated by the
  same party as the application.
- Log readers (people, SIEMs, `journalctl`) treat a record as data and
  do not execute it.
- The `unsafe` code listed in `SECURITY.md` is reviewed with every
  change to it.
