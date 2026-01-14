# CADI Forward Acceleration Implementation Roadmap

## 🎯 North Star Vision

**CADI's true power: Enable forward acceleration of novel, unseen problems.**

Yesterday's todo app contributes reusable atoms (JWT auth, generic CRUD, error-handling patterns) to today's entirely new project (real-time collaboration tool, AI agent framework). This reuse happens semantically and automatically, reducing boilerplate and letting LLMs focus on innovative "glue" logic.

The system works without manual chore—it's designed for seamless scaffolding where LLMs interact at a high semantic level, while low-level details (hashes, linking) are abstracted away until build time.

---

## 📊 Current State Assessment

### ✅ Already Implemented
- **Semantic Extraction Core** (`internal/cadi-core/src/semantic.rs`): SemanticNorm, SemanticOperation, SemanticEffect types defined
- **Normalizer** (`internal/cadi-core/src/normalizer.rs`): Alpha-renaming for semantic equivalence
- **Search Engine Skeleton** (`internal/cadi-registry/src/search.rs`): Multi-modal search structure (Semantic, Textual, Structural, Compositional, Hybrid)
- **Embeddings Infrastructure** (`internal/llm/src/embeddings.rs`): EmbeddingProvider trait, OpenAI & mock implementations
- **MCP Tools** (`cmd/cadi-mcp-server/src/tools.rs`): 8+ tools defined (cadi_search, cadi_get_chunk, cadi_import, cadi_publish, cadi_build, cadi_plan, etc.)
- **Manifest Schema** (`cadi-spec/manifest.schema.json`): Build graph structure for DAG composition
- **Registry** (`internal/cadi-registry/src`): Federation and client infrastructure

### ⚠️ Gaps to Fill

1. **Semantic Search + Vector Indexing** (HIGH PRIORITY)
   - Embeddings stored but not indexed in SurrealDB with MTREE
   - Text search only partial—needs inverted index with scoring
   - No chunked metadata normalization for search relevance
   - Gap: LLM must manually discover atoms; no semantic matching

2. **Semantic Hashing & Deduplication** (HIGH PRIORITY)
   - Normalizer works but not used in import pipeline
   - Chunks not deduplicated by semantic hash
   - Gap: Novel projects can't discover equivalent solutions from past

3. **Graph-Based Linking** (HIGH PRIORITY)
   - No DEPENDS_ON, REFINES, EQUIVALENT_TO edges
   - No automatic dependency resolution
   - Gap: Manual wiring required; composition feels like a chore

4. **CBS (CADI Build Spec) Schema & Parser** (MEDIUM PRIORITY)
   - Manifest schema exists but not the simpler CBS format for LLMs
   - No declarative query/generate sections
   - Gap: LLM can't easily express "use this atom + generate this new part"

5. **Build Engine Scaffolding** (MEDIUM PRIORITY)
   - No automatic composition (inject deps, generate imports, link modules)
   - No "needs_generation" feedback loop
   - Gap: Build doesn't feel automatic or AI-friendly

6. **MCP Tool Implementation** (MEDIUM PRIORITY)
   - Tool definitions exist but not the backend logic
   - cadi_search returns placeholders, not real results
   - cadi_build doesn't scaffold

---

## 🛠️ Implementation Plan (Phase 2: Forward Acceleration)

### Stage 1: Semantic Search + Vector Indexing (~2-3 days)
**Goal**: LLM can search semantically and discover atoms even with different keywords.

#### 1.1 Enhance Search Engine (`internal/cadi-registry/src/search.rs`)
- **Implement hybrid search**:
  - Text: Tokenize metadata/summaries with BM25 scoring
  - Semantic: Use embeddings with cosine similarity
  - Structural: Match function signatures, interfaces
  - Score ranking: Semantic match (50%) + Text match (30%) + Usage/quality (20%)
- **Add search result ranking**:
  - Relevance score based on embedding distance
  - Boost high-usage, well-tested components
  - Language/platform compatibility filtering

#### 1.2 Integrate SurrealDB Vector Search (`internal/cadi-registry/src/db.rs`)
- Add MTREE index on embedding vectors
- Query: Semantic search with <100ms latency
- Schema: Each chunk stores normalized metadata + embedding

