# CADI: "NO READ" Pattern - Implementation Guide

## 🎯 Core Concept

**LLMs should NEVER read source code to use CADI components.**

Instead:
1. **LLM searches** → Gets component INTERFACES (summaries)
2. **LLM chooses** → Gets detailed INTERFACE metadata
3. **LLM builds** → Specifies what to compose
4. **System builds** → ONLY THEN retrieves source code

---

## 📋 What's Been Built

### ✅ Foundation (DONE)

1. **`ComponentInterface` model** (`internal/cadi-core/src/interface.rs`)
   - Complete data structure for component metadata
   - Everything an LLM needs to understand + use a component
   - NO source code storage
   - ~500 bytes per component (vs 50KB for source)

2. **`InterfaceExtractor`** (`internal/cadi-core/src/interface_extractor.rs`)
   - Extracts interfaces FROM source code automatically
   - Uses tree-sitter for robust AST parsing
   - Creates ComponentInterface without storing source

3. **MCP Response Format** (`cmd/cadi-mcp-server/src/responses.rs`)
   - `search_response()` - Returns interfaces only
   - `get_interface_response()` - Detailed interface metadata
   - `composition_advice()` - What fits together
   - `compatibility_check()` - Can I use X with Y?
   - NO full source code in responses

4. **`CompositionMatcher`** (`internal/cadi-core/src/composition.rs`)
   - Answers: "What can compose with what?"
   - Type checking (output → input)
   - Role-based heuristics
   - Suggestion ranking

5. **Documentation** (`docs/NO_READ_PATTERN.md`)
   - Complete explanation
   - Examples
   - Token accounting
   - Success metrics

---

## 🚀 What Needs to Happen Next

### Phase 1: Extract & Store Interfaces (Week 1)

#### Task 1.1: Batch Extract Interfaces from Existing Chunks
```rust
// In a new tool: cadi extract-interfaces

for chunk in all_stored_chunks {
    let interface = InterfaceExtractor::for_typescript()
        .extract(&chunk.source_code, ExtractionContext {
            id: chunk.id.clone(),
            name: chunk.name.clone(),
            // ... 
        })?;
    
    storage.store_interface(&interface)?;
}
```

**Location**: `cmd/cadi/src/commands/extract_interfaces.rs`

**Deliverable**: CLI command that runs extraction on all existing chunks

---

#### Task 1.2: Store Interfaces in Database
```sql
-- Update SurrealDB schema

DEFINE TABLE component_interface SCHEMAFULL;
DEFINE FIELD id ON TABLE component_interface TYPE string UNIQUE;
DEFINE FIELD component_id ON TABLE component_interface TYPE string;
DEFINE FIELD signature ON TABLE component_interface TYPE string;
DEFINE FIELD role ON TABLE component_interface TYPE string;
DEFINE FIELD summary ON TABLE component_interface TYPE string;
DEFINE FIELD inputs ON TABLE component_interface TYPE array;
DEFINE FIELD output ON TABLE component_interface TYPE object;
DEFINE FIELD usage_examples ON TABLE component_interface TYPE array;
DEFINE FIELD compatible_with ON TABLE component_interface TYPE array;
DEFINE FIELD side_effects ON TABLE component_interface TYPE array;

-- Link to chunks
DEFINE TABLE chunk_has_interface TYPE RELATION FROM chunk TO component_interface;
```

**Location**: `schemas/surreal.surql`

**Deliverable**: Schema update + migration script

---

### Phase 2: Update MCP Server (Week 1-2)

#### Task 2.1: Redesign `cadi_search` Tool
```rust
// OLD (WRONG)
cadi_search("HTTP server")
  → returns [{ id, source_code: "2000 lines..." }]

// NEW (RIGHT)
cadi_search("HTTP server")
  → returns [{
      id: "cadi://fn/...",
      signature: "fn(config) -> Server",
      summary: "Express.js HTTP server",
      inputs: [...],
      output: {...},
      examples: ["..."],
      compatible_with: [...]
    }]
```

**Location**: `cmd/cadi-mcp-server/src/tools/search.rs`

**File Changes**:
- Remove source_code from search results
- Use `to_mcp_response()` method
- Add to_json/from_json for interface structs

---

#### Task 2.2: Add `cadi_get_interface` Tool
```rust
// New tool: Returns full interface (still no source)
cadi_get_interface("cadi://fn/...")
  → returns {
      id, signature, summary, inputs, output,
      behavior, preconditions, postconditions,
      side_effects, composition_points,
      all_examples, all_compatible_with,
      quality_metrics
    }
```

