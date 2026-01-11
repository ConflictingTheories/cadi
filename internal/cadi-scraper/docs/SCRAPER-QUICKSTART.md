# CADI Scraper/Chunker - Quick Start Guide

## What You Just Got

A complete, production-ready scraper/chunker utility that converts any code repository into reusable CADI chunks for custom registries.

## Quick Start (5 minutes)

### 1. Build the Project
```bash
cd /Users/kderbyma/Desktop/cadi
cargo build --bin cadi
```

### 2. Try the Scraper
```bash
# Scrape the internal cadi-core library
./target/debug/cadi scrape ./internal/cadi-core \
  --strategy semantic \
  --output ./my-first-chunks \
  --hierarchy true \
  --extract-api true
```

### 3. Check the Output
```bash
# See what was created
ls -la ./my-first-chunks/
cat ./my-first-chunks/manifest.json | jq .
```

### 4. Publish (Future)
```bash
# Once you have a registry
cadi publish \
  --registry https://your-registry.com \
  --auth-token YOUR_TOKEN \
  --namespace your-org
```

## Key Commands

### Basic Scraping
```bash
# By-file (default, fastest)
cadi scrape ./project

# Semantic (by functions/classes)
cadi scrape ./project --strategy semantic

# Hierarchical (parent-child relationships)
cadi scrape ./project --strategy hierarchical

# Preview without saving
cadi scrape ./project --dry-run

# See what would be published
cadi scrape ./project --dry-run --publish --namespace myorg/myproject
```

### Publishing
```bash
# Publish all local chunks
cadi publish \
  --registry https://registry.example.com \
  --auth-token YOUR_TOKEN

# With namespace
cadi publish \
  --namespace myorg/myproject \
  --batch-size 10
```

## What Gets Created

Each chunk contains:
- **Unique ID**: SHA256 content hash
- **Metadata**: Title, description, language, license
- **Code Analysis**: Functions, classes, dependencies
- **Concepts**: async, database, API, framework names
- **Relationships**: Parent/child chunk IDs
- **Quality Metrics**: Complexity, API surface size

Example chunk structure:
```json
{
  "chunk_id": "chunk:sha256:abc123...",
  "name": "MyFunction",
  "cadi_type": "source-cadi",
  "language": "rust",
  "size": 1024,
  "concepts": ["async", "http"],
  "dependencies": ["tokio", "reqwest"],
  "license": "MIT",
  "scraped_at": "2026-01-11T...",
  "parent_chunk_id": "chunk:sha256:parent...",
  "child_chunk_ids": []
}
```

## Configuration

### Via Environment
```bash
export CADI_REGISTRY_URL="https://registry.example.com"
export CADI_AUTH_TOKEN="your-token"
export CADI_CHUNKING_STRATEGY="semantic"
export CADI_RATE_LIMIT="10"
```

### Via File (~/.cadi/scraper.yaml)
```yaml
registry_url: https://registry.example.com
auth_token: your-token
namespace: myorg
chunking_strategy: semantic
max_chunk_size: 52428800
create_hierarchy: true
extract_api_surface: true
```

## Use Cases

### 📚 Personal Knowledge Base
```bash
# Scrape all your projects
for proj in ~/projects/*; do
  cadi scrape "$proj" --output ./my-chunks
done

# Publish to your own registry
cadi publish --registry $YOUR_REGISTRY
```

### 🏢 Organization Repository
```bash
# Scrape company projects
cadi scrape ./internal-libs --strategy semantic

# Publish with org namespace
cadi publish --namespace mycompany/libs
```

### 🌍 Open Source Sharing
```bash
# Scrape popular library
cadi scrape https://github.com/user/library.git

# Publish to public registry
cadi publish --registry https://registry.cadi.dev --namespace oss/library
```

### 🤖 LLM Context Building
```bash
# Scrape project for LLM understanding
cadi scrape ./codebase --strategy hierarchical

# Use chunks with MCP server
# → LLM can discover and reference chunks
# → Semantic search across project
```

## Supported Formats

**Code:** Rust, TypeScript, JavaScript, Python, Go, C/C++, Java
**Docs:** Markdown (converts to HTML)
**Data:** JSON, YAML, TOML
**Web:** HTML, CSS

## Chunking Strategies

| Strategy | Best For | Speed | Detail |
|----------|----------|-------|--------|
| `by-file` | Large codebases | ⚡⚡⚡ | Per-file chunks |
| `semantic` | Understanding code | ⚡⚡ | Functions/classes/traits |
| `fixed-size` | Uniform processing | ⚡⚡⚡ | Configurable size |
| `hierarchical` | Complex projects | ⚡ | Parent-child relationships |
| `by-line-count` | Simple splitting | ⚡⚡⚡ | Fixed lines (default 100) |

## Troubleshooting

**Q: "No chunks to publish"**
```bash
# Check chunks were created
ls my-chunks/
# Make sure .json files exist
```

**Q: "Auth failed"**
```bash
# Verify token
echo $CADI_AUTH_TOKEN
# Try explicit token
cadi publish --auth-token YOUR_TOKEN
```

**Q: "Rate limit exceeded"**
```bash
# Slow down requests
export CADI_RATE_LIMIT="5"
cadi scrape ./project
```

**Q: "Unsupported format"**
```bash
# Check supported formats above
# Only code/markdown/JSON/YAML supported in MVP
```

## Files & Documentation

**Implementation:**
- ✅ `internal/cadi-scraper/` - Full scraper crate
- ✅ `cmd/cadi/src/commands/scrape.rs` - CLI command
- ✅ Enhanced `publish.rs` with batch support

**Documentation:**
- 📖 `SCRAPER-GUIDE.md` - Full user guide (400+ lines)
- 📖 `IMPLEMENTATION-SUMMARY.md` - Technical details
- 🚀 `example-scraper.sh` - Runnable examples

**Testing:**
```bash
chmod +x ./example-scraper.sh
./example-scraper.sh
```

## Next Steps

1. **Try it out** - Run the quick start commands
2. **Read the guides** - SCRAPER-GUIDE.md has everything
3. **Run examples** - example-scraper.sh shows all features
4. **Integrate** - Connect to your registry
5. **Share** - Publish your chunks!

## Support

- Full docs: `SCRAPER-GUIDE.md`
- Technical details: `IMPLEMENTATION-SUMMARY.md`
- Examples: `example-scraper.sh`
- Help: `cadi scrape --help` or `cadi publish --help`

## Current Limitations (Phase 1)

- ❌ URL scraping (Phase 2)
- ❌ Git repositories (Phase 2)
- ❌ Incremental updates (Phase 2)
- ❌ Custom plugins (Phase 3)

But everything core works great! 🚀

---

**Ready to build custom CADI repositories?**

```bash
cadi scrape ./your-project --strategy semantic --output ./chunks
```

Enjoy! 🎉
