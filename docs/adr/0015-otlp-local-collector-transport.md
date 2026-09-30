<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR 0015 — OTLP Through a Local Collector (no TLS in-process)

- **Status:** Accepted
- **Date:** 2026-09-30
- **Deciders:** repository maintainers
- **Supersedes:** the transport choices of ADR 0010 phases 19b
  (`reqwest`) and 19c (`tonic`). Its reliability primitives
  (`RetryPolicy`, `CircuitBreaker`) stand.

## Context

`rlg-otlp` carried two optional transports from ADR 0010:

- `async`: `reqwest` with `rustls-tls`.
- `grpc`: `tonic` with `tls-ring`. It was a scaffold only: the
  send path returned `GrpcNotImplemented`.

Both brought rustls and `ring` into the tree. `cargo deny
--all-features` failed on them: `webpki-roots` is licensed
CDLA-Permissive-2.0, outside the allow list, and `ring` pulled
duplicate `getrandom` (0.2) and `windows-sys` (0.52) versions.
The CI gate could only check default features.

The default, blocking exporter already had no TLS: `ureq` is built
without its `rustls` feature, so an `https://` endpoint failed at
runtime with "TLS required, but transport is unsecured", while the
crate's own documentation showed one.

## Decision

The exporter speaks plain OTLP/HTTP to an OpenTelemetry Collector
(or another OTLP/HTTP forwarder) on the same host or in the same
pod. The Collector owns TLS, credentials, batching and buffering
towards the backend.

- **`async`** keeps `AsyncOtlpExporter` and its API, with its
  HTTP/1.1 exchange written in-house (`src/http.rs`) over a Tokio
  `TcpStream`: one connection per request, `Connection: close`,
  a `Content-Length` body, and a response read only as far as the
  final status line.
- **`grpc`** is removed. Collectors accept OTLP/HTTP on 4318, and
  an in-house HTTP/2 + HPACK client would be 1–2k lines to own and
  fuzz for no capability OTLP/HTTP lacks.
- Both builders default to `http://localhost:4318/v1/logs`
  (`DEFAULT_ENDPOINT`). The async builder rejects `https://`
  endpoints, header names that are not RFC 9110 tokens, header
  values with CR, LF or NUL, and the headers the client writes
  itself.

The same change removes rlg's two other optional dependencies that
failed the all-features check: `miette` (replaced by
`RlgError::code`, `help` and `report`) and `notify` (replaced by a
polling watcher in `Config::hot_reload_async`).

## Consequences

- `cargo deny --all-features check` passes, and the CI gate checks
  all features. `ring`, `rustls`, `webpki-roots`, `reqwest`,
  `hyper`, `tonic` and `prost` are gone from `rlg-otlp`'s tree.
- No cryptographic code runs in the exporter's process. CVE
  tracking for TLS moves to the Collector, which is patched on its
  own release cycle.
- **Breaking** (0.0.x): the `grpc` feature, `GrpcOtlpExporter`,
  `OtlpError::GrpcEndpoint` and `GrpcNotImplemented` are removed;
  `OtlpError::AsyncTransport` now wraps `std::io::Error`; new
  variants `InvalidEndpoint` and `InvalidHeader`;
  `AsyncOtlpExporterBuilder::build` fails on an `https://` endpoint.
  In rlg, the `miette` feature is removed and
  `ConfigError::WatcherError` wraps `std::io::Error`.
- Deployments that exported straight to a SaaS endpoint over
  `https://` with the `async` feature must add a Collector. The
  crate documentation carries a minimal configuration.
- The blocking exporter still uses `ureq` (a required dependency,
  not an optional one). Moving it onto `src/http.rs` would remove
  `ureq` as well; that is left as a separate decision.

## Alternatives considered

- **Keep rustls, own only the HTTP layer.** Rejected: `ring` keeps
  the duplicate `getrandom` and `windows-sys` versions and the
  crypto-audit burden, for a capability the Collector provides.
- **Allow CDLA-Permissive-2.0 and skip the duplicates.** Rejected:
  it silences the gate instead of shrinking the tree.
- **Own HTTP/2 to keep OTLP/gRPC.** Rejected on cost; see above.
