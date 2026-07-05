<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Verifying an `rlg` Release

Every tagged release on <https://github.com/sebastienrousseau/rlg/releases>
ships with:

- `sbom.spdx.json` — SPDX SBOM (industry default).
- `sbom.cyclonedx.json` — CycloneDX SBOM (EU CRA baseline).
- `<file>.sig` + `<file>.crt` for each SBOM — keyless sigstore
  signature + certificate bundle.

This document is the consumer runbook for verifying those artefacts
end-to-end. Design rationale in
[`docs/adr/0005-sigstore-and-sbom.md`](../docs/adr/0005-sigstore-and-sbom.md).

## One-time setup

Install `cosign` from the sigstore project:

```bash
# macOS
brew install cosign

# Debian / Ubuntu
curl -sSfLO "https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64" \
  && sudo install -m0755 cosign-linux-amd64 /usr/local/bin/cosign

# Arch Linux
sudo pacman -S cosign

# Go
go install github.com/sigstore/cosign/v2/cmd/cosign@latest
```

Verify:

```bash
cosign version
```

## Verify a release SBOM

Given a release tag (`v0.1.0` in this example), download the SBOM
and its signature bundle:

```bash
TAG=v0.1.0
BASE="https://github.com/sebastienrousseau/rlg/releases/download/${TAG}"

# SPDX
curl -sLO "${BASE}/sbom.spdx.json"
curl -sLO "${BASE}/sbom.spdx.json.sig"
curl -sLO "${BASE}/sbom.spdx.json.crt"

# CycloneDX
curl -sLO "${BASE}/sbom.cyclonedx.json"
curl -sLO "${BASE}/sbom.cyclonedx.json.sig"
curl -sLO "${BASE}/sbom.cyclonedx.json.crt"
```

Verify:

```bash
for f in sbom.spdx.json sbom.cyclonedx.json; do
  cosign verify-blob \
    --certificate "${f}.crt" \
    --signature "${f}.sig" \
    --certificate-identity-regexp \
        "https://github.com/sebastienrousseau/rlg/.github/workflows/release.yml@refs/tags/v[0-9]+.*" \
    --certificate-oidc-issuer \
        https://token.actions.githubusercontent.com \
    "$f"
done
```

A successful verification prints `Verified OK` per file. Any other
outcome — mismatched signature, wrong issuer, revoked certificate,
non-matching identity — is a **stop-the-line** event: do not consume
the artefact.

## What each certificate identity means

- `certificate-identity-regexp` pinned to
  `https://github.com/sebastienrousseau/rlg/.github/workflows/release.yml@refs/tags/v*`
  means the signing job was **this repository's release workflow**,
  triggered by a **`v`-prefixed tag push**. Any other identity —
  including a workflow file at a non-tag ref, or a fork — fails the
  check.
- `certificate-oidc-issuer` pinned to
  `https://token.actions.githubusercontent.com` means the OIDC
  token came from **GitHub Actions**, not another IdP.

## Compare an SBOM against your Cargo.lock

The SBOMs enumerate every transitive dependency the release was
built against. To confirm your consumer build resolves to the same
set:

```bash
cargo audit --db-path /tmp/rustsec --file Cargo.lock \
  --json | jq '.[].dependencies[]' | sort -u > my.deps

jq -r '.packages[] | "\(.name) \(.version)"' sbom.spdx.json \
  | sort -u > release.deps

diff <(sort my.deps) <(sort release.deps)
```

Any diff means your build's dependency closure differs from the
released one — either because you enabled different features or
because a transitive dep resolved to a different version.

## Trust chain summary

```text
sigstore (Fulcio CA, transparency log)
    │
    ├─ certifies OIDC identity of the signer
    │
    ▼
GitHub Actions OIDC token
    │
    ├─ issued only to workflows running in
    │  sebastienrousseau/rlg on a v* tag ref
    │
    ▼
release.yml at tags/v<version>
    │
    ├─ generates sbom.{spdx,cyclonedx}.json
    ├─ signs each with `cosign sign-blob --yes`
    │
    ▼
sbom.<fmt>.json + .sig + .crt on the release page
```

Break any link in that chain and verification fails. That is the
guarantee.
