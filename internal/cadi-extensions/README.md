# CADI Extensions

The plugin system that makes CADI infinitely extensible.

## Overview

CADI Extensions enable developers to extend CADI's capabilities with custom:
- **Language Atomizers**: Support for new programming languages
- **Build Backends**: Custom build targets (Docker, WASM, mobile apps)
- **Registry Plugins**: Alternative storage backends (S3, IPFS, GitHub)
- **MCP Tools**: Additional AI agent capabilities
- **UI Extensions**: IDE integrations and dashboards

## Architecture

Extensions are dynamically loaded plugins that implement well-defined traits:

```rust
#[async_trait]
pub trait Extension: Send + Sync {
    fn metadata(&self) -> ExtensionMetadata;
    async fn initialize(&mut self, context: &ExtensionContext) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
}
```

## Extension Types

### Atomizer Extensions
```rust
#[async_trait]
pub trait AtomizerExtension: Extension {
    fn language(&self) -> &str;
    async fn extract_atoms(&self, source: &str) -> Result<Vec<AtomicChunk>>;
    async fn resolve_imports(&self, source: &str) -> Result<Vec<ResolvedImport>>;
}
```

### Build Backend Extensions
```rust
#[async_trait]
pub trait BuildBackendExtension: Extension {
    fn target_formats(&self) -> Vec<&str>;
    async fn build(&self, chunk: &AtomicChunk, target: &str) -> Result<BuildArtifact>;
}
```

### Registry Extensions
```rust
#[async_trait]
pub trait RegistryExtension: Extension {
    async fn store(&self, chunk: &AtomicChunk) -> Result<String>;
    async fn retrieve(&self, id: &str) -> Result<Option<AtomicChunk>>;
    async fn search(&self, query: &SearchQuery) -> Result<Vec<AtomicChunk>>;
}
```

## Creating Extensions

1. **Create a new Rust library crate**
2. **Implement the appropriate extension trait**
3. **Export the extension constructor function**:
   ```rust
   #[no_mangle]
   pub extern "C" fn cadi_extension_create() -> *mut dyn Extension {
       Box::into_raw(Box::new(MyExtension::new()))
   }
   ```
4. **Create extension manifest** (`extension.toml`):
   ```toml
   [extension]
   name = "my-extension"
   version = "1.0.0"
   type = "atomizer"
   description = "Atomizer for MyLanguage"

   [dependencies]
   cadi-core = "2.0"
   ```

## Loading Extensions

Extensions can be loaded from:
- **Local filesystem**: `./extensions/`
- **CADI Registry**: `cadi install my-extension`
- **System paths**: `/usr/local/lib/cadi/extensions/`

```rust
let loader = ExtensionLoader::new()
    .with_search_paths(vec!["./extensions".into()])
    .load_all()
    .await?;
```

## Distribution

Extensions are distributed as:
- **Crates on crates.io**: `cargo install cadi-atomizer-java`
- **Binary releases**: Platform-specific downloads
- **CADI Registry**: `cadi install my-extension`

## Commercial Extensions

Premium extensions are available through CADI Pro/Enterprise:
- Advanced language support (Java, Swift, C++)
- Cloud storage backends
- Enterprise security features
- Custom development services

## Development

```bash
# Build the extensions crate
cargo build

# Run tests
cargo test

# Build example extension
cd examples/java-atomizer
cargo build --release
```