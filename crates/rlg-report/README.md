<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/rlg/v1/logos/rlg.svg" alt="RLG logo" width="128" />
</p>

<h1 align="center">rlg-report</h1>

<p align="center">
  Log digest / analytics for <code>rlg</code> streams. Count by level,
  group by component, rank top descriptions, compute latency
  percentiles.
</p>

<p align="center">
  <a href="https://crates.io/crates/rlg-report"><img src="https://img.shields.io/crates/v/rlg-report.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/rlg-report"><img src="https://img.shields.io/badge/docs.rs-rlg--report-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
</p>

---

## Install

```bash
cargo install rlg-report
```

## CLI

```bash
# Pretty-printed digest of a log file.
rlg-report /var/log/app.ndjson

# JSON output for a dashboard.
rlg-report --format json /var/log/app.ndjson

# Read from stdin (e.g. piped from `rlg`).
my-service | rlg --format ndjson | rlg-report

# Keep only the top 3 description buckets.
rlg-report --top 3 /var/log/app.ndjson
```

### Sample output

```
── rlg report ───────────────────────────────────────────
total records:      18421
unparseable lines:  0

── by level ─────────────────
  INFO       16842
  WARN       912
  ERROR      640
  FATAL      27

── by component ─────────────
  api        12001
  db         3920
  orchestrator 2500

── top descriptions ─────────
   1240  GET /v1/users -> 200
    320  POST /v1/orders -> 201
    274  user authenticated
    ...

── latency (ms) ─────────────
  samples  16980
  p50      14
  p95      82
  p99      210
  max      4012
```

## Library mode

```rust
use rlg_report::Report;

let lines: Vec<&str> = my_log_lines();
let report = Report::from_lines(lines.into_iter());

println!("errors-and-above: {}", report.error_count());
if let Some(latency) = report.latency {
    println!("p99 latency: {} ms", latency.p99);
}
```

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.
