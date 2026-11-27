<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# rlg implementation plan: 10/10 in every category

> **Status:** Proposed, 2026-11-27. Supersedes
> [`IMPLEMENTATION-PLAN-v0.1.0.md`](IMPLEMENTATION-PLAN-v0.1.0.md),
> which is kept because ADRs 0003 to 0014 cite its phase numbers.
>
> **Baseline:** rlg 0.0.20 (`main` at `1f95916`). Every finding below
> was raised by the 2026-11-27 audit; those marked **verified** were
> reproduced by running code or confirmed by reading it, the rest are
> audit findings still to be confirmed by the work package's
> fail-first test.
>
> **Audience:** maintainers and contributors picking up a work
> package, and adopters who want to know what rlg commits to.

## How to read this plan

- Section 1 defines the ten categories, scores rlg today, and states
  the **measurable** condition for 10/10 in each, plus the CI gate
  that keeps it there. A category is 10/10 only when every condition
  holds and its gate is required on `main`.
- Section 2 lists the ceilings that code cannot lift.
- Section 3 is the work: 37 work packages (WP) grouped into ten
  milestones, one release each. Every WP names the problem with its
  evidence, the change, the files, the public API (with code), the
  fail-first test, the gate, and the semver impact.
- Section 4 maps milestones to releases; section 5 lists the
  decisions only the maintainer can make; section 6 is the final
  definition of done.

Every WP follows `AGENTS.md`: work lands on `feat/v<next>` and
reaches `main` through one release PR; commits are signed with an
`Assisted-by:` trailer; a bug fix starts with a test seen to fail;
complexity ceilings (cyclomatic 10, cognitive 15, Halstead 30, 60
lines per function, 500 per file) hold for touched code; the hot path
(`ingest()`) takes no lock and does no formatting; decisions that
will be questioned get an ADR.

## 1. Categories, scores and the 10/10 bar

Scores are the auditors' judgement on a 0 to 10 scale against the
2026 best-in-class baseline (tracing-subscriber with
tracing-appender, fastrace, logforth, spdlog-rs, the OpenTelemetry
Rust SDK 0.33, Vector).

| # | Category | Today | Main reasons |
| --- | --- | --- | --- |
| C1 | Correctness and standards conformance | 4 | Double newline, OTLP logs off-spec, CEF/GELF/MCP/journald off-spec, `init()` format ignored |
| C2 | Architecture and extensibility | 5 | Closed sink enum, config never reaches the pipeline, one global everything |
| C3 | Reliability and delivery guarantees | 3 | Silent drops, no stats API, shutdown is terminal, exit loses records, flusher panic invisible |
| C4 | Performance and resource efficiency | 6 | Fast enqueue, but formatting at build time, 2 syscalls per record, eviction on the producer |
| C5 | Security and hardening | 5 | MCP reads any file, log and journald injection, OTLP request-line injection, world-readable files |
| C6 | Supply chain and release integrity | 8 | Scorecard 8.3; 172 vet exemptions; unsigned images; unpinned `mcp-publisher` |
| C7 | Observability | 5 | Good span context and OTLP spans; no engine metrics, logs export manual, no `EventName` |
| C8 | Developer experience and documentation | 5 | Journald-by-default surprise, stale tutorial, docs drift, `RUST_LOG` flattened |
| C9 | Verification depth | 6 | Miri, loom, Kani and fuzz exist but test models, not the real queue or outputs |
| C10 | Operations and packaging | 6 | Signed releases and distroless image; PKGBUILD skips checksums, no SBOM in image, no rotation |

### C1. Correctness and standards conformance: the 10/10 bar

1. Every record is framed exactly once: one `\n` per record on
   stdout and file; journald uses the native protocol with binary
   framing for values containing newlines.
2. `LogFormat::OTLP` and rlg-otlp's log export validate against the
   OTLP/JSON protobuf mapping (`opentelemetry-proto` v1.x schema):
   `timeUnixNano` and `observedTimeUnixNano` as decimal strings,
   attributes as `KeyValue[]`, `severityNumber` on the 1 to 24 scale,
   ids omitted when absent, `eventName` when set.
3. Every named format either follows its specification (CEF 25,
   GELF 1.1, ECS 8.x, MCP `notifications/message`, logfmt quoting
   rules) or is removed or renamed. No invented "ELF" or "W3C" lines.
4. Every builder setting takes effect on every path (`Log::fire`,
   `log` facade, `tracing`, macros).
5. Every output parses back with rlg's own reader (`rlg-cli`),
   for every format that has a reader.

**Gate:** `conformance` CI job: an OTLP collector container
(`otel/opentelemetry-collector-contrib`, pinned by digest) receives
rlg's logs and spans and its `debug` exporter output is checked;
snapshot tests per format; a fuzz target that renders random records
in every format and re-parses them.

### C2. Architecture and extensibility: the 10/10 bar

1. Sinks are a trait; stdout, file, rotating file, journald, os_log,
   OTLP and test capture are implementations; fan-out is a sink.
2. `Config` drives the whole pipeline (sink, format, level, filters,
   rotation, retention) and hot-reload re-applies it.
3. Filtering is per target (`RUST_LOG` directive grammar) and
   reloadable without restarting the flusher.
4. Public enums that will grow are `#[non_exhaustive]`; internal
   fields are private; no feature-dependent variants on public enums.
5. No dependency cycles between workspace crates.

**Gate:** `cargo semver-checks` (already required), a `public-api`
diff check in CI, `cargo hack check --each-feature`, and an ADR for
every change to these rules.

### C3. Reliability and delivery guarantees: the 10/10 bar

