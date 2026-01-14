# CADI: The "NO READ" Pattern

**The core innovation that makes CADI work:**

## The Problem

```
Traditional LLM workflow:
  LLM: "I need to use function X"
  System: [hands LLM 2,000 lines of code]
  LLM: [wastes 800 tokens understanding the code]
  LLM: [finally uses 50 lines of it]
  
Result: 900+ wasted tokens per component
```

## The CADI Solution

```
CADI workflow:
  LLM: "What HTTP servers exist?"
  CADI: [returns 10 ComponentInterface summaries]
       {
         "id": "cadi://fn/express-server/abc",
         "summary": "Express.js HTTP server",
         "signature": "createServer(config: ServerConfig): HTTPServer",
         "inputs": [{"name": "config", "type": "ServerConfig"}],
         "output": {"type": "HTTPServer"},
         "examples": [
           "const server = createServer({port: 3000})",
           "server.on('request', handler)"
         ],
         "compatible_with": ["jwt-auth", "cors-middleware"],
         "side_effects": ["Listens on port 3000"]
       }
  LLM: [understands it in <100 tokens]
  LLM: [decides to use it]
  LLM: "Build me an API with this"
  CADI: [retrieves FULL SOURCE only when building]
  
Result: 87% token savings, LLM never reads the code
```

## Key Design Principles

### 1. Separation of Concerns

```
┌─────────────────────────────────────────────────────────────┐
│                     LLM Context Window                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ComponentInterface (small)                                 │
│  ├─ ID: cadi://fn/express-server/abc                       │
│  ├─ Signature: fn(config) -> Server                        │
│  ├─ Inputs: [config: ServerConfig]                         │
│  ├─ Output: Server                                          │
│  ├─ Examples: ["server.listen(3000)"]                      │
│  ├─ Compatible: [jwt-auth, cors]                           │
│  └─ Side effects: [Listens on port]                        │
│                                                             │
│  ← LLM NEVER READS:                                        │
│  Source code (50-2000 lines)                               │
│  Implementation details                                    │
│  Internal functions                                        │
│  Comments/documentation                                   │
│                                                             │
│  ← ONLY RETRIEVED WHEN:                                   │
│  Building/compiling                                        │
│  Running tests                                             │
│  Actual execution needed                                  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 2. Interface-First Design

Every component has TWO representations:

```rust
// 1. INTERFACE (what LLM reads) - <1KB
pub struct ComponentInterface {
    pub id: String,                              // Unique ID
    pub signature: String,                       // fn signature
    pub inputs: Vec<Parameter>,                  // What it takes
    pub output: TypeSignature,                   // What it returns
    pub behavior: String,                        // What it does
    pub usage_examples: Vec<String>,             // How to use
    pub compatible_with: Vec<CompatibleComponent>, // What works with
    pub side_effects: Vec<SideEffect>,          // Does I/O?
    pub constraints: Vec<String>,               // Can't do X?
    pub composition_points: Vec<CompositionPoint>, // Where to plug stuff
}

// 2. SOURCE CODE (what build system reads) - stored separately
pub struct Chunk {
    pub source_code: String,  // 50-2000 lines
    pub metadata: ChunkMetadata,
    pub interface: ComponentInterface,  // Always paired
}
```

### 3. MCP Tools Return ONLY Interfaces

```rust
// WRONG - includes source code
cadi_get("id") -> {
    source: "2000 lines of code...",  // ❌ WRONG
    interface: {...}
}

// RIGHT - interface only
cadi_get_interface("id") -> {
    interface: {
        signature: "...",
        examples: [...],
        compatible_with: [...]
    }
}

// Source only retrieved when:
cadi_get_source("id") -> { source: "..." }
cadi_build(spec) -> [retrieves sources only during compilation]
```

### 4. Composition API

LLM asks "What can I build?" → Gets interfaces

```rust
// LLM queries
cadi_search("HTTP server") 
  → returns 5 ComponentInterface objects
  → total <500 tokens

// LLM then asks "Can I use X with Y?"
cadi_compose([component_a, component_b])
  → returns compatibility check + composition plan
  → <100 tokens

// LLM builds the spec
build_spec = {
  components: [
    { id: "cadi://fn/express", as: "server" },
    { id: "cadi://fn/jwt-auth", as: "auth" },
    { id: "cadi://fn/postgres", as: "db" }
  ]
}

