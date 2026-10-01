// completions.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! `rlg-report --completions <SHELL>` prints a script generated from the CLI
//! definition, and refuses to be combined with other arguments.

#![allow(missing_docs)]

use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rlg-report"))
        .args(args)
        .output()
        .expect("binary runs")
}

#[test]
fn every_shell_gets_a_script_naming_the_binary() {
    for shell in ["bash", "zsh", "fish", "elvish", "powershell"] {
        let out = run(&["--completions", shell]);
        assert!(out.status.success(), "{shell}");
        let script = String::from_utf8(out.stdout).unwrap();
        assert!(script.contains("rlg-report"), "{shell}: {script}");
        assert!(script.contains("format"), "{shell} lists the options");
    }
}

#[test]
fn completions_are_exclusive() {
    let out = run(&["--completions", "zsh", "--format", "json"]);
    assert!(!out.status.success());
}

#[test]
fn the_manpage_is_generated_from_the_cli() {
    let out = run(&["--manpage"]);
    assert!(out.status.success());
    let page = String::from_utf8(out.stdout).unwrap();
    assert!(page.contains(".TH rlg-report 1"), "{page}");
    assert!(page.contains("format"), "lists the options");
}
