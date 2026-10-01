# Engine Design

RLG separates log ingestion from formatting and I/O. Application threads push events into a ring buffer; a single background thread drains, formats, and writes them.

---

## 1. The Ring Buffer

The engine uses a `ShardedQueue` with a fixed capacity of 65,536 slots: one `crossbeam::ArrayQueue` by default, or eight when the `fast-queue` feature spreads producers across shards to reduce cache-line contention ([ADR 0009](../adr/0009-sharded-producer-queue.md)). Each `ArrayQueue` is a bounded, multi-producer, multi-consumer queue backed by contiguous memory and atomic operations.

Call flow:

1. `Log::info("msg").fire()` builds a `LogEvent` and calls `ENGINE.ingest()`.
2. `ingest()` checks the event's level against an atomic filter. Events below the threshold are dropped immediately.
3. `ingest()` pushes the event into the caller's shard. If the shard is full, it evicts the oldest entry on that shard and retries, up to three times. Every event that does not stay in the buffer is counted once in `TuiMetrics::dropped_events`: each eviction that removes one, and the new event if every retry loses the race.
4. `ingest()` unparks the flusher thread via a cached `std::thread::Thread` handle — no `Mutex` on the hot path.

## 2. The Flusher Thread

A single OS thread named `rlg-flusher` parks itself between batches and is unparked by each `ingest()`; a 5 ms park timeout is the fallback when no wake-up arrives. On wake:

1. Drain up to 64 events from the queue into a local batch.
2. Format each event into a reused byte buffer using `Display::fmt`.
3. Write each formatted event to the configured sink (file, journald, os_log, or stdout).
4. Stop if shutdown was requested and the queue is empty; otherwise park again.

The flusher reuses its format buffer across batches to avoid repeated heap allocation.

## 3. Deferred Formatting

Formatting happens on the flusher thread, never on the caller's thread. `Log::build()` captures metadata (level, description, component, attributes) without serialising to a string. The `Display` implementation on `Log` handles serialisation when the flusher calls `write!`.

This design keeps the ingestion path fast: one atomic level check, one `ArrayQueue::push`, one `thread::unpark`.

## 4. Platform Sinks

The flusher dispatches formatted output to a `PlatformSink`:

| Platform | Sink | Mechanism |
| ---------- | ------ | ----------- |
| macOS | `os_log` | FFI call to `libsystem` |
| Linux | `journald` | `UnixDatagram` to `/run/systemd/journal/socket` |
| Fallback | File / stdout | `std::fs::File` or `std::io::stdout` |

Sink selection happens once at startup via `PlatformSink::from_config()` or `PlatformSink::native()`.

## 5. Shutdown

Call `ENGINE.shutdown()` or drop the `FlushGuard` returned by `init()`. This:

1. Drains all remaining events from the queue.
2. Joins the flusher thread.
3. Closes the sink.

**If you exit without shutdown, buffered events are lost.** Always hold the `FlushGuard` until process exit.
