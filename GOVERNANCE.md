<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Governance

## Maintainer

rlg has a single maintainer, Sebastien Rousseau
([@sebastienrousseau](https://github.com/sebastienrousseau)), who owns
every path in the repository ([`.github/CODEOWNERS`](.github/CODEOWNERS))
and decides what is merged and released.

## How decisions are made

- **Changes** arrive as pull requests into the one active release branch,
  `feat/v<next-version>`, and reach `main` through that branch's single
  release pull request. CI must be green before a merge.
- **Decisions that will be questioned later** are recorded as
  Architecture Decision Records in [`docs/adr/`](docs/adr/). A reversal
  gets a new ADR that supersedes the old one, never an edit that hides it.
- **Releases** follow the lockstep `0.0.x` scheme in
  [`AGENTS.md`](AGENTS.md): every publishable crate shares one version,
  and each release is a signed tag.

## Conduct

Everyone taking part follows the [Code of Conduct](CODE_OF_CONDUCT.md).
