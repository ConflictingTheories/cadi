# cadi-builder

CADI build engine and transformation pipeline for WASM and binaries.

## About CADI

CADI is a universal build and distribution system. `cadi-builder` implements the transformation pipeline that converts source code and artifacts into WASM IR, binaries, and containers.

## Features

- **Multi-stage transformation pipeline**: Source → IR → WASM → Binary/Container
- **Content-addressed output**: All build artifacts identified by SHA256
- **Efficient incremental builds**: Skip already-built chunks
- **Build receipts**: Full provenance with build metadata and timestamps
- **Async/await**: Tokio-based concurrent transformations
- **Comprehensive logging**: Tracing integration for detailed build info

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
cadi-builder = "1.0"
```

## Basic Usage

```rust
use cadi_builder::Builder;
use cadi_core::Chunk;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut builder = Builder::new();
    
    // Add input chunks
    let chunk = Chunk {
        id: "chunk-1".to_string(),
        content: b"fn main() {}".to_vec(),
        ..Default::default()
    };
    
    builder.add_chunk(chunk)?;
    
    // Transform to WASM IR
    let ir_output = builder.transform_to_ir().await?;
    
    // Build to binary
    let binary = builder.build_binary().await?;
    
    Ok(())
}
```

## Pipeline Stages

1. **Source Analysis**: Parse and analyze source code
2. **IR Generation**: Convert to WASM intermediate representation
3. **Optimization**: Apply optimizations and transformations
4. **Code Generation**: Generate binary from IR
5. **Attestation**: Create build receipts and sign artifacts

## Integration

- Depends on **cadi-core** for chunk types
- Used by **cadi-registry** for chunk distribution
- Works with **cadi-scraper** for source code ingestion

## Error Handling

Uses `thiserror` for comprehensive error types:

```rust
match builder.build() {
    Ok(result) => println!("Build succeeded"),
    Err(e) => eprintln!("Build failed: {}", e),
}
```

## Documentation

Full API documentation at [docs.rs/cadi-builder](https://docs.rs/cadi-builder)

## License

MIT License
