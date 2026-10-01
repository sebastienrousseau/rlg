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

[ "$status" -eq 0 ] && echo "ok: every install snippet and PKGBUILD names $want"
exit $status