1. `accepted == written + dropped + pending` holds at all times and
   is observable through `ENGINE.stats()`.
2. The overflow policy is chosen by the user (`DropNewest` default,
   `DropOldest`, `Block { timeout }`), documented with its cost.
3. `flush(timeout)` drains without stopping the engine; shutdown
   counts late records; `log::Log::flush` is real.
4. A flusher panic is caught, counted and the flusher restarted; a
   failed sink write is counted, never silent.
5. Exit paths are documented: what `FlushGuard`, `process::exit`,
   `panic = "abort"` and signals lose, and how to avoid it.

**Gate:** a `reliability` test suite (saturation, kill-the-flusher,
failing sink, shutdown race under loom), required in CI.

### C4. Performance and resource efficiency: the 10/10 bar

1. Producer path: no formatting and at most one allocation per
   non-static attribute; measured with a counting allocator.
2. Flusher: one `write` per drained batch (up to 64 records).
3. Published, reproducible numbers on one machine, against
   tracing-appender non-blocking, fastrace, logforth and spdlog-rs:
   p50, p99 and p999 per call (1 and 8 producers), end-to-end drain
   time to a file, drop rate at a sustained rate, RSS.
4. No regression over 5% on any tracked benchmark between releases.
5. Idle CPU: the flusher does not wake more than 10 times a second
   when nothing is logged.

**Gate:** `bench-publish.yml` runs the suite on a fixed runner class
and fails on a regression past the threshold; `docs/BENCHMARKS.md`
is generated from its output.

### C5. Security and hardening: the 10/10 bar

1. No input can forge a record or a field in any format or sink
   (fuzzed).
2. rlg-mcp reads only under an explicit root, with size, line, file
   and time limits; non-loopback binds require a token.
3. Network clients validate every byte they put on the wire and
   refuse credentials over plaintext to non-local hosts unless the
   caller opts in; TLS is available behind a feature.
4. Files are created `0640` with `O_NOFOLLOW`; rotation never
   overwrites an archive; retention is bounded.
5. Redaction can be enforced engine-wide and covers the common
   secret and PII classes plus key-based rules.
6. `SECURITY.md` is current; a threat model (`docs/THREAT-MODEL.md`)
   names every trust boundary.

**Gate:** fuzz targets for every parser and renderer in
`fuzz-smoke.yml`; CodeQL and Scorecard with zero open alerts;
`cargo deny` with `yanked = "deny"`.

### C6. Supply chain and release integrity: the 10/10 bar

1. Every release artifact, the container images and the crates have
   Sigstore signatures and SLSA build provenance attached to the
   GitHub release (Scorecard Signed-Releases 10 requires the
   provenance files on the release).
2. SBOM (CycloneDX) for binaries and images.
3. Every tool downloaded in CI is pinned by version and digest.
4. cargo-vet exemptions shrink every release (target: under 40, all
   network and parsing crates audited or imported from trusted
   audit sets).
5. OpenSSF Best Practices: Silver, then Gold when the ceilings in
   section 2 allow.

**Gate:** Scorecard in CI with a minimum per check; `cargo vet`
with a committed exemption count that can only shrink.

### C7. Observability: the 10/10 bar

1. Logs, traces and engine metrics leave through OTLP from one
   configuration, without the OpenTelemetry SDK.
2. Log records carry trace context on every path, and `EventName`
   for event-style records (OpenTelemetry's replacement for span
   events).
3. The engine exports its own metrics: accepted, written, dropped,
   write errors, queue depth, flusher restarts.
4. Resource attributes (`service.name`, `service.version`,
   `deployment.environment.name`, host) are configurable and follow
   semantic conventions.

**Gate:** the C1 conformance job also checks metrics and resource
attributes.

### C8. Developer experience and documentation: the 10/10 bar

1. `rlg::init()` with no arguments prints readable output to stdout
   on every OS, honouring `RUST_LOG` per target.
2. Every public item has a compiling doc example (no `ignore`;
   `no_run` only where it needs a network or a socket).
3. Every claim in the READMEs and docs is backed by a test or a
   benchmark, and a docs check fails on version or feature drift.
4. Each satellite is either complete or labelled experimental in its
   README, Cargo description and docs.rs page.
5. Error messages name the cause and the fix (`RlgError` help text
   extended to `InitError`, config and sink errors).

**Gate:** `cargo test --doc` with a count of `ignore` blocks that
can only fall to zero; `scripts/check-doc-versions.sh` extended to
feature tables; link and lint checks (already required).

### C9. Verification depth: the 10/10 bar

1. loom runs the real `ShardedQueue` and the real park/wake
   handshake, not a model.
2. Kani proves the overflow accounting invariant and the timestamp
   and severity mappings.
3. Fuzz targets cover every renderer, every parser, the journald
   encoder, the OTLP endpoint parser and the MCP path guard.
4. Property tests: render then parse is the identity for every
   format with a reader.
5. Coverage tests are replaced by behaviour tests with assertions;
   line coverage stays at or above 90% with no test that asserts
   nothing.

**Gate:** loom, Kani, Miri and fuzz-smoke required on `main`;
a lint (`scripts/check-assertions.py`) that fails on a test function
with no assertion.

### C10. Operations and packaging: the 10/10 bar

1. File output rotates by size, time or date with retention, and
   survives external rotation (reopen on `SIGHUP` or inode change).
2. Packages (PKGBUILD, Debian, Nix, container) are reproducible,
   checksummed and install every binary.
3. Container images are signed, carry an SBOM and run as non-root
   with a read-only root filesystem.
