<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Migrating from `slog` to `rlg`

`slog` was the first structured-logging library for Rust to gain
traction. Its `o!(...)` context macro and `Logger::new(root,
o!(...))` inheritance model don't have a direct rlg equivalent
— rlg records are flat and self-contained.

## Context inheritance

```rust
// slog
let root = slog::Logger::root(drain, o!("service" => "api"));
let child = root.new(o!("user_id" => user_id));
info!(child, "authenticated");

// rlg
Log::info("authenticated")
    .component("api")
    .with("user_id", user_id)
    .fire();
```

For a shared context, wrap the fluent calls in a helper:

```rust
fn service_log(msg: &str) -> Log {
    Log::info(msg).component("api")
}

service_log("authenticated")
    .with("user_id", user_id)
    .fire();
```

## Async drain

`slog-async` runs a channel between call sites and the actual
drain. rlg's engine already does this — every `Log::fire()`
pushes to an atomic ring buffer, and a background flusher thread
drains it. No migration needed.

## Level mapping

| slog | rlg |
|---|---|
| `trace!` | `Log::trace` |
| `debug!` | `Log::debug` |
| `info!` | `Log::info` |
| `warn!` | `Log::warn` |
| `error!` | `Log::error` |
| `crit!` | `Log::critical` |

## Setup

```rust
// slog
let drain = slog_async::Async::new(slog_json::Json::default(io::stdout()).fuse()).build().fuse();
let root = slog::Logger::root(drain, o!());

// rlg
let _guard = rlg::init().unwrap();
```

## Related

- [`from-tracing.md`](from-tracing.md).
- [`from-log.md`](from-log.md).
