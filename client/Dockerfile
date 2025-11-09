# Build stage
FROM rust:1.85-slim as builder

WORKDIR /build

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY sim ./sim
COPY client ./client

# Build the client
RUN cargo build --release -p client

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /build/target/release/client /app/client

# Set default server URL (can be overridden)
ENV SERVER_URL=https://evo-islands.rackspace.koski.co

# Run the client
CMD ["/app/client"]
