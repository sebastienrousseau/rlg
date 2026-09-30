#!/usr/bin/env python3
"""The workspace README follows the portfolio template (README-TEMPLATE.md).

Checks the headings the template requires, in its order; that no
{{TEMPLATE_TOKEN}} is left unresolved outside code; and the header
structure: SPDX comment, centred logo and h1, the OpenSSF Scorecard
badge, and the demo GIF.

Usage: scripts/check-readme.py [README.md]
"""

import re
import sys
from pathlib import Path

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
STRUCTURE = [
    "<!-- SPDX-License-Identifier:",
    '<p align="center">',
    '<h1 align="center">',
    "ossf-scorecard",
    ".github/demo.gif",
]


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


def main() -> int:
    path = Path(sys.argv[1] if len(sys.argv) > 1 else "README.md")
    issues = problems(path.read_text())
    for issue in issues:
        print(f"FAIL {path}: {issue}")
    if not issues:
        print(f"ok: {path} follows the README template")
    return 1 if issues else 0


if __name__ == "__main__":
    sys.exit(main())
