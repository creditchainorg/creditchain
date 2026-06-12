# syntax=docker/dockerfile:1.7
#
# Multi-stage build for creditchain-faucet — yields a ~20MB distroless image.
# Build context is `creditchain/services/faucet/`, not the full repo.

FROM rust:1.93-slim-bookworm AS builder

WORKDIR /build

# Install minimal toolchain bits we need for k256 + reqwest+rustls.
RUN apt-get update \
 && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Cache deps separately for fast incremental builds. Copy the committed
# Cargo.lock and build `--locked` so the image is reproducible and immune to
# upstream version drift / transient resolver hiccups.
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release --locked --bin creditchain-faucet \
 && cp /build/target/release/creditchain-faucet /usr/local/bin/

# ── runtime stage ───────────────────────────────────────────────────────────
# Docker Hub base (reachable everywhere). Distroless via gcr.io is leaner but
# is not reachable in all build environments; debian-slim + a non-root user is
# the portable choice and still small.
FROM debian:bookworm-slim

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 --no-create-home faucet

COPY --from=builder /usr/local/bin/creditchain-faucet /usr/local/bin/creditchain-faucet

# Expose the default port; override via FAUCET_BIND.
EXPOSE 8080

USER faucet
ENTRYPOINT ["/usr/local/bin/creditchain-faucet"]
