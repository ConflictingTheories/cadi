# Stage 3: Graph-Based Linking
## Detailed Implementation Specification

### Overview
Build the graph infrastructure that automatically links chunks via semantic edges (DEPENDS_ON, REFINES, EQUIVALENT_TO, IMPLEMENTS, SATISFIES). This enables the build engine to walk the graph and automatically resolve dependencies without manual wiring.

---

## 3.1 Graph Schema & Data Model

### File: `internal/cadi-core/src/graph/mod.rs` (new module)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Edge types in the semantic dependency graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GraphEdgeType {
    /// Chunk A directly uses/imports Chunk B
    DEPENDS_ON,
    
    /// Chunk B is an optimized/improved variant of Chunk A
    /// (e.g., a faster O(n) implementation of a sort function)
    REFINES,
    
    /// Chunks A and B are semantically equivalent
    /// (same functionality, possibly different languages)
    EQUIVALENT_TO,
    
    /// Chunk A implements interface/specification B
    IMPLEMENTS,
    
    /// Chunk A satisfies constraint/requirement B
    SATISFIES,
    
    /// Chunk B is a more generic version of Chunk A
    /// (e.g., generic CRUD adapts to specific domain)
    SPECIALIZES,
    
    /// Chunk A provides a type used by Chunk B
    PROVIDES_TYPE,
}

/// Metadata about an edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMetadata {
    /// Edge type
    pub edge_type: GraphEdgeType,
    
    /// Timestamp of creation
    pub created_at: String,
    
    /// Version compatibility info
    pub compatible_versions: Option<Vec<String>>,
    
    /// Additional context (e.g., semantic hash, interface signature)
    pub context: serde_json::Value,
    
    /// Confidence/strength of the edge (0.0 - 1.0)
    pub confidence: f32,
}

/// Node in the semantic graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkNode {
    /// Unique chunk ID
    pub id: String,
    
    /// Semantic hash
    pub semantic_hash: String,
    
    /// Chunk metadata
    pub name: String,
    pub description: String,
    pub language: String,
    pub concepts: Vec<String>,
    
    /// Interface exported by this chunk
    pub interface: Option<ChunkInterface>,
    
    /// Created timestamp
    pub created_at: String,
    
    /// Quality metrics
    pub usage_count: usize,
    pub quality_score: f32,
}

/// Interface/contract exposed by a chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInterface {
    /// Function/class name
    pub name: String,
    
    /// Input parameters
    pub inputs: Vec<Parameter>,
    
    /// Output/return type
    pub output: TypeSignature,
    
    /// Side effects
    pub effects: Vec<String>,
    
    /// Example usage
    pub example: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_sig: TypeSignature,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeSignature {
    pub name: String,
    pub is_generic: bool,
    pub constraints: Vec<String>,
}

/// Graph query results
#[derive(Debug, Clone)]
pub struct DependencyPath {
    pub from: String,
    pub to: String,
    pub path: Vec<(String, GraphEdgeType)>,  // (node_id, edge_type)
    pub depth: usize,
}

pub struct TransitiveDependencies {
    pub direct: Vec<String>,
    pub transitive: Vec<String>,
    pub all: Vec<String>,
}
```

---

## 3.2 SurrealDB Graph Implementation

### File: `internal/cadi-registry/src/graph.rs` (new file)

**Goal**: Store and query the semantic graph using SurrealDB's graph capabilities.

```rust
use surrealdb::Surreal;
use surrealdb::sql::Thing;
use crate::graph::*;

pub struct GraphDB {
    client: Surreal<Client>,
}

impl GraphDB {
    pub async fn new(url: &str, namespace: &str, database: &str) -> Result<Self> {
        let client = Surreal::new::<Wss>(url)
            .await?
            .use_ns(namespace)
            .use_db(database)
            .await?;
        
        Ok(Self { client })
    }
    
