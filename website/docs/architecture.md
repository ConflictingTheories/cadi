# CADI Architecture

This document describes the architecture of the CADI system.

## Overview

CADI is built around the principle of content-addressed software artifacts.
Every piece of code, intermediate representation, or binary is identified by
the hash of its content, creating an immutable and verifiable software supply chain.

```
┌─────────────────────────────────────────────────────────────────┐
│                        CADI Ecosystem                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐  │
│  │  Source  │───▶│    IR    │───▶│   Blob   │───▶│Container │  │
│  │  CADI    │    │   CADI   │    │   CADI   │    │   CADI   │  │
│  └──────────┘    └──────────┘    └──────────┘    └──────────┘  │
│        │              │              │              │           │
│        └──────────────┴──────────────┴──────────────┘           │
│                              │                                   │
│                              ▼                                   │
│                      ┌──────────────┐                           │
│                      │   Manifest   │                           │
│                      │ (Build Graph)│                           │
│                      └──────────────┘                           │
│                              │                                   │
│                              ▼                                   │
│                      ┌──────────────┐                           │
│                      │   Registry   │                           │
│                      │  Federation  │                           │
│                      └──────────────┘                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Chunk Types

#### Source CADI
Contains source code files with metadata about language, version, and dependencies.

```json
{
  "chunk_id": "chunk:sha256:abc123...",
  "cadi_type": "source",
  "source": {
    "language": "rust",
    "version": "1.75",
    "files": [
      { "path": "lib.rs", "hash": "sha256:..." }
    ]
  }
}
```

#### IR CADI
Contains intermediate representations like WebAssembly or LLVM IR.

```json
{
  "chunk_id": "chunk:sha256:def456...",
  "cadi_type": "intermediate",
  "intermediate": {
    "format": "wasm",
    "version": "1.0",
    "module": { "hash": "sha256:...", "exports": [...] }
  }
}
```

#### Blob CADI
Contains compiled binaries for specific architectures.

```json
{
  "chunk_id": "chunk:sha256:ghi789...",
  "cadi_type": "blob",
  "blobs": [
    {
      "architecture": "x86_64-linux",
      "format": "elf",
      "hash": "sha256:..."
    }
  ]
}
```

#### Container CADI
Contains container image references and layer information.

```json
{
  "chunk_id": "chunk:sha256:jkl012...",
  "cadi_type": "container",
  "container": {
    "format": "oci",
    "image_ref": "cadi.dev/app:latest",
    "layers": [...]
  }
}
```

### 2. Manifest System

The manifest describes an application as a directed acyclic graph (DAG) of chunks.

```yaml
manifest_id: "my-app-manifest"
build_graph:
  nodes:
    - id: "core-lib"
      source_cadi: "chunk:sha256:..."
      representations:
        - form: "source"
          language: "rust"
        - form: "ir"
          format: "wasm"
        - form: "blob"
          architecture: "x86_64-linux"
          
  edges:
    - from: "main-app"
      to: "core-lib"
      interface: "CoreAPI"

build_targets:
  - name: "web"
    platform: "wasm32"
    nodes:
      - id: "core-lib"
        prefer: ["ir:wasm"]
```

### 3. Registry Federation

Multiple registries can form a federation for redundancy and locality.

```
┌─────────────────────────────────────────────────────────────┐
│                    Federation Manager                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Primary    │  │    Mirror    │  │   Private    │      │
│  │   Registry   │  │   Registry   │  │   Registry   │      │
│  │  (priority 0)│  │  (priority 1)│  │  (priority 2)│      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│         │                 │                 │                │
│         └─────────────────┼─────────────────┘                │
│                           │                                  │
│                           ▼                                  │
│                    ┌──────────────┐                         │
│                    │   Unified    │                         │
│                    │     API      │                         │
│                    └──────────────┘                         │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 4. Build Engine

The build engine transforms chunks through the representation pipeline.

```
Source → Parse → IR → Compile → Blob → Link → Bundle
           │           │           │
           ▼           ▼           ▼
        Cache       Cache       Cache
```

Features:
- Incremental builds (content-addressed caching)
- Parallel execution where possible
- Cross-language linking via common IR
- Reproducible builds

### 5. Trust & Security

CADI provides multiple layers of security:

1. **Content Verification**: Hash-based identity ensures integrity
2. **Signature Verification**: Publisher signatures prove authenticity
3. **Attestations**: Build receipts prove provenance
4. **Trust Policies**: Configurable policies for accepting chunks

```yaml
trust_policy:
  default_action: "verify"
  trusted_publishers:
    - id: "publisher:abc123"
      trust_level: "full"
  required_attestations:
    - type: "reproducible-build"
```

## Data Flow

### Publishing Flow

```
1. Developer writes code
2. cadi import → Creates source-cadi chunk
3. cadi build → Creates ir-cadi and blob-cadi chunks
4. cadi publish → Uploads to registry with signatures
```

### Fetching Flow

```
1. Consumer runs cadi fetch or cadi build
2. Manifest is parsed to determine needed chunks
3. Registry is queried for chunk availability
4. Best representation is selected based on target
5. Chunks are downloaded and verified
6. Build proceeds with local chunks
```

### Verification Flow

```
1. Chunk is fetched from registry
2. Hash is computed and compared to chunk_id
3. Signature is verified against publisher key
4. Attestations are checked against trust policy
5. Lineage is traced back to source if required
```

## MCP Integration

CADI provides an MCP (Model Context Protocol) server for LLM integration:

```
┌─────────────────────────────────────────┐
│              LLM (Claude)                │
├─────────────────────────────────────────┤
│                   │                      │
│         MCP Protocol (JSON-RPC)          │
│                   │                      │
│                   ▼                      │
│         ┌─────────────────┐              │
│         │  CADI MCP Server│              │
│         ├─────────────────┤              │
│         │ Tools:          │              │
│         │ - cadi_search   │              │
│         │ - cadi_get_chunk│              │
│         │ - cadi_build    │              │
│         │ - cadi_verify   │              │
│         │                 │              │
│         │ Resources:      │              │
│         │ - cadi://config │              │
│         │ - cadi://cache  │              │
│         └─────────────────┘              │
│                   │                      │
│                   ▼                      │
│         ┌─────────────────┐              │
│         │  CADI Core Libs │              │
│         └─────────────────┘              │
│                                          │
└─────────────────────────────────────────┘
```

## Extension Points

CADI is designed for extensibility:

1. **Custom Transformers**: Add support for new languages/formats
2. **Registry Backends**: Implement custom storage backends
3. **Trust Providers**: Integrate with external PKI systems
4. **Build Hooks**: Custom pre/post build actions
5. **MCP Tools**: Add custom tools for LLM integration
