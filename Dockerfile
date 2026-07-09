# syntax=docker/dockerfile:1.7
# Dockerfile — rlg-mcp Model Context Protocol server image.
#
# The MCP server speaks JSON-RPC 2.0 over stdio so AI agents (Claude
# Desktop, Cursor, on-call/SRE tooling) can call the rlg log-stream
# tools (`tail_log`, `filter_log`, `summarize_errors`) without needing
# a Rust toolchain.
#
#   docker run -i --rm ghcr.io/sebastienrousseau/rlg-mcp
#
# Only the `rlg-mcp` workspace member (and its path deps) is built; the
# default rlg feature set pulls no journald/os_log system libraries.

FROM rust:1.96-bookworm AS build

WORKDIR /src
COPY . .

ARG SOURCE_DATE_EPOCH
ENV SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-0}
ENV RUSTFLAGS="--remap-path-prefix=/src=/build --remap-path-prefix=/usr/local/cargo=/cargo"

RUN cargo build --release --locked -p rlg-mcp

# ── Runtime stage ────────────────────────────────────────────────
FROM gcr.io/distroless/cc-debian12:nonroot

LABEL org.opencontainers.image.title="rlg-mcp" \
      org.opencontainers.image.description="Model Context Protocol server exposing rlg (RustLogs) log streams as tools for on-call / SRE agent workflows." \
      org.opencontainers.image.source="https://github.com/sebastienrousseau/rlg" \
      org.opencontainers.image.licenses="MIT OR Apache-2.0"

COPY --from=build /src/target/release/rlg-mcp /usr/local/bin/rlg-mcp

USER nonroot:nonroot
ENTRYPOINT ["/usr/local/bin/rlg-mcp"]
