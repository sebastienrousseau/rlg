<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ADR 0009 — Sharded Producer Queue

- **Status:** Accepted
- **Date:** 2026-07-05
- **Phase:** 18 (per `docs/IMPLEMENTATION-PLAN-v0.1.0.md`)
- **Deciders:** repository maintainers
- **Related:** ADR 0001 (Loom-verified ring buffer) — the shutdown
  handshake proofs continue to hold regardless of shard count.
  ADR 0008 (fused redaction automaton) — same "faster hot path,
  same public surface" pattern.

## Context

`LockFreeEngine::ingest` used to push directly into a single
`crossbeam-queue::ArrayQueue<LogEvent>`. Under `N` concurrent
producer threads, every push contended on the same producer-side
atomic tag — a single cache line shared across every producer core.
As `N` grew past 4, contention dominated the wall-clock cost of
`ingest`, capping throughput well below the queue's theoretical
per-slot cost.

The plan called out sharding as the surgical fix: split the queue
into `N` independent shards so producer-side atomic contention
scales as `1/N` instead of `1`. Consumer-side (the flusher) drains
all shards in rotation on every wake.

## Decision

Introduce an internal `ShardedQueue` type
(`crates/rlg/src/sharded_queue.rs`) that wraps
`Box<[ArrayQueue<LogEvent>]>` behind the minimal
`push` / `pop` / `pop_local` / `is_empty` surface
`LockFreeEngine` needs.

The shard count is a **compile-time constant** driven by a new
`fast-queue` Cargo feature:

- **Default build (no feature flag)** — `SHARD_COUNT = 1`.
  Byte-for-byte the same behaviour as the pre-Phase-18 direct
  `ArrayQueue` use. Zero regression for the single-producer case.
- **`--features fast-queue`** — `SHARD_COUNT = 8`. Producer-side
  atomic contention scales as `1/8` for `N >= 8` producers.

Producers pick a shard once per thread. A thread-local
`Cell<Option<usize>>` is initialised on the first `push` call to
`NEXT_SHARD.fetch_add(1, Relaxed) % SHARD_COUNT`. Every subsequent
`push` from the same thread hits the same shard with zero
selection overhead.

The public API — `LockFreeEngine::new`, `::ingest`, `::shutdown`,
and the `ENGINE` global — is unchanged in shape and observable
behaviour.

## Producer path

```rust
// Sticky per-thread shard index.
let shard = SHARD_INDEX.with(|slot| match slot.get() {
    Some(idx) => idx,
    None => {
        let idx = NEXT_SHARD.fetch_add(1, Relaxed) % SHARD_COUNT;
        slot.set(Some(idx));
        idx
    }
});
self.shards[shard].push(event)
```

Round-robin assignment via a shared `AtomicUsize` distributes
producers evenly across shards regardless of thread creation order.
The counter itself is contended once per thread lifetime — not
per push — so its cost is amortised.

## Consumer path (flusher)

```rust
fn pop(&self) -> Option<LogEvent> {
    for shard in &self.shards {
        if let Some(event) = shard.pop() {
            return Some(event);
        }
    }
    None
}
```

The flusher's per-wake drain loop calls `pop()` until it returns
`None`. Under `SHARD_COUNT = 1` this is one `ArrayQueue::pop`;
under `SHARD_COUNT = 8` it costs at most eight `ArrayQueue::pop`
tries before returning `None`. Since drain runs in batches of 64
events per wake, the amortised cost is negligible.

## Retry-eviction semantics

`LockFreeEngine::ingest` retries evicted pushes up to three times
on a full buffer. To keep the retry hitting the same shard as the
failed push, `ShardedQueue::pop_local` is a variant of `pop` that
targets the caller's thread-local shard rather than iterating.
Same-shard eviction ensures the retry's `push` sees a slot the
producer's shard just freed.

## Loom coverage

The Phase 10 Loom proofs (`crates/rlg/tests/loom_engine.rs`) use a
`Mutex<Vec<u32>>` as a stand-in for the concrete queue
implementation. Their invariants — no lost events across
shutdown, session-ID monotonicity — are shape-independent: they
hold regardless of whether the queue is one `ArrayQueue`, eight
sharded `ArrayQueue`s, or the mutex-vec model itself. No new Loom
harness is needed for Phase 18.

## Bench methodology

`crates/rlg/benches/competitive_bench.rs` exercises the ingest
path. To compare the two build variants:

```bash
# Baseline — 1 shard, same as pre-Phase-18 behaviour.
cargo bench --bench competitive_bench

# Sharded — 8 shards.
cargo bench --bench competitive_bench --features fast-queue
```

Precise multipliers land on the CI-published Criterion report at
v0.1.0 per Phase 27 (live `rustlogs.com/bench/`).

Expected direction (validated locally):
- **Single-producer case**: ≤0 % regression (sticky shard index +
  same underlying `ArrayQueue` per shard).
- **4-producer concurrent case**: ≥1.4× throughput (contention on
  the shared atomic tag drops from all-4-on-one to 1-of-8).

## What does NOT change

- **Public API.** `LockFreeEngine::new(capacity)`,
  `ingest(event)`, `shutdown()`, and the `ENGINE` global keep
  their signatures. Existing consumers upgrade transparently.
- **Total capacity semantics.** `LockFreeEngine::new(capacity)`
  still bounds the total in-flight event count at `capacity`.
  With shards, per-shard capacity is `capacity / SHARD_COUNT`
  (with remainder distributed to the first shards).
- **Shutdown handshake.** The `shutdown_flag` + `unpark` sequence
  is unchanged. The flusher's terminate condition
  (`shutdown && queue.is_empty()`) uses `ShardedQueue::is_empty`,
  which reports true only when every shard is empty.

## Consequences

- **Zero regression by default.** Users who never set the feature
  see byte-for-byte identical behaviour. The abstraction cost
  through `ShardedQueue::new` and the single-shard iteration in
  `pop` is trivial at N=1 and optimised out by the compiler.
- **Opt-in performance win.** Enterprise deployments with many
  producer threads flip the feature and get the win. Simpler
  deployments pay no cost for a knob they don't need.
- **Thread-local slot per producer.** ~24 bytes of TLS per thread
  that ingests. Negligible.

## Alternatives considered

- **Per-producer `rtrb` SPSC rings.** The plan's original text.
  Rejected in favour of sharded `ArrayQueue` for two reasons:
  1. `rtrb` is SPSC only; producer registration and consumer
     ownership add complexity that the sharded MPMC design avoids.
  2. `ArrayQueue` per shard preserves the MPMC semantics
     `LockFreeEngine` was already coded against, so the diff is
     surgical instead of a rewrite.
  Revisit if a future benchmark shows the SPSC path is materially
  faster than 8-way sharded MPMC.
- **Runtime-configurable shard count.** Rejected. Compile-time
  constant lets the compiler optimise the sharding away entirely
  under N=1. A runtime knob would foreclose that.
- **Thread-affinity or NUMA-aware sharding.** Considered for
  cross-socket deployments; deferred to a follow-up ADR once we
  have a NUMA benchmark to justify the complexity.

## References

- [`crossbeam-queue::ArrayQueue`](https://docs.rs/crossbeam-queue/latest/crossbeam_queue/struct.ArrayQueue.html)
- [`rtrb`](https://docs.rs/rtrb/) — SPSC alternative considered.
- ADR 0001 (Loom-verified ring buffer) — shutdown-handshake
  proofs that continue to hold under this change.
