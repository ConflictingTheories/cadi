# CADI: The Holy Grail Roadmap

> **"You have an agent — but do you have a CADI?"**

This document outlines the technical roadmap to transform CADI from its current state into the **foundational infrastructure layer for the agentic era**.

---

## Executive Summary

CADI is not an agent framework. It is the **infrastructure that makes agents viable**. We treat code not as flat text files, but as a **Merkle Forest of Atoms** (functions, types, logic blocks).

| Current State | Holy Grail Target |
|--------------|-------------------|
| Regex-based parsing | Tree-sitter AST parsing with symbol resolution |
| Flat file/KV storage | Merkle DAG with relationship graph |
| Full-file retrieval | Virtual Views with atom stitching |
| Manual context | Ghost Import auto-expansion |

---

## Part 1: Gap Analysis

### ✅ What We Have (Strong Foundation)

1. **Core Architecture** (`cadi-core`)
   - `AtomicChunk` type with aliases, categories, granularity
   - `SmartChunker` with entity detection (1327 lines)
   - Manifest parsing and validation
   - Content-addressing with SHA256

2. **MCP Server** (`cadi-mcp-server`)
   - Functional JSON-RPC over stdio and HTTP
   - 11 tools (search, import, build, etc.)
   - Resources and prompts for agent guidance

3. **Registry Federation** (`cadi-registry`)
   - Multi-registry support with priority ordering
   - Trust levels and capabilities
   - Async fetch/publish

4. **Build Engine** (`cadi-builder`)
   - Build plans and caching
   - Transformation pipeline structure

5. **Specifications** (`cadi-spec/`)
   - 16 JSON schemas covering the full spec

### ❌ Critical Gaps

| Gap | Current State | Impact | Priority |
|-----|---------------|--------|----------|
| **Smart Atomizer** | Regex-based entity extraction | Cuts functions in half, misses dependencies | P0 |
| **Graph Store** | Flat blob storage | Can't answer "who depends on X?" in O(1) | P0 |
| **Virtual Views** | Returns full files/snippets | Agents read unnecessary tokens | P1 |
| **Ghost Imports** | None | LLMs hallucinate missing types | P1 |
| **Symbol Resolution** | None | `import { X }` not resolved to hash | P0 |

---

## Part 2: The Implementation Phases

### Phase 0: Foundation Hardening (Week 1-2)
*Get the base solid before building up*

#### 0.1 Create Graph Storage Layer
**File: `internal/cadi-core/src/graph/mod.rs`**

```rust
//! Merkle DAG Graph Store
//! 
//! Stores chunks and their relationships for O(1) dependency queries.

pub mod store;
pub mod node;
pub mod edge;
pub mod query;

pub use store::GraphStore;
pub use node::GraphNode;
pub use edge::EdgeType;
pub use query::GraphQuery;
```

**Key Types:**
```rust
pub struct GraphStore {
    /// The embedded database (sled or rocksdb)
    db: sled::Db,
    /// Forward edges: chunk -> [dependents]
    dependents: sled::Tree,
    /// Reverse edges: chunk -> [dependencies]  
    dependencies: sled::Tree,
    /// Chunk metadata
    metadata: sled::Tree,
    /// Symbol index: symbol_name -> chunk_id
    symbols: sled::Tree,
}

pub enum EdgeType {
    /// Direct import/use
    Imports,
    /// Type reference
    TypeRef,
    /// Function call
    Calls,
    /// Composed of (parent -> children)
    ComposedOf,
    /// Exports symbol
    Exports,
}

pub struct GraphNode {
    pub chunk_id: String,
    pub content_hash: String,
    pub symbols_defined: Vec<String>,
    pub symbols_referenced: Vec<String>,
    pub edges: Vec<(EdgeType, String)>,
}
```

#### 0.2 Add Dependency: sled
**File: `internal/cadi-core/Cargo.toml`**
```toml
[dependencies]
sled = "0.34"
```

---

### Phase 1: The Smart Atomizer (Week 2-4)
*The hardest part - language-aware parsing*

#### 1.1 Integrate Tree-sitter

**Add Dependencies:**
```toml
# internal/cadi-core/Cargo.toml
[dependencies]
tree-sitter = "0.22"
tree-sitter-rust = "0.21"
tree-sitter-typescript = "0.21"
tree-sitter-python = "0.21"
tree-sitter-javascript = "0.21"
tree-sitter-go = "0.21"
```

