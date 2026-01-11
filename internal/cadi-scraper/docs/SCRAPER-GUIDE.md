# CADI Scraper/Chunker - Implementation Guide

## Overview

The CADI Scraper/Chunker utility converts any source code repository or file data into reusable CADI chunks. It enables users to:

1. **Extract content** from local directories, files, or URLs
2. **Parse and analyze** code with language-specific AST extraction
3. **Create chunks** using multiple strategies (file-based, semantic, hierarchical, etc.)
4. **Extract metadata** automatically (titles, descriptions, licenses, dependencies)
5. **Publish chunks** to authenticated registry servers
6. **Build custom CADI repositories** for personal or organizational use

## Architecture

### Core Modules

```
cadi-scraper/
├── lib.rs                 # Main library exports
├── types.rs              # Core types and configuration
├── config.rs             # Configuration management
├── error.rs              # Error types
├── fetcher.rs            # HTTP/file fetching with rate limiting
├── parser.rs             # Multi-format content parsing
├── chunker.rs            # Semantic and hierarchical chunking
├── metadata.rs           # Metadata extraction
├── transformer.rs        # Language-specific AST extraction
└── scraper.rs            # Main orchestrator
```

### Data Flow

```
Input (Path/URL/Directory)
    ↓
[Fetcher] → Fetch content with rate limiting
    ↓
[Parser] → Parse multi-format (code, markdown, YAML, JSON)
    ↓
[Metadata Extractor] → Auto-extract titles, descriptions, licenses
    ↓
[Transformer] → Extract AST, compute quality metrics
    ↓
[Chunker] → Split using selected strategy
    ↓
[Scraper] → Create manifest and relationships
    ↓
Output (Chunks + Manifest)
    ↓
[Publish] → Registry or local storage
```

## Features

### 1. Multiple Input Sources

#### Local Files
```bash
cadi scrape /path/to/file.rs
```

#### Directories
```bash
cadi scrape /path/to/project --strategy semantic
```

#### URLs (Future)
```bash
cadi scrape https://raw.githubusercontent.com/user/repo/main/src/lib.rs
```

#### Git Repositories (Future)
```bash
cadi scrape git@github.com:user/repo.git --branch main
```

### 2. Chunking Strategies

#### By File (Default)
- Each file becomes a single chunk
- Fastest, suitable for small files
- No splitting or overlap

```bash
cadi scrape /path --strategy by-file
```

#### Semantic
- Splits by functions, classes, traits, methods
- Language-aware boundary detection
- Best for understanding code structure

```bash
cadi scrape /path --strategy semantic
```

#### Fixed Size
- Splits content into fixed byte sizes
- Configurable via `--max-chunk-size`
- Useful for uniform processing

```bash
cadi scrape /path --strategy fixed-size --max-chunk-size 102400
```

#### Hierarchical
- Creates parent-child chunk relationships
- File chunk as parent
- Semantic sub-chunks as children

```bash
cadi scrape /path --strategy hierarchical
```

#### By Line Count
- Splits by fixed line count (default 100 lines)
- Simple, predictable chunking

```bash
cadi scrape /path --strategy by-line-count
```

### 3. Language Support

#### Supported Languages
- **Rust** (.rs)
- **TypeScript/JavaScript** (.ts, .tsx, .js, .jsx)
- **Python** (.py)
- **Go** (.go)
- **C/C++** (.c, .h, .cpp)
- **Java** (.java)
- **Markdown** (.md)
- **JSON/YAML/TOML** (structured data)
- **HTML/CSS** (.html, .css)

#### Language-Specific Features

**Rust:**
- Extract functions, structs, traits
- Detect async/await patterns
- Identify macro usage
- Mark unsafe code blocks

**TypeScript/JavaScript:**
- Extract classes, interfaces, functions
- Detect React components
- Find decorators and metadata
- Track imports/exports

**Python:**
- Extract classes and functions
- Identify decorators
- Track imports
- Detect async code

### 4. Metadata Extraction

Automatic extraction of:

| Item | Detection Method |
|------|-----------------|
| **Title** | Markdown heading, JSON name, Cargo.toml package.name |
| **Description** | Markdown after heading, JSON description, Cargo.toml |
| **Keywords** | JSON keywords array |
| **Concepts** | Pattern matching (database, API, UI, testing, etc.) |
| **License** | SPDX detection, JSON license field |
| **Authors** | JSON author/contributors, Cargo.toml authors |
| **Frameworks** | React, Vue, Angular, Express, FastAPI, Django, Rails, etc. |
| **Dependencies** | AST extraction, package.json, Cargo.toml |

