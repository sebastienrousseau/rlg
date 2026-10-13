#!/usr/bin/env bash
# Install snippets in the docs must name the version the workspace ships.
#
# Every publishable crate moves in lockstep, but the `rlg = "…"` lines in
# the READMEs and guides were edited by hand and fell a release (and in
# places five) behind, so a copy-pasted snippet pulled an old rlg. This
# compares each such line in the Markdown docs with crates/rlg/Cargo.toml.
# CHANGELOG, release notes and ADRs are history and are not checked.
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

want=$(sed -n 's/^version = "\([^"]*\)"/\1/p' crates/rlg/Cargo.toml | head -1)
[ -n "$want" ] || {
  echo "could not read crates/rlg/Cargo.toml version"
  exit 1
}

# `rlg = "X"`, `rlg-otlp  = "X"`, `rlg = { version = "X", … }`.
pattern='^[[:space:]]*rlg(-[a-z]+)?[[:space:]]*=[[:space:]]*(\{[[:space:]]*version[[:space:]]*=[[:space:]]*)?"[0-9]+\.[0-9]+\.[0-9]+"'

status=0
while IFS= read -r hit; do
  got=$(printf '%s\n' "$hit" | sed -E 's/.*"([0-9]+\.[0-9]+\.[0-9]+)".*/\1/')
  if [ "$got" != "$want" ]; then
    echo "STALE ${hit%%:*}: ${hit#*:} (workspace is $want)"
    status=1
  fi
done < <(git ls-files '*.md' |
  grep -vE '^(CHANGELOG\.md|RELEASE-NOTES|docs/adr/)' |
  xargs grep -nE "$pattern" /dev/null || true)

# The Arch recipe carries the version too.
pkgver=$(sed -n 's/^pkgver=//p' PKGBUILD)
if [ "$pkgver" != "$want" ]; then
  echo "STALE PKGBUILD: pkgver=$pkgver (workspace is $want)"
  status=1
fi

# The MSRV. crates/rlg/Cargo.toml's rust-version is the floor; every
# crate manifest and every other statement of it must agree, or a raise
# leaves a stale promise behind. Each pattern must still match, so a
# reworded statement fails here instead of dropping out of the check.
msrv=$(sed -n 's/^rust-version = "\([^"]*\)"/\1/p' crates/rlg/Cargo.toml | head -1)
msrv_sites=(
  "crates/rlg*/Cargo.toml fuzz/Cargo.toml|^rust-version = \"[0-9.]+\""
  ".github/workflows/ci.yml|MSRV \([0-9.]+\) builds|toolchain install [0-9.]+|cargo \+[0-9.]+ check"
  "crates/rlg/build.rs|let min_version = \"[0-9.]+\""
  "flake.nix|rust-bin\.stable\.\"[0-9.]+\""
  "README.md|badge/MSRV-[0-9.]+-|Rust version [0-9.]+\"|\*\*Rust [0-9.]+ or newer\*\*|binary on [0-9.]+;|Minimum toolchain\*\*: Rust [0-9.]+"
  "crates/rlg/README.md CONTRIBUTING.md|targets Rust \*\*[0-9.]+\*\* \(MSRV\)"
  "crates/rlg-*/README.md|Requires Rust \*\*[0-9.]+\*\* or newer"
  "DEVELOPMENT.md|minimum supported version is \*\*[0-9.]+\*\*"
  "docs/POLICIES.md|The floor is Rust [0-9.]+\*\*|MSRV \([0-9.]+\) builds|cargo \+[0-9.]+ check"
  "docs/packaging.md|minimum supported Rust is \*\*[0-9.]+\*\*|builds with the [0-9.]+ toolchain"
)
for site in "${msrv_sites[@]}"; do
  IFS='|' read -r -a parts <<<"$site"
  read -r -a globs <<<"${parts[0]}"
  files=()
  for g in "${globs[@]}"; do
    matched=$(compgen -G "$g" || true)
    [ -n "$matched" ] || { echo "STALE MSRV $g: no such file"; status=1; continue; }
    while IFS= read -r m; do files+=("$m"); done <<<"$matched"
  done
  for f in "${files[@]}"; do
    for p in "${parts[@]:1}"; do
      hits=$(grep -oE "$p" "$f" || true)
      if [ -z "$hits" ]; then
        echo "STALE MSRV $f: no statement matches $p"
        status=1
        continue
      fi
      while read -r v; do
        if [ "$v" != "$msrv" ]; then
          echo "STALE MSRV $f: states $v (rust-version is $msrv)"
          status=1
        fi
      done < <(grep -oE '[0-9]+\.[0-9]+\.[0-9]+' <<<"$hits")
    done
  done
done

[ "$status" -eq 0 ] && echo "ok: every install snippet and PKGBUILD names $want, every MSRV statement names $msrv"
exit $status
