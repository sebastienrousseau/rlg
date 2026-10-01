# How-To: The Fluent API

Build structured log entries with a chainable builder. Every method returns `Self` — chain freely, then dispatch with `.fire()`.

## 1. Start with a Severity Level

Every log begins with a level shortcut. This returns a builder with sensible defaults.

```rust
use rlg::log::Log;

Log::info("Connection established").fire();
```

Available shortcuts: `info`, `warn`, `error`, `debug`, `trace`, `fatal`, `critical`, `verbose`.

## 2. Attach Structured Context

Add key-value attributes with `.with()`. Accepts any `T: Serialize`.

```rust
Log::warn("Potential breach detected")
    .with("ip_address", "192.168.1.100")
    .with("attempts", 5)
    .with("target_resource", "/admin/login")
    .fire();
```

Attributes are stored in a `BTreeMap<String, serde_json::Value>` and serialized in sorted order.

## 3. Override Component and Format

Tag the originating module with `.component()`. Switch the output format per-entry with `.format()`.

```rust
use rlg::log_format::LogFormat;

Log::error("Database query failed")
    .component("db-client-pool")
    .format(LogFormat::OTLP)
    .with("query_time_ms", 1250)
    .fire();
```

## 4. Manual Control

`.fire()` consumes the builder and pushes it into the ring buffer. For deferred dispatch, store the builder and fire later.

```rust
let entry = Log::info("Ready")
    .session_id(42)
    .time("2026-03-05T12:00:00Z");

// ... additional processing ...

entry.fire();
```

**`.fire()` vs `.log()`**: `.fire()` consumes `self` (no clone). `.log()` borrows and clones — use it only when you need to retain the entry.

## AI Format Guidelines

For `LogFormat::MCP` and `LogFormat::OTLP`, use descriptive `snake_case` keys in `.with()`. AI orchestrators map these keys automatically for anomaly detection and state tracking.
