# CADI - Content-Addressed Development Interface

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-v1.0.0--dev-orange.svg)](CHANGELOG.md)

CADI is a universal build and distribution system for software artifacts, treating all artifacts as content-addressed chunks with multiple interchangeable representations (source, IR/WASM, binaries, containers). It acts as a "global linker" and provenance-aware registry so humans, tools, and LLMs can discover, assemble, and verify software components.

## Features

- **Content-Addressed Artifacts**: All chunks are immutable and identified by their content hash
- **Multi-Representation Support**: Source, IR (WASM), native binaries, and OCI containers
- **Build Graph Resolution**: Intelligent dependency resolution and caching
- **Provenance & Verification**: SLSA-compliant build receipts and attestations
- **LLM Optimization**: Token-efficient summaries and semantic search for AI-assisted development
- **MCP Integration**: Model Context Protocol server for LLM tool access
- **Cross-Platform**: Support for Linux (x86_64, ARM64), macOS (Intel, Apple Silicon), and WASM

## Quick Start

### Installation

```bash
# macOS (Homebrew)
brew install cadi

# Linux (from binary)
curl -sSL https://get.cadi.dev | sh

# From source
cargo install cadi
```

### Basic Usage

```bash
# Initialize CADI configuration
cadi init

# Import a project
cadi import ./my-project

# Build all targets
cadi build manifest.cadi.yaml

# Run the built artifact
cadi run manifest.cadi.yaml --target web-dev

# Verify provenance
cadi verify chunk:sha256:abc123...
```

## Project Structure

```
cadi/
├── cmd/                    # CLI and server binaries
│   ├── cadi/              # Main CLI
│   ├── cadi-server/       # Registry server
│   └── cadi-mcp-server/   # MCP server for LLM integration
├── cadi-spec/             # CADI specification schemas
├── internal/              # Internal packages
│   ├── builder/           # Build system
│   ├── registry/          # Registry client/server
│   ├── resolver/          # Dependency resolution
│   ├── transform/         # Transformation engine
│   ├── llm/               # LLM optimization layer
│   └── security/          # Signing and verification
├── examples/              # Demo projects
│   └── todo-suite/        # Full-stack todo application
└── docs/                  # Documentation
```

## Core Concepts

### Chunks

Immutable content-addressed units identified by `chunk:sha256:<digest>`. Each chunk contains:
- Metadata (name, description, version, tags)
- Representations (source, IR, binary, container)
- Lineage (parent chunks, build receipts)
- Licensing information

### Representations

A specific form of a chunk:
- `source.*` - Source code (TypeScript, JavaScript, C, etc.)
- `intermediate.*` - Portable representations (WASM)
- `binary.*` - Architecture-specific binaries (x86_64-linux, arm64-darwin, etc.)
- `container.oci` - OCI container images

### Manifests

Application build graphs describing:
- Nodes (components with their representations)
- Edges (interfaces and dependencies between components)
- Build targets (platform-specific configurations)

### Build Receipts

Provenance records capturing:
- Input/output chunks
- Build tools and versions
- Environment digest
- Cryptographic signatures

## Documentation

- [Getting Started](docs/getting-started.md)
- [Specification Overview](docs/spec-overview.md)
- [MCP Integration Guide](docs/mcp-integration.md)
- [Security Model](docs/security-model.md)
- [Demo Walkthrough](docs/demo-walkthrough.md)

## Demo Suite

The included todo-suite demonstrates CADI across multiple platforms:

```bash
# Run the web development target
cadi demo todo-suite --target web-dev

# Build for production with Linux containers
cadi build examples/todo-suite/todo-suite.cadi.yaml --target web-prod

# Run the C server with WASM fallback
cadi run examples/todo-suite/todo-suite.cadi.yaml --target c-server-prod
```

Components:
- **Web Frontend** (React/TypeScript) - Basic and styled variants
- **Node.js REST Server** - Express-based API server
- **Node.js WebSocket Server** - Real-time updates
- **C REST Server** - Minimal HTTP server with WASM support
- **Shared PostgreSQL Schema** - Common database

## Configuration

Default configuration file: `~/.cadi/config.yaml`

```yaml
registry:
  url: "https://registry.cadi.dev"
  namespace: "github.com/myorg"

cache:
  dir: "~/.cadi/store"
  max_size_gb: 10

security:
  trust_policy: "standard"
  verify_on_fetch: true

llm:
  embedding_model: "text-embedding-3-large"
  summary_max_tokens: 500
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Inspired by Nix, Bazel, and OCI registries
- Built for the AI-assisted development era
- Follows Model Context Protocol (MCP) specification
