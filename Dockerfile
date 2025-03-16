# Build stage
FROM rust:1.85.0-slim-bookworm as builder

WORKDIR /app

# Install OpenSSL development packages and pkg-config for the build process
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy over manifests
COPY Cargo.toml Cargo.lock ./

# Copy source files
COPY *.rs ./

# Build the application with release optimizations
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install dependencies for runtime
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/discord-pushover-notifier .

# process based healthcheck since theres no http server inside, and i dont want to run a http server just for healthchecks
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
  CMD ps aux | grep discord-pushover-notifier | grep -v grep || exit 1

# Set the binary as the entrypoint
ENTRYPOINT ["./discord-pushover-notifier"]