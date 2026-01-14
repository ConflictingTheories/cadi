# CADI Forward Acceleration: Complete Implementation Plan

## 🎯 The Vision (What We're Building)

**CADI's core innovation: Transform LLM-assisted development from "paste entire codebases into context" to "reference semantic atoms and compose."**

Yesterday's todo app doesn't just get rebuilt—it actively accelerates building entirely new projects. A blog app, a real-time collaboration tool, an AI agent framework—they all benefit from yesterday's JWT auth, generic CRUD, error handling, logging patterns.

**The invariant holds**: LLM + CADI > LLM alone (cost, tokens, speed, quality)

**Key difference from caching:**
- ❌ Not about rebuilding yesterday's project
- ✅ About accelerating tomorrow's novel problems  
- ✅ Semantic reuse (not keyword search)  
- ✅ Automatic composition (not manual wiring)  
- ✅ Forward acceleration (learning compounds)  

---

## 📊 The 8-Stage Implementation Roadmap

### Stage 1: Semantic Search + Vector Indexing (~2-3 days)
**Goal**: Enable LLMs to discover atoms semantically, even with different keywords.

**Deliverables**:
- Hybrid search engine (text BM25 + semantic cosine similarity)
- SurrealDB MTREE vector index for <100ms queries
- Metadata extraction from code (name, description, signatures, concepts)
- MCP tool `cadi_search` returns top 5 results + metadata (~50 tokens)

**Result**: `cadi_search("JWT auth for user sessions")` finds todo's JWT chunk

**Key file**: [STAGE1_SEMANTIC_SEARCH_SPEC.md](STAGE1_SEMANTIC_SEARCH_SPEC.md)

---

### Stage 2: Semantic Hashing & Deduplication (~2-3 days)
**Goal**: Identical code semantics = identical chunks across projects

**Deliverables**:
- Enhanced normalizer (all languages: TypeScript, Python, Rust, Go, Java)
- Alpha-renaming + comment stripping + whitespace normalization
- Semantic hash computation (canonical form → SHA256)
- Deduplication engine detects equivalents
- Import pipeline integration (auto-deduplicate on import)
- `EQUIVALENT_TO` graph edges for equivalent chunks

**Result**: Blog's `addTask` function automatically links to todo's identical `addTask`

**Key file**: [STAGE2_SEMANTIC_HASHING_SPEC.md](STAGE2_SEMANTIC_HASHING_SPEC.md)

---

### Stage 3: Graph-Based Linking (~2-3 days)
**Goal**: Automatic dependency resolution and scaffolding via semantic edges

**Deliverables**:
- Graph schema: DEPENDS_ON, REFINES, EQUIVALENT_TO, IMPLEMENTS, SATISFIES edges
- SurrealDB graph queries (transitive closure, cycle detection)
- Automatic dependency resolver (BFS traversal)
- Interface compatibility checking
- Build scaffolder (auto-generates imports for TypeScript, Python, Rust)

**Result**: Building blog pulls JWT + CRUD + logger automatically via graph

**Key file**: [STAGE3_GRAPH_LINKING_SPEC.md](STAGE3_GRAPH_LINKING_SPEC.md)

---

### Stage 4: CBS (Build Spec) for LLMs (~2 days)
**Goal**: Simple YAML format for LLMs to express projects without writing code

**Deliverables**:
- CBS schema (`build-spec.schema.json`): reuse, search, generate patterns
- CBS parser + validator
- Conversion to build plan
- LLM prompt template (explains CADI-first workflow)
- Examples: `blog-app.build-spec.yaml` (~80 lines, 70% reuse)

**Result**: LLM writes 80-100 line build spec instead of 2000+ lines

**Key file**: [STAGE4_CBS_SPEC.md](STAGE4_CBS_SPEC.md)

---

### Stage 5: Build Engine Scaffolding (~3-4 days)
**Goal**: Transform CBS + chunks into runnable project

**Deliverables**:
- Build pipeline: Resolve → Validate → Compose → Generate → Execute
- Query resolution (search queries → chunks)
- Dependency injection (graph walk → all transitive deps)
- Auto-import generation (TypeScript, Python, Rust)
- Scaffolding for novel parts
- Feedback loop (LLM iterates on needs_generation)
- Build output: runnable project + attestation