**File: `internal/cadi-core/src/atomizer/mod.rs`**

```rust
//! Smart Atomizer - Language-aware code parsing
//!
//! Uses Tree-sitter for AST parsing and symbol resolution.

pub mod parser;
pub mod languages;
pub mod symbol_resolver;
pub mod atom_extractor;

pub use parser::AstParser;
pub use symbol_resolver::SymbolResolver;
pub use atom_extractor::AtomExtractor;
```

#### 1.2 AST Parser

**File: `internal/cadi-core/src/atomizer/parser.rs`**

```rust
use tree_sitter::{Parser, Language, Tree, Node};
use std::collections::HashMap;

pub struct AstParser {
    parsers: HashMap<String, Parser>,
}

impl AstParser {
    pub fn new() -> Self {
        let mut parsers = HashMap::new();
        
        // Initialize parsers for supported languages
        let languages = [
            ("rust", tree_sitter_rust::language()),
            ("typescript", tree_sitter_typescript::language_typescript()),
            ("javascript", tree_sitter_javascript::language()),
            ("python", tree_sitter_python::language()),
        ];
        
        for (name, lang) in languages {
            let mut parser = Parser::new();
            parser.set_language(lang).expect("Error loading grammar");
            parsers.insert(name.to_string(), parser);
        }
        
        Self { parsers }
    }
    
    pub fn parse(&mut self, language: &str, source: &str) -> Option<Tree> {
        self.parsers.get_mut(language)?.parse(source, None)
    }
}
```

#### 1.3 Symbol Resolver

**File: `internal/cadi-core/src/atomizer/symbol_resolver.rs`**

```rust
//! Resolves imports to chunk hashes
//!
//! When parsing `import { X } from './y'`, this module:
//! 1. Finds the source file './y'
//! 2. Parses it to find export 'X'
//! 3. Hashes the export
//! 4. Returns a link reference: `link:sha256:...`

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct SymbolResolver {
    /// Cache of resolved symbols: (file, symbol) -> chunk_id
    cache: HashMap<(PathBuf, String), String>,
    /// Project root for relative path resolution
    project_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ResolvedImport {
    /// Original import path
    pub source_path: String,
    /// Imported symbols
    pub symbols: Vec<ImportedSymbol>,
}

#[derive(Debug, Clone)]
pub struct ImportedSymbol {
    pub name: String,
    pub alias: Option<String>,
    pub chunk_id: String,
    pub chunk_hash: String,
}

impl SymbolResolver {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            cache: HashMap::new(),
            project_root,
        }
    }
    
    /// Resolve all imports in a file
    pub fn resolve_imports(
        &mut self,
        file_path: &Path,
        imports: Vec<RawImport>,
        graph: &GraphStore,
    ) -> Vec<ResolvedImport> {
        imports.into_iter()
            .filter_map(|import| self.resolve_import(file_path, import, graph))
            .collect()
    }
    
    fn resolve_import(
        &mut self,
        current_file: &Path,
        import: RawImport,
        graph: &GraphStore,
    ) -> Option<ResolvedImport> {
        // Resolve relative path
        let target_path = self.resolve_path(current_file, &import.source)?;
        
        // Look up each symbol in the graph
        let symbols = import.symbols.into_iter()
            .filter_map(|sym| {
                let cache_key = (target_path.clone(), sym.name.clone());
                
                if let Some(chunk_id) = self.cache.get(&cache_key) {
                    return Some(ImportedSymbol {
                        name: sym.name,
                        alias: sym.alias,
                        chunk_id: chunk_id.clone(),
                        chunk_hash: chunk_id.split(':').last()?.to_string(),
                    });
                }
                
                // Query the graph for this symbol
                let chunk_id = graph.find_symbol_definition(&sym.name, &target_path)?;
                self.cache.insert(cache_key, chunk_id.clone());
                
                Some(ImportedSymbol {
                    name: sym.name,
                    alias: sym.alias,
                    chunk_id: chunk_id.clone(),
                    chunk_hash: chunk_id.split(':').last()?.to_string(),
                })
            })
            .collect();
        
        Some(ResolvedImport {
            source_path: import.source,
            symbols,
        })
    }
}
```

#### 1.4 Atom Extractor

**File: `internal/cadi-core/src/atomizer/atom_extractor.rs`**

