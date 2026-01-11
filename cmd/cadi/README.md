# cadi

CADI CLI - Content-Addressed Development Interface

A universal build and distribution system treating all artifacts as content-addressed chunks with support for multiple representations (source → WASM IR → binaries → containers), registry federation, provenance tracking, and LLM optimization.

## Installation

```bash
cargo install cadi
```

## Quick Start

### Scrape a Project

```bash
cadi scrape ./my-project --strategy semantic --output ./chunks
```

### Publish to Registry

```bash
cadi publish \
  --registry https://registry.example.com \
  --auth-token YOUR_TOKEN \
  --namespace myorg/project
```

### Build Artifacts

```bash
cadi build ./cadi.yaml
```

### Query Registry

```bash
cadi query --registry URL --namespace myorg
```

## Commands

### scrape

Convert source code and files into CADI chunks.

```bash
cadi scrape <INPUT> [OPTIONS]

Options:
  --output DIR                    Output directory (default: ./cadi-chunks)
  --strategy <STRATEGY>           by-file|semantic|fixed-size|hierarchical|by-line-count
  --max-chunk-size BYTES          Maximum chunk size (default: 50MB)
  --include-overlap               Include context between chunks
  --hierarchy                     Create hierarchical relationships
  --extract-api                   Extract API surfaces
  --detect-licenses               Detect licenses
  --publish                       Publish after scraping
  --namespace NAMESPACE           Registry namespace
  --format FORMAT                 table|json
```

### publish

Publish chunks to a registry.

```bash
cadi publish [OPTIONS]

Options:
  --registry URL                  Registry URL
  --auth-token TOKEN              Authentication token
  --namespace NAMESPACE           Namespace/project
  --batch-size N                  Chunks per request
  --dry-run                       Show what would be published
```

### build

Execute build recipes.

```bash
cadi build <RECIPE> [OPTIONS]

Options:
  --output DIR                    Output directory
  --config FILE                   Configuration file
```

### query

Query registry for chunks.

```bash
cadi query [OPTIONS]

Options:
  --registry URL                  Registry URL
  --namespace NAMESPACE           Filter by namespace
  --search QUERY                  Search terms
  --limit N                       Result limit
```

## Configuration

### Global Config

Create `~/.cadi/config.yaml`:

```yaml
default_registry: https://registry.example.com
default_namespace: myorg
auth:
  token: your-token-here
scraper:
  strategy: semantic
  max_chunk_size: 52428800
  extract_api_surface: true
```

### Project Config

Create `./cadi.yaml` in your project:

```yaml
name: my-project
version: 1.0.0
namespace: myorg/my-project

scraping:
  strategy: semantic
  include_overlap: true
  languages:
    - rust
    - python

publishing:
  registry: https://registry.example.com
  namespace: myorg/my-project
```

## Examples

### Scrape and Publish

```bash
# Scrape with semantic chunking
cadi scrape ./src --strategy semantic --output ./chunks

# Publish to registry
cadi publish --registry https://registry.example.com \
  --auth-token $CADI_TOKEN \
  --namespace myorg/project
```

### Batch Processing

```bash
for dir in ./projects/*; do
  echo "Processing $(basename $dir)..."
  cadi scrape "$dir" --strategy semantic
done
```

### Integration with Pipelines

```bash
# GitHub Actions
cadi scrape . --strategy semantic --publish \
  --registry $REGISTRY_URL \
  --auth-token $REGISTRY_TOKEN \
  --namespace $GITHUB_REPOSITORY
```

## Environment Variables

```bash
CADI_REGISTRY_URL          # Default registry
CADI_AUTH_TOKEN            # Authentication token
CADI_NAMESPACE             # Default namespace
CADI_CHUNKING_STRATEGY     # Default strategy
CADI_MAX_CHUNK_SIZE        # Default chunk size
CADI_EXTRACT_API           # Extract API surfaces
CADI_DETECT_LICENSES       # Detect licenses
```

## Output Formats

### Table Format (default)

```
ID                              Content Type    Size     Chunks
abc123def456                    application/rs  2,048    3
xyz789                          text/markdown   512      1
```

### JSON Format

```bash
cadi scrape ./project --format json
```

Outputs structured JSON with full metadata:

```json
{
  "chunks": [
    {
      "id": "abc123def456",
      "content_type": "application/rs",
      "size": 2048,
      "metadata": {
        "title": "hello.rs",
        "description": "Main binary",
        "keywords": ["rust", "cli"]
      }
    }
  ],
  "statistics": {
    "total_chunks": 10,
    "total_bytes": 102400,
    "duration_ms": 1200
  }
}
```

## Troubleshooting

### Authentication Failed

```bash
export CADI_AUTH_TOKEN="your-token"
cadi query --registry https://registry.example.com
```

### Rate Limiting

Configure in `~/.cadi/config.yaml`:

```yaml
scraper:
  rate_limit_per_sec: 20
  request_timeout_secs: 30
```

### Large Projects

For large projects, use hierarchical chunking:

```bash
cadi scrape ./large-project --strategy hierarchical
```

## Integration with Other Tools

### Docker

```dockerfile
FROM rust:latest
RUN cargo install cadi
WORKDIR /project
COPY . .
RUN cadi scrape . --strategy semantic
```

### CI/CD

```yaml
# GitLab CI
build_chunks:
  image: rust:latest
  script:
    - cargo install cadi
    - cadi scrape . --strategy semantic --publish
```

## System Requirements

- Rust 1.70+
- 4GB RAM (2GB minimum)
- 100MB disk space
- Network access for registry operations

## Performance

Typical performance on modern hardware:

- Scraping: 50-100 MB/sec
- Publishing: Limited by network bandwidth
- Querying: < 100ms per chunk

## Documentation

- **User Guide**: See SCRAPER-GUIDE.md for detailed documentation
- **API Docs**: `cargo doc --open` (library crates)
- **Examples**: See examples/ directory
- **Quick Start**: SCRAPER-QUICKSTART.md

## License

MIT License

## Contributing

Contributions welcome! Please check repository for guidelines.

## Support

- GitHub Issues: https://github.com/ConflictingTheories/cadi/issues
- Documentation: https://github.com/ConflictingTheories/cadi