**Result**: `cadi build blog.build-spec.yaml` → project ready to `npm start`

**Implementation**: `internal/cadi-builder/src/builder.rs` (enhance)

---

### Stage 6: MCP Tool Implementation (~2-3 days)
**Goal**: LLMs use CADI naturally within existing workflows

**Deliverables**:
- Backend logic for all 8+ MCP tools
- `cadi_search`: semantic discovery
- `cadi_get_chunk`: fetch interfaces (not full code by default)
- `cadi_build`: execute build specs
- `cadi_import`: analyze + atomize projects
- `cadi_suggest`: AI-assisted discovery
- Response formatting (NO READ pattern: interfaces, not source)

**Result**: Tools feel like natural LLM extensions

**Implementation**: `cmd/cadi-mcp-server/src/tools.rs` (enhance)

---

### Stage 7: Forward-Acceleration Demo (~2-3 days)
**Goal**: Validate end-to-end: Todo app → Blog app via semantic reuse

**Scenario**:
1. **Day 1**: Import todo app
   - Atomize JWT, CRUD, error handling, logging
   - Publish to registry with semantic hashes
2. **Day 2**: Build blog app with CADI
   - Search for JWT → find todo's chunk
   - Search for CRUD → find todo's adapter
   - Generate comment threading + import
   - Build scaffolds all wiring
3. **Day 3**: Build real-time sync tool
   - Reuse JWT, CRUD, logger
   - Novel: WebSocket layer
   - Estimated reuse: 70-80%

**Measurements**:
- **Tokens**: Day 1 (300 for import) + Day 2 (600) vs. ~5000 from scratch
- **Build time**: <5 minutes
- **Code reuse**: 65-75% from past

**Files**: `examples/forward-acceleration-demo/`

---

### Stage 8: Documentation & Guidelines (~2 days)
**Goal**: Help LLMs and developers use CADI's forward acceleration

**Deliverables**:
- [docs/cadi-first-workflow.md](docs/cadi-first-workflow.md)
- [docs/semantic-search-guide.md](docs/semantic-search-guide.md)
- [docs/build-spec-reference.md](docs/build-spec-reference.md)
- [docs/graph-query-patterns.md](docs/graph-query-patterns.md)
- LLM prompt templates
- Integration guides (Claude, GPT-4, others)

---

## 🛠️ Technical Architecture

### Key Components

```
┌─────────────────────────────────────────────────────┐
│             CADI Ecosystem                          │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌────────────────────────────────────────────┐   │
│  │ Semantic Search + Vector Indexing (S1)    │   │
│  │ - BM25 text search                        │   │
│  │ - Cosine similarity on embeddings         │   │
│  │ - SurrealDB MTREE index                   │   │
│  │ - Result: Discover atoms semantically    │   │
│  └────────────────────────────────────────────┘   │
│                        ↓                           │
│  ┌────────────────────────────────────────────┐   │
│  │ Semantic Hashing (S2)                     │   │
│  │ - Normalizer (all languages)              │   │
│  │ - Alpha-renaming for equivalence          │   │
│  │ - Deduplication engine                    │   │
│  │ - Result: Identical semantics = same ID  │   │
│  └────────────────────────────────────────────┘   │
│                        ↓                           │
│  ┌────────────────────────────────────────────┐   │
│  │ Graph-Based Linking (S3)                  │   │
│  │ - DEPENDS_ON, REFINES, EQUIVALENT_TO      │   │
│  │ - Graph DB with MTREE                     │   │
│  │ - Transitive resolution                   │   │
│  │ - Result: Auto dependency injection       │   │
│  └────────────────────────────────────────────┘   │
│                        ↓                           │
│  ┌────────────────────────────────────────────┐   │
│  │ CBS (Build Spec) (S4)                     │   │
│  │ - YAML schema for LLMs                    │   │
│  │ - Reuse, search, generate patterns        │   │
│  │ - Parser + validator                      │   │
│  │ - Result: LLMs express projects, not code│   │
│  └────────────────────────────────────────────┘   │
│                        ↓                           │
│  ┌────────────────────────────────────────────┐   │
│  │ Build Engine (S5)                         │   │
│  │ - Resolve → Validate → Compose → Generate│   │
│  │ - Auto-import generation                  │   │
│  │ - Scaffolding                             │   │
│  │ - Result: Runnable project                │   │
│  └────────────────────────────────────────────┘   │
│                        ↓                           │
│  ┌────────────────────────────────────────────┐   │
│  │ MCP Tools (S6)                            │   │
│  │ - cadi_search, cadi_build, cadi_import    │   │
│  │ - LLM integration                         │   │
│  │ - Result: Tools feel native               │   │
│  └────────────────────────────────────────────┘   │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Data Flow: Forward Acceleration

```
Project A (Yesterday)
    ↓
