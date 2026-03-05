# Getting Started with RLG

Welcome to the future of high-performance observability. This tutorial will take you from a blank terminal to a fully functioning, lock-free logging pipeline in under 5 minutes.

## 1. Installation

Add `rlg` to your `Cargo.toml`. For the 2026 standards, we recommend enabling the `reqwest` feature if you plan on streaming to OTLP collectors.

```toml
[dependencies]
rlg = { version = "0.0.7", features = ["reqwest"] }
```

## 2. Your First "Liquid" Log

`rlg` uses a Fluent API designed for speed and ergonomics. Unlike traditional loggers, `rlg` never blocks your main thread.

Create a `main.rs` file:

```rust
use rlg::log::Log;
use rlg::log_format::LogFormat;

fn main() {
    // 1. Initialize a simple log entry
    // 2. Add semantic context
    // 3. Fire it into the background engine
    Log::info("System initialization complete")
        .component("kernel")
        .with("version", "0.0.7")
        .with("mode", "production")
        .format(LogFormat::MCP)
        .fire();

    println!("Log has been handed off to the engine in <10ns.");
}
```

## 3. The Generative TUI Dashboard

During development, you don't want to tail flat files. `rlg` includes a built-in "Liquid Glass" dashboard.

To enable it, set the `RLG_TUI` environment variable:

```bash
export RLG_TUI=1
cargo run
```

You will see a 60FPS real-time dashboard at the bottom of your terminal showing throughput, error rates, and active spans.

## 4. Platform-Native Sinks

One of the unique powers of `rlg` is its ability to talk directly to your OS:

- **On macOS:** Your logs automatically flow into the **Console.app** via `os_log`.
- **On Linux:** Your logs are injected directly into the **Systemd Journal** via binary socket.

Try viewing your logs with the native tools:

```bash
# macOS
log show --predicate 'subsystem == "com.rlg.logger"' --last 1m

# Linux
journalctl -t rlg --since "1 min ago"
```

## Next Steps
Now that you have the basics running, dive into the [How-To: Streaming OTLP to Grafana Loki](../how-to/otlp-loki.md) or explore the [Architecture Deep-Dive](../explanation/engine-design.md).