    /// Initialize graph schema
    pub async fn init_schema(&self) -> Result<()> {
        self.client.query(
            r#"
            -- Chunks table (nodes)
            DEFINE TABLE chunks SCHEMAFULL;
            DEFINE FIELD id ON TABLE chunks TYPE string;
            DEFINE FIELD semantic_hash ON TABLE chunks TYPE string;
            DEFINE FIELD name ON TABLE chunks TYPE string;
            DEFINE FIELD description ON TABLE chunks TYPE string;
            DEFINE FIELD language ON TABLE chunks TYPE string;
            DEFINE FIELD interface ON TABLE chunks TYPE object;
            DEFINE FIELD created_at ON TABLE chunks TYPE datetime;
            DEFINE FIELD usage_count ON TABLE chunks TYPE number;
            DEFINE FIELD quality_score ON TABLE chunks TYPE float;
            
            -- Dependencies table (edges)
            DEFINE TABLE dependencies TYPE RELATION IN out;
            DEFINE FIELD edge_type ON TABLE dependencies TYPE string;
            DEFINE FIELD confidence ON TABLE dependencies TYPE float;
            DEFINE FIELD context ON TABLE dependencies TYPE object;
            DEFINE FIELD created_at ON TABLE dependencies TYPE datetime;
            
            -- Indexes
            DEFINE INDEX idx_chunk_hash ON TABLE chunks FIELDS semantic_hash;
            DEFINE INDEX idx_chunk_language ON TABLE chunks FIELDS language;
            "#
        )
        .await?;
        
        Ok(())
    }
    
    /// Create a chunk node
    pub async fn create_chunk_node(
        &self,
        id: &str,
        metadata: &ChunkNode,
    ) -> Result<Thing> {
        let record: Vec<Thing> = self.client.create("chunks")
            .content(serde_json::json!({
                "id": id,
                "semantic_hash": metadata.semantic_hash,
                "name": metadata.name,
                "description": metadata.description,
                "language": metadata.language,
                "interface": metadata.interface,
                "created_at": chrono::Utc::now(),
                "usage_count": 0,
                "quality_score": 0.8,
            }))
            .await?;
        
        Ok(record[0].clone())
    }
    
    /// Create an edge between two chunks
    pub async fn create_edge(
        &self,
        from_id: &str,
        to_id: &str,
        edge_type: GraphEdgeType,
        context: Option<serde_json::Value>,
    ) -> Result<Thing> {
        let edge_name = format!("{:?}", edge_type);
        
        let record: Vec<Thing> = self.client.query(
            r#"
            RELATE $from -> dependencies -> $to SET 
                edge_type = $edge_type,
                confidence = $confidence,
                context = $context,
                created_at = now()
            "#
        )
        .bind(("from", format!("chunks:{}", from_id)))
        .bind(("to", format!("chunks:{}", to_id)))
        .bind(("edge_type", edge_name))
        .bind(("confidence", 1.0))
        .bind(("context", context.unwrap_or(serde_json::json!({}))))
        .await?
        .take(0)?;
        
        Ok(record[0].clone())
    }
    
    /// Get all direct dependencies of a chunk
    pub async fn get_dependencies(
        &self,
        chunk_id: &str,
    ) -> Result<Vec<DependencyInfo>> {
        let results: Vec<DependencyInfo> = self.client.query(
            r#"
            SELECT 
                type::string(out) AS dependency_id,
                edge_type,
                confidence
            FROM (
                SELECT out FROM dependencies 
                WHERE in == chunks:$chunk_id
            )
            "#
        )
        .bind(("chunk_id", chunk_id))
        .await?
        .take(0)?;
        
        Ok(results)
    }
    
    /// Get transitive closure of dependencies
    pub async fn get_transitive_dependencies(
        &self,
        chunk_id: &str,
        max_depth: usize,
    ) -> Result<TransitiveDependencies> {
        let results: Vec<TransitiveResult> = self.client.query(
            r#"
            SELECT 
                id,
                depth
            FROM (
                SELECT ->dependencies->chunks AS dep FROM chunks:$chunk_id 
                LIMIT $depth
            ) FLATTEN(dep)
            "#
        )
        .bind(("chunk_id", chunk_id))
        .bind(("depth", max_depth))
        .await?
        .take(0)?;
        
        let mut direct = Vec::new();
        let mut transitive = Vec::new();
        let mut all = Vec::new();
        
        for result in results {
            all.push(result.id.clone());
            if result.depth == 1 {
                direct.push(result.id);
            } else {
                transitive.push(result.id);
            }
        }
        
        Ok(TransitiveDependencies {
            direct,
            transitive,
            all,
        })
    }
    