4. A runbook (`docs/how-to/operate.md`) covers sizing the buffer,
   reading the metrics, journald limits and upgrades.

**Gate:** the existing `make install` and reproducible-package jobs,
plus an image signature and SBOM check in `release.yml`.

## 2. Ceilings code cannot lift

These are recorded so that "10/10" is honest. Each needs a people or
organisation change, not a pull request.

| Item | Today | Ceiling while rlg has one maintainer | What lifts it |
| --- | --- | --- | --- |
| Scorecard Code-Review | 0 | 0 | A second maintainer approving pull requests |
| Scorecard Branch-Protection | 3 | 4 to 5 (rulesets make enforced rules visible) | Required approvals, possible only with a second reviewer |
| Scorecard Contributors | 0 | 0 | Contributors from at least three organisations |
| OpenSSF Best Practices Gold | Passing | Silver | Gold needs two unassociated significant contributors |

Alerts #6 and #76 were dismissed as "won't fix" on 2026-11-27 for
this reason. WP-S6 below moves the protection rules into a
repository ruleset so the parts that are enforced become visible.

## 3. Work packages

Each WP lists: **Problem** (with evidence), **Change**, **Files**,
**API**, **Test first**, **Gate**, **Semver**. "Verified" means
reproduced on 2026-11-27.

### Milestone M1 (0.0.21): correct output

#### WP-C1. One newline per record (verified)

- **Problem:** `engine.rs:248` formats with `writeln!`, then
  `sink.rs:221-226` and `:246` write another `\n`; journald gets
  `MESSAGE=...\n\n`. A JSON log file reads `}\n\n{`.
- **Change:** `emit` formats with `write!`; each sink adds the
  framing it needs. `PlatformSink::File` wraps a `BufWriter` flushed
  when the queue drains and at shutdown.
- **Files:** `crates/rlg/src/engine.rs`, `crates/rlg/src/sink.rs`.
- **Test first:** `tests/test_set_sink.rs` reads the file back and
  asserts `written.matches('\n').count() == records` and no `"\n\n"`.
- **Gate:** the test; a fuzz target asserting framing for random
  payloads.
- **Semver:** none (output fix, noted in CHANGELOG).

#### WP-C2. Spec-conformant OTLP log records (verified)

- **Problem:** `log.rs:425-449` writes `timeUnixNano` as ISO-8601,
  attributes as a JSON map, `severityNumber` on rlg's scale (INFO 6,
  ERROR 8), and `""` ids; `rlg-otlp/src/lib.rs:363-388` hard-codes
  `service.name = "rlg"`.
- **Change:** a `crate::otlp` module shared by the renderer and
  rlg-otlp; a resource builder in rlg-otlp.

```rust
/// OTLP SeverityNumber (logs data model, 1..=24).
pub const fn otlp_severity(level: LogLevel) -> u8 {
    match level {
        LogLevel::TRACE => 1,
        LogLevel::DEBUG | LogLevel::VERBOSE => 5,
        LogLevel::INFO => 9,
        LogLevel::WARN => 13,
        LogLevel::ERROR => 17,
        LogLevel::FATAL | LogLevel::CRITICAL => 21,
        _ => 0, // SEVERITY_NUMBER_UNSPECIFIED
    }
}
```

  Records carry the time as nanoseconds (WP-P1); until then the
  renderer parses `Log::time` back to nanoseconds once. Attributes go
  through the same `AnyValue` encoder as `rlg-otlp/src/traces.rs`.

- **Files:** `crates/rlg/src/log.rs`, new `crates/rlg/src/otlp.rs`,
  `crates/rlg-otlp/src/lib.rs`, `crates/rlg-otlp/src/traces.rs`.
- **API:** `rlg_otlp::Resource::builder().service_name(..)
  .service_version(..).attribute(k, v)`; both exporters take it.
- **Test first:** a snapshot test that decodes the JSON with the
  `opentelemetry-proto` serde types (dev-dependency) and fails on
  today's output.
- **Gate:** the `conformance` CI job (section 1, C1).
- **Semver:** output change; `serialise_batch` gains a `&Resource`
  parameter (breaking, 0.0.x).

#### WP-C3. Builder settings reach every path (verified)

- **Problem:** `Log::build` hard-codes `LogFormat::MCP`
  (`log.rs:158-167`); `RlgSubscriber::event` never sets a format;
  after `builder().format(JSON).init()`, `Log::info().fire()` prints
  an MCP envelope. `rlg_time_it!` and `rlg_span!` force their own
  formats.
- **Change:** an engine-wide default format (`AtomicU8` in the
  engine, set by `init()`); `Log` carries `Option<LogFormat>`; the
  flusher resolves `None` to the default. Macros stop forcing a
  format.
- **Files:** `crates/rlg/src/log.rs`, `engine.rs`, `init.rs`,
  `tracing.rs`, `macros.rs`.
- **API:** `Log.format` becomes `Option<LogFormat>` (field), with
  `Log::format(fmt)` unchanged for builders.
- **Test first:** the reproducer from the audit as an integration
  test: `init(JSON)` then `Log::info().fire()` must produce JSON.
- **Semver:** breaking for code reading `log.format` directly.

#### WP-C4. `caller` correct on every path (verified for `log`)

- **Problem:** `logger.rs:84`, `tracing.rs:92` and
  `rlg-tower/src/lib.rs:268` call `fire()` from inside rlg, so
  `caller` is rlg's own line (`rlg/src/logger.rs:84` observed).
- **Change:** those paths ingest with `LogEvent::new` and set
  `caller` from `log::Record::file()/line()`, tracing metadata, or
  (tower) leave it out.