**Location**: `cmd/cadi-mcp-server/src/tools/get_interface.rs`

**Deliverable**: New MCP tool

---

#### Task 2.3: Add `cadi_composition_advice` Tool
```rust
// New tool: "How do I compose these?"
cadi_composition_advice("cadi://fn/A", "cadi://fn/B")
  → returns {
      component_a: {...},
      component_b: {...},
      compatible: true/false,
      composition_modes: ["direct", "middleware", ...],
      examples: ["..."],
      next_steps: "Use cadi_build"
    }
```

**Location**: `cmd/cadi-mcp-server/src/tools/composition.rs`

**Deliverable**: New MCP tool

---

#### Task 2.4: Separate Source Code Retrieval
```rust
// Only for actual builds - NOT for interface inspection
cadi_build_get_source("cadi://fn/...")
  → { source_code: "..." }
  // Called only during cadi_build execution
  // Never exposed to LLM directly
```

**Location**: `cmd/cadi-mcp-server/src/build.rs`

**Deliverable**: Build system retrieves source only when needed

---

### Phase 3: Update CLI (Week 2)

#### Task 3.1: Redesign `cadi search` Command
```bash
# OLD
$ cadi search "HTTP server"
Found: express-server
Source:
  (2000 lines displayed)

# NEW
$ cadi search "HTTP server"
Found:
  ID: cadi://fn/express-server/abc
  Role: http_server
  Signature: createServer(config: ServerConfig): HTTPServer
  Summary: Express.js HTTP server
  
  Inputs:
    - config: ServerConfig (required)
  
  Output:
    - type: HTTPServer
  
  Examples:
    const server = createServer({port: 3000})
    server.on('request', handler)
  
  Compatible with:
    - cadi://fn/jwt-auth/...
    - cadi://fn/cors-middleware/...
  
  To see full source, run: cadi get --source <id>
```

**Location**: `cmd/cadi/src/commands/search.rs`

---

#### Task 3.2: Add `cadi get` Interface Display
```bash
$ cadi get cadi://fn/express-server/abc
(same as above - default to interface)

$ cadi get cadi://fn/express-server/abc --source
(shows source with warning: "For reference only, not needed to use this component")

$ cadi get cadi://fn/express-server/abc --composition
(shows composition points and what can be plugged in)
```

**Location**: `cmd/cadi/src/commands/get.rs`

---

### Phase 4: Testing & Validation (Week 2-3)

#### Task 4.1: Integration Test: LLM Uses CADI Without Reading

```rust
#[tokio::test]
async fn test_llm_never_reads_source() {
    // Setup: Add 10 components to CADI
    setup_test_components().await;
    
    // Phase 1: LLM searches (should get interfaces)
    let search_response = mcp.cadi_search("HTTP server");
    for result in search_response {
        assert!(!result.contains("source_code"), 
                "Search result should NOT contain source");
        assert!(result.contains("signature"), 
                "But MUST contain signature");
    }
    
    // Phase 2: LLM chooses components
    let interface = mcp.cadi_get_interface("cadi://fn/express");
    assert!(!interface.contains("fn app()"), 
            "Interface should NOT contain source");
    
    // Phase 3: LLM composes
    let composition = mcp.cadi_composition_advice("express", "jwt-auth");
    assert!(composition["compatible"].is_boolean());
    
    // Phase 4: Only during BUILD do we retrieve source
    let build_result = mcp.cadi_build(build_spec);
    // At this point, source IS retrieved (internally)
    // But LLM never sees it
    
    assert_token_cost_under(1000); // ~87% savings
}
```

**Location**: `tests/no_read_integration.rs`

---

#### Task 4.2: Token Accounting Test

```rust
#[test]
fn test_token_efficiency() {
    let interface = test_interface();
    let json = serde_json::to_string(&interface).unwrap();
    
    // Interface should be <500 bytes
    assert!(json.len() < 500,
            "Interface too large: {} bytes (should be <500)",
            json.len());
    
    // Estimate tokens (1 token ≈ 4 bytes)
    let estimated_tokens = json.len() / 4;
    assert!(estimated_tokens < 125, // 500 bytes / 4
            "Interface costs {} tokens (should be <125)",
            estimated_tokens);
}
```

**Location**: `tests/token_efficiency.rs`

---

### Phase 5: Documentation & Guides (Week 3)

#### Task 5.1: LLM Prompt Template

Create `docs/LLM_PROMPTS.md`:

