# How-To: Using the Liquid Fluent API

The "Liquid" API in `rlg` is designed to feel effortless. Inspired by Apple's Human Interface Guidelines, it prioritizes the developer's intent and minimizes boilerplate.

## 1. Basic Semantic Entry
Every log starts with a severity level. This creates a builder that defaults to standard settings.

```rust
use rlg::log::Log;

// Minimal
Log::info("Connection established").fire();
```

## 2. Chainable Context
Logging is only useful if it's structured. Add context using the `.with()` method. It accepts any type that implements `serde::Serialize`.

```rust
Log::warn("Potential security breach")
    .with("ip_address", "192.168.1.100")
    .with("attempts", 5)
    .with("target_resource", "/admin/login")
    .fire();
```

## 3. Component & Format Overrides
You can override defaults on a per-log basis. This is particularly useful for multi-service binaries or when certain logs need to be OTLP-compliant while others remain in simple Logfmt.

```rust
use rlg::log_format::LogFormat;

Log::error("Database query failed")
    .component("db-client-pool")
    .format(LogFormat::OTLP) // Ensure this log is AI-readable
    .with("query_time_ms", 1250)
    .fire();
```

## 4. Manual Control (Advanced)
While `.fire()` is the recommended way to interact with the lock-free engine, you can also control the exact timing of the handoff if you need to perform additional processing.

```rust
let log_entry = Log::info("Ready")
    .session_id("CUSTOM-ID")
    .time("2026-03-05T12:00:00Z");

// ... do something else ...

log_entry.fire();
```

## AI-Readability Tip
When using `LogFormat::MCP` or `LogFormat::OTLP`, ensure your keys in `.with(key, value)` are descriptive. AI orchestrators use these keys to automatically map your application's state and detect anomalies. Use `snake_case` for maximum compatibility.
