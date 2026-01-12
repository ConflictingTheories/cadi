# CADI - Content-Addressed Development Interface

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-v1.0.0--dev-orange.svg)](CHANGELOG.md)

CADI is a universal build and distribution system for software artifacts, treating all artifacts as content-addressed chunks with multiple interchangeable representations (source, IR/WASM, binaries, containers). It acts as a "global linker" and provenance-aware registry so humans, tools, and LLMs can discover, assemble, and verify software components.

## Features

- **Content-Addressed Artifacts**: All chunks are immutable and identified by their content hash
- **CADL v2 Interface Contracts**: Advanced semantic contracts for behavior, effects, ABI, and safety
- **Multi-Representation Support**: Source, IR (WASM), native binaries, and OCI containers
- **Build Graph Resolution**: Intelligent dependency resolution and caching
- **Provenance & Verification**: SLSA-compliant build receipts and attestations
- **LLM Optimization**: Token-efficient summaries and semantic search for AI-assisted development
- **MCP Integration**: Model Context Protocol server for LLM tool access
- **Cross-Platform**: Support for Linux (x86_64, ARM64), macOS (Intel, Apple Silicon), and WASM

## CADL - CADI Definition Language

CADI uses CADL v2 to define interfaces with comprehensive semantic contracts, addressing common integration blindspots:

- `@contract`: Semantic behavior and complexity guarantees
- `@effects`: Concurrency, IO, and side-effect contracts
- `@abi`: Binary encoding and calling convention stability
- `@protocol`: Lifecycle and state machine constraints
- `@security`: Sandbox and capability permissions

```cadl
interface VideoCodec {
    @contract(codec: "h264", profile: "high")
    @effects(concurrency: "thread_safe", blocking: "none")
    fn encode(frame: Image) -> Bitstream;
}
```

## MCP Integration

CADI includes a Model Context Protocol (MCP) server that enables AI assistants and coding agents to interact with CADI's build system and chunk registry.

### Setup

1. **Install the Copilot MCP extension** in VS Code:
   ```
   ext install automatalabs.copilot-mcp
   ```

2. **Configure the MCP server** in `.vscode/settings.json`:
   ```json
   {
     "mcp": {
       "servers": {
         "cadi": {
           "command": "target/release/cadi-mcp-server",
           "args": [],
           "env": {
             "CADI_REGISTRY": "http://localhost:8080",
             "CADI_STORAGE": ".cadi-repo",
             "RUST_LOG": "cadi_mcp_server=info"
           }
         }
       }
     },
     "github.copilot.chat.mcp.enabled": true
   }
   ```

### Available Tools

The MCP server exposes these CADI tools to AI assistants:

- **`cadi_search`**: Search for CADI chunks by concept, language, or keyword
- **`cadi_get_chunk`**: Retrieve a CADI chunk by its ID (includes metadata and source)
- **`cadi_build`**: Build a CADI manifest for a specific target
- **`cadi_plan`**: Show the build plan for a manifest without executing it
- **`cadi_verify`**: Verify a chunk's integrity and provenance
- **`cadi_explain`**: Explain a chunk's purpose, dependencies, and lineage
- **`cadi_suggest`**: Suggest chunks that might be useful for a task

### Available Resources

- **`cadi://config`**: Current CADI configuration settings
- **`cadi://cache/stats`**: Local cache usage statistics
- **`cadi://registries`**: Configured registry endpoints and federation status
- **`cadi://trust/policy`**: Current trust policy configuration
- **`cadi://chunk/{id}`**: Direct access to specific chunk details

### Testing MCP Integration

Run the MCP test script to verify everything works:

```bash
./scripts/test-mcp-integration.sh
```

This will test the MCP protocol communication and list all available tools and resources.

## Project Structure

```
cadi/
├── cmd/                    # Execution binaries
│   ├── cadi/              # Principal CLI
│   ├── cadi-server/       # Federated Registry server
│   └── cadi-mcp-server/   # MCP bridge for LLMs
├── internal/              # Core implementations
│   ├── cadi-core/         # AST, Parser, Validator, and CADL v2 logic
│   ├── cadi-builder/      # Cross-platform build engine
│   ├── cadi-registry/     # Registry client and federation logic
│   ├── cadi-scraper/      # Semantic chunking and metadata extraction
│   └── llm/               # Embedding and optimization layer
├── cadi-spec/             # Formal CADI and CADL specifications
├── examples/              # Sample projects and demo suites
└── website/               # Project landing page and documentation
```

## Core Concepts

### CADL v2 (CADI Definition Language)

Advanced interface definitions that go beyond types, capturing behavior, performance, and side-effects.

### Chunks

Immutable content-addressed units identified by `chunk:sha256:<digest>`. Each chunk contains:
- Metadata (name, description, version, tags)
- Representations (source, IR, binary, container)
- Lineage (parent chunks, build receipts)
- Contracts (expressed in CADL)

### Representations

A specific form of a chunk:
- `source.*` - Source code (TypeScript, JavaScript, Rust, C, etc.)
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
