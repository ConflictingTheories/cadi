# CADI Workflow Guide: From Search to Build

## Overview
This guide demonstrates the complete CADI workflow for LLM agents and developers building code with 80-90% token efficiency through semantic code reuse.

---

## The CADI-First Principle

**Traditional LLM Approach:**
```
Generate HTTP server (2,000 tokens)
Generate auth system (1,500 tokens) 
Generate database layer (1,800 tokens)
Total: 5,300 tokens - only 30% complete
```

**CADI-First Approach:**
```
Search for existing components (100 tokens) → 3 found
Compose them together (200 tokens) → gaps identified
Generate only glue code (400 tokens) → minimal generation
Total: 700 tokens - 100% complete, 87% token savings ✅
```

---

## Workflow Steps

### Step 1: Initialize CADI Project

```bash
cadi init --language typescript --template web-service
```

Creates:
- `.cadi/` - Local CADI repository
- `cadi.yaml` - Project configuration
- `build.cadi.yaml` - Build specification template

### Step 2: Search for Components (ALWAYS START HERE)

```bash
# Semantic search for functionality
cadi search "HTTP server with routing and middleware"
→ Returns: Express.js, Fastify, Hapi implementations
  - Relevance scores
  - Usage statistics
  - Quality metrics
  - Token saved: ~1,000 per search

# Language-specific search
cadi search "JWT authentication" --language typescript
→ Returns: TypeScript/Node.js auth libraries

# Concept-based search
cadi search "database connection pool" --concepts ["postgres", "async"]
→ Returns: Relevant database clients
```

**Token Cost**: ~50 tokens per search (IDs + metadata)
**Time Saved**: 5 minutes of reading docs

### Step 3: Examine Components

```bash
# Get component metadata and signature
cadi get cadi://fn/http-server-express/abc123 --format summary
→ Shows:
  - Function signature
  - Dependencies
  - Type information
  - Test coverage
  - Usage examples

# Get full source if needed
cadi get cadi://fn/http-server-express/abc123 --format full
→ Full implementation code

# Get detailed documentation
cadi get cadi://fn/http-server-express/abc123 --include docs
→ Usage patterns, examples, best practices
```

**Token Cost**: ~100 tokens for summary, ~500 for full code

### Step 4: Create Build Specification (CBS)

Edit `build.cadi.yaml`:

```yaml
cadi_version: "1.0"
project:
  name: "task-management-api"
  type: package
  language: typescript
  description: "REST API for task management with auth"

components:
  # Reuse existing HTTP server
  - id: "cadi://fn/http-server-express/abc123"
    as: "http_server"
  
  # Reuse authentication
  - id: "cadi://fn/jwt-auth-express/def456"
    as: "auth_middleware"
  
  # Reuse database client
  - id: "cadi://fn/db-client-postgres/ghi789"
    as: "db"
  
  # Generate only unique business logic
  - generate:
      description: "Express route handlers for task CRUD operations"
      interface:
        input:
          method: string
          path: string
          body: object
        output:
          status: number
          data: object
      dependencies: ["http_server", "auth_middleware", "db"]
    as: "task_routes"
  
  # Generate only error handler
  - generate:
      description: "Express error handling middleware"
      interface:
        input: { error: Error, context: object }
        output: { status: number, message: string }
      dependencies: ["http_server"]
    as: "error_handler"

build:
  steps:
    - type: dependency-check
      config:
        validate_compatibility: true
    
    - type: transpile
      config:
        source: typescript
        target: javascript
    
    - type: test
      config:
        framework: jest
        coverage_target: 80
    
    - type: bundle
      config:
        tool: esbuild
        minify: true

output:
  - type: npm-package
    path: ./dist
  
  - type: docker-image
    config:
      registry: docker.io
      tag: "my-app:latest"
```

**Why this works:**
- 5 components, 3 reused (60% reuse)
- 2 generated (unique business logic)
- Only glue code needs generation
- Composition is explicit and verifiable

### Step 5: Validate Components

Before building, validate that components work together:

```bash
# Check compatibility of selected components
cadi compose \
  cadi://fn/http-server-express/abc123 \
  cadi://fn/jwt-auth-express/def456 \
  cadi://fn/db-client-postgres/ghi789
→ Shows:
  - Compatibility: ✅ All compatible
  - Gaps: Error handler missing
  - Suggested fixes
  - Composition order

# Validate existing component
cadi validate cadi://fn/http-server-express/abc123
→ Shows:
  - Tests: 94 passed, 0 failed
  - Type safety: ✅ All correct
  - Contracts: ✅ All satisfied
  - Security scan: ✅ No issues
  - Coverage: 92%
```