- **Test first:** `log::info!` from a test file must give that file
  and line.
- **Semver:** none.

#### WP-C5. Formats follow their specs or go

- **Problem:** CEF header misassigned and unescaped
  (`log.rs:258-268`); "ELF" and "W3C" invented (`:270-292`); Apache
  puts the level in the status slot (`:294-304`); GELF `timestamp`
  is a string and `level` uses rlg's scale; MCP uses
  `notifications/log` instead of `notifications/message`; logfmt
  escapes neither `\` nor newlines.
- **Change:** an ADR deciding per format (keep and fix, rename, or
  remove), then one renderer module per kept format under
  `crates/rlg/src/log/format/`, each with its escaper. Proposed:
  fix CEF, GELF, ECS, logfmt, MCP; remove ELF and W3C; rename CLF to
  `KeyValue` or make it real NCSA CLF; keep Apache only as true
  Combined Log Format for HTTP records (rlg-tower).
- **Test first:** per-format conformance snapshots, starting with a
  CEF record whose description contains `|` and `\n`.
- **Semver:** breaking (`LogFormat` variants change); `LogFormat`
  becomes `#[non_exhaustive]` in the same release.

#### WP-C6. journald native protocol

- **Problem:** `sink.rs:328-335` sends `PRIORITY` and `MESSAGE` as
  text, so a newline in the payload forges fields; no
  `SYSLOG_IDENTIFIER`; datagrams over `SO_SNDBUF` fail silently.
- **Change:** binary framing for values containing `\n`,
  `SYSLOG_IDENTIFIER` (default: program name), structured fields
  (`RLG_COMPONENT`, `CODE_FILE`, `CODE_LINE`, `TRACE_ID`), and a
  memfd fallback (`memfd_create` + `SCM_RIGHTS`) for large records,
  counted in stats if it fails.

```rust
fn push_field(buf: &mut Vec<u8>, key: &[u8], value: &[u8]) {
    buf.extend_from_slice(key);
    if value.contains(&b'\n') {
        buf.push(b'\n');
        buf.extend_from_slice(&(value.len() as u64).to_le_bytes());
        buf.extend_from_slice(value);
    } else {
        buf.push(b'=');
        buf.extend_from_slice(value);
    }
    buf.push(b'\n');
}
```

- **Test first:** encode `"x\nPRIORITY=0"` and assert the datagram
  decodes to one `MESSAGE` field (a small decoder in the test).
- **Semver:** none.

#### WP-C7. Small correctness fixes

- `SpanExporter::flush` uses blocking `send` and can wait past its
  timeout (`rlg-otlp/src/spans.rs:111`): use `try_send` and fall back
  to waiting with the remaining timeout. Test with a full queue.
- Per-record `std::env::var` in the journald and os_log sinks
  (`sink.rs:263`, `:312`): decide once at construction.
- `LogLevel` ordering puts `NONE`/`DISABLED` below `TRACE`
  (`log_level.rs:147-158`): reorder so `NONE` filters everything;
  property test that `includes` matches the engine gate.
- `parse_rust_log` ignores `off`: parse it.

### Milestone M2 (0.0.21, private advisory first): security fixes

#### WP-S1. rlg-mcp reads only under a root (verified)

- **Problem:** `crates/rlg-mcp/src/ops.rs:26,48,68,97,104` open any
  path or glob; `tail_log` keeps every record; `tail_logs_glob`
  reads whole files; no auth on HTTP transports. Classified as a
  vulnerability: handled through a GitHub private security advisory
  (`SECURITY.md`), fixed on the advisory's private fork, released,
  then disclosed.
- **Change:**

```rust
pub struct Root(PathBuf); // canonical, from --root or RLG_MCP_ROOT

impl Root {
    pub fn resolve(&self, requested: &Path) -> Result<PathBuf, Refused> {
        let full = self.0.join(requested).canonicalize()
            .map_err(|_| Refused::NotFound)?;           // generic error
        if !full.starts_with(&self.0) { return Err(Refused::OutsideRoot); }
        let meta = std::fs::symlink_metadata(&full).map_err(|_| Refused::NotFound)?;
        if !meta.is_file() { return Err(Refused::NotARegularFile); }
        if meta.len() > MAX_FILE_BYTES { return Err(Refused::TooLarge); }
        Ok(full)
    }
}
```

  Limits: 64 MiB per file, 1 MiB per line (reuse
  `transport/bounded_lines.rs`), 1,000 files per glob, `n <= 10,000`,
  tail through a `VecDeque` ring; tool I/O in `spawn_blocking` with
  a 10 s timeout; a non-loopback bind requires `--allow-remote` and
  `--token <file>` (bearer check in `transport/guard.rs`).

- **Test first:** `../`, absolute paths, symlinks out of the root,
  `/dev/zero`, a FIFO and a glob of `/**/*` each refused.
- **Gate:** a fuzz target over `Root::resolve`.
- **Semver:** `--root` becomes required (breaking CLI change,
  documented; the default can be the current directory with a
  warning for one release).

#### WP-S2. No injection in any renderer

