# Multi-stage build for Rust web application using cargo-chef
# Stage 1: Plan dependencies
FROM ghcr.io/daksha-rc/cargo-chef-base:latest AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin rc-web

# Stage 2: Build dependencies (this layer will be cached)
FROM ghcr.io/daksha-rc/cargo-chef-base:latest AS cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin rc-web

# Stage 3: Build application
FROM ghcr.io/daksha-rc/cargo-chef-base:latest AS builder
WORKDIR /app
COPY . .
# Copy the built dependencies from cacher stage
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release --bin rc-web

# Stage 4: Runtime image
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create app user
RUN groupadd -r appuser && useradd -r -g appuser -s /bin/false appuser

# Copy the binary from builder stage
COPY --from=builder /app/target/release/rc-web /usr/local/bin/rc-web

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