#!/usr/bin/env python3
"""Hold production Rust code to the complexity ceilings.

Per function: cyclomatic <= 10, cognitive <= 15, Halstead difficulty
<= 30, <= 60 source lines. Per file: <= 500 lines. Code over a
ceiling today is recorded in scripts/complexity-baseline.json; the gate
fails when a function or file goes over a ceiling without being in the
baseline, or when a baselined one gets worse on any metric. Shrinking
an offender, or fixing it outright, always passes; run with --update
to record the improvement.

Measured: crates/*/src/**/*.rs, minus `#[cfg(test)]` modules. Tests,
examples, benches and build scripts are not production code.
Metrics come from rust-code-analysis-cli (pinned in CI).

Usage: scripts/complexity-gate.py [--update]
"""

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "scripts" / "complexity-baseline.json"
CEILINGS = {"cyclomatic": 10, "cognitive": 15, "halstead": 30, "sloc": 60}
FILE_CEILING = 500
TEST_MOD = re.compile(r"#\[cfg\((?:all\()?\s*test\b[^\]]*\]\s*(?:pub\s+)?mod\s+\w+\s*\{")


def test_ranges(text):
    """1-based line ranges of `#[cfg(test)] mod … { … }` blocks."""
    ranges = []
    for m in TEST_MOD.finditer(text):
        depth, i = 0, m.end() - 1
        while i < len(text):
            depth += {"{": 1, "}": -1}.get(text[i], 0)
            if depth == 0:
                break
            i += 1
        start = text.count("\n", 0, m.start()) + 1
        ranges.append((start, text.count("\n", 0, i) + 1))
    return ranges


def measure_file(path):
    """(functions, lines) for one production source file."""
    rel = path.relative_to(ROOT).as_posix()
    text = path.read_text()
    out = subprocess.run(
        ["rust-code-analysis-cli", "-m", "-O", "json", "-p", str(path)],
        capture_output=True, text=True, check=True,
    ).stdout
    skip = test_ranges(text)
    funcs, seen = {}, {}

    def walk(node, scope):
        name = node.get("name") or "?"
        is_fn = node["kind"] == "function" and name != "<anonymous>"
        if is_fn and not any(a <= node["start_line"] <= b for a, b in skip):
            m = node["metrics"]
            qual = "::".join(scope + [name])
            seen[qual] = seen.get(qual, 0) + 1
            key = f"{rel}::{qual}" + (f"#{seen[qual]}" if seen[qual] > 1 else "")
            funcs[key] = {
                "cyclomatic": m["cyclomatic"]["sum"],
                "cognitive": m["cognitive"]["sum"],
                "halstead": round(m["halstead"].get("difficulty") or 0, 1),
                "sloc": m["loc"]["sloc"],
                "line": node["start_line"],
            }
        inner = scope + [name] if node["kind"] in ("impl", "trait", "function") and not is_fn else scope
        for child in node.get("spaces", []):
            walk(child, inner)

    walk(json.loads(out), [])
    return funcs, text.count("\n") + 1


def measure():
    funcs, files = {}, {}
    for path in sorted(ROOT.glob("crates/*/src/**/*.rs")):
        f, lines = measure_file(path)
        funcs.update(f)
        if lines > FILE_CEILING:
            files[path.relative_to(ROOT).as_posix()] = lines
    offenders = {
        k: {m: v[m] for m in CEILINGS}
        for k, v in funcs.items()
        if any(v[m] > c for m, c in CEILINGS.items())
    }
    return funcs, offenders, files


def check(funcs, offenders, files, base):
    problems = []
    for key, metrics in offenders.items():
        where = f"{key} (line {funcs[key]['line']})"
        old = base["functions"].get(key)
        if old is None:
            over = [f"{m} {metrics[m]} > {CEILINGS[m]}" for m in CEILINGS if metrics[m] > CEILINGS[m]]
            problems.append(f"new offender {where}: {', '.join(over)}")
            continue
        worse = [f"{m} {old[m]} -> {metrics[m]}" for m in CEILINGS if metrics[m] > old[m]]
        if worse:
            problems.append(f"worse {where}: {', '.join(worse)}")
    for path, lines in files.items():
        old = base["files"].get(path)
        if old is None:
            problems.append(f"new file over {FILE_CEILING} lines: {path} ({lines})")
        elif lines > old:
            problems.append(f"file grew: {path} {old} -> {lines} lines")
    return problems


def main():
    funcs, offenders, files = measure()
    if "--update" in sys.argv:
        # Why a remaining offender is accepted, kept across updates for
        # the offenders that are still there.
        notes = json.loads(BASELINE.read_text()).get("notes", {}) if BASELINE.exists() else {}
        notes = {k: v for k, v in notes.items() if k in offenders or k in files}
        BASELINE.write_text(json.dumps(
            {"ceilings": {**CEILINGS, "file_lines": FILE_CEILING},
             "functions": offenders, "files": files, "notes": notes},
            indent=2, sort_keys=True) + "\n")
        print(f"baseline: {len(offenders)} functions, {len(files)} files")
        return 0
    base = json.loads(BASELINE.read_text())
    problems = check(funcs, offenders, files, base)
    for p in problems:
        print(f"FAIL {p}")
    fixed = sorted(set(base["functions"]) - set(offenders)) + sorted(set(base["files"]) - set(files))
    for key in fixed:
        print(f"fixed {key}: run with --update to shrink the baseline")
    print(f"{len(funcs)} functions measured; {len(offenders)} over a ceiling "
          f"(baseline {len(base['functions'])}); {len(files)} files over "
          f"{FILE_CEILING} lines (baseline {len(base['files'])})")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
