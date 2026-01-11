# cadi-core

Core types and utilities for the CADI (Content-Addressed Development Interface) system.

## About CADI

CADI is a universal build and distribution system that treats all artifacts as content-addressed chunks. It enables:

- **Multi-representation support**: Source code → WASM IR → binaries → containers
- **Content addressing**: All artifacts identified by SHA256 hashes
- **Registry federation**: Distribute chunks across multiple registries with trust policies
- **Provenance tracking**: Security attestations and build receipts
- **LLM optimization**: Hierarchical summaries and semantic embeddings

## cadi-core

This crate provides the foundational types and utilities used throughout the CADI ecosystem:

### Core Types

- **Chunk**: Base unit of content-addressed data with metadata
- **ChunkId**: SHA256-based content identifiers
- **Manifest**: Collection metadata and dependency graphs
- **ChunkMetadata**: Title, description, keywords, author, license
- **BuildReceipt**: Build provenance and attestation
- **SecurityAttestation**: Signature and verification metadata

### Features

- Serialization/deserialization with serde (JSON, YAML)
- UUID generation for tracking
- Timestamp handling with chrono
- Hash computation with SHA2
- Error handling with thiserror

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
cadi-core = "1.0"
```

## Basic Usage

```rust
use cadi_core::{Chunk, ChunkMetadata};
use std::collections::HashMap;

// Create chunk metadata
let metadata = ChunkMetadata {
    title: Some("My Code Chunk".to_string()),
    description: Some("A reusable code component".to_string()),
    keywords: vec!["rust".to_string(), "library".to_string()],
    ..Default::default()
};

// Create a chunk
let chunk = Chunk {
    id: "abc123def456".to_string(),
    content: "fn hello() { println!(\"Hello, World!\"); }".as_bytes().to_vec(),
    metadata,
    content_type: "application/rust".to_string(),
    timestamp: chrono::Utc::now(),
    ..Default::default()
};
```

## Integration with Other CADI Crates

- **cadi-builder**: Build chunks from source code
- **cadi-registry**: Publish and retrieve chunks from registries
- **cadi-scraper**: Automatically scrape projects into chunks

## Documentation

Full API documentation available at [docs.rs/cadi-core](https://docs.rs/cadi-core)

## License

MIT License - See LICENSE file in the repository
