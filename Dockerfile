# Multi-stage build for TRX Tracker
FROM rust:1.70 as rust-builder

WORKDIR /app

# Copy Rust source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY config ./config

# Build the Rust application
RUN cargo build --release

# Node.js stage for building admin UI
FROM node:18-alpine as node-builder

WORKDIR /app

# Copy admin UI source
COPY admin-ui/package.json admin-ui/pnpm-lock.yaml ./
RUN npm install -g pnpm && pnpm install

COPY admin-ui ./
RUN pnpm run build

# Final runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false trontracker

# Create app directory
WORKDIR /app

# Copy built Rust binary
COPY --from=rust-builder /app/target/release/tron-tracker /usr/local/bin/

# Copy built admin UI
COPY --from=node-builder /app/dist ./admin-ui/dist

# Copy configuration
COPY config ./config

# Create necessary directories
RUN mkdir -p /app/logs /app/data && \
    chown -R trontracker:trontracker /app

# Switch to app user
USER trontracker

# Expose ports
EXPOSE 8080 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Start the application
CMD ["tron-tracker"]

