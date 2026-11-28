#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0 OR MIT
"""Hold third-party dependencies to a baseline that only goes down.

Four kinds are counted (standards AGENTS.md section 11):

- cargo:   every registry or git package in Cargo.lock, by name
- actions: every GitHub Action or reusable workflow in .github/
- images:  every container base image (Dockerfiles, dev container)
- tools:   every tool CI, make or pre-commit installs
           (install-action `tool:`, `cargo install`, `npx`, `pipx run`,
           hook repos)

The gate fails when any of them names something that is not in
scripts/dependency-baseline.json, and when a direct crate dependency
has no row in docs/DEPENDENCIES.md giving its reason. Removing a
dependency always passes; run with --update to record the smaller set.
A new dependency needs the maintainer's approval of that named
dependency, recorded in docs/DEPENDENCIES.md, before --update adds it.

Usage: scripts/dependency-gate.py [--update]
"""

import json
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "scripts" / "dependency-baseline.json"
INVENTORY = ROOT / "docs" / "DEPENDENCIES.md"
DEP_TABLES = ("dependencies", "dev-dependencies", "build-dependencies")

USES = re.compile(r"^\s*(?:-\s*)?uses:\s*([^@\s#]+)", re.M)
FROM = re.compile(r"^\s*FROM\s+(?:--\S+\s+)*([^\s@:]+)", re.M)
STAGE = re.compile(r"^\s*FROM\s+.*\s+AS\s+(\S+)", re.M | re.I)
IMAGE = re.compile(r'"image"\s*:\s*"([^"@:]+)')
TOOL = re.compile(r"^\s*tool:\s*(.+)$", re.M)
CARGO_INSTALL = re.compile(r"cargo (?:install|binstall)\s+(?:--\S+\s+)*([A-Za-z0-9_-]+)")
PIPX = re.compile(r"pipx run\s+(?:--\S+\s+)*([A-Za-z0-9_.-]+)")
NPX = re.compile(r"npx\s+(?:-y\s+)?(@?[A-Za-z0-9_./-]+)")
HOOK_REPO = re.compile(r"^\s*-\s*repo:\s*(\S+)", re.M)


def read(path):
    """Text of `path`, or "" when it does not exist."""
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def cargo_packages():
    """Names of the third-party packages in Cargo.lock."""
    lock = tomllib.loads(read(ROOT / "Cargo.lock"))
    return {p["name"] for p in lock.get("package", []) if "source" in p}


def ci_files():
    """Workflow and composite-action files under .github/."""
    return sorted((ROOT / ".github").rglob("*.y*ml"))


def actions():
    """Actions and reusable workflows referenced by `uses:`, minus local ones."""
    refs = set()
    for path in ci_files():
        refs.update(m for m in USES.findall(read(path)) if not m.startswith("./"))
    return refs


def images():
    """Container base images, without tag or digest."""
    found = set()
    for path in ROOT.rglob("Dockerfile*"):
        if "target" in path.parts:
            continue
        text = read(path)
        stages = {s.lower() for s in STAGE.findall(text)} | {"scratch"}
        found.update(n for n in FROM.findall(text) if n.lower() not in stages)
    found.update(IMAGE.findall(read(ROOT / ".devcontainer" / "devcontainer.json")))
    return found


def unversioned(name):
    """`name` without an `@version` suffix (a leading `@` is an npm scope)."""
    name = name.split("==")[0]
    at = name.find("@", 1)
    return name[:at] if at > 0 else name


def tools():
    """Tools installed by CI, make or pre-commit, without versions."""
    found = set()
    for path in ci_files():
        text = read(path)
        for line in TOOL.findall(text):
            found.update(t.strip() for t in line.split(","))
        found.update(NPX.findall(text))
        found.update(PIPX.findall(text))
    for name in ("Makefile", "GNUmakefile"):
        found.update(CARGO_INSTALL.findall(read(ROOT / name)))
    found.update(HOOK_REPO.findall(read(ROOT / ".pre-commit-config.yaml")))
    return {unversioned(t) for t in found if t}


def current():
    """Every counted dependency, by kind."""
    return {"cargo": cargo_packages(), "actions": actions(), "images": images(), "tools": tools()}


def table_deps(table):
    """Third-party crate names in one dependency table."""
    for key, spec in table.items():
        if isinstance(spec, dict) and "path" in spec and "version" not in spec:
            continue
        name = spec.get("package", key) if isinstance(spec, dict) else key
        if not name.startswith("rlg"):
            yield name


def manifest_tables(manifest):
    """Every dependency table in one Cargo.toml, target-specific ones included."""
    tables = [manifest.get(k, {}) for k in DEP_TABLES]
    for target in manifest.get("target", {}).values():
        tables.extend(target.get(k, {}) for k in DEP_TABLES)
    tables.append(manifest.get("workspace", {}).get("dependencies", {}))
    return tables


def direct_crates():
    """Crates some manifest in the workspace names directly."""
    names = set()
    manifests = [ROOT / "Cargo.toml", ROOT / "fuzz" / "Cargo.toml"]
    manifests += sorted((ROOT / "crates").glob("*/Cargo.toml"))
    for path in manifests:
        for table in manifest_tables(tomllib.loads(read(path))):
            names.update(table_deps(table))
    return names


def undocumented(direct):
    """Direct crates with no `| \\`name\\` |` row in the inventory."""
    rows = set(re.findall(r"^\|\s*`([^`]+)`\s*\|", read(INVENTORY), re.M))
    return sorted(direct - rows)


def additions(now, base):
    """`kind: name` for every dependency not in the baseline."""
    return [f"{k}: {n}" for k in sorted(now) for n in sorted(now[k] - set(base.get(k, [])))]


def update(now):
    """Write the current set as the baseline."""
    data = {k: sorted(v) for k, v in sorted(now.items())}
    BASELINE.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    counts = ", ".join(f"{k} {len(v)}" for k, v in data.items())
    print(f"dependency baseline written: {counts}")
    return 0


def report(added, missing, now, base):
    """Print the verdict; 1 on any failure."""
    for line in added:
        print(f"new dependency not in the baseline: {line}")
    for name in missing:
        print(f"direct dependency with no reason in docs/DEPENDENCIES.md: {name}")
    if added or missing:
        print("A new dependency needs the maintainer's named approval (AGENTS.md).")
        return 1
    shrunk = sum(len(base.get(k, [])) - len(v) for k, v in now.items())
    counts = ", ".join(f"{k} {len(v)}" for k, v in sorted(now.items()))
    hint = f"; {shrunk} fewer than the baseline, run --update" if shrunk > 0 else ""
    print(f"dependencies within the baseline ({counts}){hint}")
    return 0


def main(argv):
    """Entry point."""
    now = current()
    if "--update" in argv:
        return update(now)
    base = json.loads(read(BASELINE) or "{}")
    return report(additions(now, base), undocumented(direct_crates()), now, base)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
