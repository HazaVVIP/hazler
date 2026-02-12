# Hazler Docker Image
# Multi-stage build for minimal final image size

# Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build for release
RUN cargo build --release --bin hazler

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Copy the built binary from builder
COPY --from=builder /app/target/release/hazler /usr/local/bin/hazler

# Create a non-root user
RUN useradd -m -u 1000 hazler && \
    chown hazler:hazler /usr/local/bin/hazler

USER hazler
WORKDIR /home/hazler

# Set entrypoint
ENTRYPOINT ["hazler"]
CMD ["--help"]

# Labels
LABEL org.opencontainers.image.title="Hazler"
LABEL org.opencontainers.image.description="Next-Generation Intelligent Web Crawler"
LABEL org.opencontainers.image.url="https://github.com/HazaVVIP/hazler"
LABEL org.opencontainers.image.source="https://github.com/HazaVVIP/hazler"
LABEL org.opencontainers.image.version="0.1.0"
LABEL org.opencontainers.image.licenses="MIT"
