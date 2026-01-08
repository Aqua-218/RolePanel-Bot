# Build stage
FROM rust:1.88-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests and vendor directory
COPY Cargo.toml Cargo.lock ./
COPY .cargo ./.cargo
COPY vendor ./vendor

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application using vendored dependencies (offline)
RUN cargo build --release --offline

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/role-panel-bot /app/role-panel-bot

# Copy migrations for runtime execution
COPY --from=builder /app/migrations /app/migrations

# Create non-root user
RUN useradd -r -s /bin/false bot
USER bot

# Expose health port
EXPOSE 8080

# Run the bot
CMD ["/app/role-panel-bot"]