```markdown
# Using CADI as an LLM

## Your tools (never read source code)

### 1. Search for components
TOOL: cadi_search(query: string)
- Returns: List of component interfaces
- Contains: Signature, inputs, outputs, examples, compatibility
- Does NOT contain: Source code

### 2. Get component details
TOOL: cadi_get_interface(id: string)
- Returns: Complete interface metadata
- Contains: All details about what it does
- Does NOT contain: Source code

### 3. Check composition
TOOL: cadi_composition_advice(id_a: string, id_b: string)
- Returns: Can these components work together?
- Contains: Compatibility info, composition modes
- Does NOT contain: Source code

### 4. Build your project
TOOL: cadi_build(spec: BuildSpec)
- Spec contains: Component IDs to use + glue code you write
- Returns: Compiled output
- System internally retrieves source (you never see it)

## Workflow: Build an API in <1000 tokens

1. Search: "HTTP server" (100 tokens)
2. Get interface: examine choices (200 tokens)
3. Check composition: Will auth work? (50 tokens)
4. Build spec: What to assemble (200 tokens)
5. Generate glue code: Routes, handlers (400 tokens)
6. Build: System compiles (returned to you)

TOTAL: ~950 tokens
Traditional approach: 5,000+ tokens
SAVINGS: 81%
```

**Location**: `docs/LLM_PROMPTS.md`

---

#### Task 5.2: Developer Guide

Create `docs/CONTRIBUTING_INTERFACES.md`:

```markdown
# Contributing Components to CADI

## If you're adding a new component:

### 1. Write great source code
```typescript
/**
 * Creates an HTTP server with the given configuration
 * 
 * Behavior:
 * - Listens on the specified port
 * - Routes requests through middleware pipeline
 * - Supports both HTTP and HTTPS
 * 
 * Side Effects:
 * - Listens on network port
 * - May write access logs
 * - Establishes TCP connections
 */
export function createServer(config: ServerConfig): HTTPServer {
  // ... implementation
}
```

### 2. Let CADI extract the interface
```bash
$ cadi extract-interfaces ./my-component.ts
```

### 3. Verify the interface
```bash
$ cadi get cadi://fn/my-component/hash --interface
```

That's it! Your component is now usable by LLMs without code reading.
```

**Location**: `docs/CONTRIBUTING_INTERFACES.md`

---

## 📊 Success Metrics

### Token Usage
- [ ] `cadi_search` response: <200 tokens
- [ ] `cadi_get_interface` response: <300 tokens
- [ ] `cadi_composition_advice` response: <100 tokens
- [ ] Total per component: <1,000 tokens vs 5,000+ baseline

### Code Reuse
- [ ] LLM can build complete project from components
- [ ] Code reuse >70%
- [ ] Zero unnecessary source code reading

### Quality
- [ ] All existing tests pass
- [ ] New integration tests pass
- [ ] No performance regression
- [ ] Interfaces accurate and up-to-date

---

## 🔄 Validation Workflow

```bash
# 1. Extract interfaces from all chunks
$ cadi extract-interfaces --all

# 2. Start MCP server (will use interfaces)
$ cadi-mcp-server start

# 3. Run integration test
$ cargo test --test no_read_integration

# 4. Measure tokens
$ cargo test --test token_efficiency

# 5. Check quality
$ cadi quality-report
```

---

## 💡 Key Design Decisions

### Why separate Interface from Source?
- **Efficiency**: 500 bytes vs 50KB per component
- **Security**: Source never exposed unless building
- **Composability**: LLM understands contract without internals
- **Scalability**: Can have 100K components, LLM sees only summaries

### Why MCP tools return interfaces?
- **LLM context**: Small responses = more reasoning space
- **Trust**: LLM knows exactly what it's using
- **Composability**: Interface defines what can compose
- **Auditability**: Clear what LLM sees vs doesn't

### Why only build retrieves source?
- **Separation of concerns**: Planning vs execution
- **No accidental reads**: LLM can't accidentally request source
- **Clear boundaries**: Source is build artifact, not interface

---

## 🎬 Next Actions

1. **This week**: 
   - [ ] Implement interface extraction for all stored chunks
   - [ ] Update SurrealDB schema
   - [ ] Redesign `cadi_search` MCP tool

2. **Next week**:
   - [ ] Add `cadi_get_interface` tool
   - [ ] Add `cadi_composition_advice` tool
   - [ ] Update CLI commands
   - [ ] Integration test

3. **Week 3**:
   - [ ] Validation & testing
   - [ ] Documentation
   - [ ] Performance optimization
   - [ ] Release

---

**THE INVARIANT:**
> LLM + CADI > LLM alone (Cost, Tokens, Speed, Quality)

This implementation ensures that's true by keeping LLMs focused on what they're good at (composition & generation) while CADI handles code reuse.