    /// Find shortest path between two chunks
    pub async fn find_path(
        &self,
        from_id: &str,
        to_id: &str,
    ) -> Result<Option<DependencyPath>> {
        // Use GRAPH traversal
        let results: Vec<PathResult> = self.client.query(
            r#"
            SELECT * FROM (
                SELECT ->dependencies->(chunks)* AS path FROM chunks:$from 
                WHERE id == chunks:$to
            ) LIMIT 1
            "#
        )
        .bind(("from", from_id))
        .bind(("to", to_id))
        .await?
        .take(0)?;
        
        if results.is_empty() {
            return Ok(None);
        }
        
        // Construct DependencyPath from results
        // (Implementation depends on path format)
        Ok(None)  // Placeholder
    }
    
    /// Find all chunks that implement a specific interface
    pub async fn find_implementing_chunks(
        &self,
        interface_name: &str,
    ) -> Result<Vec<ChunkNode>> {
        let results: Vec<ChunkNode> = self.client.query(
            r#"
            SELECT * FROM chunks 
            WHERE interface.name == $interface_name
            "#
        )
        .bind(("interface_name", interface_name))
        .await?
        .take(0)?;
        
        Ok(results)
    }
    
    /// Find semantic equivalents using graph
    pub async fn find_equivalents(
        &self,
        chunk_id: &str,
    ) -> Result<Vec<EquivalentChunkInfo>> {
        let results: Vec<EquivalentChunkInfo> = self.client.query(
            r#"
            SELECT 
                type::string(->dependencies->(chunks WHERE edge_type == 'EQUIVALENT_TO')) AS equivalent_id,
                semantic_hash,
                name,
                language
            FROM chunks:$chunk_id
            "#
        )
        .bind(("chunk_id", chunk_id))
        .await?
        .take(0)?;
        
        Ok(results)
    }
    
