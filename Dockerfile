# Hazler Docker Image
# Multi-stage build for a minimal glibc-based image

# Build stage
FROM rust:1-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build for release with symbol stripping
ENV RUSTFLAGS="-C strip=symbols"
RUN cargo build --release --bin hazler

# Runtime stage - use Debian slim to match the glibc release binaries
FROM debian:bookworm-slim

# Install CA certificates and the OpenSSL runtime library
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder
COPY --from=builder /app/target/release/hazler /usr/local/bin/hazler

# Create a non-root user
RUN useradd -u 1000 -m hazler && \
    chown hazler:hazler /usr/local/bin/hazler

USER hazler
WORKDIR /home/hazler

# Set entrypoint
ENTRYPOINT ["hazler"]
CMD ["--help"]

# Build argument allows the CI pipeline to stamp the correct version at build time
ARG BUILD_VERSION=dev

# Labels
LABEL org.opencontainers.image.title="Hazler"
LABEL org.opencontainers.image.description="Next-Generation Intelligent Web Crawler"
LABEL org.opencontainers.image.url="https://github.com/HazaVVIP/hazler"
LABEL org.opencontainers.image.source="https://github.com/HazaVVIP/hazler"
LABEL org.opencontainers.image.version="${BUILD_VERSION}"
LABEL org.opencontainers.image.licenses="MIT"

