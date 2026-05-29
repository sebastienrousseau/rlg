# Security Policy

## Supported Versions

Only the latest published `0.0.x` series receives security patches.

| Version | Supported |
| ------- | --------- |
| `0.0.9` | yes       |
| `< 0.0.9` | no      |

## Reporting a Vulnerability

Report security issues privately. Do **not** open a public GitHub issue.

- Email: `sebastian.rousseau@gmail.com` with subject `[SECURITY][rlg]`
- Or use GitHub's "Report a vulnerability" link under the **Security** tab

Include a minimal reproducer, the affected version, and your assessment of impact. Expect an acknowledgement within 5 business days.

## Supply Chain & Provenance

- Releases are built from signed commits on `main`. See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the signing policy.
- `cargo audit` runs as part of CI via the centralized [`sebastienrousseau/pipelines/security.yml`](https://github.com/sebastienrousseau/pipelines) workflow.
- `unsafe_code = "deny"` is enforced project-wide. The only exception is the macOS `os_log` FFI in `src/sink.rs`, which is documented and gated behind `#[cfg(target_os = "macos")]`.

## Verifying a Release

```bash
# Verify the latest tag's signature
git verify-tag v0.0.9

# Verify HEAD on main is signed
git log --show-signature -1 main
```
