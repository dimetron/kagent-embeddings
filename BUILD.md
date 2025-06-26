# Docker Deployment Guide for Embeddings Service

## Quick Start

### 1. Basic Docker Build and Run

```bash
# Build the image
docker build -t embeddings-service .

# Run the container
docker run -p 3000:3000 embeddings-service
```

### 2. Using Docker Compose (Recommended)

```bash
# Start the service
docker-compose up -d

# View logs
docker-compose logs -f embeddings-service

# Stop the service
docker-compose down
```

### 3. Production Deployment with Nginx

```bash
# Start with nginx reverse proxy
docker-compose --profile production up -d

# Service will be available on port 80
curl http://localhost/health
```

## LibTorch Dependencies

### Automatic Download (Default)
The `Cargo.toml` includes the `download-libtorch` feature which automatically downloads LibTorch during build:

```toml
rust-bert = { version = "0.22", features = ["download-libtorch"] }
```

**Important Notes:**
- **CPU version**: Downloads ~500MB LibTorch CPU library
- **CUDA version**: Set `TORCH_CUDA_VERSION=cu124` environment variable (downloads ~2GB)
- **First build**: Takes 5-15 minutes due to LibTorch download
- **Cached**: Subsequent builds are much faster as LibTorch is cached

### Build Time Considerations
```bash
# CPU build (faster, smaller)
docker build -t embeddings-service .

# GPU build (larger, requires CUDA)
docker build --build-arg TORCH_CUDA_VERSION=cu124 -t embeddings-service-gpu .
```

## Dockerfile Features

### Multi-stage Build
- **Builder stage**: Installs all build dependencies and compiles the Rust application
- **Runtime stage**: Minimal Debian image with only runtime dependencies
- **Result**: ~200MB final image vs ~2GB+ if built in single stage

### Security Features
- **Non-root user**: Application runs as `embeddings` user, not root
- **Minimal dependencies**: Only essential runtime libraries included
- **Health checks**: Built-in container health monitoring

### Performance Optimizations
- **Dependency caching**: Docker layers cache Cargo dependencies separately from source code
- **Release build**: Optimized compilation with LTO and single codegen unit
- **Minimal runtime**: Debian slim base with only necessary packages

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `3000` | Server port |
| `RUST_LOG` | `info` | Logging level (error, warn, info, debug, trace) |

## Volume Management

### Model Cache Persistence
```bash
# The docker-compose.yml includes a named volume for model cache
# This prevents re-downloading models on container restart

# View volume
docker volume ls

# Inspect volume
docker volume inspect embeddings_model_cache

# Clean up volume (will trigger model re-download)
docker volume rm embeddings_model_cache
```

### Manual Volume Mount
```bash
# Mount host directory for model cache
docker run -p 3000:3000 \
  -v ./models:/app/.cache/.rustbert \
  embeddings-service
```

## Resource Requirements

### Minimum Requirements
- **RAM**: 512MB (1GB recommended)
- **CPU**: 0.5 cores (1 core recommended)
- **Disk**: 2GB for models + application

### Production Requirements
- **RAM**: 2GB (for concurrent requests)
- **CPU**: 2+ cores for better concurrency
- **Disk**: 5GB+ (logs, models, temporary files)

## Build Optimization

### CPU-Specific Optimizations
```dockerfile
# Add to Dockerfile builder stage for target CPU optimization
ENV RUSTFLAGS="-C target-cpu=native"
```

### Multi-architecture Build
```bash
# Build for multiple architectures
docker buildx build --platform linux/amd64,linux/arm64 -t embeddings-service .
```

### Build Arguments
```dockerfile
# Add build arguments for customization
ARG RUST_VERSION=1.75
FROM rust:${RUST_VERSION}-slim as builder

ARG TARGET_CPU=x86-64
ENV RUSTFLAGS="-C target-cpu=${TARGET_CPU}"
```

## Monitoring and Logging

