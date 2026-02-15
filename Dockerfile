# Hazler Docker Image
# Multi-stage build for minimal final image size (<50MB target)

# Build stage - use Rust Alpine for smaller base
FROM rust:1.75-alpine as builder

# Install build dependencies (musl for static linking)
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static

# Create a new empty shell project
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY benches ./benches

# Build for release with optimizations
# - Strip debug symbols
# - Use musl for static linking
# - Optimize for size
ENV RUSTFLAGS="-C target-feature=+crt-static -C strip=symbols -C opt-level=z"
RUN cargo build --release --bin hazler --target x86_64-unknown-linux-musl

# Runtime stage - use Alpine for minimal footprint
FROM alpine:3.19

# Install only CA certificates for HTTPS (no other runtime deps needed with musl)
RUN apk add --no-cache ca-certificates

# Copy the statically-linked binary from builder
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/hazler /usr/local/bin/hazler

# Create a non-root user
RUN adduser -D -u 1000 hazler && \
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
LABEL org.opencontainers.image.version="0.2.0"
LABEL org.opencontainers.image.licenses="MIT"

