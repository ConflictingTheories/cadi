# cadi-registry

CADI registry client for chunk storage, retrieval and distribution.

## About CADI

CADI is a universal build and distribution system where all artifacts are content-addressed chunks. The registry system enables:

- **Distributed storage**: Store chunks across multiple registries
- **Federation**: Trust policies and cross-registry replication
- **Deduplication**: Automatic detection and reuse of identical content
- **Authentication**: Token-based access control
- **Namespace isolation**: Organize chunks by namespace/project

## Features

- **HTTP-based distribution**: Standard REST API for chunk operations
- **Batch operations**: Publish multiple chunks efficiently
- **Rate limiting**: Configurable request throttling
- **Async/await**: Tokio-based concurrent registry operations
- **Authentication**: Bearer token and API key support
- **Error handling**: Comprehensive error types with context

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
cadi-registry = "1.0"
```

## Basic Usage

```rust
use cadi_registry::RegistryClient;
use cadi_core::Chunk;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = RegistryClient::new(
        "https://registry.example.com",
        "my-auth-token"
    );
    
    let chunk = Chunk {
        id: "chunk-1".to_string(),
        content: b"fn hello() {}".to_vec(),
        ..Default::default()
    };
    
    // Publish chunk
    client.publish_chunk(&chunk).await?;
    
    // Retrieve chunk
    let retrieved = client.get_chunk("chunk-1").await?;
    
    // List chunks
    let chunks = client.list_chunks("my-namespace").await?;
    
    Ok(())
}
```

## Registry API

### Publish Chunks

```rust
client.publish_chunk(&chunk).await?;
client.publish_batch(&chunks).await?;
```

### Retrieve Chunks

```rust
let chunk = client.get_chunk("chunk-id").await?;
let chunks = client.get_chunks(&ids).await?;
```

### Query Chunks

```rust
let results = client.search("keyword").await?;
let namespace_chunks = client.list_chunks("namespace").await?;
```

## Configuration

### From Environment Variables

```bash
export CADI_REGISTRY_URL="https://registry.example.com"
export CADI_AUTH_TOKEN="your-token-here"
```

### Programmatically

```rust
let config = RegistryConfig {
    registry_url: "https://registry.example.com".to_string(),
    auth_token: Some("token".to_string()),
    namespace: Some("myorg".to_string()),
    timeout: Duration::from_secs(30),
    ..Default::default()
};

let client = RegistryClient::from_config(config);
```

## Error Handling

```rust
use cadi_registry::error::{Error, Result};

match client.publish_chunk(&chunk).await {
    Ok(_) => println!("Published"),
    Err(Error::Http(e)) => eprintln!("HTTP error: {}", e),
    Err(Error::Authentication) => eprintln!("Auth failed"),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Integration

- Uses **cadi-core** for chunk types
- Works with **cadi-builder** to distribute built artifacts
- Integrates with **cadi-scraper** for publishing scraped chunks

## Documentation

Full API documentation at [docs.rs/cadi-registry](https://docs.rs/cadi-registry)

## License

MIT License