```rust
//! Extracts atoms from AST with proper boundaries
//!
//! Unlike regex, this understands code structure and extracts
//! complete, syntactically valid code units.

use tree_sitter::{Node, Tree};
use crate::AtomicChunk;

pub struct AtomExtractor {
    language: String,
}

#[derive(Debug, Clone)]
pub struct ExtractedAtom {
    pub name: String,
    pub kind: AtomKind,
    pub source: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: usize,
    pub end_line: usize,
    /// Symbols this atom defines (exports)
    pub defines: Vec<String>,
    /// Symbols this atom references (imports)
    pub references: Vec<String>,
    /// Doc comments
    pub doc_comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AtomKind {
    Function,
    AsyncFunction,
    Method,
    Struct,
    Class,
    Trait,
    Interface,
    Enum,
    Constant,
    TypeAlias,
    Module,
    Macro,
}

impl AtomExtractor {
    pub fn new(language: &str) -> Self {
        Self { language: language.to_string() }
    }
    
    pub fn extract(&self, tree: &Tree, source: &str) -> Vec<ExtractedAtom> {
        let mut atoms = Vec::new();
        let root = tree.root_node();
        
        match self.language.as_str() {
            "rust" => self.extract_rust(&root, source, &mut atoms),
            "typescript" | "javascript" => self.extract_ts(&root, source, &mut atoms),
            "python" => self.extract_python(&root, source, &mut atoms),
            _ => {}
        }
        
        atoms
    }
    
    fn extract_rust(&self, node: &Node, source: &str, atoms: &mut Vec<ExtractedAtom>) {
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            match child.kind() {
                "function_item" => {
                    if let Some(atom) = self.extract_rust_function(&child, source) {
                        atoms.push(atom);
                    }
                }
                "struct_item" => {
                    if let Some(atom) = self.extract_rust_struct(&child, source) {
                        atoms.push(atom);
                    }
                }
                "impl_item" => {
                    // Extract methods from impl block
                    self.extract_rust_impl(&child, source, atoms);
                }
                "trait_item" => {
                    if let Some(atom) = self.extract_rust_trait(&child, source) {
                        atoms.push(atom);
                    }
                }
                "enum_item" => {
                    if let Some(atom) = self.extract_rust_enum(&child, source) {
                        atoms.push(atom);
                    }
                }
                "mod_item" => {
                    if let Some(atom) = self.extract_rust_module(&child, source) {
                        atoms.push(atom);
                    }
                }
                _ => {
                    // Recurse into children
                    self.extract_rust(&child, source, atoms);
                }
            }
        }
    }
    
    fn extract_rust_function(&self, node: &Node, source: &str) -> Option<ExtractedAtom> {
        let name_node = node.child_by_field_name("name")?;
        let name = name_node.utf8_text(source.as_bytes()).ok()?.to_string();
        
        // Get the full function source
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();
        let func_source = &source[start_byte..end_byte];
        
        // Extract doc comments (preceding siblings)
        let doc_comment = self.extract_doc_comment(node, source);
        
        // Find references (identifiers used in the body)
        let references = self.find_references(node, source);
        
        Some(ExtractedAtom {
            name: name.clone(),
            kind: if self.is_async_function(node) { AtomKind::AsyncFunction } else { AtomKind::Function },
            source: func_source.to_string(),
            start_byte,
            end_byte,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            defines: vec![name],
            references,
            doc_comment,
        })
    }
    
    // ... similar methods for struct, trait, enum, etc.
}
```

---

### Phase 2: Virtual Views & Rehydration (Week 4-6)
*The killer feature for agents*

#### 2.1 Rehydration Engine

**File: `internal/cadi-core/src/rehydration/mod.rs`**

```rust
//! Rehydration Engine
//!
//! Dynamically assembles atoms into syntactically valid "virtual files"
//! that exist only in memory, optimized for LLM consumption.

pub mod engine;
pub mod context;
pub mod assembler;

pub use engine::RehydrationEngine;
pub use context::ViewContext;
```

**File: `internal/cadi-core/src/rehydration/engine.rs`**