#### 1.3 Metadata Normalization for Search
- Extract summaries from code (top comments, function sigs)
- Store: name, description, language, concepts, dependencies
- Example: JWT auth chunk → "user authentication middleware, JWT tokens, Node.js Express"

**Result**: 
```bash
cadi_search("JWT authentication for user sessions")
→ Returns top 5 matches with scores + minimal metadata (~50 tokens)
```

---

### Stage 2: Semantic Hashing & Deduplication (~2-3 days)
**Goal**: Identical code semantics always produce identical chunks, enabling automatic reuse discovery.

#### 2.1 Enhance Normalizer (`internal/cadi-core/src/normalizer.rs`)
- Extend beyond TypeScript to all languages (Python, Rust, etc.)
- Use tree-sitter for robust parsing
- Normalize:
  - Variable names (var_0, var_1, ...)
  - Whitespace & formatting
  - Comment stripping
  - Import order
- Output: canonical code + semantic hash

#### 2.2 Import Pipeline with Deduplication
- **Atomizer** integrates normalizer:
  1. Parse code
  2. Normalize to canonical form
  3. Compute semantic hash
  4. Check registry: If hash exists, deduplicate (create alias link)
  5. If new, create chunk + store hash
- Example: Todo app's `addTask` function → same hash in blog app → automatically linked

#### 2.3 Semantic Equivalence Detection
- Hashes matching → Semantically equivalent
- Create `EQUIVALENT_TO` edge in graph
- LLM sees: "This CRUD function from todo app is semantically equivalent to what you need"

**Result**:
```bash
Import blog project → Auto-detect JWT auth matches todo's → Link as reusable
Tokens saved: ~500 (no need to reimplement or describe equivalent logic)
```

---

### Stage 3: Graph-Based Linking (~2-3 days)
**Goal**: Automatic dependency resolution and scaffolding via edges.

#### 3.1 Graph Schema & Edges (`internal/cadi-core/src/graph/`)
Define edge types:
```rust
enum GraphEdge {
    DEPENDS_ON,      // Chunk A uses Chunk B
    REFINES,         // Chunk B is an optimized variant of A
    EQUIVALENT_TO,   // Chunks A and B are semantically equivalent
    IMPLEMENTS,      // Chunk A implements interface X
    SATISFIES,       // Chunk A satisfies constraint/requirement
}
```

#### 3.2 SurrealDB Graph Queries
- Store edges as relations
- Query: "Find all chunks that JWT chunk depends on"
- Transitive closure: "What does blog.comments.threading depend on?"

#### 3.3 Automatic Dependency Resolution
- **When building**: Walk graph edges to collect all transitive deps
- **Inject automatically**: Map deps to imports/module federation in generated code
- Example: Resolve `blog.content` → pulls `todo.postgres_crud` + `todo.logger` auto-imported

#### 3.4 Interface Compatibility Checking
- Each chunk exports semantic interface (signature, I/O types)
- Edges include `SATISFIES` with interface requirement
- Build engine verifies composition is type-safe

**Result**:
```yaml
# CBS references IDs; build engine walks graph
nodes:
  - id: auth
    source_cadi: chunk:sha256:abc123  # from todo app
  - id: content_db
    query: "generic CRUD for PostgreSQL"  # auto-resolves via search + graph
```

---

### Stage 4: CBS (CADI Build Spec) for LLMs (~2 days)
**Goal**: Simple, human-readable YAML format that LLMs generate effortlessly.

#### 4.1 CBS Schema (`cadi-spec/build-spec.schema.json`)
```yaml
build_spec:
  version: "1.0"
  project:
    name: "blog-app"
    description: "Blog with comments and real-time sync"
  
  # Reuse from existing projects
  components:
    - id: auth
      source: chunk:sha256:abc123  # or query
      language: typescript
    
    - id: content_db
      query: "generic CRUD operations for PostgreSQL with TypeScript"
      # Build engine searches + resolves semantically
      expected_interface:
        inputs: [userId, data]
        outputs: [result]
    
    - id: comments_threading
      generate: true  # Mark as novel
      depends_on: [auth, content_db]
      description: "Threaded comments with conflict resolution"
      # LLM provides code snippet + dependencies
  
  # Build targets
  targets:
    - name: api
      platform: nodejs
      components: [auth, content_db, comments_threading]
    
    - name: worker
      platform: nodejs
      components: [content_db, comments_threading]
```