- **Change:** one escaping module (`crates/rlg/src/log/escape.rs`)
  used by every text format: C0 and C1 controls (including ESC) as
  `\u{..}` in text formats; XML entities; CEF rules; logfmt quoting
  when a value contains space, `=`, `"`, `\` or a control character;
  attribute keys restricted to `[A-Za-z0-9_.-]` (others are
  percent-encoded).
- **Test first:** a fuzz target `render_roundtrip` that renders
  random records in every format and asserts one record per line and
  no field the input did not contain.

#### WP-S3. OTLP endpoint and credentials

- **Problem:** `rlg-otlp/src/http.rs:183` writes the endpoint path
  into the request line unchecked; auth headers go in plaintext to
  any host; DNS resolution runs outside the deadline
  (`blocking_http.rs:60`); `OTEL_EXPORTER_OTLP_HEADERS` is not
  percent-decoded.
- **Change:** `Endpoint::parse` rejects bytes `<= 0x20`, `0x7f` and
  non-ASCII in host and path; headers to a non-loopback, non-private
  address require `.allow_insecure_remote()`; DNS on a helper thread
  bounded by the timeout; percent-decode header values per the OTel
  spec and report malformed pairs as an error from `from_env`.
- **Test first:** `http://c:4318/v1 HTTP/1.1\r\nX: y` refused.

#### WP-S4. Files created safely, rotation without loss

- `OpenOptionsExt::mode(0o640)` and `custom_flags(O_NOFOLLOW |
  O_CLOEXEC)` on every log file; `DirBuilder::mode(0o750)`.
- Archive names gain a sequence suffix and are created with
  `O_EXCL`; `max_files` and `max_total_bytes` retention.
- `Count(0)` rejected by validation.
- **Test first:** two rotations in the same second keep both
  archives; a symlinked target is refused.

#### WP-S5. Secrets out of config and requests

- `Config::expand_env_vars` values are redacted in `Debug`, `diff`
  and never written by `save_to_file`; `TryFrom<env::Vars>` reads
  only `RLG_`-prefixed variables.
- rlg-tower: trace header values validated against W3C and
  truncated; query strings dropped from the logged path by default.

#### WP-S6. CI and repository hardening

- Pin `mcp-publisher` by version and SHA-256 in `publish-mcp.yml`;
  pass workflow values to `run:` through `env:`;
  `persist-credentials: false` on every checkout; restrict
  `workflow_dispatch` image pushes to tags.
- Sign container images with cosign and attach
  `provenance: mode=max` and an SBOM.
- Add `.dockerignore`; remove the duplicate root `Dockerfile` or make
  it the single source.
- `deny.toml`: `yanked = "deny"`, `unmaintained = "workspace"`;
  correct the rustls comment.
- Move `main` protection into a repository ruleset (Scorecard reads
  rulesets) without bypass actors.
- `SECURITY.md`: supported versions generated by
  `scripts/check-doc-versions.sh`; document every `unsafe` site
  (sink FFI and rlg-ebpf).
- New `docs/THREAT-MODEL.md`: trust boundaries (application input,
  config files, environment, network collector, MCP clients, log
  files read by tools).

### Milestone M3 (0.0.22): the sink trait and stats

#### WP-A1. `Sink` trait and fan-out

- **Problem:** `PlatformSink` is a closed enum holding a bare `File`
  (`sink.rs:46`), so rotation, test capture, network sinks, OTLP log
  export and fan-out cannot plug in.
- **Change:** an ADR, then:

```rust
pub trait Sink: Send + 'static {
    /// Called once per drained batch, records already formatted and
    /// framed for this sink.
    fn write_batch(&mut self, records: &[Record<'_>]) -> std::io::Result<()>;
    /// Make buffered records durable (called when the queue drains
    /// and at flush and shutdown).
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
    /// Per-sink format; `None` uses the engine default.
    fn format(&self) -> Option<LogFormat> { None }
}

pub struct Record<'a> {
    pub level: LogLevel,
    pub bytes: &'a [u8],
    pub log: &'a Log,
}

pub struct Fanout(Vec<Box<dyn Sink>>);
```

  `PlatformSink` variants become `StdoutSink`, `FileSink`,
  `RotatingFileSink` (from `rotation.rs`), `JournaldSink`,
  `OsLogSink`; `set_sink` takes `impl Sink`; rlg-test gains a
  `CaptureSink` that finally captures records fired by code under
  test.

- **Gate:** `cargo semver-checks`; the reliability suite (WP-R1)
  runs against every built-in sink.
- **Semver:** breaking; `PlatformSink` kept one release as a
  deprecated alias that builds the matching sink.

#### WP-R1. Stats and an overflow policy

```rust
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowPolicy {
    /// Refuse the new record, count it. O(1) on the producer.
    DropNewest,
    /// Evict the oldest record (today's behaviour).
    DropOldest,
    /// Wait for space, at most `timeout`, then drop and count.
    Block { timeout: std::time::Duration },
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EngineStats {
    pub accepted: u64,
    pub written: u64,
    pub dropped: u64,
    pub write_errors: u64,
    pub queue_depth: u64,
    pub flusher_restarts: u64,
}

impl LockFreeEngine {
    pub fn stats(&self) -> EngineStats { /* Relaxed loads */ }
    pub fn set_overflow_policy(&self, policy: OverflowPolicy) { /* AtomicU8 + timeout */ }
}
```

- Counters live in a cache-line-padded struct
  (`crossbeam_utils::CachePadded`) so producers and the flusher do not
  share a line; producers touch only `accepted` and `dropped`.
- `DropNewest` becomes the default: it keeps the producer O(1) and
  frees nothing on the producer thread (today `push_evicting` frees
  evicted records on the caller, `sharded_queue.rs:115-134`).
- **Test first:** saturate with a sink that sleeps; assert
  `accepted == written + dropped + queue_depth` after flush, for each
  policy; a sink returning errors raises `write_errors`.
- **Semver:** additive, but the default policy changes (CHANGELOG).

### Milestone M4 (0.0.23): lifecycle and config