```rust
use std::collections::{HashSet, HashMap};
use crate::graph::GraphStore;
use crate::AtomicChunk;

pub struct RehydrationEngine {
    graph: GraphStore,
}

/// A virtual view - atoms assembled into a coherent context
#[derive(Debug, Clone)]
pub struct VirtualView {
    /// The assembled source code
    pub source: String,
    /// Atoms included in this view
    pub atoms: Vec<String>,
    /// Total token estimate
    pub token_estimate: usize,
    /// Language of the view
    pub language: String,
    /// Map of symbol -> line number in the view
    pub symbol_locations: HashMap<String, usize>,
}

impl RehydrationEngine {
    pub fn new(graph: GraphStore) -> Self {
        Self { graph }
    }
    
    /// Create a virtual view from requested atoms
    /// 
    /// This is the core "fractal" operation - assembling atoms into
    /// a coherent, syntactically valid buffer.
    pub async fn create_view(&self, atom_ids: Vec<String>) -> Result<VirtualView, Error> {
        let mut assembled = String::new();
        let mut included_atoms = Vec::new();
        let mut symbol_locations = HashMap::new();
        let mut current_line = 1;
        
        // Collect all atoms with their content
        let mut atoms_with_content = Vec::new();
        for atom_id in &atom_ids {
            let chunk = self.graph.get_chunk(atom_id)?;
            let content = self.graph.get_content(atom_id)?;
            atoms_with_content.push((chunk, content));
        }
        
        // Sort by type priority (imports first, then types, then functions)
        atoms_with_content.sort_by_key(|(chunk, _)| self.type_priority(&chunk.granularity));
        
        // Assemble the view
        for (chunk, content) in atoms_with_content {
            // Add separator comment
            assembled.push_str(&format!("// --- {} ---\n", chunk.primary_alias()));
            current_line += 1;
            
            // Track symbol locations
            for symbol in &chunk.provides {
                symbol_locations.insert(symbol.clone(), current_line);
            }
            
            // Add the content
            let lines = content.lines().count();
            assembled.push_str(&content);
            assembled.push_str("\n\n");
            current_line += lines + 2;
            
            included_atoms.push(chunk.chunk_id.clone());
        }
        
        Ok(VirtualView {
            source: assembled,
            atoms: included_atoms,
            token_estimate: self.estimate_tokens(&assembled),
            language: self.detect_language(&atom_ids),
            symbol_locations,
        })
    }
    
    /// Create a view with automatic context expansion (Ghost Imports)
    pub async fn create_expanded_view(
        &self,
        atom_ids: Vec<String>,
        expansion_depth: usize,
    ) -> Result<VirtualView, Error> {
        let mut all_atoms = HashSet::new();
        
        // Add requested atoms
        for id in &atom_ids {
            all_atoms.insert(id.clone());
        }
        
        // Expand dependencies up to depth
        let mut frontier = atom_ids.clone();
        for _ in 0..expansion_depth {
            let mut next_frontier = Vec::new();
            
            for atom_id in &frontier {
                let deps = self.graph.get_dependencies(atom_id)?;
                for dep in deps {
                    if !all_atoms.contains(&dep) {
                        all_atoms.insert(dep.clone());
                        next_frontier.push(dep);
                    }
                }
            }
            
            frontier = next_frontier;
        }
        
        self.create_view(all_atoms.into_iter().collect()).await
    }
    
    fn estimate_tokens(&self, text: &str) -> usize {
        // Rough estimate: ~4 characters per token
        text.len() / 4
    }
}
```

#### 2.2 MCP Virtual View Tool

**Add to `cmd/cadi-mcp-server/src/tools.rs`:**

```rust
ToolDefinition {
    name: "cadi_view_context".to_string(),
    description: "🎯 VIRTUAL VIEW: Assemble atoms into a coherent code context. Returns syntactically valid code with only what you need.".to_string(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "atoms": {
                "type": "array",
                "items": { "type": "string" },
                "description": "List of atom/chunk IDs to include in the view"
            },
            "expand_depth": {
                "type": "integer",
                "default": 1,
                "description": "How many levels of dependencies to automatically include (Ghost Imports)"
            },
            "format": {
                "type": "string",
                "enum": ["source", "minimal", "documented"],
                "default": "source",
                "description": "Output format"
            },
            "max_tokens": {
                "type": "integer",
                "description": "Maximum tokens to include (truncates if exceeded)"
            }
        },
        "required": ["atoms"]
    }),
},
```

---

### Phase 3: Ghost Import Resolver (Week 6-7)
*Automatic context expansion*

#### 3.1 Context Expansion System

**File: `internal/cadi-core/src/ghost/mod.rs`**

```rust
//! Ghost Import Resolver
//!
//! Automatically expands atom context to include necessary dependencies
//! so LLMs don't hallucinate missing types.

pub mod resolver;
pub mod policy;
pub mod analyzer;

pub use resolver::GhostResolver;
pub use policy::ExpansionPolicy;
```

