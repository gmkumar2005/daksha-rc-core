# Multi-stage Dockerfile for rc-web with reusable base image
# The base-builder stage (up to line 22 equivalent) can be built and reused independently

# Base builder stage - reusable intermediate image with all build dependencies
# This stage includes Rust toolchain, cargo-chef, and all build dependencies
FROM lukemathwalker/cargo-chef:latest-rust-1.86.0-alpine AS base-builder

ENV DEBIAN_FRONTEND=noninteractive
# Install build dependencies including OpenSSL
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    postgresql-dev \
    ca-certificates

# Set OpenSSL environment variables for static linking
ENV OPENSSL_STATIC=1
ENV OPENSSL_LIB_DIR=/usr/lib
ENV OPENSSL_INCLUDE_DIR=/usr/include
ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /app

# Planner stage - analyze dependencies
# This stage analyzes the workspace to create a dependency recipe
FROM base-builder AS planner
# Copy the entire workspace to analyze dependencies
COPY . .
# Generate recipe.json for dependency caching - this enables efficient rebuilds
RUN cargo chef prepare --recipe-path recipe.json --bin rc-web

# Cacher stage - build dependencies only
# This stage builds only dependencies, enabling excellent layer caching
FROM base-builder AS cacher
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies (this layer will be cached)
RUN cargo chef cook --release --recipe-path recipe.json --bin rc-web

# Builder stage - build the application
# This stage builds the actual application with pre-built dependencies
FROM base-builder AS builder
# Copy the entire source code
COPY . .
# Copy cached dependencies from cacher stage
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

# Build the rc-web binary
RUN cargo build --release --bin rc-web

# Runtime stage - minimal production image
FROM debian:bookworm-slim AS runtime
ENV DEBIAN_FRONTEND=noninteractive
# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN groupadd -r appuser && useradd -r -g appuser -s /bin/false appuser

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/rc-web /usr/local/bin/rc-web

# Copy static files if they exist
# COPY --from=builder /app/rc-web/static ./static 2>/dev/null || true

# Set proper permissions
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8000

# Add OCI-compliant labels for better container metadata
LABEL org.opencontainers.image.title="RC Web Application"
LABEL org.opencontainers.image.description="Daksha RC web application built with Rust"
LABEL org.opencontainers.image.source="https://github.com/daksha-rc/daksha-rc-core"
LABEL org.opencontainers.image.vendor="Daksha RC"
LABEL org.opencontainers.image.licenses="MIT"

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Run the application
CMD ["rc-web"]
