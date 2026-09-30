<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Benchmarks

What the benchmarks measure, how to run them, and what the numbers do
and do not mean.

## What is measured

`crates/rlg/benches/competitive_bench.rs` times the cost to the
**calling thread** of emitting a record, in four groups:

| Group | Each contender emits |
| :--- | :--- |
| Simple Emission | one record with a string message |
| Structured Emission | one record with three key-value attributes |
| Burst 10k | 10,000 records in a row |
| Latency Distribution | one record, sampled for its spread |

The three contenders do different work, and the numbers only make sense
with that in mind:

- **rlg `fire()`** checks the level and pushes the record into the ring
  buffer. Formatting and I/O happen later on the flusher thread and are
  **not** in the timed path. That is the design being measured.
- **`tracing::info!`** runs through a `tracing_subscriber::fmt`
  subscriber that formats the event on the calling thread and writes it
  to `std::io::sink`.
- **`log::info!`** goes to a logger that does nothing: no formatting, no
  I/O. It is the floor, the cost of the facade alone.

So rlg against `tracing` compares "enqueue and return" with "format on
this thread", and neither is comparable to the `log` floor.

## Running them

```bash
cargo bench -p rlg --bench competitive_bench      # this suite
cargo bench --workspace                           # every crate's benches
```

Numbers from a laptop swing by tens of percent between runs; compare
results from the same machine, back to back.

## Published results

`bench-publish.yml` runs every workspace benchmark on a GitHub-hosted
`ubuntu-latest` runner for each release tag, and uploads the Criterion
report and a JSON summary as workflow artifacts (kept 90 days). Until
0.0.13 that workflow ran no benchmarks at all: its output directory did
not exist, and the failure was swallowed.

### 0.0.13 (release branch)

[Run 36773354952](https://github.com/sebastienrousseau/rlg/actions/runs/36773354952),
GitHub-hosted `ubuntu-latest`, stable Rust, release profile. Typical
time per iteration with Criterion's 95% confidence interval.

| Scenario | rlg `fire()` | `tracing::info!` | `log::info!` (no-op) |
| :--- | ---: | ---: | ---: |
| Simple Emission | 848 ns (833–864) | 353 ns (351–355) | 1.7 ns |
| Structured Emission, 3 attributes | 1,145 ns (1,128–1,162) | 676 ns (672–679) | 1.9 ns |
| Burst of 10,000 records | 9.44 ms (9.18–9.83) | 4.00 ms (3.98–4.02) | 0.02 ms |
| Latency Distribution | 793 ns (787–800) | 333 ns (332–333) | 1.5 ns |

**Reading these honestly.** On the calling thread, rlg's `fire()` costs
about 2.4 times what `tracing::info!` costs, even though `tracing`
formats the event there and rlg does not. rlg's advantage is not a
cheaper call; it is that the caller never waits on a sink's I/O, which
this suite's discarding writer does not exercise. The per-record cost
has not been profiled yet; the likely candidates are the flusher
wake-up (`unpark`) on every `fire()` and the per-record metrics
counters.