### Health Checks
```bash
# Check container health
docker ps  # Look for "(healthy)" status

# Manual health check
curl http://localhost:3000/health
```

### Log Management
```bash
# View logs
docker-compose logs embeddings-service

# Follow logs
docker-compose logs -f embeddings-service

# Log rotation in production
# Add to docker-compose.yml:
logging:
  driver: "json-file"
  options:
    max-size: "10m"
    max-file: "3"
```

### Metrics Collection
```bash
# Add metrics endpoint to your service
# Monitor with Prometheus/Grafana

# Example docker-compose addition:
prometheus:
  image: prom/prometheus
  ports:
    - "9090:9090"
  volumes:
    - ./prometheus.yml:/etc/prometheus/prometheus.yml
```

## Scaling and Load Balancing

### Horizontal Scaling
```yaml
# docker-compose.yml scaling
services:
  embeddings-service:
    # ... existing config
    deploy:
      replicas: 3

  nginx:
    # Load balancer configuration
    volumes:
      - ./nginx-lb.conf:/etc/nginx/nginx.conf:ro
```

### Load Balancer Configuration
```nginx
upstream embeddings_backend {
    least_conn;
    server embeddings-service_1:3000;
    server embeddings-service_2:3000;
    server embeddings-service_3:3000;
}
```

## Troubleshooting

### Common Issues

#### 1. Out of Memory
```bash
# Check memory usage
docker stats embeddings-service

# Increase memory limit
# In docker-compose.yml:
deploy:
  resources:
    limits:
      memory: 4G
```

#### 2. Model Download Failures
```bash
# Check internet connectivity from container
docker exec -it embeddings-service curl -I https://huggingface.co

# Check disk space
docker system df
docker volume inspect embeddings_model_cache
```

#### 3. Slow Startup
```bash
# Monitor startup process
docker-compose logs -f embeddings-service

# Increase startup timeout
# In docker-compose.yml healthcheck:
start_period: 120s
```

#### 4. Permission Issues
```bash
# Check file permissions
docker exec -it embeddings-service ls -la /app

# Rebuild with correct user
docker-compose build --no-cache
```

### Debug Mode
```bash
# Run with debug logging
docker run -p 3000:3000 -e RUST_LOG=debug embeddings-service

# Interactive debugging
docker run -it --entrypoint=/bin/bash embeddings-service
```

## Production Deployment Checklist

- [ ] Set appropriate resource limits
- [ ] Configure log rotation
- [ ] Set up monitoring and health checks
- [ ] Configure nginx reverse proxy
- [ ] Enable HTTPS with SSL certificates
- [ ] Set up backup for model cache volume
- [ ] Configure firewall rules
- [ ] Set up log aggregation (ELK, Fluentd, etc.)
- [ ] Configure auto-restart policies
- [ ] Set up alerting for service health
- [ ] Test disaster recovery procedures

## Cloud Deployment Examples

### AWS ECS
```json
{
  "family": "embeddings-service",
  "networkMode": "awsvpc",
  "cpu": "1024",
  "memory": "2048",
  "containerDefinitions": [
    {
      "name": "embeddings",
      "image": "your-registry/embeddings-service:latest",
      "portMappings": [
        {
          "containerPort": 3000,
          "protocol": "tcp"
        }
      ],
      "healthCheck": {
        "command": ["CMD-SHELL", "curl -f http://localhost:3000/health || exit 1"],
        "interval": 30,
        "timeout": 5,
        "retries": 3
      }
    }
  ]
}
```

### Kubernetes
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: embeddings-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: embeddings-service
  template:
    metadata:
      labels:
        app: embeddings-service
    spec:
      containers:
      - name: embeddings
        image: embeddings-service:latest
        ports:
        - containerPort: 3000
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 60
          periodSeconds: 30
```

This comprehensive Docker setup provides a production-ready deployment with security, performance optimization, monitoring, and scaling capabilities.