Example output:
```json
{
  "chunk_id": "chunk:sha256:abc123...",
  "name": "Hello Function",
  "description": "A simple greeting function",
  "language": "rust",
  "concepts": ["function", "async"],
  "license": "MIT",
  "frameworks": ["tokio"],
  "dependencies": ["tokio", "serde"]
}
```

### 5. API Surface Extraction

Extracts public API:

**From Rust:**
```rust
pub fn function_name() {}
pub struct MyStruct {}
pub trait MyTrait {}
```

**From TypeScript:**
```typescript
export function functionName() {}
export class MyClass {}
export interface MyInterface {}
```

**From Python:**
```python
def function_name():
class ClassName:
```

### 6. Chunk Relationships

Hierarchical chunks track:
- **Parent ID**: Parent chunk in hierarchy
- **Child IDs**: Child chunks
- **Dependencies**: Referenced chunks
- **Concepts**: Semantic tags

Example hierarchy:
```
file-chunk (parent)
├── function-chunk-1 (child)
├── function-chunk-2 (child)
├── class-chunk-1 (child)
│   ├── method-chunk-1 (grandchild)
│   └── method-chunk-2 (grandchild)
└── trait-chunk-1 (child)
```

### 7. Manifest Generation

Creates manifest linking all chunks:

```json
{
  "version": "1.0.0",
  "cadi_type": "manifest",
  "scraped_at": "2026-01-11T12:00:00Z",
  "chunk_count": 15,
  "chunks": [
    {
      "chunk_id": "chunk:sha256:...",
      "name": "MyModule",
      "source": "src/main.rs",
      "language": "rust",
      "concepts": ["async", "http"]
    }
  ],
  "dependency_graph": {
    "chunk:sha256:abc": ["chunk:sha256:def"],
    "chunk:sha256:def": []
  }
}
```

## CLI Usage

### Basic Scraping

```bash
# Scrape a directory with semantic chunking
cadi scrape ./src --strategy semantic --output ./chunks

# Scrape a single file
cadi scrape main.rs --output ./chunks

# Dry run (preview without saving)
cadi scrape ./project --dry-run

# Verbose output
cadi scrape ./project -v
```

### Advanced Options

```bash
# Custom chunk size
cadi scrape ./project \
  --strategy fixed-size \
  --max-chunk-size 102400

# Include/exclude overlap context
cadi scrape ./project \
  --strategy semantic \
  --include-overlap true

# Create hierarchical relationships
cadi scrape ./project \
  --strategy hierarchical \
  --hierarchy true

# Extract API surfaces
cadi scrape ./project \
  --extract-api true

# Detect licenses
cadi scrape ./project \
  --detect-licenses true
```

### Output Formats

```bash
# Table format (default)
cadi scrape ./project --format table

# JSON format
cadi scrape ./project --format json | jq .

# YAML format (future)
cadi scrape ./project --format yaml
```

## Publishing Workflow

### Publishing to Registry

```bash
# Step 1: Scrape repository
cadi scrape ./my-project --output ./chunks

# Step 2: Configure registry and auth
export CADI_REGISTRY_URL="https://registry.example.com"
export CADI_AUTH_TOKEN="your-token"

# Step 3: Publish chunks
cadi publish \
  --registry $CADI_REGISTRY_URL \
  --auth-token $CADI_AUTH_TOKEN \
  --namespace myorg/myproject

# Step 4: Verify in registry
cadi query --name myproject --registry $CADI_REGISTRY_URL
```

### Batch Publishing

```bash
# Publish with deduplication (skip existing chunks)
cadi publish --no-dedup false

# Batch size control
cadi publish --batch-size 10

# Sign chunks during publish
cadi publish --no-sign false
```

## Configuration

### Config File (~/.cadi/scraper.yaml)

