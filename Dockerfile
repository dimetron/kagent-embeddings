# Multi-stage build for optimized production image
FROM rust:1.87-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    build-essential \
    curl \
    wget \
    unzip \
    libc6-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached unless Cargo files change)
# The download-libtorch feature will automatically download LibTorch (~500MB-2GB)
RUN cargo build --release && rm -rf src target/release/deps/embeddings*

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage with minimal dependencies
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libgomp1 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security
RUN groupadd -r embeddings && useradd -r -g embeddings embeddings

# Create directory for models and set permissions
RUN mkdir -p /app/.cache/.rustbert && \
    chown -R embeddings:embeddings /app

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/embeddings-service .

# Change ownership to non-root user
RUN chown embeddings:embeddings embeddings-service

# Switch to non-root user
USER embeddings

# Set environment variables
ENV RUST_LOG=info
ENV PORT=3000

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the application
CMD ["./embeddings-service"]