#### WP-R2. Flush, shutdown, panics, exit

- `ENGINE.flush(timeout) -> Result<(), FlushTimeout>`: the flusher
  drains, flushes sinks and acknowledges through a one-shot; the
  engine keeps running. `RlgLogger::flush` calls it.
- `ingest` after shutdown counts the record as dropped
  (`shutdown_flag` checked with `Relaxed`, no lock).
- Close the shutdown race: the flusher re-checks the queue after
  seeing the flag (loom test on the real queue, WP-V1).
- The flusher body runs under `catch_unwind`; a panic increments
  `flusher_restarts` and restarts the loop with a fresh sink handle;
  thread spawn failure leaves the engine in a "no flusher" state that
  `stats()` reports instead of poisoning `ENGINE`.
- `FlushGuard` stays; `docs/how-to/operate.md` documents exit paths
  (`process::exit` and `panic = "abort"` lose the queue; install the
  guard in `main`; optional `rlg::install_panic_flush()` hook that
  flushes from the panic hook with a bounded timeout).

#### WP-A2. Config drives the pipeline

- **Problem:** `apply_config` sets only the level (`engine.rs:371`);
  `from_config` and `RotatingFile` are used only by tests; hot-reload
  swaps an `Arc` nothing reads; `Network` destinations are skipped.
- **Change:** `ENGINE.apply(&Config)` builds a `Fanout` from
  `logging_destinations` (file with rotation and retention, stdout,
  journald, OTLP once WP-O1 lands), sets format, level and filters,
  and swaps the sink in the flusher through a reload slot (the
  `sink_slot` from 0.0.20 generalised to a `ArcSwap`-style handoff the
  flusher checks between batches). Hot-reload calls `apply` and
  reports errors through a callback instead of swallowing them.
  `Network` is rejected by validation until it exists.
- **Test first:** load a config naming a file and `Size(1 KiB)`
  rotation, log 10 KiB, assert archives exist.
- **Semver:** behaviour change for configs that relied on being
  ignored (CHANGELOG, migration note).

### Milestone M5 (0.0.24): filters, one tracing path, defaults

#### WP-A3. Per-target filtering

- Parse the `RUST_LOG` directive grammar (`target=level`, `off`,
  span-name filters later) into a sorted directive table held in an
  `ArcSwap`; the hot path does one atomic load plus a prefix match
  on `&'static str` targets, cached per callsite.
- `RlgLayer` uses `tracing_subscriber::layer::Filter` (per-layer
  filtering) so it no longer filters other layers through `enabled`;
  level changes call `tracing_core::callsite::rebuild_interest_cache`.
- The `log` facade's `set_max_level` follows the table's maximum.
- **Test first:** `warn,mycrate=debug` lets `mycrate` debug through
  and stops `hyper` info.

#### WP-A4. `init()` installs `RlgLayer`

- With the `tracing-layer` feature, `init()` installs
  `registry().with(RlgLayer)` so span context, scrub, span export and
  `traceparent` work without hand-built setup; `RlgSubscriber` is
  deprecated for one release then removed.

#### WP-D1. Defaults that work on first run

- The default sink is stdout on every OS; journald and os_log are
  chosen explicitly (`set_sink`, config, or `RLG_SINK=journald`).
  ADR required: it reverses the native-first default.
- macOS: real `os_log` through `os_log_create` with a subsystem, or
  document that the sink is syslog and rename it.
- Fix `docs/tutorials/getting-started.md` (`journalctl -t rlg` once
  `SYSLOG_IDENTIFIER` exists, the macOS predicate, the fallback).

### Milestone M6 (0.0.25): producer-path performance

#### WP-P1. No formatting on the producer

- **Problem:** `Log::build` formats an ISO timestamp and allocates
  (`log.rs:161`, `datetime.rs:144-164`), copies the description, and
  `with()` runs `serde_json::to_value` per attribute.
- **Change:** `Log.time` becomes a `Timestamp(u64)` (Unix nanos from
  `SystemTime`, or a TSC-calibrated clock behind a feature),
  formatted on the flusher; `description: Cow<'static, str>`;
  attributes in `SmallVec<[(Key, Value); 4]>` with `Key =
  Cow<'static, str>`; `Value` a small enum (`Str`, `I64`, `U64`,
  `F64`, `Bool`, `Json`) so integers and static strings never touch
  `serde_json`.
- **Test first:** a counting allocator test (`#[global_allocator]` in
  a dedicated test binary): `Log::info("static").fire()` performs 0
  allocations on the caller; one `with("k", 1u64)` performs 0.
- **Gate:** the allocation test in CI; the bench regression gate.
- **Semver:** breaking (`Log` fields); a migration note.

#### WP-P2. Buffer sizing and memory

- Capacity configurable at engine start (`RLG_BUFFER`,
  `Config::buffer_capacity`); default stays 65,536.
- `LogEvent` size tracked by a `const` assertion (target: 64 bytes
  after WP-P1).

### Milestone M7 (0.0.26): flusher, layer and exporter performance

#### WP-P3. Batch writes

- The flusher formats a whole drained batch into `fmt_buf` and calls
  `Sink::write_batch` once; `FileSink` and `StdoutSink` issue one
  `write_all` per batch; stdout is locked once per batch.
- Park without the 5 ms timeout when the queue is empty (the
  `flusher_idle` handshake already wakes it); keep a 100 ms backstop.
- Drain `fast-queue` shards round-robin (`sharded_queue.rs:146-153`).

#### WP-P4. `RlgLayer` without per-event cloning