#### 4.2 CBS Parser & Validator
- Parse YAML to structured build spec
- Validate:
  - All referenced chunks exist (or marked for generation)
  - Dependencies form DAG (no cycles)
  - Interfaces compatible

#### 4.3 LLM Guidance for CBS Generation
- Provide template + examples
- Instruct: "Use queries for 'reuse first'; mark novel parts as generate: true"
- Result: LLM generates ~50-100 lines YAML + code snippets (vs. 2000+ lines)

**Result**:
```bash
LLM generates blog.build-spec.yaml
→ 60 lines (reuses auth, CRUD; generates comments logic)
vs. ~2000 lines if all written from scratch
```

---

### Stage 5: Build Engine with Automatic Scaffolding (~3-4 days)
**Goal**: Transform CBS + chunks into runnable project without manual wiring.

#### 5.1 Build Engine Pipeline
1. **Resolve**: Queries → Search → Chunks; collect transitive deps via graph
2. **Validate**: Type/interface compatibility checks
3. **Compose**: Link chunks (inject imports, module federation)
4. **Generate**: Fill in gaps for novel parts (scaffolding)
5. **Execute**: Transpile, test, bundle; output runnable project

#### 5.2 Composition Logic
- **For TypeScript**: Auto-generate imports + module exports
  ```typescript
  // Generated by build engine
  import { authenticate } from "./chunks/auth";
  import { createTask } from "./chunks/crud";
  import { createComment } from "./chunks/comments";  // novel
  
  // Re-export for use
  export { authenticate, createTask, createComment };
  ```
- **For Python**: Auto-import classes, inject factory functions
- **For Rust**: Workspace setup, trait implementations

#### 5.3 Feedback Loop for Novel Parts
- If novel code can't be resolved:
  - Return "needs_generation" with specifics
  - LLM iterates with summaries (not full code)
  - Minimal context switching

#### 5.4 Output & Caching
- Output: Runnable project (npm, pip, cargo ready)
- Cache intermediate builds (MTREE indexing helps)
- Attestation: Provenance from chunks → build receipt

**Result**:
```bash
cadi build blog.build-spec.yaml --target api
→ Resolves chunks, scaffolds imports, outputs src/ ready to `npm start`
Tokens saved in iteration: ~3000 (no back-and-forth on wiring)
```

---

### Stage 6: MCP Tool Implementation (~2-3 days)
**Goal**: LLM uses CADI tools naturally within existing workflows.

#### 6.1 Implement Backend for Each Tool
- **cadi_search**: Call search engine, return top 5 results + metadata
- **cadi_resolve_alias**: Lookup alias in registry, return chunk ID
- **cadi_get_chunk**: Fetch chunk + dependencies, return interface (not full code by default)
- **cadi_import**: Analyze project, atomize, deduplicate, return chunk IDs + aliases
- **cadi_build**: Parse CBS, build, return runnable project + receipt
- **cadi_suggest**: AI-assisted discovery (use LLM to rank search results for task)

#### 6.2 Response Formatting (NO READ Pattern)
- **Always return interfaces, not source code**
- Interface includes: function signature, input/output types, dependencies, examples
- Keep response ~100-200 tokens (vs. 500+ for source)
- Lazy load: If LLM needs full code, use `include_source=true`

#### 6.3 Error Handling & Iteration
- Clear error messages guide LLM
- Suggest alternatives: "JWT not found; try 'auth middleware' or browse options"
- Support refinement: `cadi_search` with filters, `cadi_suggest` for ranking

**Result**: CADI tools feel like natural extensions of LLM workflow.

---

### Stage 7: Forward-Acceleration Demo (~2-3 days)
**Goal**: Demonstrate the full vision: Todo app → Blog app via semantic reuse.

