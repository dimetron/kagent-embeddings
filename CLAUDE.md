# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based embeddings service that provides sentence embeddings via a REST API. The service uses `fastembed-rs` with the all-MiniLM-L6-v2 model to generate 384-dimensional embeddings. It's built with Axum for the web framework and designed for high-performance NLP workloads.

## Development Commands

### Building and Running
- `cargo build` - Build in debug mode
- `cargo build --release` - Build optimized release version
- `cargo run` - Run the service locally (listens on port 9000)
- `cargo test` - Run unit tests
- `cargo check` - Quick syntax and type checking

### Docker Development
- `docker build -t embeddings-service .` - Build Docker image
- `docker run -p 9000:9000 embeddings-service` - Run container
- `docker-compose up -d` - Start with docker-compose
- `docker-compose logs -f embeddings-service` - View logs
- `docker-compose down` - Stop services

### Production Deployment
- `docker-compose --profile production up -d` - Deploy with nginx reverse proxy
- `docker build -f Dockerfile.cuda -t embeddings-service-gpu .` - Build with GPU support

## Architecture

### Core Components
- **Main Application** (`src/main.rs`): Axum-based REST API server with CORS support
- **Model Management**: Uses `fastembed-rs` TextEmbedding with async mutex protection
- **Request/Response Types**: Structured with serde for JSON serialization
- **State Management**: Arc-wrapped AppState containing model instances

### API Endpoints
- `GET /health` - Health check with available models list
- `POST /embeddings` - Batch embedding generation (JSON body with texts array)
- `GET /embeddings?text=<text>` - Single text embedding via query parameter

### Model Configuration
- Default model: all-MiniLM-L6-v2 (384-dimensional embeddings)
- Models cached in `~/.cache/fastembed` or `/app/.cache/fastembed` in Docker
- Additional models can be added in `AppState::new()` method

### Performance Characteristics
- ONNX Runtime backend with automatic model download during first run
- Thread-safe model access with async mutex
- Fast inference with optimized ONNX models
- Memory-efficient with Docker multi-stage builds

### Docker Architecture
- Multi-stage build: builder stage (Rust compilation) + runtime stage (Debian slim)
- Non-root user execution for security
- Health checks and resource limits configured
- Model cache persistence via Docker volumes
- Optional nginx reverse proxy for production

## Environment Variables
- `PORT` - Server port (default: 9000)
- `RUST_LOG` - Logging level (default: info)

## Testing
- Unit tests in `src/main.rs` using `axum-test`
- Test endpoints: health check and embeddings generation
- Models must be loaded for tests (may take time on first run)

## Performance Notes
- First run downloads ONNX models (1-2 minutes depending on connection)
- Model loading happens at startup (check logs for "Models loaded successfully!")
- Batch processing recommended for multiple texts
- GPU acceleration available with CUDA builds
- Memory requirements: 256MB minimum, 1GB recommended for production