    /// Get chunks that can be composed (find SATISFIES relationships)
    pub async fn find_composable_chunks(
        &self,
        requirement: &str,
    ) -> Result<Vec<ComposableChunk>> {
        let results: Vec<ComposableChunk> = self.client.query(
            r#"
            SELECT 
                id,
                name,
                description,
                language,
                interface
            FROM chunks
            WHERE description @@ $requirement 
               OR concepts @@ $requirement
            ORDER BY quality_score DESC
            LIMIT 20
            "#
        )
        .bind(("requirement", requirement))
        .await?
        .take(0)?;
        
        Ok(results)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub dependency_id: String,
    pub edge_type: String,
    pub confidence: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransitiveResult {
    pub id: String,
    pub depth: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathResult {
    pub path: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EquivalentChunkInfo {
    pub equivalent_id: String,
    pub semantic_hash: String,
    pub name: String,
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComposableChunk {
    pub id: String,
    pub name: String,
    pub description: String,
    pub language: String,
    pub interface: Option<ChunkInterface>,
}
```

---

## 3.3 Automatic Dependency Resolution

### File: `internal/cadi-builder/src/dependency_resolver.rs` (new file)

**Goal**: Walk the graph to resolve all transitive dependencies automatically.

```rust
use crate::graph::{GraphDB, GraphEdgeType};
use std::collections::{HashSet, VecDeque};

pub struct DependencyResolver {
    graph_db: Arc<GraphDB>,
}

impl DependencyResolver {
    pub fn new(graph_db: Arc<GraphDB>) -> Self {
        Self { graph_db }
    }
    
    /// Resolve all dependencies for a chunk (BFS traversal)
    pub async fn resolve_all_dependencies(
        &self,
        chunk_id: &str,
    ) -> Result<ResolutionResult> {
        let mut resolved = HashSet::new();
        let mut queue = VecDeque::new();
        let mut dependency_graph = HashMap::new();
        
        queue.push_back(chunk_id.to_string());
        resolved.insert(chunk_id.to_string());
        
        while let Some(current) = queue.pop_front() {
            // Get direct dependencies
            let deps = self.graph_db.get_dependencies(&current).await?;
            
            for dep in deps {
                if !resolved.contains(&dep.dependency_id) {
                    resolved.insert(dep.dependency_id.clone());
                    queue.push_back(dep.dependency_id.clone());
                }
                
                // Record edge
                dependency_graph
                    .entry(current.clone())
                    .or_insert_with(Vec::new)
                    .push((dep.dependency_id.clone(), dep.edge_type));
            }
        }
        
        Ok(ResolutionResult {
            root: chunk_id.to_string(),
            all_dependencies: resolved,
            dependency_graph,
        })
    }
    
    /// Check if composition is valid (no cycles, compatible versions)
    pub async fn validate_composition(
        &self,
        chunks: &[String],
    ) -> Result<CompositionValidation> {
        let mut issues = Vec::new();
        
        // Check for cycles
        for chunk in chunks {
            for other in chunks {
                if chunk != other {
                    if let Some(_path) = self.graph_db.find_path(chunk, other).await? {
                        if let Some(_reverse) = self.graph_db.find_path(other, chunk).await? {
                            issues.push(CompositionIssue {
                                severity: "error".to_string(),
                                message: format!("Circular dependency: {} -> {}", chunk, other),
                            });
                        }
                    }
                }
            }
        }
        
        Ok(CompositionValidation {
            is_valid: issues.is_empty(),
            issues,
        })
    }
    
    /// Get interface compatibility between two chunks
    pub async fn check_interface_compatibility(
        &self,
        provider_id: &str,
        consumer_id: &str,
    ) -> Result<InterfaceCompatibility> {
        // Query interfaces
        let provider = self.graph_db.get_chunk_node(provider_id).await?;
        let consumer = self.graph_db.get_chunk_node(consumer_id).await?;
        
        let provider_interface = provider.interface
            .ok_or("Provider has no interface")?;
        let consumer_interface = consumer.interface
            .ok_or("Consumer has no interface")?;
        
        // Check type compatibility
        let mut incompatibilities = Vec::new();
        
        // Compare output of provider with input of consumer
        if provider_interface.output.name != consumer_interface.inputs[0].type_sig.name {
            incompatibilities.push(format!(
                "Type mismatch: {} output vs {} input",
                provider_interface.output.name,
                consumer_interface.inputs[0].type_sig.name
            ));
        }
        
        Ok(InterfaceCompatibility {
            is_compatible: incompatibilities.is_empty(),
            incompatibilities,
        })
    }
}

#[derive(Debug)]
pub struct ResolutionResult {
    pub root: String,
    pub all_dependencies: HashSet<String>,
    pub dependency_graph: HashMap<String, Vec<(String, String)>>,
}

#[derive(Debug)]
pub struct CompositionValidation {
    pub is_valid: bool,
    pub issues: Vec<CompositionIssue>,
}

#[derive(Debug)]
pub struct CompositionIssue {
    pub severity: String,  // "error", "warning"
    pub message: String,
}

#[derive(Debug)]
pub struct InterfaceCompatibility {
    pub is_compatible: bool,
    pub incompatibilities: Vec<String>,
}
```

---

## 3.4 Build Engine with Graph Traversal

### File: `internal/cadi-builder/src/builder.rs` (enhance)

**Goal**: Use resolved dependencies to automatically scaffold and link chunks.

```rust
pub struct BuildScaffolder {
    graph_db: Arc<GraphDB>,
    dependency_resolver: Arc<DependencyResolver>,
}

impl BuildScaffolder {
    /// Generate import/linking code for a set of chunks
    pub async fn scaffold_imports(
        &self,
        chunks: &[String],
        language: &str,
        output_path: &Path,
    ) -> Result<()> {
        // Resolve all dependencies
        let mut all_deps = HashSet::new();
        for chunk_id in chunks {
            let resolved = self.dependency_resolver
                .resolve_all_dependencies(chunk_id)
                .await?;
            all_deps.extend(resolved.all_dependencies);
        }
        
        // Get chunk info
        let mut imports = Vec::new();
        for chunk_id in all_deps.iter() {
            let node = self.graph_db.get_chunk_node(chunk_id).await?;
            if let Some(interface) = &node.interface {
                imports.push(ImportStatement {
                    chunk_id: chunk_id.clone(),
                    name: interface.name.clone(),
                    language: node.language.clone(),
                });
            }
        }
        
        // Generate import code
        let import_code = self.generate_imports(&imports, language)?;
        
        // Write to file
        let import_file = output_path.join("generated_imports.ts");  // or .py, .rs
        tokio::fs::write(&import_file, import_code).await?;
        
        Ok(())
    }
    
    fn generate_imports(
        &self,
        imports: &[ImportStatement],
        language: &str,
    ) -> Result<String> {
        match language {
            "typescript" => {
                let mut code = String::new();
                for import in imports {
                    code.push_str(&format!(
                        "import {{ {} }} from './chunks/{}';\n",
                        import.name,
                        import.chunk_id
                    ));
                }
                Ok(code)
            }
            "python" => {
                let mut code = String::new();
                for import in imports {
                    code.push_str(&format!(
                        "from .chunks.{} import {}\n",
                        import.chunk_id,
                        import.name
                    ));
                }
                Ok(code)
            }
            "rust" => {
                let mut code = String::new();
                for import in imports {
                    code.push_str(&format!(
                        "use chunks::{}::{};\n",
                        import.chunk_id,
                        import.name
                    ));
                }
                Ok(code)
            }
            _ => Err(format!("Unsupported language: {}", language)),
        }
    }
}

pub struct ImportStatement {
    pub chunk_id: String,
    pub name: String,
    pub language: String,
}
```

---

## 3.5 Testing

### File: `tests/graph_linking_test.rs`

```rust
#[tokio::test]
async fn test_graph_creation_and_traversal() {
    let graph = setup_test_graph().await;
    
    // Create nodes
    let jwt_chunk = ChunkNode {
        id: "jwt_auth".to_string(),
        semantic_hash: "hash1".to_string(),
        name: "JWT Authenticator".to_string(),
        /* ... */
    };
    
    let crud_chunk = ChunkNode {
        id: "crud_ops".to_string(),
        semantic_hash: "hash2".to_string(),
        name: "CRUD Operations".to_string(),
        /* ... */
    };
    
    graph.create_chunk_node("jwt_auth", &jwt_chunk).await.unwrap();
    graph.create_chunk_node("crud_ops", &crud_chunk).await.unwrap();
    
    // Create edge: CRUD depends on JWT
    graph.create_edge(
        "crud_ops",
        "jwt_auth",
        GraphEdgeType::DEPENDS_ON,
        None,
    ).await.unwrap();
    
    // Query dependencies
    let deps = graph.get_dependencies("crud_ops").await.unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].dependency_id, "jwt_auth");
}

#[tokio::test]
async fn test_transitive_dependency_resolution() {
    let resolver = setup_test_resolver().await;
    
    // Create dependency chain: blog -> crud -> jwt
    setup_dependency_chain(&resolver).await;
    
    // Resolve all dependencies from "blog"
    let result = resolver.resolve_all_dependencies("blog").await.unwrap();
    
    assert!(result.all_dependencies.contains("crud"));
    assert!(result.all_dependencies.contains("jwt"));
}

#[tokio::test]
async fn test_cycle_detection() {
    let validation = setup_cyclic_dependencies().await;
    let result = validation.validate_composition(&["a", "b", "c"]).await.unwrap();
    
    assert!(!result.is_valid);
    assert!(!result.issues.is_empty());
}
```

---

## Success Criteria

✅ Graph stores all edges (DEPENDS_ON, REFINES, EQUIVALENT_TO, etc.)  
✅ Transitive dependency resolution <100ms for 1000+ chunks  
✅ Cycle detection prevents invalid compositions  
✅ Interface compatibility checking prevents type errors  
✅ Auto-generated imports work in TypeScript, Python, Rust  
✅ Build scaffolding requires zero manual wiring  
✅ Demo: Blog app pulls JWT + CRUD + logger from todo via graph  

---

## Next: Stage 4 (CBS for LLMs)
Build the CADI Build Spec schema optimized for LLM generation and build execution.