// CADI builds (NOW it retrieves sources)
cadi_build(build_spec)
  → retrieves 3 source files
  → assembles them
  → returns compiled output
```

## Token Accounting

### Traditional Approach
```
Input to LLM:
├─ User request (200 tokens)
├─ Express.js source (1,500 tokens)
├─ JWT auth source (800 tokens)
├─ Postgres client source (600 tokens)
└─ LLM generates (1,500 tokens)

TOTAL: ~5,000 tokens

PROBLEM: 90% is already-written code the LLM doesn't need to understand deeply
```

### CADI Approach
```
Step 1: Search (LLM reads interfaces)
├─ User request (200 tokens)
├─ 10 interface summaries (300 tokens)
└─ LLM chooses 3 components (100 tokens)

TOTAL COST: ~600 tokens

Step 2: Get details (LLM understands what it's using)
├─ 3 detailed interfaces (200 tokens)
└─ Glue code LLM writes (300 tokens)

TOTAL COST: ~500 tokens

FINAL: ~1,100 tokens total

SAVINGS: 78% reduction from traditional approach!
```

## Implementation Checklist

### Phase 1: Build the Interface Layer ✓
- [x] ComponentInterface data model (✓ done)
- [x] InterfaceExtractor (✓ done)
- [x] MCP response redesign (✓ done)

### Phase 2: Extract Interfaces from Existing Code
- [ ] Run interface extractor on all stored chunks
- [ ] Store interfaces in SurrealDB alongside source
- [ ] Create index for interface queries

### Phase 3: Update MCP Server
- [ ] Redesign cadi_search to return interfaces
- [ ] Add cadi_get_interface (interface only)
- [ ] Add cadi_get_source (source only, requires build context)
- [ ] Update cadi_build to retrieve sources during compilation

### Phase 4: Update CLI
- [ ] `cadi search` returns interfaces
- [ ] `cadi get <id>` returns interface summary
- [ ] `cadi get <id> --source` returns source (with warning)
- [ ] `cadi build` only retrieves source when needed

### Phase 5: Validation
- [ ] LLM can build project using only interfaces
- [ ] No direct source code reading required
- [ ] Token usage <1,200 per component
- [ ] Quality matches traditional approach

## Example: Building an API

### What the LLM sees

```json
{
  "search": "HTTP server framework",
  "results": [
    {
      "id": "cadi://fn/express-server/a1b2",
      "signature": "createServer(config: Config): Server",
      "summary": "Express.js HTTP server",
      "inputs": [
        {"name": "config", "type": "Config", "required": true}
      ],
      "output": {"type": "Server"},
      "examples": [
        "const server = createServer({port: 3000})",
        "server.on('request', handler)"
      ],
      "compatible_with": [
        {"id": "cadi://fn/jwt-auth/c3d4", "mode": "middleware"},
        {"id": "cadi://fn/cors-middleware/e5f6", "mode": "middleware"}
      ]
    }
  ]
}
```

LLM reasoning:
- "This is an Express server ✓"
- "Takes a config object ✓"
- "Returns a Server ✓"
- "Examples show how to use ✓"
- "Compatible with JWT auth ✓"

LLM NEVER SEES:
- 5,000 lines of Express.js implementation
- Internals of how it handles connections
- Dependency resolution
- Stream handling code

### What happens during build

```yaml
# LLM generates this build spec
cadi_version: "1.0"
project:
  name: "my-api"
  type: "rest-service"
  language: "typescript"

components:
  - id: "cadi://fn/express-server/a1b2"
    as: "http_server"
  - id: "cadi://fn/jwt-auth/c3d4"
    as: "auth_middleware"
  - id: "cadi://fn/postgres-client/e5f6"
    as: "db"

build:
  steps:
    - type: "compile"
      language: "typescript"
```

CADI's cadi_build tool:
1. Resolves all component IDs
2. Retrieves FULL SOURCE for each (now!)
3. Compiles/assembles
4. Returns executable

**Key: Source code retrieved ONCE, during build, not repeated in LLM context**

## Success Metrics

- [ ] LLM can search components in <100 tokens
- [ ] LLM can choose components in <200 tokens
- [ ] LLM can understand interfaces in <300 tokens
- [ ] LLM can generate glue code in <400 tokens
- [ ] **TOTAL: <1,000 tokens (vs 5,000+ traditional)**
- [ ] Code reuse >70%
- [ ] Zero source code read by LLM unnecessarily
- [ ] Build completes in <10 seconds
- [ ] All integration tests pass
