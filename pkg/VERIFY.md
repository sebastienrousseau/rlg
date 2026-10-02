<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Verifying an `rlg` Release

Every tagged release on <https://github.com/sebastienrousseau/rlg/releases>
ships with:

- `sbom.spdx.json` — SPDX SBOM (industry default).
- `sbom.cyclonedx.json` — CycloneDX SBOM (EU CRA baseline).
- `<file>.sigstore.json` for each SBOM: a keyless Sigstore bundle
  holding the signature, the signing certificate and the
  transparency-log proof. Releases up to v0.0.14 shipped
  `<file>.sig` + `<file>.crt` instead; see
  [Releases up to v0.0.14](#releases-up-to-v0014).

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
go install github.com/sigstore/cosign/v3/cmd/cosign@latest
```

Bundles need cosign v3 or later.

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

for f in sbom.spdx.json sbom.cyclonedx.json; do
  curl -sLO "${BASE}/${f}"
  curl -sLO "${BASE}/${f}.sigstore.json"
done
```

Verify:

```bash
for f in sbom.spdx.json sbom.cyclonedx.json; do
  cosign verify-blob \
    --bundle "${f}.sigstore.json" \
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

### Releases up to v0.0.14

Those releases carry a detached signature and certificate per SBOM.
Download `${f}.sig` and `${f}.crt` instead of the bundle, and pass
`--certificate "${f}.crt" --signature "${f}.sig"` in place of
`--bundle`; the identity and issuer flags are the same.

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

## Verify a signed tag

Release tags are signed with the maintainer's SSH keys, listed in
[`KEYS.asc`](../KEYS.asc) as allowed-signers lines. In a clone:

```bash
grep '^sebastian.rousseau@gmail.com namespaces=' KEYS.asc > allowed_signers
git -c gpg.ssh.allowedSignersFile=allowed_signers verify-tag v0.0.14
```

A good tag prints `Good "git" signature for
sebastian.rousseau@gmail.com with ED25519 key SHA256:...`; the
fingerprint must be one `KEYS.asc` lists. `git verify-commit` checks a
commit the same way. Commits from 2024 carry the OpenPGP key at the
end of `KEYS.asc` (`gpg --import KEYS.asc`), and merge commits made on
github.com carry GitHub's key from <https://github.com/web-flow.gpg>.

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
sbom.<fmt>.json + .sigstore.json on the release page
```

Break any link in that chain and verification fails. That is the
guarantee.
