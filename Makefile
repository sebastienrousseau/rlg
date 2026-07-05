# Makefile using cargo for managing builds and dependencies in a Rust project.

# Default target executed when no arguments are given to make.
.PHONY: all
all: help ## Display this help.

# Local verification pipeline. Runs everything a contributor needs
# to pass locally before opening a PR — fmt, clippy, tests,
# semver-checks, deny, vet.
.PHONY: verify
verify: ## Run the full local pre-PR verification pipeline.
	@echo "▶ cargo fmt --check"
	@cargo fmt --check
	@echo "▶ cargo clippy --workspace --all-features --tests --benches -- -D warnings"
	@cargo clippy --workspace --all-features --tests --benches -- -D warnings
	@echo "▶ cargo test --workspace --all-features"
	@cargo test --workspace --all-features
	@echo "▶ cargo semver-checks --workspace --exclude rlg-ebpf"
	@cargo semver-checks check-release --workspace --exclude rlg-ebpf
	@echo "▶ cargo deny check"
	@cargo deny check
	@echo "▶ cargo vet check"
	@cargo vet check
	@echo "✓ Local verification passed."

# Build the project including all workspace members.
.PHONY: build
build: ## Build the project.
	@echo "Building all project components..."
	@cargo build --all

# Lint the project with stringent rules using Clippy, install Clippy if not present.
.PHONY: lint
lint: ensure-clippy ## Lint the project with Clippy.
	@echo "Linting with Clippy..."
	@cargo clippy --all-features --all-targets --all -- \
		--deny clippy::dbg_macro --deny clippy::unimplemented --deny clippy::todo --deny warnings \
		--deny missing_docs --deny broken_intra_doc_links --forbid unused_must_use --deny clippy::result_unit_err

# Run all unit and integration tests in the project.
.PHONY: test
test: ## Run tests for the project.
	@echo "Running tests..."
	@cargo test

# Check the project for errors without producing outputs.
.PHONY: check
check: ## Check the project for errors without producing outputs.
	@echo "Checking code formatting..."
	@cargo check

# Format all code in the project according to rustfmt's standards, install rustfmt if not present.
.PHONY: format
format: ensure-rustfmt ## Format the code.
	@echo "Formatting all project components..."
	@cargo fmt --all

# Check code formatting without making changes, with verbose output, install rustfmt if not present.
.PHONY: format-check-verbose
format-check-verbose: ensure-rustfmt ## Check code formatting with verbose output.
	@echo "Checking code format with verbose output..."
	@cargo fmt --all -- --check --verbose

# Apply fixes to the project using cargo fix, install cargo-fix if necessary.
.PHONY: fix
fix: ensure-cargo-fix ## Automatically fix Rust lint warnings using cargo fix.
	@echo "Applying cargo fix..."
	@cargo fix --all

# Use cargo-deny to check for security vulnerabilities, licensing issues, and more, install if not present.
.PHONY: deny
deny: ensure-cargo-deny ## Run cargo deny checks.
	@echo "Running cargo deny checks..."
	@cargo deny check

# Verify a release's SBOM + sigstore signature.
# Usage: make verify-release TAG=v0.1.0
.PHONY: verify-release
verify-release: ensure-cosign ## Verify a release's sigstore-signed SBOMs (TAG=<vX.Y.Z> required).
	@if [ -z "$(TAG)" ]; then echo "Usage: make verify-release TAG=<vX.Y.Z>"; exit 1; fi
	@echo "Verifying release $(TAG)..."
	@BASE="https://github.com/sebastienrousseau/rlg/releases/download/$(TAG)"; \
	tmpdir="$$(mktemp -d)"; \
	cd "$$tmpdir" && \
	for f in sbom.spdx.json sbom.cyclonedx.json; do \
		curl -sfLO "$${BASE}/$${f}"          && \
		curl -sfLO "$${BASE}/$${f}.sig"      && \
		curl -sfLO "$${BASE}/$${f}.crt"      && \
		cosign verify-blob                    \
			--certificate "$${f}.crt"        \
			--signature   "$${f}.sig"        \
			--certificate-identity-regexp 'https://github.com/sebastienrousseau/rlg/.github/workflows/release.yml@refs/tags/v[0-9]+.*' \
			--certificate-oidc-issuer     https://token.actions.githubusercontent.com \
			"$${f}"; \
	done
	@echo "Release $(TAG) verified."

# Check for outdated dependencies only for the root package, install cargo-outdated if necessary.
.PHONY: outdated
outdated: ensure-cargo-outdated ## Check for outdated dependencies for the root package only.
	@echo "Checking for outdated dependencies..."
	@cargo outdated --root-deps-only

# Installation checks and setups
.PHONY: ensure-clippy ensure-rustfmt ensure-cargo-fix ensure-cargo-deny ensure-cargo-outdated
ensure-clippy:
	@cargo clippy --version || rustup component add clippy

ensure-rustfmt:
	@cargo fmt --version || rustup component add rustfmt

ensure-cargo-fix:
	@cargo fix --version || rustup component add rustfix

ensure-cargo-deny:
	@command -v cargo-deny || cargo install cargo-deny

ensure-cargo-outdated:
	@command -v cargo-outdated || cargo install cargo-outdated

ensure-cosign:
	@command -v cosign || { echo "cosign not installed. See pkg/VERIFY.md"; exit 1; }

# Help target to display callable targets and their descriptions.
.PHONY: help
help: ## Display this help.
	@echo "Usage: make [target]..."
	@echo ""
	@echo "Targets:"
	@awk 'BEGIN {FS = ":.*?##"} /^[a-zA-Z_-]+:.*?##/ {printf "  %-30s %s\n", $$1, $$2}' $(MAKEFILE_LIST)