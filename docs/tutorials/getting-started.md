# Getting Started

Install RLG, emit your first log, and verify output — all in under five minutes.

## 1. Add the Dependency

```toml
[dependencies]
rlg = "0.0.7"
```

For OTLP streaming to Grafana Loki or similar collectors, enable the `reqwest` feature:

```toml
rlg = { version = "0.0.7", features = ["reqwest"] }
```

## 2. Initialise and Log

Call `init::init()` once at startup. Store the returned `FlushGuard` — dropping it flushes all buffered events and shuts down the background thread.

```rust
use rlg::init;
use rlg::log::Log;
use rlg::log_format::LogFormat;

fn main() {
    let _guard = init::init().expect("failed to initialise RLG");

    Log::info("System initialisation complete")
        .component("kernel")
        .with("version", "0.0.7")
        .format(LogFormat::JSON)
        .fire();
}
```

`fire()` pushes the event into a ring buffer and returns immediately. The background flusher thread handles formatting and I/O.

## 3. Enable the TUI Dashboard

Set `RLG_TUI=1` to display a live metrics dashboard in your terminal:

```bash
RLG_TUI=1 cargo run
```

The dashboard shows throughput, error rates, active spans, and format distribution at 60 FPS.

## 4. Verify Platform-Native Output

RLG routes logs to your OS-native sink automatically:

- **macOS** — appears in Console.app via `os_log`:
  ```bash
  log show --predicate 'subsystem == "com.rlg.logger"' --last 1m
  ```

- **Linux** — appears in the systemd journal via `journald`:
  ```bash
  journalctl -t rlg --since "1 min ago"
  ```

If neither sink is available, RLG falls back to the configured file path or stdout.

## Next Steps

- [Fluent API](../how-to/fluent-api.md) — chain `.with()`, `.component()`, `.format()`, then `.fire()`
- [Engine Design](../explanation/engine-design.md) — how the ring buffer and flusher thread work
