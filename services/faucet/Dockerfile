# syntax=docker/dockerfile:1.7
#
# Multi-stage build for creditchain-faucet — yields a ~20MB distroless image.
# Build context is `creditchain/services/faucet/`, not the full repo.

FROM rust:1.83-slim-bookworm AS builder

WORKDIR /build

# Install minimal toolchain bits we need for k256 + reqwest+rustls.
RUN apt-get update \
 && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Cache deps separately for fast incremental builds.
COPY Cargo.toml ./
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release --bin creditchain-faucet \
 && cp /build/target/release/creditchain-faucet /usr/local/bin/

# ── runtime stage ───────────────────────────────────────────────────────────
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/local/bin/creditchain-faucet /usr/local/bin/creditchain-faucet

# Expose the default port; override via FAUCET_BIND.
EXPOSE 8080

USER nonroot:nonroot
ENTRYPOINT ["/usr/local/bin/creditchain-faucet"]
