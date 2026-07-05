<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Migrating from `tracing` to `rlg`

This guide covers the concrete replacements for the surface most
`tracing` codebases use. It does **not** cover advanced tracing
features (spans-as-context-propagation via `#[instrument]`,
`Subscriber` layering across observability backends) — those
have direct rlg equivalents via `RlgLayer` (the `tracing-layer`
feature), which is the smoothest migration path.

## When to migrate

- You want a single fluent API instead of `tracing::event!` +
  `#[instrument]` macro machinery.
- You want structured logs by default (rlg records carry `Cow`
  attributes at ~1 alloc per record) rather than tracing's
  span-scoped attributes.
- You want to ship OTLP directly from the process without a
  separate exporter crate — `rlg-otlp` is first-party.
- You want PII redaction on the write path — `rlg-redact` is
  first-party.

## When NOT to migrate

- You need distributed span propagation across processes and
  your infrastructure already speaks tracing/OTel spans.
- You want per-span context you can enter/exit — rlg's model is
  flat records with attributes.
- You want `#[instrument]` macros to auto-generate span code —
  rlg doesn't have this pattern.

If you're in this category, keep `tracing` and use `RlgLayer` to
bridge tracing events into rlg's engine for structured storage +
redaction + OTLP export.

## Level mapping

| tracing | rlg |
|---|---|
| `trace!` | `Log::trace` |
| `debug!` | `Log::debug` |
| `info!` | `Log::info` |
| `warn!` | `Log::warn` |
| `error!` | `Log::error` |
| — | `Log::verbose`, `Log::fatal`, `Log::critical` (extra) |

## Event emission

```rust
// tracing
tracing::info!(user_id = 42, region = "eu-west-1", "authenticated");

// rlg
rlg::log::Log::info("authenticated")
    .with("user_id", 42_u64)
    .with("region", "eu-west-1")
    .fire();
```

## Structured attributes

`tracing` fields are macro-magic key/value pairs. rlg uses
`.with(key, value)` fluent calls; every value type must implement
`Into<serde_json::Value>`.

```rust
// tracing
tracing::info!(order_id = %uuid, amount = 4200_u64, "payment posted");

// rlg
Log::info("payment posted")
    .with("order_id", uuid.to_string())
    .with("amount", 4200_u64)
    .fire();
```

## Filter / subscriber setup

```rust
// tracing
tracing_subscriber::fmt::init();

// rlg
let _guard = rlg::init().unwrap();
// _guard flushes on drop
```

## Span-adjacent patterns

If you use `#[instrument]` spans for latency measurement:

```rust
// tracing
#[tracing::instrument(fields(order_id = %id))]
async fn checkout(id: Uuid) { … }

// rlg
async fn checkout(id: Uuid) {
    let start = std::time::Instant::now();
    // … work …
    Log::info("checkout completed")
        .with("order_id", id.to_string())
        .with("latency_ms", start.elapsed().as_millis() as u64)
        .fire();
}
```

For automated timing, rlg ships the `rlg_time_it!` macro.

## Bridging: keep `tracing` + use rlg for storage

Add `rlg` with the `tracing-layer` feature:

```toml
rlg = { version = "0.0.11", features = ["tracing-layer"] }
```

Install both subscribers:

```rust
use tracing_subscriber::layer::SubscriberExt;

let subscriber = tracing_subscriber::registry()
    .with(rlg::RlgLayer::default());
tracing::subscriber::set_global_default(subscriber).unwrap();
```

Every `tracing::info!` now routes through rlg's engine —
structured storage, redaction, OTLP export all apply.

## Related

- [`from-log.md`](from-log.md) — migrating from `log` (much
  smaller diff).
- [`from-slog.md`](from-slog.md) — migrating from `slog`.
