FROM rust:1.87-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r embeddings && useradd -r -g embeddings embeddings
RUN mkdir -p /app/.cache/fastembed && chown -R embeddings:embeddings /app

WORKDIR /app
COPY --from=builder /app/target/release/embeddings-service .
RUN chown embeddings:embeddings embeddings-service

USER embeddings
ENV RUST_LOG=info
ENV PORT=9000

EXPOSE 9000
CMD ["./embeddings-service"]
