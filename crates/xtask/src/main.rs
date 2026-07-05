// main.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `cargo xtask` — internal automation for the rlg workspace.
//!
//! Run `cargo xtask <subcommand>`. Implementations are intentionally
//! thin wrappers over `cargo` subprocesses; the value is having
//! conventions in typed Rust rather than scattered shell snippets.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use std::process::{Command, ExitStatus};

/// Workspace automation tasks.
#[derive(Parser, Debug)]
#[command(name = "xtask", version, about, long_about = None)]
struct Cli {
    /// The task to run.
    #[command(subcommand)]
    command: Task,
}

/// All supported automation tasks.
#[derive(Subcommand, Debug)]
enum Task {
    /// `cargo fmt --check && cargo clippy -D warnings && cargo test`.
    /// The standard pre-PR gate.
    CheckAll,
    /// Generate a coverage summary with `cargo llvm-cov`.
    Coverage,
    /// Run `cargo audit` and exit non-zero on any advisory.
    Audit,
    /// Build the public API documentation.
    Doc {
        /// Open the docs in a browser on completion.
        #[arg(long)]
        open: bool,
    },
    /// Run every example in the workspace.
    Examples,
    /// Run criterion benchmarks for the core crate.
    Bench,
    /// Bump every crate's `version =` to `<new>` and print the
    /// recommended release-checklist next steps. Does **not** push,
    /// tag, or call `cargo publish`.
    Release {
        /// New version, e.g. `0.0.11`.
        new: String,
    },
}

fn cargo(args: &[&str]) -> Result<ExitStatus> {
    let status =
        Command::new("cargo").args(args).status().with_context(
            || format!("failed to invoke `cargo {}`", args.join(" ")),
        )?;
    Ok(status)
}

fn require_success(label: &str, status: ExitStatus) -> Result<()> {
    if status.success() {
        Ok(())
    } else {
        bail!("{label} exited with {status}")
    }
}

fn check_all() -> Result<()> {
    require_success(
        "cargo fmt --all -- --check",
        cargo(&["fmt", "--all", "--", "--check"])?,
    )?;
    require_success(
        "cargo clippy",
        cargo(&[
            "clippy",
            "--workspace",
            "--all-features",
            "--tests",
            "--benches",
            "--",
            "-D",
            "warnings",
        ])?,
    )?;
    require_success(
        "cargo test",
        cargo(&["test", "--workspace", "--all-features"])?,
    )?;
    println!("✔ check-all green");
    Ok(())
}

fn coverage() -> Result<()> {
    require_success(
        "cargo llvm-cov",
        cargo(&[
            "llvm-cov",
            "--workspace",
            "--all-features",
            "--summary-only",
            "--show-missing-lines",
        ])?,
    )?;
    Ok(())
}

fn audit() -> Result<()> {
    require_success(
        "cargo audit",
        cargo(&["audit", "--deny", "warnings"])?,
    )?;
    Ok(())
}

fn doc(open: bool) -> Result<()> {
    let mut args =
        vec!["doc", "--workspace", "--all-features", "--no-deps"];
    if open {
        args.push("--open");
    }
    require_success("cargo doc", cargo(&args)?)?;
    Ok(())
}

fn examples() -> Result<()> {
    // Each example is built and run with the `tokio` feature on so
    // the tokio-gated demos compile too.
    for name in [
        "example",
        "example_lib",
        "example_macros",
        "example_log_level",
        "example_log_format",
        "example_config",
        "example_utils",
    ] {
        println!("» running example: {name}");
        require_success(
            name,
            cargo(&[
                "run",
                "-p",
                "rlg",
                "--example",
                name,
                "--features",
                "tokio,tui,miette,tracing-layer",
            ])?,
        )?;
    }
    Ok(())
}

fn bench() -> Result<()> {
    require_success(
        "cargo bench",
        cargo(&["bench", "-p", "rlg", "--bench", "competitive_bench"])?,
    )?;
    Ok(())
}

fn release(new: &str) -> Result<()> {
    // Conservative: only print the checklist. Bumping every crate
    // is deferred to a follow-up PR per release until we have a
    // stable cross-crate semver policy.
    println!("Release checklist for v{new}:");
    println!("  1. Bump version in every `Cargo.toml`.");
    println!("  2. Update `CHANGELOG.md`.");
    println!("  3. `cargo xtask check-all`.");
    println!("  4. `git tag -s v{new} -m 'release v{new}'`.");
    println!("  5. `git push origin main --tags`.");
    println!(
        "  6. CI's `release.yml` workflow handles `cargo publish`."
    );
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Task::CheckAll => check_all(),
        Task::Coverage => coverage(),
        Task::Audit => audit(),
        Task::Doc { open } => doc(open),
        Task::Examples => examples(),
        Task::Bench => bench(),
        Task::Release { new } => release(&new),
    }
}
