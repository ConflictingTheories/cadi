# Getting Started with CADI

CADI (Content-Addressed Development Interface) is a system for managing software artifacts
using content-addressed identifiers. This guide will help you get started.

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/cadi-dev/cadi.git
cd cadi

# Build all packages
cargo build --release

# Install the CLI
cargo install --path cmd/cadi
```

### From Binary Releases

Download the latest release from GitHub and add it to your PATH.

## Quick Start

### 1. Initialize a Project

```bash
# Create a new CADI project
cadi init my-project
cd my-project
```

This creates a `cadi.yaml` manifest file and basic project structure.

### 2. Import Existing Code

```bash
# Import source code as a CADI chunk
cadi import ./src --language rust --name my-library
```

### 3. Build & Validate

```bash
# Validate your CADL interface definitions
cadi validate my-interface.cadl

# Build for the default target
cadi build

# Build for a specific target
cadi build --target web

# See what would be built without building
cadi plan --target web
```

### 4. Publish to Registry

```bash
# Publish all chunks to the registry
cadi publish

# Publish with signing (if configured)
cadi publish --sign
```

### 5. Fetch Dependencies

```bash
# Fetch a specific chunk
cadi fetch chunk:sha256:abc123...

# Fetch all dependencies for a manifest
cadi fetch --manifest cadi.yaml
```

## Key Concepts

### CADL v2 (CADI Definition Language)

CADI uses CADL v2 to define interfaces with comprehensive semantic contracts. You can use the `cadi validate` command to ensure your definitions are compliant with the specification.

**Note:** CADI's `cadi-core` crate now enables Tree-sitter based AST parsing by default (feature: `ast-parsing`). This provides more accurate atom extraction and significantly improved performance for some languages (e.g., Rust). If you need to reduce compile time or disable Tree-sitter parsing for any reason, build without the feature:

```bash
cargo build -p cadi-core --no-default-features
```

If you prefer to enable it explicitly in a downstream project, use:

```bash
cargo build -p cadi-core --features ast-parsing
```

This setting is visible in `internal/cadi-core/Cargo.toml` under the `[features]` section.

### Chunks

A chunk is a content-addressed piece of software. The chunk ID is derived from its content hash, ensuring immutability and verifiability.

Types of chunks (serialized as lowercase):
- **source**: Source code files and project structure
- **intermediate**: Portable representations (WASM)
- **blob**: Native binary artifacts for specific architectures
- **container**: OCI-compatible container images

### Manifests

A manifest describes an application as a graph of chunks with build targets. It specifies:
- Which chunks make up the application
- How chunks depend on each other
- What representations are available for each chunk
- How to build for different platforms

### Build Targets

A build target specifies:
- The platform (e.g., `wasm32`, `x86_64-linux`, `aarch64-darwin`)
- Which representations to prefer for each chunk
- How to bundle the final output

### Registry

The registry stores and serves CADI chunks. Features:
- Content-addressed storage
- Federated architecture
- Cryptographic verification
- SLSA provenance support

## Configuration

CADI configuration is stored in `~/.cadi/config.yaml`:

```yaml
registry:
  url: https://registry.cadi.dev
  namespace: github.com/myorg

auth:
  token: ${CADI_TOKEN}

cache:
  dir: ~/.cadi/store
  max_size_gb: 10

build:
  parallelism: auto
  prefer_representation: 
    - binary
    - wasm
    - source

security:
  trust_policy: standard
  verify_on_fetch: true
  sandbox_untrusted: true

llm:
  embedding_model: text-embedding-3-large
  summary_max_tokens: 500
```

## Tree-sitter AST Parsing (default)

**Note:** CADI's `cadi-core` crate now enables Tree-sitter based AST parsing by default (feature: `ast-parsing`). This provides more accurate atom extraction and significantly improved performance for some languages (e.g., Rust). If you need to reduce compile time or disable Tree-sitter parsing for any reason, build without the feature:

```bash
cargo build -p cadi-core --no-default-features
```

If you prefer to enable it explicitly in a downstream project, use:

```bash
cargo build -p cadi-core --features ast-parsing
```

This setting is visible in `internal/cadi-core/Cargo.toml` under the `[features]` section.

## Next Steps

- Read the [Architecture Guide](./architecture.md)
- Explore the [CLI Reference](./cli-reference.md)
- Check out [Example Projects](../examples/)
- Learn about [MCP Integration](../MCP-INTEGRATION.md)