```yaml
registry_url: https://registry.example.com
auth_token: YOUR_TOKEN_HERE
namespace: myorg

chunking_strategy: semantic
max_chunk_size: 52428800  # 50MB
include_overlap: true
overlap_size: 500

language_options:
  rust:
    min_semantic_size: 100
    split_by_semantic_boundary: true
    extract_functions: true
    extract_types: true
    extract_classes: true

exclude_patterns:
  - "**/.git"
  - "**/node_modules"
  - "**/target"
  - "**/dist"

create_hierarchy: true
extract_api_surface: true
detect_licenses: true

request_timeout: 30
rate_limit: 10.0  # requests per second
cache_dir: ~/.cadi/scraper-cache
```

### Environment Variables

```bash
# Registry configuration
export CADI_REGISTRY_URL="https://registry.example.com"
export CADI_AUTH_TOKEN="your-token"
export CADI_NAMESPACE="myorg"

# Chunking strategy
export CADI_CHUNKING_STRATEGY="semantic"

# Rate limiting
export CADI_RATE_LIMIT="10"

# Timeout
export CADI_REQUEST_TIMEOUT="30"
```

## Examples

### Example 1: Scrape Todo Suite Project

```bash
cadi scrape ./examples/todo-suite \
  --strategy hierarchical \
  --output ./todo-suite-chunks \
  --extract-api true \
  --detect-licenses true
```

Creates chunks for each component with hierarchy:
- todo-core (functions, types)
- todo-cli (CLI structures)
- todo-web (React components)

### Example 2: Create Custom Organization Repository

```bash
# Scrape all internal projects
for project in internal/*/; do
  cadi scrape "$project" \
    --strategy semantic \
    --output "./org-chunks/$(basename $project)"
done

# Publish to org registry
cadi publish \
  --registry https://registry.myorg.com \
  --namespace myorg
```

### Example 3: Share Open Source Chunks

```bash
# Scrape popular open source projects
cadi scrape https://github.com/tokio-rs/tokio.git \
  --strategy semantic \
  --output ./public-chunks

# Publish to public CADI registry
cadi publish \
  --registry https://registry.cadi.dev \
  --namespace community/tokio
```

## Performance

### Benchmarks (on typical laptop)

| Operation | Time | Notes |
|-----------|------|-------|
| Scrape 100 small files | 2-3s | By-file strategy |
| Scrape 1000 LOC with semantic | 5-10s | Full AST extraction |
| Hierarchical chunking | +15% | Overhead from parent/child tracking |
| Metadata extraction | <1s | Per-chunk |
| Publish 50 chunks | 3-5s | Sequential, 10 req/s |

### Optimization Tips

1. **Use by-file strategy** for large codebases (faster)
2. **Enable deduplication** to skip existing chunks
3. **Batch publishing** with higher batch_size for better throughput
4. **Configure rate_limit** appropriately for your registry
5. **Use hierarchical** only when parent-child relationships needed

## Roadmap

### Phase 1 (MVP - Current)
- ✅ Local file/directory scraping
- ✅ Multi-format parsing (code, markdown, JSON, YAML)
- ✅ Semantic chunking with AST extraction
- ✅ Metadata auto-extraction
- ✅ Batch publishing with authentication
- ✅ CLI command integration

### Phase 2 (Planned)
- 🔄 URL/HTTP scraping with caching
- 🔄 Git repository support
- 🔄 Incremental scraping (track changes)
- 🔄 HTML/Web scraping
- 🔄 PDF parsing
- 🔄 Custom transformer plugins

### Phase 3 (Future)
- 📋 Multi-language support (more languages)
- 📋 Federated scraping (coordinate across registries)
- 📋 Semantic chunking v2 (ML-based boundaries)
- 📋 Compression and delta encoding
- 📋 Browser extension for web scraping

## Troubleshooting

### Common Issues

**Issue: "No chunks to publish"**
```bash
# Solution: Verify chunks were created
ls -la ./chunks
# Check for .json files
```

**Issue: "Authentication failed"**
```bash
# Solution: Verify auth token
echo $CADI_AUTH_TOKEN
# Try with explicit token
cadi publish --auth-token YOUR_TOKEN
```

**Issue: "Chunk already exists at registry"**
```bash
# Solution: Skip deduplication check
cadi publish --no-dedup
```

**Issue: Rate limit exceeded**
```bash
# Solution: Reduce rate limit
export CADI_RATE_LIMIT="5"
cadi scrape ./project
```

## Contributing

Contributions welcome for:
- Additional language support
- New chunking strategies
- Better metadata extraction
- Performance optimizations
- Bug fixes

See main README.md for contribution guidelines.
