use serde::{Deserialize, Serialize};

// Submodules
pub mod edge;
pub mod node;
pub mod query;
pub mod store;

// Re-export types from submodules
pub use edge::EdgeType;
pub use node::GraphNode;
pub use query::{GraphQuery, QueryNode, QueryResult, TraversalDirection};
pub use store::GraphStore;

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
