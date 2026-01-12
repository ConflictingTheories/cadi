# CADI Server

Registry server for CADI (Content-Addressed Development Interface).

## Overview

The CADI server provides HTTP APIs for storing, retrieving, and managing content-addressed code chunks. It serves as the backend for CADI registries.

## Installation

```bash
cargo install cadi-server
```

## Usage

```bash
cadi-server
```

With custom bind address:
```bash
CADI_BIND_ADDRESS=0.0.0.0:8080 cadi-server
```

## API Endpoints

### Chunks

- `GET /chunks/:id` - Retrieve a chunk by content hash
- `POST /chunks` - Store a new chunk
- `GET /chunks/:id/metadata` - Get chunk metadata

### Health

- `GET /health` - Health check endpoint

### Search

- `GET /search?q=<query>` - Search for chunks

## Configuration

Environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `CADI_BIND_ADDRESS` | `0.0.0.0:8080` | Server bind address |
| `CADI_STORAGE` | `/data` | Storage path for chunks |
| `RUST_LOG` | `cadi_server=info` | Log level |

## Docker

```bash
docker run -p 8080:8080 -v ./data:/data cadi/registry:latest
```

## License

MIT