[Normalize, Hash, Extract Metadata]
    ↓
SurrealDB
├── chunks table (semantic hash, metadata)
├── MTREE vector index (embeddings)
└── dependencies table (graph edges)
    ↓
Project B (Today) - Novel Idea
    ↓
[LLM: Decompose into components]
    ↓
cadi_search queries
    ├─ "JWT auth" → finds A's auth chunk (semantic match)
    ├─ "CRUD ops" → finds A's CRUD chunk (semantic match)
    └─ "novel feature" → LLM generates (marked in build spec)
    ↓
Build engine:
    ├─ Resolve queries → chunks
    ├─ Walk graph → all transitive deps
    ├─ Validate composition
    └─ Scaffold imports + wiring
    ↓
Runnable Project B
    ├─ 70-80% code from Project A (reused)
    └─ 20-30% novel code
```

---

## 📈 Success Metrics

### Token Efficiency
| Metric | Without CADI | With CADI | Savings |
|--------|------------|----------|---------|
| Read existing code | 500 | 0 | 100% |
| Understand patterns | 300 | 0 | 100% |
| Search/discovery | 100 | 50 | 50% |
| Build spec | 200 | 200 | 0% |
| Novel code | 1000 | 500 | 50% |
| **Total** | **2100** | **750** | **64%** |

### Development Speed
- **Without CADI**: 4-6 hours (read, understand, write, debug)
- **With CADI**: 30-45 minutes (search, spec, build, refine)
- **Speedup**: 6-10x faster

### Code Quality
- **Reuse rate**: 60-80% from past projects
- **Bug reduction**: Tested code from production
- **Consistency**: Patterns stay consistent
- **Type safety**: Interface checks prevent errors

### Developer Experience
- **No boilerplate**: Semantic composition handles linking
- **Natural workflow**: Search → compose → refine
- **Compound learning**: Each project makes future faster
- **Confidence**: Reused code is proven code

---

## 🚀 Implementation Timeline

### Phase 2A: Foundation (Weeks 1-2)
- **Stage 1**: Semantic Search (3 days)
- **Stage 2**: Semantic Hashing (3 days)
- **Stage 3**: Graph Linking (3 days)
- **Checkpoint**: Search and hash working; graph queries verified

### Phase 2B: Builder (Weeks 3-4)
- **Stage 4**: CBS Schema (2 days)
- **Stage 5**: Build Engine (4 days)
- **Checkpoint**: End-to-end CBS → runnable project

### Phase 2C: Integration & Demo (Weeks 5-6)
- **Stage 6**: MCP Tools (3 days)
- **Stage 7**: Forward-Acceleration Demo (3 days)
- **Checkpoint**: Todo → Blog demo validates vision

### Phase 2D: Documentation (Week 7)
- **Stage 8**: Docs + Prompts (2 days)
- **Polish**: Tests, examples, CI integration
- **Release**: Ready for public use

### Parallel (Throughout)
- **Testing**: Unit tests for each stage
- **CI/CD**: Ensure cargo test --all-features passes
- **Performance**: Monitor <100ms search, <50ms hash
- **Documentation**: Inline code comments + architecture docs

---

## 📋 File Structure (After Implementation)

```
cadi/
├── FORWARD_ACCELERATION_ROADMAP.md          # This document
├── STAGE1_SEMANTIC_SEARCH_SPEC.md           # Search engine design
├── STAGE2_SEMANTIC_HASHING_SPEC.md          # Hash & deduplicate
├── STAGE3_GRAPH_LINKING_SPEC.md             # Graph DB & linking
├── STAGE4_CBS_SPEC.md                       # Build spec schema
├── docs/
│   ├── cadi-first-workflow.md               # LLM workflow guide
│   ├── semantic-search-guide.md             # How to search
│   ├── build-spec-reference.md              # CBS format
│   └── graph-query-patterns.md              # Common queries
├── cadi-spec/
│   ├── build-spec.schema.json               # CBS JSON schema
│   └── graph-edges.schema.json              # Graph edge types
├── internal/
│   ├── cadi-core/src/
│   │   ├── normalizer.rs                    # (enhanced)
│   │   ├── deduplication.rs                 # (new)
│   │   ├── graph/mod.rs                     # (new)
│   │   ├── graph/edges.rs                   # (new)
│   │   └── semantic.rs                      # (enhanced)
│   ├── cadi-registry/src/
│   │   ├── search.rs                        # (enhanced)
│   │   ├── graph.rs                         # (new)
│   │   └── db.rs                            # (new)
│   ├── cadi-builder/src/
│   │   ├── build_spec.rs                    # (new)
│   │   ├── dependency_resolver.rs           # (new)
│   │   ├── builder.rs                       # (enhanced)
│   │   └── importer.rs                      # (enhanced)
│   └── llm/src/
│       ├── embeddings.rs                    # (enhanced)
│       └── store.rs                         # (new)
├── cmd/cadi-mcp-server/src/
│   ├── tools.rs                             # (enhanced)
│   ├── protocol.rs                          # (may enhance)
│   └── resources.rs                         # (update examples)
├── examples/
│   ├── forward-acceleration-demo/
│   │   ├── day1-import-todo/
│   │   ├── day2-build-blog/
│   │   ├── day3-build-realtime/
│   │   └── README.md
│   └── blog-app.build-spec.yaml             # Example CBS
├── tests/
│   ├── semantic_search_test.rs              # (new)
│   ├── semantic_deduplication_test.rs       # (new)
│   ├── graph_linking_test.rs                # (new)
│   ├── build_spec_test.rs                   # (new)
│   └── forward_acceleration_e2e.rs          # (new)
```

---

## 🎓 Key Concepts

### Semantic Atoms
Smallest unit of reusable code carrying:
- Normalized semantics (identical for equivalent code)
- Content hash (immutable ID)
- Metadata (name, description, concepts, interface)
- Dependencies (other atoms it needs)
- Graph edges (relationships to other atoms)

### NO READ Pattern
LLMs never read full source code; they work with interfaces:
- Function signature (~50 bytes)
- Input/output types (~30 bytes)
- Concepts/tags (~100 bytes)
- Example usage (~100 bytes)
- **Total: ~300 bytes vs. 50KB of source**

### Forward Acceleration
Each project compounds learning for future projects:
- **Day 1**: Todo app contributes atoms (auth, CRUD, logging)
- **Day 2**: Blog app reuses 70% from todo
- **Day 3**: Real-time tool reuses 80% from todo + blog
- **Day N**: Projects get 2x faster as atom library grows

### CADI Build Spec (CBS)
Declarative composition language for LLMs:
- Express: "Use this chunk" → `source: chunk:sha256:...`
- Express: "Find this" → `query: "JWT auth"`
- Express: "Generate this" → `generate: true`
- **Result**: Meaningful composition without manual wiring

---

## ✅ Acceptance Criteria

### Stage 1: Semantic Search
- [ ] Hybrid search returns relevant results for semantic queries
- [ ] Latency <100ms for 10K+ chunks
- [ ] MCP tool returns <200 tokens
- [ ] Embeddings generated and indexed in SurrealDB
- [ ] Unit tests pass with >90% coverage

### Stage 2: Semantic Hashing
- [ ] Identical code produces identical hash (all languages)
- [ ] Deduplication engine identifies equivalents
- [ ] Import pipeline auto-deduplicates
- [ ] Demo: Blog finds todo's auth by semantic hash
- [ ] Performance: <50ms hash for typical function

### Stage 3: Graph Linking
- [ ] Graph stores and queries edges (all types)
- [ ] Transitive dependency resolution <100ms
- [ ] Cycle detection prevents invalid compositions
- [ ] Interface compatibility checking works
- [ ] Auto-generated imports are valid code

### Stage 4: CBS
- [ ] Schema validates correctly
- [ ] Parser handles YAML → build plan
- [ ] Supports reuse, search, generate patterns
- [ ] LLM can generate valid specs
- [ ] Examples: blog app spec <100 lines, >60% reuse

### Stage 5: Build Engine
- [ ] Resolves queries → chunks
- [ ] Walks graph → all transitive deps
- [ ] Scaffolds imports for TS, Python, Rust
- [ ] Outputs runnable project
- [ ] Demo: `cadi build blog.build-spec.yaml` → npm start

### Stage 6: MCP Tools
- [ ] All 8+ tools implemented and working
- [ ] Responses follow NO READ pattern
- [ ] Integration with Claude, GPT-4 verified
- [ ] Error handling with clear guidance
- [ ] End-to-end workflows documented

### Stage 7: Demo
- [ ] Todo app atomized and published
- [ ] Blog app built using CBS
- [ ] 60-80% reuse demonstrated
- [ ] Token savings 64%+ vs. traditional
- [ ] Speed 6-10x faster

### Stage 8: Documentation
- [ ] Workflow guides for LLMs and developers
- [ ] API documentation complete
- [ ] 5+ examples showing different patterns
- [ ] Integration guides for popular LLMs
- [ ] README updated with forward acceleration

---

## 🔗 Interdependencies

```
Stage 1 (Search)
    ↓
