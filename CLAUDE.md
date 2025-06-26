# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based embeddings service that provides sentence embeddings via a REST API. The service uses `rust-bert` with the all-MiniLM-L6-v2 model to generate 384-dimensional embeddings. It's built with Axum for the web framework and designed for high-performance NLP workloads.

## Development Commands

### Building and Running
- `cargo build` - Build in debug mode
- `cargo build --release` - Build optimized release version
- `cargo run` - Run the service locally (listens on port 3000)
- `cargo test` - Run unit tests
- `cargo check` - Quick syntax and type checking

### Docker Development
- `docker build -t embeddings-service .` - Build Docker image
- `docker run -p 3000:3000 embeddings-service` - Run container
- `docker-compose up -d` - Start with docker-compose
- `docker-compose logs -f embeddings-service` - View logs
- `docker-compose down` - Stop services

### Production Deployment
- `docker-compose --profile production up -d` - Deploy with nginx reverse proxy
- `docker build --build-arg TORCH_CUDA_VERSION=cu124 -t embeddings-service-gpu .` - Build with GPU support

## Architecture

### Core Components
- **Main Application** (`src/main.rs`): Axum-based REST API server with CORS support
- **Model Management**: Uses `rust-bert` SentenceEmbeddingsModel with async mutex protection
- **Request/Response Types**: Structured with serde for JSON serialization
- **State Management**: Arc-wrapped AppState containing model instances

### API Endpoints
- `GET /health` - Health check with available models list
- `POST /embeddings` - Batch embedding generation (JSON body with texts array)
- `GET /embeddings?text=<text>` - Single text embedding via query parameter

### Model Configuration
- Default model: all-MiniLM-L6-v2 (384-dimensional embeddings)
- Models cached in `~/.cache/.rustbert` or `/app/.cache/.rustbert` in Docker
- Additional models can be added in `AppState::new()` method

### Performance Characteristics
- LibTorch backend with automatic download during build (~500MB for CPU, ~2GB for CUDA)
- Thread-safe model access with async mutex
- 2-4x faster text generation compared to Python equivalents
- Memory-efficient with Docker multi-stage builds (~200MB final image)

### Docker Architecture
- Multi-stage build: builder stage (Rust compilation) + runtime stage (Debian slim)
- Non-root user execution for security
- Health checks and resource limits configured
- Model cache persistence via Docker volumes
- Optional nginx reverse proxy for production

## Environment Variables
- `PORT` - Server port (default: 3000)
- `RUST_LOG` - Logging level (default: info)
- `TORCH_CUDA_VERSION` - CUDA version for GPU builds (e.g., cu124)

## Testing
- Unit tests in `src/main.rs` using `axum-test`
- Test endpoints: health check and embeddings generation
- Models must be loaded for tests (may take time on first run)

## Performance Notes
- First build downloads LibTorch (5-15 minutes)
- Model loading happens at startup (check logs for "Models loaded successfully!")
- Batch processing recommended for multiple texts
- GPU acceleration available with CUDA builds (6x improvement)
- Memory requirements: 512MB minimum, 2GB recommended for production