- Span fields stored as `Arc<[(Key, Value)]>` snapshots rebuilt on
  `on_record`; events hold `Arc`s to their scope's snapshots instead
  of cloning maps (`layer.rs:149`); ids cloned for the innermost span
  only; `component` and `span` as `&'static str`; `caller` deferred
  as `(&'static str, u32)`; scrub moves to the flusher (WP-O2).
- **Gate:** a bench with three nested spans and five fields each,
  compared with `tracing_subscriber::fmt` into `io::sink`.

#### WP-P5. Exporter efficiency

- Keep-alive connections reused across batches (drop
  `Connection: close`, read the response body to completion), cached
  `Endpoint` and resolved address, retries scheduled by the worker
  without blocking new spans (a retry queue with deadlines), and a
  streaming encoder writing JSON directly (`serde::Serialize` structs
  or hand-written writers) instead of `serde_json::Value` trees.
- **Gate:** an exporter bench (spans per second to a local
  collector) in the suite.

### Milestone M8 (0.0.27): measured and verified

#### WP-B1. A reproducible benchmark suite

- New `crates/rlg/benches/suite/` with criterion and an `hdrhistogram`
  latency harness:
  - per-call latency p50, p99, p999 with 1, 4 and 8 producers;
  - end-to-end: time to drain 1,000,000 records into a file;
  - sustained rate: records per second until drops begin, and drop
    rate at 2x that rate;
  - `RlgLayer` with nested spans; OTLP export throughput;
  - RSS after start and after a burst.
- Competitors on the same machine: `tracing` with
  `tracing-appender::non_blocking`, `fastrace`, `logforth`,
  `spdlog-rs`. Method, hardware and versions committed in
  `docs/BENCHMARKS.md`, generated from the run.
- `bench-publish.yml` fails on a regression over 5% against the
  stored baseline.

#### WP-V1. Verification that tests the real code

- loom: run `ShardedQueue`, `push` and the `flusher_idle` park/wake
  handshake under `--cfg loom` (the queue gains a loom-compatible
  atomics shim), replacing the `Mutex<Vec>` model in
  `tests/loom_engine.rs`.
- Kani: prove `accepted == written + dropped + depth` for the
  counter logic, the OTLP severity mapping, and timestamp
  conversion.
- Fuzz: `render_roundtrip` (WP-S2), `journald_encode` (WP-C6),
  `otlp_endpoint` (WP-S3), `mcp_root` (WP-S1); all in
  `fuzz-smoke.yml` and proposed to OSS-Fuzz (`docs/OSS-FUZZ.md`).
- Property tests: render then parse is the identity for JSON,
  NDJSON, logfmt, OTLP and MCP.
- Replace `tests/test_coverage_*.rs` with behaviour tests; add
  `scripts/check-assertions.py` (fails on a test with no assertion).

### Milestone M9 (0.0.28 to 0.0.29): observability that leads

#### WP-O1. OTLP logs and metrics from the engine

- `rlg_otlp::OtlpSink` implements `Sink`: batches formatted records
  as OTLP log records (WP-C2), with resource attributes, retries and
  stats; logs from `Log::fire` and the `log` facade carry the
  current trace context when `RlgLayer` is installed (a thread-local
  "current span ids" the layer maintains on enter and exit).
- `Log::event(name)` sets OTLP `eventName` (OpenTelemetry's
  replacement for span events).
- `rlg_otlp::MetricsExporter` exports `EngineStats` and
  `SpanExportStats` as OTLP sums and gauges every 30 s.
- One configuration: `rlg_otlp::install_from_env()` reads the
  standard `OTEL_*` variables and wires logs, traces and metrics.
- **Gate:** the conformance job checks all three signals.

#### WP-O2. Redaction enforced engine-wide

- `ENGINE.set_scrub(impl Scrub)` runs in the flusher for every record
  on every path; `RlgLayer::with_scrub` keeps working for span
  attributes.
- rlg-redact packs: `Redactor::secrets()` (OpenAI and Anthropic
  `sk-`, GitHub `ghp_`/`gho_`/`ghs_`/`github_pat_`, Slack `xox`,
  Stripe `sk_live_`, Google `AIza`, PEM private keys, Basic auth, URL
  credentials, AWS secret keys), `Redactor::pii()` (email, phone,
  IPv4, IPv6, card numbers with a Luhn check); key-based rules
  (`password`, `passwd`, `secret`, `token`, `api_key`,
  `authorization`, `cookie`, `set-cookie`) that replace the whole
  value; `Cow` fast path that allocates only on a match.
- **Test first:** every pattern with a positive and a negative case;
  a `Log::fire` record with `password=hunter2` is scrubbed with the
  engine hook set.

### Milestone M10 (0.0.30 to 0.0.32): differentiators and finish

#### WP-X1. TLS behind a feature

- A `tls` feature in rlg-otlp using `rustls` with `webpki-roots` or
  platform verification; `https://` accepted when the feature is on.
  ADR superseding 0015 (local-collector transport stays the
  documented default).

#### WP-X2. rlg-mcp for agents

- Bearer or OAuth 2.1 resource-server auth for HTTP transports
  (validate tokens, scopes `logs:read`); idle session reaping;
  stdin line cap lowered to 1 MiB.
- Emit the OpenTelemetry MCP semantic conventions (development
  status; pinned to a version through `OTEL_SEMCONV_STABILITY_OPT_IN`).
- Read every rlg output format (MCP envelopes, logfmt, OTLP, ECS) in
  `rlg-cli::parse_record`.
- Fix `glama.json` (volume mount, tool count).

#### WP-X3. Audit mode (conditional: see decision 1)

