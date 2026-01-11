# cadi-scraper

CADI Scraper/Chunker utility for converting source code repos and file data into reusable CADI chunks.

## Overview

`cadi-scraper` automatically analyzes source code projects and converts them into optimized, content-addressed chunks ready for distribution through CADI registries. It handles multiple programming languages, diverse file formats, and provides intelligent semantic chunking.

## Features

- **Multi-language support**: Rust, TypeScript, Python, JavaScript, Go, C/C++
- **Format-agnostic**: Source code, Markdown, JSON, YAML, HTML, CSS
- **5 chunking strategies**: By-file, Semantic, Fixed-size, Hierarchical, By-line-count
- **Automatic metadata extraction**: Titles, descriptions, licenses, frameworks, API surfaces
- **Rate-limited fetching**: HTTP and filesystem access with configurable throttling
- **Semantic analysis**: AST-based code understanding via tree-sitter
- **Framework detection**: Identifies 20+ popular frameworks (React, Django, Spring, etc.)
- **License detection**: Recognizes SPDX licenses automatically
- **Async/await**: High-performance concurrent processing

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
cadi-scraper = "1.0"
```

## Quick Start

### Basic Scraping

```rust
use cadi_scraper::{Scraper, ScraperConfig, ScraperInput, ChunkingStrategy};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ScraperConfig {
        chunking_strategy: ChunkingStrategy::Semantic,
        max_chunk_size: 50_000,
        ..Default::default()
    };
    
    let scraper = Scraper::new(config);
    
    let input = ScraperInput::LocalPath("./my-project".into());
    let output = scraper.scrape(input).await?;
    
    println!("Created {} chunks", output.chunks.len());
    println!("Total bytes: {}", output.statistics.total_bytes);
    
    Ok(())
}
```

### CLI Usage

```bash
# Install
cargo install cadi

# Scrape a project
cadi scrape ./my-project --strategy semantic --output ./chunks

# Publish to registry
cadi publish --registry https://registry.example.com \
  --auth-token TOKEN \
  --namespace myorg/project

# See all options
cadi scrape --help
```

## Chunking Strategies

### By-File
Creates one chunk per file. Fast, simple, preserves file structure.

```rust
ChunkingStrategy::ByFile
```

### Semantic
Analyzes code structure and chunks at logical boundaries (functions, classes, modules).

```rust
ChunkingStrategy::Semantic
```

### Fixed-Size
Creates fixed-byte chunks, useful for uniform processing.

```rust
ChunkingStrategy::FixedSize
```

### Hierarchical
Creates parent chunks per file with children chunks for functions/classes.

```rust
ChunkingStrategy::Hierarchical
```

### By-Line-Count
Creates chunks every N lines (default 100).

```rust
ChunkingStrategy::ByLineCount
```

## Configuration

### Via Environment Variables

```bash
export CADI_CHUNKING_STRATEGY="semantic"
export CADI_MAX_CHUNK_SIZE="52428800"  # 50MB
export CADI_INCLUDE_OVERLAP="true"
export CADI_EXTRACT_API_SURFACE="true"
export CADI_DETECT_LICENSES="true"
```

### Via Config File

Create `~/.cadi/scraper-config.yaml`:

```yaml
chunking_strategy: semantic
max_chunk_size: 52428800
include_overlap: true
extract_api_surface: true
detect_licenses: true
languages:
  rust:
    enabled: true
    custom_patterns: []
  python:
    enabled: true
    custom_patterns: []
```

### Programmatically

```rust
let config = ScraperConfig {
    chunking_strategy: ChunkingStrategy::Semantic,
    max_chunk_size: 50_000,
    include_overlap: true,
    hierarchy: true,
    extract_api: true,
    detect_licenses: true,
    ..Default::default()
};
```

## Output

Scraping produces `ScraperOutput` with:

```rust
pub struct ScraperOutput {
    pub chunks: Vec<ScrapedChunk>,      // Generated chunks
    pub manifest: Manifest,              // Dependency graph
    pub statistics: ScrapingStatistics,  // Metrics
    pub errors: Vec<String>,             // Non-fatal errors
}
```

## Advanced Usage

### Custom Language Patterns

```rust
let mut config = ScraperConfig::default();
config.languages.insert("rust".to_string(), LanguageConfig {
    enabled: true,
    custom_patterns: vec![
        r"#\[derive\((.*?)\)\]".to_string(),
    ],
});
```

### Publishing Chunks

```rust
use cadi_registry::RegistryClient;

let output = scraper.scrape(input).await?;
let client = RegistryClient::new(registry_url, auth_token);

for chunk in output.chunks {
    client.publish_chunk(&chunk).await?;
}
```

### Batch Processing

```rust
let inputs = vec![
    ScraperInput::LocalPath("./project1".into()),
    ScraperInput::LocalPath("./project2".into()),
    ScraperInput::Url("https://github.com/user/repo".into()),
];

for input in inputs {
    let output = scraper.scrape(input).await?;
    // Process output...
}
```

## Framework Detection

Automatically detects:

- **Frontend**: React, Vue, Angular, Svelte, Next.js
- **Backend**: Express, Fastify, Django, FastAPI, Spring, Rails
- **Async Runtimes**: Tokio, async-std
- **Testing**: Jest, pytest, RSpec
- **Build Tools**: Webpack, Vite, Cargo, Maven

## License Detection

Recognizes SPDX identifiers:

- MIT
- Apache-2.0
- GPL-3.0
- BSD-3-Clause
- ISC
- And many more...

## Performance

Typical performance on modern hardware:

- **By-file chunking**: ~100 MB/sec
- **Semantic chunking**: ~50 MB/sec
- **Metadata extraction**: Included in above
- **Rate limiting**: Configurable (default 10 req/sec)

## Error Handling

```rust
use cadi_scraper::error::Error;

match scraper.scrape(input).await {
    Ok(output) => {
        if !output.errors.is_empty() {
            eprintln!("Warnings: {:?}", output.errors);
        }
    }
    Err(Error::InvalidInput(msg)) => eprintln!("Invalid input: {}", msg),
    Err(Error::Fetch(msg)) => eprintln!("Fetch failed: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Integration

Part of the CADI ecosystem:

- **cadi-core**: Chunk and manifest types
- **cadi-registry**: Publish scraped chunks
- **cadi**: CLI integration
- **cadi-builder**: Transform scraped chunks

## Documentation

- Full API docs: [docs.rs/cadi-scraper](https://docs.rs/cadi-scraper)
- User guide: Check repository SCRAPER-GUIDE.md
- Examples: See repository examples/ directory

## License

MIT License