#### 7.1 Scenario
1. **Day 1**: Import todo app into CADI
   - Atomize JWT auth, PostgreSQL CRUD, error handling
   - Publish to registry with aliases
2. **Day 2**: Build blog app using CADI
   - Search for "JWT auth" → find todo's chunk
   - Search for "CRUD operations" → find todo's CRUD adapted for blog posts
   - Generate novel "comment threading" + import from CADI
   - Build scaffolds all wiring automatically
3. **Day 3**: Build real-time sync tool
   - Reuse JWT, CRUD, logger from todo
   - Novel: WebSocket layer
   - Estimated reuse: 70-80%

#### 7.2 Measurement
- Tokens: Day 1 (300 for import) + Day 2 (600 for search + CBS) vs. ~5000 from scratch
- Build time: <5 minutes (vs. hours of manual wiring)
- Code reuse: 65-75% from past projects

#### 7.3 LLM Prompt & Workflow
```markdown
# Forward Acceleration with CADI

You're building new projects. ALWAYS search CADI first.

**Workflow:**
1. Task: "Build a blog app with JWT auth, content storage, comments"
2. Decompose: "I need (1) auth, (2) DB ops, (3) comment threading"
3. Search:
   cadi_search("JWT authentication middleware for Node.js")
   → Find todo app's auth chunk
   cadi_search("generic CRUD operations for PostgreSQL")
   → Find todo app's CRUD chunk
4. Reuse: Create build-spec.yaml with these chunks + novel comment code
5. Build: `cadi_build blog.build-spec.yaml` → scaffolded project
6. Save: cadi_import → publish new chunks for next project's reuse

**Token Savings**: 
- No need to read/understand existing implementations
- Search: 50 tokens
- Interface checks: 50 tokens
- Novel code: 200 tokens
- Total: 300 tokens vs. 2000+ from scratch (85% savings!)
```

---

### Stage 8: Documentation & Guidelines (~2 days)
**Goal**: Help LLMs and developers use CADI's forward acceleration.

#### 8.1 Documentation
- [CADI-First Workflow](docs/cadi-first-workflow.md): How to search, reuse, build
- [Semantic Search Guide](docs/semantic-search.md): Effective queries for discovery
- [CBS Specification](docs/build-spec.md): Format + LLM prompt template
- [Graph Query Patterns](docs/graph-patterns.md): Tracing dependencies, finding variants

#### 8.2 LLM Prompts
- Embed CADI-first prompt in MCP server resources
- Prompt template for build-spec generation
- Iterative guidance for composition/novel parts

---

## 🎯 Success Metrics

### Token Efficiency
- Forward project: **<1000 tokens** (vs. 5000+ without CADI)
- 80%+ token savings on reusable components

### Development Speed
- Build time: **<5 minutes** (search + resolve + scaffold)
- Novel code focus: **60-80% reuse**, only **20-40% new**

### Code Quality
- No manual wiring → fewer bugs
- Semantic deduplication → DRY across projects
- Interface checks → composition safety

### Developer Experience
- No "paste codebases" → semantic discovery
- Seamless scaffolding → feels effortless
- Clear iteration loops → debugging is guided

---

## 📋 Implementation Order (Recommended)

1. **Stage 1**: Semantic Search + Vector DB (foundation)
2. **Stage 2**: Semantic Hashing + Dedup (enable discovery)
3. **Stage 3**: Graph Linking (enable auto-composition)
4. **Stage 4**: CBS for LLMs (enable declarative builds)
5. **Stage 5**: Build Engine (enable scaffolding)
6. **Stage 6**: MCP Tools (enable LLM integration)
7. **Stage 7**: Demo (validate end-to-end)
8. **Stage 8**: Documentation (guide future use)

---

## 🚀 Expected Outcome

**CADI becomes the semantic backbone of LLM-assisted development:**
- LLMs search for atoms semantically (not grep)
- LLMs compose via declarative specs (not manual wiring)
- Novel projects benefit from all past solutions (not just rebuilds)
- Development feels like **collaborative problem-solving**, not boilerplate writing

**The invariant holds:**
$$\text{LLM} + \text{CADI} > \text{LLM alone}$$

Cost, tokens, speed, and quality all improve. This is the north star.
