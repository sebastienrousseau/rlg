#!/usr/bin/env bash
# The MCP directory manifests must name the version the workspace ships.
#
# glama.json is what Glama shows for rlg-mcp and server.json is what the
# MCP registry publishes from a tag. Both sat at 0.0.11 for a whole
# release because nothing compared them with Cargo.toml; this does.
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

want=$(sed -n 's/^version = "\([^"]*\)"/\1/p' crates/rlg-mcp/Cargo.toml | head -1)
[ -n "$want" ] || { echo "could not read crates/rlg-mcp/Cargo.toml version"; exit 1; }

status=0
check() {
  local label=$1 got=$2
  if [ "$got" != "$want" ]; then
    echo "MISMATCH $label: $got (crate is $want)"; status=1
  else
    echo "ok $label: $got"
  fi
}
check "glama.json version"        "$(jq -r .version glama.json)"
check "glama.json docker tag"     "$(jq -r '.installation.docker' glama.json | sed 's/.*://')"
check "glama.json mcpServers tag" "$(jq -r '.mcpServers.rlg.args[-1]' glama.json | sed 's/.*://')"
check "server.json version"       "$(jq -r .version server.json)"
check "server.json image tag"     "$(jq -r '.packages[0].identifier' server.json | sed 's/.*://')"
check "README lockstep line"      "$(sed -n 's/.*lockstep version `\([^`]*\)`.*/\1/p' README.md | head -1)"
exit $status
