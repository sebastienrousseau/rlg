<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Deprecations

What rlg still accepts but will stop accepting, under the policy in
[docs/POLICIES.md](docs/POLICIES.md#deprecation): each entry keeps
working, with a warning, until its removal date, and changes in the
first release on or after that date, never in the release that
announced it.

| Deprecated | Replacement | Deprecated in | Removal no earlier than |
| :--- | :--- | :--- | :--- |
| `rlg-mcp` started with neither `--root` nor `RLG_MCP_ROOT`: the tools read under the current directory, with a warning on standard error | Pass `--root <dir>` or set `RLG_MCP_ROOT` | 0.0.21 (2026-11-28) | 2027-02-26: exits with an error |
