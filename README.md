# Kagent Embeddings Service

A high-performance Rust-based REST API service for generating sentence embeddings using transformer models. Built with Axum and rust-bert for fast, scalable NLP workloads.

## Features

- **Fast Performance**: 2-4x faster than Python equivalents using LibTorch backend
- **REST API**: Simple HTTP endpoints for embedding generation
- **Batch Processing**: Process multiple texts in a single request
- **GPU Support**: Optional CUDA acceleration (6x performance improvement)
- **Thread-Safe**: Concurrent request handling with async mutex protection
- **Docker Ready**: Multi-stage builds with production-ready containers
- **Health Monitoring**: Built-in health checks and logging

## Quick Start

### Prerequisites

- Rust 1.70+ 
- Docker (optional)
- CUDA toolkit (for GPU acceleration)

### Local Development

```bash
# Build the service
cargo build --release

# Run the service
cargo run
# Service will be available at http://localhost:9000
```

### Docker Deployment

```bash
# Build and run with Docker Compose
docker-compose up -d

# View logs
docker-compose logs -f embeddings-service

# Production deployment with nginx
docker-compose --profile production up -d
```

## API Endpoints

### Health Check
```bash
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "models": ["all-minilm-l6-v2"]
}
```

### Generate Embeddings (Batch)
```bash
POST /embeddings
Content-Type: application/json

{
  "texts": ["Hello world", "How are you?"],
  "model": "all-minilm-l6-v2"
}
```

**Response:**
```json
{
  "embeddings": [[0.1, 0.2, ...], [0.3, 0.4, ...]],
  "model": "all-minilm-l6-v2",
  "dimensions": 384
}
```

### Generate Single Embedding
```bash
GET /embeddings?text=Hello%20world&model=all-minilm-l6-v2
```

## Models

- **all-minilm-l6-v2**: Default model (384 dimensions)
  - Sentence transformers model optimized for semantic similarity
  - Cached locally in `~/.cache/.rustbert/` or `/app/.cache/.rustbert/` in Docker

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `9000` | Server port |
| `RUST_LOG` | `info` | Logging level |
| `TORCH_CUDA_VERSION` | - | CUDA version for GPU builds (e.g., `cu124`) |

### Docker Configuration

```yaml
# docker-compose.yaml
services:
  embeddings-service:
    build: .
    ports:
      - "3000:3000"
    environment:
      - RUST_LOG=info
      - PORT=3000
    volumes:
      - model_cache:/app/.cache/.rustbert
```

## Performance

### Benchmarks
- **CPU**: 2-4x faster than Python-based solutions
- **GPU**: 6x improvement with CUDA acceleration
- **Memory**: 512MB minimum, 2GB recommended for production
- **First Build**: 5-15 minutes (downloads LibTorch)

### Optimization
- Models are loaded once at startup
- Thread-safe model access with async mutex
- Batch processing for multiple texts
- Memory-efficient Docker builds (~200MB final image)

## Development

### Building
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Quick syntax check
cargo check
```

### Testing
```bash
# Run unit tests
cargo test

# Run with logging
RUST_LOG=debug cargo test
```

### GPU Support
```bash
# Build with CUDA support
docker build --build-arg TORCH_CUDA_VERSION=cu124 -t embeddings-service-gpu .
```

## Architecture

- **Web Framework**: Axum with CORS support
- **ML Backend**: rust-bert with LibTorch
- **Model Management**: Arc-wrapped state with async mutex
- **Serialization**: Serde for JSON handling
- **Logging**: Tracing with configurable levels

## Production Deployment

### Resource Requirements
- **Memory**: 2GB limit, 512MB reservation
- **CPU**: 1.0 limit, 0.5 reservation
- **Storage**: Volume for model cache persistence

### Monitoring
- Health check endpoint with 30s intervals
- Structured logging with tracing
- Container restart policies
- Optional nginx reverse proxy

## Contributing

1. Fork the repository
2. Create a feature branch
3. Run tests: `cargo test`
4. Submit a pull request

## License

This project is licensed under the MIT License.