Stage 2 (Hashing) ← depends on normalizer
    ↓
Stage 3 (Graphs) ← depends on S1 + S2
    ↓
Stage 4 (CBS) ← depends on S1 + S2
    ↓
Stage 5 (Builder) ← depends on S1 + S3 + S4
    ↓
Stage 6 (MCP) ← depends on S5
    ↓
Stage 7 (Demo) ← depends on S1-S6
    ↓
Stage 8 (Docs) ← depends on S1-S7
```

---

## 💡 Implementation Tips

### Start Small
- Begin with TypeScript only for Stage 2 (normalizer)
- Expand to other languages after core works

### Test Early
- Add unit tests for each stage before moving forward
- Use mock data until real SurrealDB queries work

### Performance First
- Benchmark Stage 1 search latency early
- Optimize MTREE index if needed
- Profile Stage 2 hash computation

### Documentation Matters
- Write high-level docs as you code
- Examples help LLMs understand patterns
- Inline comments explain non-obvious logic

### Iterate with Users
- Get early feedback on CBS format from LLMs
- Adjust based on what works/doesn't
- Refine prompts after Stage 7 demo

---

## 🎉 End State

**CADI becomes the semantic backbone of LLM-assisted development.**

- **LLMs search** for atoms semantically (not grep)
- **LLMs compose** via declarative specs (not manual wiring)
- **Novel projects** benefit from all past solutions (not just rebuilds)
- **Development feels** like collaborative problem-solving (not boilerplate)

**The invariant holds:**
$$\text{LLM} + \text{CADI} > \text{LLM alone}$$

Cost ↓, Tokens ↓, Speed ↑, Quality ↑

**This is the north star. This is forward acceleration.**

---

## 📞 Questions?

Refer to individual stage specs:
- [STAGE1_SEMANTIC_SEARCH_SPEC.md](STAGE1_SEMANTIC_SEARCH_SPEC.md)
- [STAGE2_SEMANTIC_HASHING_SPEC.md](STAGE2_SEMANTIC_HASHING_SPEC.md)
- [STAGE3_GRAPH_LINKING_SPEC.md](STAGE3_GRAPH_LINKING_SPEC.md)
- [STAGE4_CBS_SPEC.md](STAGE4_CBS_SPEC.md)

Or check the main README at [README.md](README.md)