**Token Cost**: ~50 tokens

### Step 6: Build Project

```bash
# Show build plan without executing
cadi build --plan build.cadi.yaml
→ Shows:
  Step 1: Resolve components
    ✓ http-server (cached)
    ✓ auth (cached)
    ✓ database (cached)
    ⧗ Generate task-routes
    ⧗ Generate error-handler
  
  Step 2: Generate missing components
  Step 3: Assemble and link
  Step 4: Run tests
  Step 5: Generate outputs

# Execute full build
cadi build build.cadi.yaml --incremental
→ Output:
  Built: ./dist/index.js
  Generated: ./docker/Dockerfile
  Tests: 324 passed
  Coverage: 89%
  Size: 245KB (gzipped)
  Time: 12 seconds
  
  Token Usage:
    - Reused code: 0 tokens
    - Generated code: 1,200 tokens
    - Total savings: ~12,000 tokens vs baseline
```

**Token Cost**: ~1,200 tokens for generation (only unique code)
**Time Saved**: 2+ hours of development

### Step 7: Deploy

```bash
# Push to registry
cadi publish ./dist --tag "task-api:1.0.0"

# Deploy Docker image
cadi deploy docker \
  --image task-api:1.0.0 \
  --registry docker.io \
  --kubeconfig ./k8s-config.yaml

# Run locally
cadi run ./dist --port 3000
```

---

## MCP Tool Reference (for LLM Agents)

### 🔍 Search Tools

**`cadi_search`**
```json
{
  "query": "HTTP server with routing",
  "language": "typescript",
  "limit": 10
}
→ Returns ranked results with IDs
   Use cadi_get_chunk with top results
```

**`cadi_resolve_alias`**
```json
{
  "alias": "myorg/utils/logger"
}
→ Returns CADI ID
   Fast cached lookup
```

**`cadi_suggest`**
```json
{
  "task": "Build a REST API with authentication",
  "language": "typescript"
}
→ Returns suggested components to explore
```

### 📦 Retrieval Tools

**`cadi_get_chunk`**
```json
{
  "chunk_id": "cadi://fn/http-server/abc123",
  "include_source": true
}
→ Returns metadata + source code
   Use format parameter to control size
```

### 🔗 Composition Tools

**`cadi_compose`**
```json
{
  "components": [
    "cadi://fn/http-server/abc123",
    "cadi://fn/auth/def456",
    "cadi://fn/database/ghi789"
  ],
  "interface": {
    "input": {"request": "object"},
    "output": {"response": "object"}
  }
}
→ Returns composition plan with gaps
   Shows what needs to be generated
```

### ⚙️ Generation Tools

**`cadi_generate`**
```json
{
  "description": "Express route handlers for task CRUD",
  "dependencies": [
    "cadi://fn/http-server/abc123",
    "cadi://fn/auth/def456"
  ],
  "interface": {
    "input": {"method": "string", "path": "string", "body": "object"},
    "output": {"status": "number", "data": "object"}
  }
}
→ Returns: New chunk ID
   Validates against contracts
   Stores in CADI registry
```

### 🏗️ Build Tools

**`cadi_build`**
```json
{
  "spec": "cadi_version: 1.0\nproject:\n  name: api\n..."
}
→ Returns build status and results
   Handles resolution → generation → assembly → validation
```

**`cadi_validate`**
```json
{
  "chunk_id": "cadi://fn/new-handler/xyz123"
}
→ Returns validation results
   Tests, type checks, contracts, security
```

### 🌐 Cross-Language Tools

**`cadi_find_equivalent`**
```json
{
  "chunk_id": "cadi://fn/sort/typescript-abc123",
  "target_language": "rust"
}
→ Returns equivalent implementations
   Shows confidence scores
```

---

## Example: Building a Web Service (LLM Agent Workflow)

**Agent Task**: "Build a REST API for managing tasks with authentication"

