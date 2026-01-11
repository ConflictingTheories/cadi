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

### 3. Build for a Target

```bash
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

# Publish with signing
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

### Chunks

A chunk is a content-addressed piece of software. The chunk ID is derived from
its content hash, ensuring immutability and verifiability.

Types of chunks:
- **source-cadi**: Source code files
- **ir-cadi**: Intermediate representations (WASM, LLVM IR)
- **blob-cadi**: Binary artifacts
- **container-cadi**: Container images

### Manifests

A manifest describes an application as a graph of chunks with build targets.
It specifies:
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
- Tiered storage (hot/warm/cold)
- Cryptographic verification

## Configuration

CADI configuration is stored in `~/.config/cadi/config.yaml`:

```yaml
registry:
  primary: https://registry.cadi.dev
  fallback:
    - https://mirror.cadi.dev

auth:
  token: ${CADI_TOKEN}

cache:
  directory: ~/.cache/cadi
  max_size_gb: 10

build:
  parallel_jobs: auto
  prefer_cached: true

security:
  verify_signatures: true
  trusted_publishers:
    - publisher:abc123
```

## Next Steps

- Read the [Architecture Guide](./architecture.md)
- Explore the [CLI Reference](./cli-reference.md)
- Check out [Example Projects](../examples/)
- Learn about [Trust & Security](./security.md)
