# Security Policy

## Supported Versions

Only the latest published `0.0.x` series receives security patches.

| Version | Supported |
| ------- | --------- |
| `0.0.11` | yes      |
| `< 0.0.11` | no     |

## Reporting a Vulnerability

Report security issues privately. Do **not** open a public GitHub issue.

- Email: `sebastian.rousseau@gmail.com` with subject `[SECURITY][rlg]`
- Or use GitHub's "Report a vulnerability" link under the **Security** tab

Include a minimal reproducer, the affected version, and your assessment of impact. Expect an acknowledgement within 5 business days.

## Supply Chain & Provenance

- Releases are built from signed commits on `main`. See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the signing policy.
- Every release ships **SBOMs in SPDX and CycloneDX formats**, and every SBOM is signed keyless via **sigstore + GitHub Actions OIDC**. See [`pkg/VERIFY.md`](pkg/VERIFY.md) for the consumer verification runbook.
- `cargo audit` runs as part of CI via the centralized [`sebastienrousseau/pipelines/security.yml`](https://github.com/sebastienrousseau/pipelines) workflow. `cargo deny check` and `cargo semver-checks` gate every PR.
- **Miri**, **Loom**, **proptest**, and **Kani** provide layered correctness coverage — see the ADR series under `docs/adr/`.
- `unsafe_code = "deny"` is enforced project-wide. The only exception is the macOS `os_log` FFI in `src/sink.rs`, which is documented and gated behind `#[cfg(target_os = "macos")]`.

## Sigstore Trust Root

The certificate identity for every release signature is pinned to:

- **Workflow:** `https://github.com/sebastienrousseau/rlg/.github/workflows/release.yml`
- **Ref pattern:** `refs/tags/v[0-9]+.*` (only tagged releases sign)
- **OIDC issuer:** `https://token.actions.githubusercontent.com`

If a signature verifies against any *other* identity, treat it as
untrusted regardless of the source. The full verification command
is in [`pkg/VERIFY.md`](pkg/VERIFY.md).

## Verifying a Release

```bash
# 1. Verify the tag commit is signed.
git verify-tag v0.0.11

# 2. Verify HEAD on main is signed.
git log --show-signature -1 main

# 3. Verify the release SBOMs (see pkg/VERIFY.md for full runbook).
cosign verify-blob \
  --certificate sbom.spdx.json.crt \
  --signature sbom.spdx.json.sig \
  --certificate-identity-regexp \
      'https://github.com/sebastienrousseau/rlg/.github/workflows/release.yml@refs/tags/v[0-9]+.*' \
  --certificate-oidc-issuer \
      https://token.actions.githubusercontent.com \
  sbom.spdx.json
```
