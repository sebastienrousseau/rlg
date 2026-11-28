#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0 OR MIT
"""The workspace README follows the portfolio template (README-TEMPLATE.md).

Checks the headings the template requires, in its order; that no
{{TEMPLATE_TOKEN}} is left unresolved outside code; and the header
structure: SPDX comment, centred logo and h1, the OpenSSF Scorecard
badge, and the demo GIF. The ecosystem table must list exactly the
family: every publishable workspace crate plus the repositories in
EXTERNAL, each once (REPO-STANDARD section 8).

Usage: scripts/check-readme.py [README.md]
"""

import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# Lockstep members that live in their own repository.
EXTERNAL = ["rustlogs.github.io"]

HEADINGS = [
    "Contents",
    "Install",
    "Requirements",
    "Quick Start",
    "The {project} ecosystem",
    "Capabilities at a glance",
    "Ecosystem comparison",
    "Benchmarks",
    "Features",
    "Configuration",
    "Examples",
    "When not to use {project}",
    "Development",
    "Security",
    "Documentation",
    "Stability guarantees",
    "License",
]
# REUSE-IgnoreStart
STRUCTURE = [
    "<!-- SPDX-License-Identifier:",
    '<p align="center">',
    '<h1 align="center">',
    "ossf-scorecard",
    ".github/demo.gif",
]
# REUSE-IgnoreEnd


def problems(text: str) -> list[str]:
    found = []
    prose = re.sub(r"```.*?```", "", text, flags=re.S)
    prose = re.sub(r"`[^`]*`", "", prose)
    found += [f"unresolved token {t}" for t in re.findall(r"\{\{[A-Z0-9_]+\}\}", prose)]
    h1 = re.search(r'<h1 align="center">([^<]+)</h1>', text)
    if not h1:
        return found + ["centred project <h1> missing"]
    want = [h.format(project=h1.group(1).strip()) for h in HEADINGS]
    have = re.findall(r"(?m)^## (.+?)\s*$", text)
    if have != want:
        found += [f"missing heading: {h}" for h in want if h not in have]
        found += [f"unexpected heading: {h}" for h in have if h not in want]
        if set(have) == set(want):
            found.append("headings are out of the template's order")
    found += [f"missing structure: {s}" for s in STRUCTURE if s not in text]
    return found


def family() -> list[str]:
    """Publishable workspace crates plus the external members, sorted."""
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text())
    names = list(EXTERNAL)
    for member in workspace["workspace"]["members"]:
        package = tomllib.loads((ROOT / member / "Cargo.toml").read_text())["package"]
        if package.get("publish", True) is not False:
            names.append(package["name"])
    return sorted(names)


def table_rows(text: str, project: str) -> list[str]:
    """First-cell names of the table under "## The {project} ecosystem"."""
    section = re.search(rf"(?ms)^## The {re.escape(project)} ecosystem$(.*?)^## ", text)
    if not section:
        return []
    return re.findall(r"(?m)^\| \[?`([^`]+)`", section.group(1))


def family_problems(text: str) -> list[str]:
    h1 = re.search(r'<h1 align="center">([^<]+)</h1>', text)
    rows = table_rows(text, h1.group(1).strip()) if h1 else []
    want = family()
    found = [f"ecosystem table lacks {n}" for n in want if n not in rows]
    found += [f"ecosystem table lists {n}, not a family member" for n in rows if n not in want]
    found += [f"ecosystem table lists {n} twice" for n in sorted(set(rows)) if rows.count(n) > 1]
    return found


def main() -> int:
    path = Path(sys.argv[1] if len(sys.argv) > 1 else "README.md")
    text = path.read_text()
    issues = problems(text) + family_problems(text)
    for issue in issues:
        print(f"FAIL {path}: {issue}")
    if not issues:
        print(f"ok: {path} follows the README template")
    return 1 if issues else 0


if __name__ == "__main__":
    sys.exit(main())
