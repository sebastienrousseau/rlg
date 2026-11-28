# Changelog

All notable changes to this project are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- The release workflow's "already published" check now reads the
  crates.io sparse index. The API call it used failed on the runner, so
  the v0.0.23 run tried to republish `rlg-core` and stopped.

## [0.0.23] - 2026-11-28

rlg's record formatting now builds without `std`. Nothing in the public
API or in any format's output changes.

### Added

- **`rlg-core`**, a new `#![no_std]` + `alloc` crate and the eleventh
  lockstep member: `LogLevel` and `ParseLogLevelError`, a borrowed
  `Record` view with a const builder, and `Format`, which renders a
  record into each of the 14 formats through `core::fmt::Write`. It
  depends only on `serde` and `serde_json` with default features off,
  and CI builds it for `thumbv7em-none-eabihf` on stable and 1.89.0
  (`no-std.yml`). The engine, sinks, config and `LogFormat` stay in
  rlg, because they need `std`.
- CI checks spelling (codespell 2.4.3) and REUSE compliance (reuse
  6.3.0; `REUSE.toml` makes every file's licence machine-readable), and
  `features.yml` builds each feature alone on every push and every
  feature combination weekly (cargo-hack 0.6.46). The maintainer
  approved the three tools by name; `docs/DEPENDENCIES.md` records it.

### Changed

- rlg re-exports `LogLevel` and `ParseLogLevelError` from `rlg-core` at
  their old paths (the same types), and `Log`'s `Display` renders
  through `rlg_core::Record`; the format tests pass unchanged.
- The breaking milestone moves to 0.0.24, and each later milestone one
  release on (`docs/IMPLEMENTATION-PLAN.md`).
- Dependencies (Dependabot #256, #257): clap 4.6.10; new digests for the
  `rust:1.100-bookworm` build images and the dev container base, now
  tag 2.1.

## [0.0.22] - 2026-11-28

A documentation and standards release, in lockstep with the website.
Nothing in the public API or in any format's output changes.

### Added

- The website, [rustlogs.github.io](https://github.com/sebastienrousseau/rustlogs.github.io)
  (<https://rustlogs.com/>), joins the lockstep family: it ships at
  0.0.22 with the crates, and the README's ecosystem table lists it.
  `scripts/check-readme.py` now fails when that table does not list
  exactly the publishable crates and the website.
- A dependency baseline (standards 0.0.16, `AGENTS.md` section 11):
  `scripts/dependency-gate.py`, run by `make verify` and CI, holds the
  third-party crates in `Cargo.lock`, the actions, the container images
  and the installed tools to `scripts/dependency-baseline.json`, which
  may only shrink, and requires a reason for every direct crate in
  `docs/DEPENDENCIES.md`.
- `DEPRECATIONS.md` and a deprecation policy in `docs/POLICIES.md`: a
  removal or contract change is deprecated first and keeps working,
  with a warning, for at least 90 days and one release.
- Fuzz smoke runs on every pull request and push to `main` for all five
  targets (`render_one_line` was missing), replaying the committed
  corpus before fuzzing. In 0.0.21 the workflow ran only by hand,
  although the CHANGELOG said `render_one_line` ran in CI.

### Changed

- **Deprecated:** running `rlg-mcp` with neither `--root` nor
  `RLG_MCP_ROOT`. It keeps working with a warning until the first
  release on or after 2027-02-26, not 0.0.22 as 0.0.21 announced; the
  warning names the date.
- The planned breaking milestone moves to 0.0.23 and every later
  milestone one release on (`docs/IMPLEMENTATION-PLAN.md`).
- `docs/POLICIES.md` no longer says there is no deprecation window
  before 0.1.0. `AGENTS.md` adds the no-new-dependency rule and the
  four principles.
- The dev container's base image is pinned by digest and kept current
  by Dependabot; Pages deploys once per push instead of twice;
  `cargo semver-checks` covers `rlg-ebpf`; the link check covers the
  crate READMEs; `check-doc-versions.sh` covers the WIT package and
  `CITATION.cff`.
- crates.io keywords: `logging`, `structured-logging`, `tracing`,
  `opentelemetry`, `observability`.

### Fixed

- Documentation that contradicted the code: the rlg-mcp README (four
  tools with `tail_logs_glob`, the prompt, two resources, stdio,
  streamable HTTP and HTTP+SSE), "everything optional is off by
  default" (`config` and rlg-otlp's `blocking` are on), the four-target
  fuzz claim, the list of formats in the manual, versions stuck at
  0.0.11 in crate READMEs and the WIT package, links to a missing
  `doc/INPUT-FORMATS.md`, the README demo GIF on crates.io (now an
  absolute URL), and `SECURITY.md`'s SBOM check, which used the
  `.sig`/`.crt` form releases stopped shipping after 0.0.14.
- rlg-tower no longer lists `tower` as a normal dependency; its library
  uses only `tower-layer` and `tower-service` (`tower` stays for tests).

## [0.0.21] - 2026-11-28

A security release. Nothing in the public API changes; the behaviour
changes are listed under Security.

### Security

- **rlg-mcp reads log files only under a root.** The tools and the
  `rlg://tail/{path}` resource opened any path or glob the client
  sent, so a client, or a model steered by text it was shown, could
  read any file the server could, and tell from the error whether a
  file existed. Paths now resolve under `--root <dir>` (or
  `RLG_MCP_ROOT`; the current directory with a warning when neither
  is set): anything outside, through `..`, an absolute path or a
  symlink, or not a regular file, gets one refusal message. Reads are
  bounded (64 MiB per file, lines over 1 MiB skipped, 10,000 records
  per answer, 1,000 glob matches), and `/**/*` is refused before any
  walk. A non-loopback `--host` prints a warning. **Behaviour
  change:** an absolute path outside the current directory is refused
  unless `--root` covers it. `--root` becomes required in 0.0.22.
- **No field can forge a record.** The text formats (CLF, CEF, ELF,
  W3C, Apache, Log4j XML, logfmt) wrote the message, component and
  time verbatim, so a line break started a forged record and `|`,
  `<` or `"` broke the format. Every text format now escapes control
  characters and its own delimiters; logfmt keys are percent-encoded
  and values quoted when needed. JSON attribute values escape DEL and
  C1 controls, which `serde_json` left raw. Output that held such
  characters reads differently.
- **No payload can forge a journal field.** journald datagrams use
  the binary field form for values with line breaks, no longer carry
  a trailing empty line in `MESSAGE`, and set `SYSLOG_IDENTIFIER` to
  the program name. The journald sink no longer drops records when
  `RLG_FALLBACK_STDOUT` or `GITHUB_ACTIONS` is set after start-up.
- **OTLP endpoints are validated.** Control characters, spaces and
  non-ASCII in an endpoint URL are refused instead of reaching the
  request line. DNS resolution runs within the request deadline.
  `OTEL_EXPORTER_OTLP_[TRACES_]HEADERS` values are percent-decoded as
  the spec requires; a malformed pair now fails
  `SpanExporterBuilder::build` instead of being dropped silently.
- **Log files are private.** Files are created `0640` and directories
  `0750` (before the umask) on Unix. Rotation no longer overwrites an
  archive when two rotations fall in the same second.
- **Secrets stay out of logs.** `Config`'s `Debug` and `Config::diff`
  no longer print `env_vars` values. rlg-tower logs a trace header
  only when it is a `traceparent` or a plain id of up to 128 safe
  characters.

### Changed

- **The rlg-mcp image is signed.** It is built with provenance and an
  SBOM and signed keyless with cosign; `SECURITY.md` has the verify
  command. The publish job runs only for a release tag, and
  `mcp-publisher` is pinned by version and checksum.
- **Build inputs.** A `.dockerignore` keeps `.git`, build output and
  local secrets out of the image build context. `cargo deny` fails on
  yanked versions and on unmaintained crates anywhere in the tree.
- **Docs.** `docs/THREAT-MODEL.md` names every trust boundary;
  `SECURITY.md` lists every `unsafe` site and names the supported
  version, which `scripts/check-doc-versions.sh` now checks. The new
  `render_one_line` fuzz target runs in CI.

## [0.0.20] - 2026-11-27

### Added

- **W3C `traceparent` adoption.** A root span created with a valid
  `traceparent` field joins that trace, with the remote span as its
  parent, so traces continue across services. A malformed value
  starts a new trace.
- **`rlg::engine::set_sink`** chooses the engine's sink in code
  (stdout, a file, journald, `os_log`) before the first record, in
  place of `PlatformSink::native()`. No environment variable is
  needed, so it is safe once an async runtime's threads are running.
- **A default-on `config` feature.** File and environment loading
  (`Config::load`, `load_async`, `save_to_file`, `set`, hot-reload,
  `TryFrom<env::Vars>`) and the `ConfigError` variants wrapping the
  `config` and `envy` errors are behind it. With
  `default-features = false, features = ["tracing-layer"]`, rlg
  builds on 33 crates instead of 47 (no config, toml, envy,
  parking_lot or their dependencies). rlg-otlp, rlg-redact,
  rlg-tower, rlg-test, rlg-wasm and rlg-ebpf depend on rlg without
  default features, so they do not pull it back in.

### Fixed

- **`caller` names the event's call site.** Records from `RlgLayer`
  carried the layer's own `layer.rs` line; they now carry the
  event's file and line. Every `caller` path that is absolute (a
  crate built from the registry) is cut to start at the crate
  directory, so build machines' home directories no longer reach
  log lines.

### Changed

- **Breaking for `default-features = false` users:** config loading
  now needs the `config` feature. Default builds are unchanged.

### Security

- **rlg-mcp's SSE transport caps a posted message at 1 MiB.** A
  `/messages/` body over 1 MiB is refused with 413 before it is
  deserialised; it was bounded only by axum's implicit 2 MiB default.
  Clears CodeQL alert #81 (`rust/unsafe-deserialization`).

### Dependencies

- The GitHub Actions group, the minor-and-patch group and
  euxis-commons 0.0.9 (Dependabot #248, #249, #250).

## [0.0.19] - 2026-11-27

### Added

- **`RlgLayer` carries span context.** Every record emitted inside
  `tracing` spans carries the fields of each span in scope (root
  first; an inner span wins a clash and the event's own fields win
  over all), fields filled in later with `Span::record`, the
  innermost span's name (`span`), the path from the root
  (`span_path`), and a `trace_id` and `span_id`.
- **`RlgLayer::with_scrub`** runs every record, and every closed
  span's attributes, through an `rlg::tracing::Scrub` before it
  leaves the layer.
- **`RlgLayer::with_span_sink`** hands every closed span to an
  `rlg::tracing::SpanSink` as a `SpanRecord` (ids, parent, name,
  times, attributes, kind, status). `otel.kind`, `otel.status_code`
  and `otel.status_message` fields set kind and status; an `ERROR`
  event marks its span as failed.
- **rlg-otlp exports spans** (`tracing` feature): `SpanExporter`
  posts closed spans to a Collector's `/v1/traces` as OTLP/HTTP JSON
  in batches from a background thread, without the `opentelemetry`
  crates. `SpanExporter::from_env()` reads the standard
  `OTEL_EXPORTER_OTLP_*` variables and `OTEL_SERVICE_NAME`.
- **rlg-redact scrubs the `tracing` path** (`tracing` feature):
  `Redactor` implements `Scrub`, so
  `RlgLayer::with_scrub(Redactor::with_defaults())` redacts records
  and span attributes.

### Changed

- **Breaking: `RlgLayer` is no longer `Copy`** (it holds the scrub
  and span-sink hooks). It is still `Clone`.

### Security

- **rmcp 3.12 is required** (was 3.4), clearing RUSTSEC-2026-0462
  (unbounded response bodies in rmcp's Streamable HTTP client).
  rmcp 3.12 answers a malformed Streamable HTTP body with 400
  instead of 415.
- **chrono is gone from the tree** (RUSTSEC-2026-0543, unmaintained).
  rlg-mcp turns off rmcp's default features, which pulled it in, and
  `deny.toml` now bans it.
- **rustix 1.1.6** (RUSTSEC-2026-0486). The `tui` feature requires
  it directly as a version floor, since `terminal_size` pulls rustix
  into users' trees.

### Dependencies

- The GitHub Actions group, the Docker base images and the
  minor-and-patch group (Dependabot #244, #245, #246).
- parking_lot stays at 0.12 (Dependabot #239 is not taken): 0.13
  needs rustc 1.95, above the 1.89 MSRV. Dependabot now skips it.

## [0.0.18] - 2026-11-06

### Changed

- **Breaking: `rlg::commons` is removed.** rlg re-exported the whole of
  euxis-commons as `rlg::commons`, which would have made the modules
  euxis-commons 0.0.8 deprecates part of rlg's API. Code that used it
  depends on `euxis-commons` directly instead.

### Fixed

- **rlg-mcp's transports are hardened** (#236). Both HTTP transports
  refuse a Host the server does not answer to and a non-local Origin
  (403); `/messages/` accepts only `application/json` (415);
  `--max-sessions` (64 by default) caps open sessions (503 past it);
  and a stdin line past 16 MiB ends the session instead of being
  buffered until the pipe closes.
- **Release pages introduce their checksums** (#235).
  `scripts/compose-release-notes.sh` prints "SHA-256 of every asset
  attached to this release:" above the checksums block; the 0.0.16
  and 0.0.17 pages were published without it.

### Dependencies

- **euxis-commons 0.0.8 is required** (was 0.0.7), with only the
  error, config, validation, id and counter features rlg uses. The
  retry, time and env features are no longer enabled.

## [0.0.17] - 2026-11-06

### Dependencies

- **euxis-commons 0.0.7 is required** (was 0.0.5). It moves to toml
  1.0, so builds compile one copy of `toml`, `serde_spanned`,
  `toml_datetime` and `winnow` instead of two, and the four
  `deny.toml` skips that hid the duplicates are removed.
- The GitHub Actions group, the Docker base images, the
  minor-and-patch group and euxis-commons 0.0.5 (Dependabot #225,
  #227, #230, #231, #232).

## [0.0.16] - 2026-11-01

### Changed

- **Breaking: the MSRV is now Rust 1.89.0** (was 1.88.0) for every
  crate, one floor shared with the maintainer's other Rust crates. The
  manifests, `build.rs`, `flake.nix`, the MSRV CI job, the READMEs and
  the policy and packaging docs state it.
- **`scripts/check-doc-versions.sh` checks the MSRV.** Every crate
  manifest and every statement of the floor must match `rust-version`.
- **Install snippets name 0.0.15.** Four in the guides still said
  0.0.14, which kept the docs check red on `main`.

### Fixed

- **The `rlg` API docs at doc.rustlogs.com lost most of their pages.**
  The `rlg` binary in `rlg-cli` and the `rlg` library both documented
  into `rlg/` (cargo#6313), so the binary's pages replaced the
  library's: `rlg/index.html` linked items that returned 404, and the
  CLI's private `LevelArg` was published as library API. The binary is
  now `doc = false`.
- **Search engines indexed hundreds of duplicate docs URLs.** rustdoc
  pages had no canonical link and its search box creates `?search=`
  variants of every page. The Pages deploy now runs
  `scripts/docs-seo.py`, which adds a canonical link to every page,
  marks `help.html` and `settings.html` noindex, writes `sitemap.xml`
  from the pages the build ships, and writes `robots.txt` that blocks
  `?search=` URLs and `src/`. CI runs it with `--check` on every pull
  request. The internal `xtask` crate is no longer published.

### Dependencies

- **tokio 1.54 is required** by every crate, so no release affected
  by RUSTSEC-2026-0386 can resolve; the lock is at 1.54.1.
- The GitHub Actions group and the Rust 1.99 Docker base images
  (Dependabot #221, #222, #227, #228).
- The minor-and-patch group (tokio 1.53.2, toml 1.1.8, libc 0.2.190,
  rmcp 3.5.1, tokio-util 0.7.20, uuid 1.28.0), Dependabot #220. uuid
  1.28 requires rustc 1.89, the new floor. `cargo vet` needs the
  maintainer's decisions for these and for config 0.15.27, unvetted
  on `main`: `.git/vet-v0.0.16.sh` asks for each.

## [0.0.15] - 2026-10-11

### Changed

- Bumped `config` from 0.15.26 to 0.15.27 in the minor-and-patch group.
- Bumped `anchore/sbom-action` from 0.24.2 to 0.24.3 in the release workflow.
- Updated Docker builds to `rust:1.99-bookworm`.
- Release SBOMs are signed with cosign v3 and ship one Sigstore bundle
  per SBOM (`sbom.<fmt>.json.sigstore.json`) in place of the separate
  `.sig` and `.crt` files. Verify with `cosign verify-blob --bundle`;
  `pkg/VERIFY.md` keeps the steps for releases up to v0.0.14, and
  `make verify-release` handles both layouts.

### Fixed

- `make verify-release` reported success when the first SBOM failed
  verification and the second passed; it now stops at the first
  failure.

## [0.0.14] - 2026-08-27

The **follow-through** cut: what releasing 0.0.13 exposed. A
memory-safety bug in the macOS sink is fixed, `fire()` costs less than
half as much under contention, the blocking OTLP exporter drops `ureq`,
the documentation site deploys again, and release pages stop carrying
unsigned duplicate SBOMs.

Workspace-lockstep versioning: all 10 publishable crates are at
`0.0.14`. `xtask` stays at `0.0.0`.

### Changed

- Producers no longer write to the flusher's park state on every
  record: the flusher raises an idle flag before parking and only the
  first producer to see it wakes the thread. With eight threads logging
  at once, `fire()` costs about 54% less per thread (3.9 to 1.8 µs on
  the development machine; 3.0 to 0.95 µs with `fast-queue`), and a
  record fired at an idle engine is still written in about 57 µs. A new
  `Contended Emission (8 threads)` benchmark group tracks it.
- The event, level, error and format counters in `TuiMetrics` are now
  updated by the flusher as it drains each event (and on the eviction
  path for dropped ones), not by every producer in `ingest` and
  `fire()`: the counters have a single writer in the common case, and
  producers no longer contend on their cache line. Counts now trail
  `ingest` by at most one flush, and events below the level filter no
  longer reach the format counters. With eight threads, `fire()` is
  about 10% cheaper per thread (median 1.93 to 1.74 µs on a loaded
  machine). The public types and methods are unchanged.
- **Breaking**: `fire()` no longer builds the `caller` attribute on
  the calling thread. It carries the `&'static Location` in a new
  `LogEvent::caller` field, and the flusher renders `file:line` into
  the attribute before formatting, so records look exactly as before.
  That string and its map insert were about 100 ns of every `fire()`.
  With eight threads, `fire()` costs about 38% less per thread (median
  2.55 to 1.59 µs, faster in 10 of 10 interleaved runs). Code that
  builds a `LogEvent` literal needs `caller: None`, or the new
  `LogEvent::new(log)`. `Log::with` keeps converting values eagerly:
  measured, the `serde_json` conversion is 1–7 ns of the ~45 ns an
  attribute costs, and deferring it would need `T: 'static` plus a
  boxed copy.
- **Breaking** (`rlg-otlp`): the blocking `OtlpExporter` now uses the
  crate's own HTTP/1.1 client over `std::net`, like the async exporter,
  and `ureq` is gone: 38 fewer crates in the lockfile and 34 fewer
  cargo-vet exemptions. `OtlpError::Transport` wraps `std::io::Error`
  (`TimedOut` for a deadline, `InvalidData` for a non-HTTP answer). An
  `https://` endpoint or an unsafe header is reported on export as
  `InvalidEndpoint` or `InvalidHeader` before any request is sent.

### Fixed

- **Memory safety** (macOS): the `os_log` sink declared `syslog(3)` with
  a fixed third parameter, but the C function is variadic. On Apple
  Silicon, variadic arguments are passed on the stack, so `syslog` read
  a stale stack word as the message pointer: the flusher could crash in
  `strlen`, or log bytes from an arbitrary address. The declaration is
  now variadic. A release build firing 20,000 records crashed 3 runs in
  3 on the previous commit and runs clean with the fix. Present since
  0.0.9; CI never reached this path because `GITHUB_ACTIONS` routes the
  sink to stdout.
- `TuiMetrics::dropped_events` under-counted under contention. When the
  ring buffer was full, `ingest` counted one drop and then evicted up to
  three events uncounted; with eight threads on a full queue, 47% of the
  lost events went unrecorded. Each eviction that removes an event, and
  a new event that loses every retry, is now counted exactly once.
- The documentation site stopped deploying at 0.0.13: today's nightly
  rustdoc rejects module links that repeat a path its label already
  resolves, while stable cannot resolve the short form, so six links in
  `config`, `engine` and `rotation` now carry descriptive labels that
  both accept. CI builds the docs the way docs.rs and Pages do on every
  pull request.
- The config hot-reload poller read file metadata with a blocking call
  on a Tokio worker; it now uses `tokio::fs`.
- Releases attached unsigned duplicate SBOMs next to the signed ones;
  only the signed files are uploaded now.
- An MCP HTTP test could connect to another test's subprocess when both
  were handed the same port; test servers now use ports outside the
  ephemeral range.

## [0.0.13] — unreleased

The **smaller-tree** cut. rlg-mcp moves onto the official MCP SDK and
serves stdio, streamable HTTP and the older HTTP+SSE transport; the
optional dependencies that failed `cargo deny --all-features` (miette,
notify, reqwest, tonic) are replaced by in-house code or removed, taking
107 crates out of the lockfile; and the repository gains the gates that
keep it that way: an enforcing cargo-deny job over all features, a
complexity baseline, and a check that install snippets name the shipped
version.

Workspace-lockstep versioning: all 10 publishable crates are at
`0.0.13`. `xtask` stays at `0.0.0`.

This is the first `0.0.13` on crates.io. The number was used briefly
for an internal dependency batch before 0.0.12 (see the note there),
but nothing was ever tagged or published under it.

### Added

- `rlg --completions <SHELL>` and `rlg-report --completions <SHELL>`
  print shell completions (bash, zsh, fish, elvish, PowerShell)
  generated from the CLI definition; `make completions` writes them all.
- `--manpage` on both binaries prints a section-1 manual page generated
  from the CLI definition, and a `GNUmakefile` adds `make install` /
  `make uninstall` (honouring `PREFIX` and `DESTDIR`) for the binaries,
  manpages and completions.
- A user manual built with mdBook from `docs/` and published at
  <https://doc.rustlogs.com/manual/>, with `ARCHITECTURE.md`,
  `POLICIES.md`, `packaging.md`, `COMPARISON.md` and `BENCHMARKS.md`.
- CI gates: the 1.88.0 MSRV build, markdownlint, an offline link check
  of the manual and README, the README template, and OpenSSF Scorecard.
- `rlg-mcp` runs on the official MCP SDK (`rmcp`) and serves stdio (the
  default), streamable HTTP (`--transport streamable-http`) or the older
  HTTP+SSE transport (`--transport sse`), covering protocol revisions
  2024-11-05 through 2026-07-28.
- `RlgError::code`, `RlgError::help` and `RlgError::report`: stable error
  codes (`rlg::io_error`, …), a resolution hint, and a multi-line report,
  with no extra dependency.
- `rlg_otlp::DEFAULT_ENDPOINT` (`http://localhost:4318/v1/logs`), and
  `OtlpError::InvalidEndpoint` / `OtlpError::InvalidHeader`.
- CI runs `cargo deny --all-features check` as a failing gate. The shared
  security workflow's run is `continue-on-error`, so policy violations
  had gone unreported.

### Changed

- `fire()` is about 40% cheaper on the calling thread: the timestamp and
  the `caller` attribute are no longer formatted through `format!`.
  Output is byte-identical. In CI it went from 2.4x to about 1.0x the
  cost of `tracing::info!` measured in the same run.
- `rlg-otlp` sends plain OTLP/HTTP to a local OpenTelemetry Collector,
  which owns TLS and credentials towards the backend
  ([ADR 0015](docs/adr/0015-otlp-local-collector-transport.md)).
  `AsyncOtlpExporter` now runs on an in-house HTTP/1.1 client over Tokio
  instead of `reqwest`, and rejects `https://` endpoints and unsafe
  headers at `build()`. Both builders default to `DEFAULT_ENDPOINT`
  instead of panicking when no endpoint is set. **Breaking:**
  `OtlpError::AsyncTransport` wraps `std::io::Error`. Deployments that
  exported straight to a SaaS `https://` endpoint must add a Collector;
  the crate docs carry a minimal configuration.
- `Config::hot_reload_async` polls the file (`HOT_RELOAD_POLL_INTERVAL`,
  250 ms) instead of using `notify`, and now also picks up editor-style
  saves that rename a new file over the old one. **Breaking:**
  `ConfigError::WatcherError` wraps `std::io::Error`.
- rlg builds `config` with the TOML format only; YAML, JSON5, RON and INI
  parsers it never used are no longer compiled.

### Removed

- The `rlg-otlp` `grpc` feature, `GrpcOtlpExporter` and its error
  variants. It was a scaffold whose send path returned
  `GrpcNotImplemented`; collectors accept OTLP/HTTP on port 4318.
- The `rlg` `miette` feature; use `RlgError::code`, `help` and `report`.
- `reqwest`, `tonic`, `prost`, `rustls`, `ring`, `webpki-roots`,
  `miette` and `notify` from the dependency tree (82 crates in all).

### Fixed

- `rlg` did not compile on Windows: the stand-in for the Unix socket type
  lacked the documentation the crate requires. It builds again, and CI now
  checks every library and binary on Windows.
- The configuration example in the `rlg` README did not load
  (`LogRotation` and `LoggingDestination` use `{ Size = N }` and
  `{ type = "File", value = ... }`); it does now, and a test loads it.
- `bench-publish.yml` never ran a benchmark: its output directory did
  not exist and the error was swallowed. Release benchmarks now run and
  publish.
- The `rlg` README and crate docs claimed `fire()` takes ~1.4 µs against
  ~20 µs for mainstream loggers. Measured in CI it is ~0.85 µs, and
  `tracing` formatting to a discarding writer is ~0.35 µs; the claims
  are replaced by the published numbers.
- `.github/SECURITY.md`, the copy GitHub shows first, was a template
  with no reporting channel; the real policy is now the one shown.
- `PKGBUILD` said 0.0.7 and `debian/debcargo.toml` named a feature that
  does not exist; both are fixed and the `PKGBUILD` version is CI-checked.
- Install snippets in thirteen places (every crate README, the getting
  started tutorial, the introduction and the tracing migration guide)
  named 0.0.11 or 0.0.7. They name 0.0.12, and
  `scripts/check-doc-versions.sh` fails CI when a snippet drifts from
  `crates/rlg/Cargo.toml` again.
- The blocking OTLP exporter retried 4xx responses and reported every
  failing status as a transport error: ureq returned non-2xx as errors,
  so the status handling never ran. A 4xx is now final and statuses are
  reported as `OtlpError::BadStatus`.
- `glama.json` and `server.json` named 0.0.11 while the workspace shipped
  0.0.12, so the Glama listing and the registry's install command pointed
  at the previous image. Both are stamped, the README's lockstep line with
  them, and `scripts/check-mcp-manifests.sh` now fails CI when they drift
  from `crates/rlg-mcp/Cargo.toml` again.

## [0.0.12] — 2026-08-26

The **portability and supply-chain** cut. Fixes a build failure on
targets without 64-bit atomics, moves publishing to crates.io Trusted
Publishing, and folds in eight weeks of dependency and MCP work.

Workspace-lockstep versioning: all 10 publishable crates are at
`0.0.12` (`rlg`, `rlg-cli`, `rlg-ebpf`, `rlg-mcp`, `rlg-otlp`,
`rlg-redact`, `rlg-report`, `rlg-test`, `rlg-tower`, `rlg-wasm`).
`xtask` stays at `0.0.0` per workspace convention.

**A note on version numbering.** The manifests briefly carried `0.0.13`
and `0.0.14` while dependency batches were consolidated under those
names, but neither was ever tagged or published — crates.io went
straight from `0.0.11` to here. Rather than publish two versions that
predate the portability fix below and would be permanently broken on
32-bit targets, the manifests were renumbered back to `0.0.12`, which
is the version this CHANGELOG and the README already advertised. There
is no `0.0.13` or `0.0.14`, and there never was one to install.

### Fixed

- **Builds on targets without 64-bit atomics.** `SPAN_ID_COUNTER` in
  `tracing.rs` and `SESSION_COUNTER` in `log.rs` used `AtomicU64`,
  which does not exist on `powerpc-unknown-linux-gnu` and similar
  targets — the import failed outright with `E0432`. Both now use
  `euxis_commons::counter::Counter`, which selects a lock-free or
  mutex-backed implementation behind
  `#[cfg(target_has_atomic = "64")]`. Verified with
  `cargo check -p rlg --target powerpc-unknown-linux-gnu`.
- **RUSTSEC-2026-0204** patched, alongside clippy 1.97 lints and TUI
  test feature gates.
- **`rustdoc::redundant_explicit_links`** failures that were breaking
  the GitHub Pages documentation build.
- **`publish-mcp`** no longer injects `packages[0].version` at publish
  time.
- **`docsrs` `doc(cfg)`** failure in the documentation build.

### Changed

- **Publishing uses crates.io Trusted Publishing.** Releases
  authenticate over OIDC rather than a stored `CARGO_REGISTRY_TOKEN`,
  so there is no long-lived registry credential in the repository. The
  secret has been removed.
- **Releases can be re-run without moving a tag.** The release
  workflow accepts a `workflow_dispatch` with the tag to publish, after
  a GitHub Actions incident left a tag-triggered run unrecoverable —
  it reported as queued, completed and running at once and refused both
  cancel and rerun. All three jobs pin their checkout to the supplied
  tag, so a dispatch cannot package the default branch under a tag's
  version.
- **Dependabot** groups GitHub Actions updates into a single pull
  request.

### Added — MCP

- **Prompts and resources** for MCP Trinity parity.
- **`tail_logs_glob`** — a multi-file glob log tailer.
- **`glama.json`**, tool titles, MCP annotations and usage guidance.
- **Dockerfile** so Glama can build a release image.
- **CNAME written into the Pages artifact**, so the custom
  documentation domain survives each deploy.

### Dependencies

Eight weeks of consolidated updates, including `serial_test` 3.5 → 4.0,
`actions/setup-node` 6 → 7, `actions/configure-pages` 5 → 6, several
`minor-and-patch` group batches, and `euxis-commons` 0.0.2 → 0.0.4
(which carries the portable counter above). cargo-vet exemptions were
moved to match.

## [v0.0.11] - 2026-07-02

The **MCP-discoverability** cut for `rlg-mcp`. Registers `rlg-mcp`
with the official Model Context Protocol Registry (via OCI
packaging), adds MCP-spec conformance CI, ships a Glama directory
manifest, and cross-links sibling developer-tools MCP servers.

Workspace-lockstep versioning: all 9 publishable crates bump from
`0.0.10` → `0.0.11` (`rlg`, `rlg-cli`, `rlg-mcp`, `rlg-otlp`,
`rlg-redact`, `rlg-report`, `rlg-test`, `rlg-tower`, `rlg-wasm`).
`xtask` stays at `0.0.0` per workspace convention. This matches the
release workflow's "tag matches every publishable crate" check.
Only `rlg-mcp` has substantive changes in this cut; the other 8
crates ship no code changes, so existing consumers can upgrade
without any migration.

### Added — MCP registry work (rlg-mcp)

- **Official MCP Registry integration.** `rlg-mcp` is now registered
  with the official Model Context Protocol Registry
  (`registry.modelcontextprotocol.io`) as
  `io.github.sebastienrousseau/rlg-mcp`. A new `server.json` at the
  repo root provides the registry metadata using `registryType: oci`
  (the OCI image at `ghcr.io/sebastienrousseau/rlg-mcp` is the
  package artefact — crates.io is not a registry-supported
  `registryType`). `crates/rlg-mcp/README.md` carries an
  `mcp-name: io.github.sebastienrousseau/rlg-mcp` marker used by the
  registry for OCI ownership verification.
- **Auto-publish workflow** (`.github/workflows/publish-mcp.yml`) —
  on every `v*.*.*` tag push:
  1. Builds and pushes the OCI image (via the new
     `pkg/docker/Dockerfile.mcp` — Rust 1.88 builder, distroless-cc
     runtime, non-root user) to GHCR.
  2. Authenticates to the MCP Registry via GitHub OIDC (no secrets
     required), syncs the tag version into `server.json`, and runs
     `mcp-publisher publish`.
- **Protocol conformance CI** (`.github/workflows/mcp-inspect.yml`) —
  builds `rlg-mcp` release binary, then runs
  `@modelcontextprotocol/inspector --cli` against `tools/list`.
  Path-filtered to `crates/rlg-mcp/**`, `crates/rlg/**`, and
  `crates/rlg-cli/**` to keep the CI budget bounded.
- **Docker packaging** (`pkg/docker/Dockerfile.mcp`) — multi-stage
  build, distroless runtime, non-root user, reproducible via
  `SOURCE_DATE_EPOCH` and `--remap-path-prefix`.
- **Glama directory manifest** (`glama.json`) — Glama listing under
  the `developer-tools` category with OCI runtime spec.
- **Suite discoverability.** `crates/rlg-mcp/README.md` now cross-
  links sibling MCP servers — `noyalib-mcp` as a fellow developer-
  tools server, and the four ISO 20022 banking MCP servers
  (`pain001-mcp`, `bankstatementparser-mcp`, `camt053-mcp`,
  `acmt001-mcp`) as author-portfolio siblings.

### Changed

- GitHub repository description and topics — description will be
  refreshed to mention the MCP server; topics will gain `mcp-server`,
  `mcp`, `model-context-protocol`, `observability`, `sre`,
  `claude`, `claude-desktop`, and `ai-agents` (previously empty).

### No functional / API changes to non-MCP crates

- Only `rlg-mcp` has substantive changes (the MCP registry work
  above). The other 8 publishable crates (`rlg`, `rlg-cli`,
  `rlg-otlp`, `rlg-redact`, `rlg-report`, `rlg-test`, `rlg-tower`,
  `rlg-wasm`) bump to `0.0.11` as part of the workspace-lockstep
  cut but ship no code changes — existing consumers can upgrade
  without any migration.