- `OverflowPolicy::Block` plus a write-ahead `AuditFileSink`: fsync
  per batch, records chained by SHA-256 (`prev_hash` field) so
  deletion or edits are detectable, retention settings, and a
  verifier in `rlg-cli` (`rlg verify-chain`). Positioned for the EU
  AI Act Article 12 logging duty (applies from 2027-12-02 for Annex
  III systems).

#### WP-X4. Finish or label every satellite

- `uring`: implement batched submission or remove the variant.
- rlg-ebpf: real eBPF enrichment or mark experimental.
- rlg-wasm: build the WASI 0.2 component, update `rlg.wit` to the
  current version, or mark experimental.
- rlg-test: `CaptureSink` (WP-A1) delivers the missing capture.
- Each decision in an ADR; experimental crates say so in their
  Cargo `description`, README and crate docs.

#### WP-D2. Documentation to match

- Rewrite the root README feature table (the `config` default),
  `COMPARISON.md` (span context now exists), `crates/rlg/README.md`
  (empty config files, `${NAME}` substitution claim, `TuiMetrics`
  field names), and fix the `#capabilities-in-0011` anchor.
- Every `ignore` doc block compiles or becomes `no_run` with a
  reason; `InitError` and sink errors get help text like `RlgError`.
- `docs/how-to/operate.md` (runbook) and `docs/THREAT-MODEL.md`.
- Extend `scripts/check-doc-versions.sh` to feature tables and
  `SECURITY.md`.

#### WP-K1. Packaging

- PKGBUILD with real `sha256sums` and every binary
  (`rlg`, `rlg-report`, `rlg-mcp`); Debian packaging from `debian/`
  updated or removed; Nix flake outputs checked in CI.
- Release assets: SLSA provenance (`*.intoto.jsonl`) attached to the
  GitHub release (lifts Scorecard Signed-Releases from 8 to 10),
  CycloneDX SBOMs, cosign signatures for images.
- File sink reopens on `SIGHUP` or when the inode changes, for
  external rotators.

#### WP-Q1. cargo-vet burn-down

- Import audit sets from Mozilla, Google, Bytecode Alliance and
  `cargo-vet`'s registry; audit the network and parsing crates
  (rmcp, axum, hyper, tokio, serde_json, regex) or trust their
  publishers explicitly; commit the exemption count in
  `scripts/vet-exemptions.txt` and fail CI if it grows.
  Target: under 40 exemptions by 0.0.32.

#### WP-Q2. Architecture debt

- Break the `rlg` to `rlg-cli` dev-dependency cycle: move the
  round-trip tests into `rlg-cli`'s own tests.
- Make `LogEvent` fields private, mark `LogFormat`, `LogLevel`,
  `OverflowPolicy`, `EngineStats` and sink errors
  `#[non_exhaustive]`; move `FastSerializer` and engine counters out
  of the public API.
- `cargo hack check --each-feature --no-dev-deps` in CI.
- A `public-api` snapshot (`cargo public-api`) committed and diffed
  in CI.

## 4. Release map

| Release | Milestone | Work packages | Categories lifted |
| --- | --- | --- | --- |
| 0.0.21 | M1 and M2 | C1 to C7, S1 to S6 | C1, C5, C6 |
| 0.0.22 | M3 | A1, R1 | C2, C3 |
| 0.0.23 | M4 | R2, A2 | C3, C2, C10 |
| 0.0.24 | M5 | A3, A4, D1 | C2, C8 |
| 0.0.25 | M6 | P1, P2 | C4 |
| 0.0.26 | M7 | P3, P4, P5 | C4 |
| 0.0.27 | M8 | B1, V1 | C4, C9 |
| 0.0.28 | M9 | O1 | C7 |
| 0.0.29 | M9 | O2 | C5, C7 |
| 0.0.30 | M10 | X1, X2 | C5, C7 |
| 0.0.31 | M10 | X4, D2, K1, Q2 | C8, C10, C2 |
| 0.0.32 | M10 | Q1, X3 (if decided) | C6 |

Every release also clears the hygiene gate (Dependabot, alerts,
audit) before its own work, ships a CHANGELOG section and release
highlights, and keeps the satellites in lockstep.

Dependencies between packages: WP-C2 before WP-O1; WP-A1 before
WP-R1, WP-A2, WP-O1, WP-K1 (rotation reopen) and WP-X4 (rlg-test);
WP-P1 before WP-P4 and the final WP-B1 numbers; WP-S2 before WP-C5
lands its renderers.

## 5. Decisions for the maintainer

1. **Business model.** Open-core with paid features, paid support,
   or purely open source. It decides whether WP-X3 (audit mode) and
   the OAuth part of WP-X2 are built, and how hard WP-B1 is marketed.
2. **Default sink.** WP-D1 proposes stdout everywhere instead of
   journald and os_log first. Breaking for users who rely on the
   journal.
3. **Formats to cut.** WP-C5 proposes removing "ELF" and "W3C" and
   reworking CLF and Apache.
4. **A second maintainer.** The only route past the section 2
   ceilings (Code-Review, Branch-Protection, Contributors, Best
   Practices Gold).
5. **`--root` default for rlg-mcp.** Required immediately, or the
   current directory with a warning for one release.

## 6. Definition of done for 10/10

rlg rates 10/10 in a category when:

- every condition in its section 1 bar holds on `main`;
- its gate is a required status check;
- the last two releases kept it (no regression waivers);
- its claims in the docs point to the test or benchmark that proves
  them.

The plan is complete when all ten categories meet that bar, and the
section 2 ceilings are either lifted or still recorded with their
reason.
