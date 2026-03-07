# Engine Design

RLG separates log ingestion from formatting and I/O. Application threads push events into a ring buffer; a single background thread drains, formats, and writes them.

---

## 1. The Ring Buffer

The engine uses a `crossbeam::ArrayQueue` with a fixed capacity of 65,536 slots. `ArrayQueue` is a bounded, multi-producer, multi-consumer queue backed by contiguous memory and atomic operations.

Call flow:

1. `Log::info("msg").fire()` builds a `LogEvent` and calls `ENGINE.ingest()`.
2. `ingest()` checks the event's level against an atomic filter. Events below the threshold are dropped immediately.
3. `ingest()` pushes the event into the `ArrayQueue`. If the buffer is full, it evicts the oldest entry and retries.
4. `ingest()` unparks the flusher thread via a cached `std::thread::Thread` handle — no `Mutex` on the hot path.

## 2. The Flusher Thread

A single OS thread (`std::thread::spawn`) parks itself when the queue is empty and unparks on each `ingest()`. On wake:

1. Drain all available events from the queue into a local batch.
2. Format each event into a reusable `String` buffer using `Display::fmt`.
3. Write the batch to the configured sink (file, journald, os_log, or stdout).
4. Park again.

The flusher reuses its format buffer across batches to avoid repeated heap allocation.

## 3. Deferred Formatting

Formatting happens on the flusher thread, never on the caller's thread. `Log::build()` captures metadata (level, description, component, attributes) without serialising to a string. The `Display` implementation on `Log` handles serialisation when the flusher calls `write!`.

This design keeps the ingestion path fast: one atomic level check, one `ArrayQueue::push`, one `thread::unpark`.

## 4. Platform Sinks

The flusher dispatches formatted output to a `PlatformSink`:

| Platform | Sink | Mechanism |
|----------|------|-----------|
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