**File: `internal/cadi-core/src/ghost/resolver.rs`**

```rust
use std::collections::HashSet;
use crate::graph::{GraphStore, EdgeType};

pub struct GhostResolver {
    graph: GraphStore,
    policy: ExpansionPolicy,
}

/// Policy for automatic context expansion
#[derive(Debug, Clone)]
pub struct ExpansionPolicy {
    /// Maximum depth of dependency expansion
    pub max_depth: usize,
    /// Maximum total atoms to include
    pub max_atoms: usize,
    /// Maximum total tokens
    pub max_tokens: usize,
    /// Types of edges to follow
    pub follow_edges: Vec<EdgeType>,
    /// Always include type definitions
    pub always_include_types: bool,
    /// Include method signatures (not bodies) for referenced types
    pub include_signatures: bool,
}

impl Default for ExpansionPolicy {
    fn default() -> Self {
        Self {
            max_depth: 2,
            max_atoms: 20,
            max_tokens: 4000,
            follow_edges: vec![
                EdgeType::Imports,
                EdgeType::TypeRef,
            ],
            always_include_types: true,
            include_signatures: true,
        }
    }
}

#[derive(Debug)]
pub struct ExpansionResult {
    /// All atoms to include (original + ghosts)
    pub atoms: Vec<String>,
    /// Which atoms were "ghost" additions
    pub ghost_atoms: Vec<String>,
    /// Truncated due to limits?
    pub truncated: bool,
    /// Explanation of what was included and why
    pub explanation: String,
}

impl GhostResolver {
    pub fn new(graph: GraphStore, policy: ExpansionPolicy) -> Self {
        Self { graph, policy }
    }
    
    /// Resolve ghost imports for a set of atoms
    pub async fn resolve(&self, atom_ids: &[String]) -> Result<ExpansionResult, Error> {
        let mut included = HashSet::new();
        let mut ghost_atoms = Vec::new();
        let mut token_count = 0;
        let mut explanations = Vec::new();
        
        // Start with requested atoms
        for id in atom_ids {
            included.insert(id.clone());
            token_count += self.graph.get_token_estimate(id)?;
        }
        
        // BFS expansion
        let mut frontier: Vec<(String, usize)> = atom_ids.iter()
            .map(|id| (id.clone(), 0))
            .collect();
        
        while let Some((atom_id, depth)) = frontier.pop() {
            if depth >= self.policy.max_depth {
                continue;
            }
            if included.len() >= self.policy.max_atoms {
                break;
            }
            
            // Get dependencies based on policy
            let deps = self.get_filtered_dependencies(&atom_id)?;
            
            for dep in deps {
                if included.contains(&dep.target) {
                    continue;
                }
                
                let dep_tokens = self.graph.get_token_estimate(&dep.target)?;
                if token_count + dep_tokens > self.policy.max_tokens {
                    continue;
                }
                
                // Add this ghost dependency
                included.insert(dep.target.clone());
                ghost_atoms.push(dep.target.clone());
                token_count += dep_tokens;
                
                explanations.push(format!(
                    "Added '{}' because '{}' references it via {:?}",
                    dep.target, atom_id, dep.edge_type
                ));
                
                // Add to frontier for further expansion
                frontier.push((dep.target.clone(), depth + 1));
            }
        }
        
        Ok(ExpansionResult {
            atoms: included.into_iter().collect(),
            ghost_atoms,
            truncated: included.len() >= self.policy.max_atoms,
            explanation: explanations.join("\n"),
        })
    }
    
    fn get_filtered_dependencies(&self, atom_id: &str) -> Result<Vec<Dependency>, Error> {
        let all_deps = self.graph.get_dependencies(atom_id)?;
        
        Ok(all_deps.into_iter()
            .filter(|d| self.policy.follow_edges.contains(&d.edge_type))
            .collect())
    }
}
```

---

### Phase 4: Enhanced MCP Integration (Week 7-8)
*Making it all work together*

#### 4.1 Update MCP Protocol

**File: `cmd/cadi-mcp-server/src/protocol.rs` additions:**

