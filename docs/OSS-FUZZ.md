<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# OSS-Fuzz Onboarding for `rlg`

> **Status:** Draft. Submission to `google/oss-fuzz` is pending
> maintainer sign-off. See `docs/adr/0002-fuzz-strategy.md` for the
> strategy that motivates this integration.

## What OSS-Fuzz gives us

- Continuous fuzzing across all four fuzz targets defined in
  `fuzz/fuzz_targets/`.
- Multiple sanitiser passes (ASan, UBSan, MSan) at no cost to the
  project.
- Crash reports filed as private GitHub Security Advisories with a
  reproducer artefact attached.
- Corpus retention and minimisation managed by Google's
  infrastructure.

Prerequisite: fuzz targets must build on **nightly** with
`cargo fuzz build --release`. Verified locally before submission.

## Submission checklist

- [ ] Draft `projects/rlg/project.yaml` in a fork of
  `google/oss-fuzz`:

  ```yaml
  homepage: "https://github.com/sebastienrousseau/rlg"
  main_repo: "https://github.com/sebastienrousseau/rlg.git"
  language: rust
  primary_contact: "sebastian.rousseau@gmail.com"
  auto_ccs:
    - "sebastian.rousseau@gmail.com"
  sanitizers:
    - address
    - undefined
    - memory
  fuzzing_engines:
    - libfuzzer
  ```

- [ ] Draft `projects/rlg/Dockerfile`:

  ```dockerfile
  FROM gcr.io/oss-fuzz-base/base-builder-rust
  RUN git clone --depth 1 https://github.com/sebastienrousseau/rlg.git rlg
  WORKDIR /src/rlg
  COPY build.sh $SRC/
  ```

- [ ] Draft `projects/rlg/build.sh`:

  ```bash
  #!/bin/bash -eu
  cd fuzz
  cargo fuzz build --release
  for target in parse_record log_format_from_str config_load redact_scrub; do
    cp target/x86_64-unknown-linux-gnu/release/"$target" "$OUT/"
  done
  ```

- [ ] Verify the fork builds locally with OSS-Fuzz's helper:

  ```bash
  python infra/helper.py build_image rlg
  python infra/helper.py build_fuzzers --sanitizer address rlg
  python infra/helper.py check_build rlg
  ```

- [ ] Open the PR against `google/oss-fuzz` with title
  `Project: rlg` and body referencing this document, ADR 0002, and
  the workspace `SECURITY.md`.

## After acceptance

- Crashes surface as private GitHub Security Advisories in this
  repo. Triage within one working day per ADR 0002.
- The corpus lives at `gs://rlg-corpus.clusterfuzz-external.appspot.com`
  and is publicly readable. Local sync:

  ```bash
  gsutil -m rsync gs://rlg-corpus.clusterfuzz-external.appspot.com/libFuzzer/rlg_parse_record fuzz/corpus/parse_record
  ```

- The dashboard is at
  <https://oss-fuzz.com/testcase?project=rlg>.

## Local reproducer for an OSS-Fuzz crash

Once a crash lands in a security advisory with a
`clusterfuzz-testcase-*` attachment:

```bash
cargo install cargo-fuzz --locked
cd fuzz
cargo +nightly fuzz run <target-name> <path-to-testcase>
```

Fix the underlying bug, add the test case to
`crates/<crate>/tests/`, land the fix, and re-run the fuzz smoke
CI to confirm the regression seed no longer reproduces.