```
Step 1: SEARCH (100 tokens saved)
cadi_search("Express HTTP server framework")
→ [http-server/abc123, http-server/def456, http-server/ghi789]

Step 2: EVALUATE
cadi_get_chunk("http-server/abc123", include_source=false)
→ Metadata: 324 usages, 0.95 quality, 92% coverage

Step 3: COMPOSE (50 tokens saved)
cadi_search("JWT authentication")
→ [jwt-auth/def456]

cadi_search("database client")
→ [db-client/ghi789]

cadi_compose([http-server/abc123, jwt-auth/def456, db-client/ghi789])
→ Compatible, gap: error handler needed

Step 4: SEARCH for error handler
cadi_search("Express error handling middleware")
→ [error-handler/jkl012]

cadi_compose([http-server/abc123, jwt-auth/def456, db-client/ghi789, error-handler/jkl012])
→ All compatible, no gaps

Step 5: GENERATE only business logic (1,200 tokens)
cadi_generate(
  description="Task CRUD route handlers (POST /tasks, GET /tasks/:id, etc.)",
  dependencies=[http-server/abc123, auth/def456, db/ghi789],
  interface={input: {method, path, body}, output: {status, data}}
)
→ Generated: task-routes/new123

Step 6: BUILD (50 tokens)
cadi_build(spec_with_all_components)
→ Result: ./dist with tests passing

TOTAL TOKENS: 100 + 50 + 1,200 + 50 = 1,400 tokens
BASELINE: ~12,000 tokens (from scratch)
SAVINGS: 87%
```

---

## Performance Metrics

### Token Efficiency by Project Type

| Project Type | Traditional | CADI | Savings |
|---|---|---|---|
| REST API (5 endpoints) | 8,000 | 800 | 90% |
| CLI Tool | 3,000 | 600 | 80% |
| React Component Library | 12,000 | 2,000 | 83% |
| Microservice | 15,000 | 2,500 | 83% |
| Full Stack App | 25,000 | 3,500 | 86% |

### Build Performance

| Operation | Time | Notes |
|---|---|---|
| Search | <100ms | Cached, highly optimized |
| Compose | <50ms | In-memory graph analysis |
| Generate | 5-30 seconds | Depends on LLM latency |
| Build (incremental) | 2-5 seconds | Cached, parallel execution |
| Deploy | 10-60 seconds | Depends on target platform |

### Code Reuse by Component

- HTTP servers: 95% reuse (only routing differs)
- Authentication: 85% reuse (only config differs)
- Database clients: 90% reuse (schema changes only)
- Business logic: 0% reuse (custom for each project)

---

## Best Practices

### 1. Search First, Generate Last
❌ Bad: Generate all code immediately
✅ Good: Search for existing components first, generate only unique parts

### 2. Use Strong Interfaces
❌ Bad: `cadi_generate(description="Some handler")`
✅ Good: `cadi_generate(description="...", interface={input/output types})`

### 3. Validate Before Building
❌ Bad: Build with untested composition
✅ Good: Run `cadi compose` first to verify compatibility

### 4. Version Your Specifications
❌ Bad: Single build.cadi.yaml
✅ Good: build.cadi.yaml + VERSION + CHANGELOG

### 5. Track and Measure
❌ Bad: Not monitoring token savings
✅ Good: Log token usage, track metrics over time

---

## Troubleshooting

### No components found
```
1. Broaden search query: "HTTP server" → "web framework"
2. Check language filter: --language typescript
3. Ensure registry is up: cadi status --registry
```

### Composition has gaps
```
1. Review gap report: cadi compose <components>
2. Search for missing piece
3. If not found, it's a generate candidate
4. Use cadi_generate with strong interface
```

### Build fails
```
1. Check tests: cadi validate <chunk-id>
2. Verify contracts: cadi get <chunk-id> --format full
3. Check dependencies: cadi graph <chunk-id>
4. Run with --verbose for details
```

### Token usage too high
```
1. Are you searching first? cadi_search is 50 tokens
2. Use summaries not full source: include_source=false
3. Reference IDs, don't pass full code
4. Batch searches, don't repeat
```

---

## Advanced Topics

### Semantic Equivalence
Components with identical semantics have identical hashes:
```
TypeScript: function sort(arr: number[]): number[]
Rust: fn sort(arr: &[i32]) -> Vec<i32>
→ Same semantic hash → Same CADI ID
→ Transparent cross-language reuse
```

### Contract-Based Generation
When generating code, CADI validates against contracts:
```yaml
contracts:
  precondition: "array must be non-null"
  postcondition: "result must be sorted"
  complexity: "O(n log n)"
```

### Incremental Builds
Changes propagate efficiently through the graph:
```
Update: task schema
Affected: db queries → route handlers → tests
Rebuild: Only affected components
```

### Federation
Use multiple registries:
```bash
cadi registry add public https://registry.cadi.dev
cadi registry add private https://my-org.cadi.dev
cadi search --registry private "internal tools"
```

---

## Summary

CADI enables LLM agents and developers to:
1. **Search** for existing components (~50 tokens)
2. **Compose** them together (gap analysis)
3. **Generate** only unique code (~20% of typical)
4. **Build** with validation and testing
5. **Save** 80-90% of token budget

The result: **Faster development, 87% token savings, proven correctness through reuse.**

Start with `cadi search`, always compose before generating, and measure your savings.