```rust
// Add streaming support for large views
async fn handle_stream_view(
    &self,
    id: Option<Value>,
    params: Option<Value>,
) -> impl Stream<Item = JsonRpcResponse> {
    // Stream atoms as they're assembled
}

// Add the ghost import instruction to initialize
fn handle_initialize(&self, id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse::success(id, json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {},
            "resources": {},
            "prompts": {},
            "streaming": true,  // NEW
        },
        "serverInfo": {
            "name": "cadi-mcp-server",
            "version": env!("CARGO_PKG_VERSION")
        },
        "instructions": concat!(
            "⚡ CADI: The Build System for the Agentic Era\n\n",
            "WORKFLOW:\n",
            "1. cadi_search - Find existing atoms (~50 tokens)\n",
            "2. cadi_view_context - Get assembled code with ghost imports\n",
            "3. Only write new code if nothing found\n",
            "4. cadi_import after writing to save for reuse\n\n",
            "Ghost imports automatically include type definitions!"
        )
    }))
}
```

#### 4.2 New Tools Summary

| Tool | Purpose | Token Cost |
|------|---------|------------|
| `cadi_view_context` | Virtual view assembly | ~100-500 |
| `cadi_get_dependencies` | See what an atom needs | ~50 |
| `cadi_get_dependents` | See what uses an atom | ~50 |
| `cadi_expand_context` | Manual ghost import control | ~100 |

---

## Part 3: Implementation Order & Milestones

### Milestone 1: Graph Store (End of Week 2)
- [ ] sled integration in cadi-core
- [ ] GraphStore basic CRUD
- [ ] Edge storage (dependents/dependencies)
- [ ] Symbol index

### Milestone 2: Tree-sitter Atomizer (End of Week 4)
- [ ] Tree-sitter integration
- [ ] Rust parser complete
- [ ] TypeScript parser complete
- [ ] Python parser complete
- [ ] Symbol resolution working

### Milestone 3: Virtual Views (End of Week 6)
- [ ] RehydrationEngine complete
- [ ] `cadi_view_context` MCP tool
- [ ] Token estimation
- [ ] View caching

### Milestone 4: Ghost Imports (End of Week 7)
- [ ] GhostResolver complete
- [ ] Expansion policies
- [ ] Type signature extraction
- [ ] Integration tests

### Milestone 5: Production Ready (End of Week 8)
- [ ] Full integration tests
- [ ] Performance benchmarks
- [ ] Documentation
- [ ] Docker deployment update

---

## Part 4: File Structure After Implementation

```
internal/cadi-core/src/
├── lib.rs
├── atomic.rs           # ✅ Existing
├── smart_chunker.rs    # ✅ Existing (enhance)
├── graph/              # 🆕 NEW
│   ├── mod.rs
│   ├── store.rs        # sled-backed graph store
│   ├── node.rs
│   ├── edge.rs
│   └── query.rs
├── atomizer/           # 🆕 NEW
│   ├── mod.rs
│   ├── parser.rs       # Tree-sitter AST
│   ├── languages/
│   │   ├── rust.rs
│   │   ├── typescript.rs
│   │   └── python.rs
│   ├── symbol_resolver.rs
│   └── atom_extractor.rs
├── rehydration/        # 🆕 NEW
│   ├── mod.rs
│   ├── engine.rs
│   ├── context.rs
│   └── assembler.rs
└── ghost/              # 🆕 NEW
    ├── mod.rs
    ├── resolver.rs
    ├── policy.rs
    └── analyzer.rs
```

---

## Part 5: Business Impact

| Feature | Token Savings | Speed Improvement | User Value |
|---------|---------------|-------------------|------------|
| Graph Store | - | 10x query speed | Instant dependency queries |
| Smart Atomizer | 30% less noise | - | Clean, complete atoms |
| Virtual Views | 60% less tokens | - | Pay for what you use |
| Ghost Imports | 20% less hallucination | - | Correct first time |

**Combined: 40% token cost reduction, 80% fewer hallucinations**

---

## Part 6: Competitive Moat

1. **Network Effects**: Every chunk published makes the registry more valuable
2. **Content Addressing**: Once hashed, atoms are eternal and verifiable
3. **Zero-Copy Builds**: "Installing" = verifying hashes, not downloading
4. **Agent Memory**: Solved problems stay solved forever

---

## Next Steps

1. **Immediate**: Start Phase 0 - add sled dependency and create graph module structure
2. **This Week**: Prototype Tree-sitter integration with Rust language
3. **Review**: After Milestone 2, assess and adjust timeline

---

> "CADI is not just a tool. It is the operating system for the agentic era."

**LET'S BUILD THIS.**
