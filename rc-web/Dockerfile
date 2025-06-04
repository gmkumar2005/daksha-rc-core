# Multi-stage Dockerfile for rc-web with cargo-chef optimization

# Chef stage - prepare cargo-chef with OpenSSL dependencies
FROM lukemathwalker/cargo-chef:latest-rust-1.86.0-alpine as chef

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
FROM chef AS planner
# Copy the entire workspace to analyze dependencies
COPY . .
# Generate recipe.json for dependency caching
RUN cargo chef prepare --recipe-path recipe.json --bin rc-web

# Cacher stage - build dependencies only
FROM chef AS cacher
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies (this layer will be cached)
RUN cargo chef cook --release --recipe-path recipe.json --bin rc-web

# Builder stage - build the application
FROM chef AS builder
# Copy the entire source code
COPY . .
# Copy cached dependencies from cacher stage
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

# Build the rc-web binary
RUN cargo build --release --bin rc-web

# Runtime stage - minimal production image
FROM debian:bookworm-slim AS runtime

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

# Add health check with database connectivity
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8000/readyz || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Run the application
CMD ["rc-web"]