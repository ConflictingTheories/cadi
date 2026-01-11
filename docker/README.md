# CADI Docker Infrastructure

This directory contains Docker configurations for running and developing with CADI.

## Quick Start

### 1. Start the CADI Server Infrastructure

```bash
cd docker

# Start basic infrastructure (registry + MCP server)
make up

# Or with full infrastructure (PostgreSQL, Redis, MinIO)
make up-full
```

The registry will be available at `http://localhost:8080`.

### 2. Development Environment

```bash
# Start the development environment
make dev

# Open an interactive shell
make shell

# Or use the build script directly
./scripts/cadi-build.sh --shell
```

## Components

### Docker Compose Files

| File | Purpose |
|------|---------|
| `docker-compose.yml` | Production server infrastructure |
| `docker-compose.dev.yml` | Development workstation |

### Dockerfiles

| File | Purpose |
|------|---------|
| `Dockerfile.server` | CADI Registry Server |
| `Dockerfile.mcp` | MCP (Model Context Protocol) Server |
| `Dockerfile.cli` | CADI CLI tool |
| `Dockerfile.dev` | Full development environment |

## Services

### Core Services

- **cadi-registry**: The main CADI chunk registry server
  - Port: 8080
  - Stores and serves CADI chunks
  - Health check: `GET /health`

- **cadi-mcp**: Model Context Protocol server
  - Port: 9090
  - Enables AI/LLM integration with CADI

### Optional Services (Full Profile)

- **postgres**: PostgreSQL for persistent metadata
- **redis**: Caching and pub/sub
- **minio**: S3-compatible storage for large chunks
- **nginx**: Reverse proxy with TLS (production profile)

## Development Workstation

The development container includes:

- **Rust** (stable + WASM targets)
- **Node.js** (v20 + TypeScript, pnpm, yarn)
- **Python** (3.x + poetry, black, ruff)
- **Go** (1.22)
- **Docker-in-Docker** support
- **CADI CLI** pre-installed

### Building Projects

```bash
# Build current directory
./scripts/cadi-build.sh

# Build specific project
./scripts/cadi-build.sh -p ./my-project

# Build with specific target
./scripts/cadi-build.sh -t release

# Build and publish
./scripts/cadi-build.sh --publish

# Interactive shell
./scripts/cadi-build.sh --shell
```

### Watch Mode

```bash
# Start file watching for automatic rebuilds
make watch
```

## Configuration

### Environment Variables

Copy `.env.example` to `.env` and customize:

```bash
cp .env.example .env
```

Key variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `CADI_PORT` | 8080 | Registry server port |
| `CADI_MCP_PORT` | 9090 | MCP server port |
| `CADI_MAX_CHUNK_SIZE` | 100MB | Maximum chunk size |
| `RUST_LOG` | info | Log level |
| `PROJECT_PATH` | ./examples | Default project path for dev |

### Volumes

| Volume | Purpose |
|--------|---------|
| `cadi-data` | Registry chunk storage |
| `cadi-cache` | Registry cache |
| `cadi-dev-cache` | Development CADI cache |
| `cadi-build-cache` | Build artifacts cache |

## Production Deployment

### With Nginx (TLS)

1. Place SSL certificates in `nginx/ssl/`:
   - `fullchain.pem`
   - `privkey.pem`

2. Start with production profile:
   ```bash
   docker compose --profile production up -d
   ```

### Kubernetes

See `../deploy/kubernetes/` for Kubernetes manifests.

## Network

All services run on the `cadi-network` bridge network. This allows:
- Service discovery by container name
- Isolation from other Docker networks
- Easy connection from development containers

## Troubleshooting

### Registry not accessible

```bash
# Check if running
docker compose ps

# Check logs
docker compose logs cadi-registry

# Test health
curl http://localhost:8080/health
```

### Build failures

```bash
# Enter dev container for debugging
make shell

# Check CADI version
cadi --version

# Verify registry connection
curl http://cadi-registry:8080/health
```

### Clean up

```bash
# Remove containers and volumes
make clean

# Remove all unused Docker resources
make prune
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Docker Network                          │
│                        (cadi-network)                           │
│                                                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │  cadi-registry  │  │    cadi-mcp     │  │    cadi-dev     │ │
│  │    (server)     │  │   (MCP server)  │  │  (workstation)  │ │
│  │    :8080        │  │     :9090       │  │                 │ │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘ │
│           │                    │                    │          │
│           └────────────────────┴────────────────────┘          │
│                                                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │    postgres     │  │      redis      │  │      minio      │ │
│  │   (optional)    │  │   (optional)    │  │   (optional)    │ │
│  │    :5432        │  │     :6379       │  │   :9000/:9001   │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │      nginx      │
                    │  (production)   │
                    │   :80 / :443    │
                    └─────────────────┘
```
