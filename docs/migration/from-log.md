<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Migrating from `log` to `rlg`

The `log` crate is the facade. rlg can either **replace** it
(direct rlg API) or **install as its backend** (drop-in). Choose
by whether you want the fluent API or minimal diff.

## Option A: install rlg as the `log` facade backend

Zero call-site changes.

```rust
use log::info;

// Initialize once at startup:
rlg::init().unwrap();

// Every existing log::* call routes through rlg's engine now.
info!("user_id={user_id} authenticated");
```

You get structured storage, redaction, OTLP export — but records
still look like the message-formatted strings your `log::` calls
produced. Attributes are not extracted.

## Option B: rewrite to the rlg fluent API

Diff at the call site but you get first-class structured
attributes.

```rust
// before (log)
info!("user_id={user_id} authenticated");

// after (rlg)
rlg::log::Log::info("authenticated")
    .with("user_id", user_id)
    .fire();
```

## Level mapping

| log | rlg |
|---|---|
| `trace!` | `Log::trace` |
| `debug!` | `Log::debug` |
| `info!` | `Log::info` |
| `warn!` | `Log::warn` |
| `error!` | `Log::error` |

## Setup

```rust
// before
env_logger::init();

// after
let _guard = rlg::init().unwrap();
```

Filter via `RUST_LOG` continues to work — rlg parses the same
env var syntax.

## Related

- [`from-tracing.md`](from-tracing.md) — larger diff, richer
  target.
- [`from-slog.md`](from-slog.md).
