//! Graph Node representation
//!
//! A GraphNode represents a chunk in the dependency graph with all its
//! relationships and metadata for efficient querying.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::EdgeType;

/// A node in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// The chunk ID (e.g., "chunk:sha256:abc123...")
    pub chunk_id: String,

    /// Content hash of the chunk
    pub content_hash: String,

    /// Primary alias for this chunk
    pub primary_alias: Option<String>,

    /// All aliases pointing to this chunk
    pub aliases: Vec<String>,

    /// Symbols this chunk defines/exports
    pub symbols_defined: Vec<String>,

    /// Symbols this chunk references/imports
    pub symbols_referenced: Vec<String>,

    /// Language of the chunk
    pub language: String,

    /// Granularity (function, type, module, etc.)
    pub granularity: String,

    /// Byte size of content
    pub byte_size: usize,

    /// Estimated token count (for LLM budgeting)
    pub token_estimate: usize,

    /// Source file location
    pub source_file: Option<String>,

    /// Line range in source file
    pub source_lines: Option<(usize, usize)>,

    /// Outgoing edges (this chunk depends on)
    pub outgoing_edges: Vec<(EdgeType, String)>,

    /// Incoming edges (chunks that depend on this)
    pub incoming_edges: Vec<(EdgeType, String)>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// When this node was created
    pub created_at: String,

    /// When this node was last updated
    pub updated_at: String,
}

impl GraphNode {
    /// Create a new graph node
    pub fn new(chunk_id: impl Into<String>, content_hash: impl Into<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            chunk_id: chunk_id.into(),
            content_hash: content_hash.into(),
            primary_alias: None,
            aliases: Vec::new(),
            symbols_defined: Vec::new(),
            symbols_referenced: Vec::new(),
            language: "unknown".to_string(),
            granularity: "unknown".to_string(),
            byte_size: 0,
            token_estimate: 0,
            source_file: None,
            source_lines: None,
            outgoing_edges: Vec::new(),
            incoming_edges: Vec::new(),
            metadata: HashMap::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Set the primary alias
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        self.primary_alias = Some(alias.clone());
        if !self.aliases.contains(&alias) {
            self.aliases.push(alias);
        }
        self
    }

    /// Set the language
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    /// Set the granularity
    pub fn with_granularity(mut self, granularity: impl Into<String>) -> Self {
        self.granularity = granularity.into();
        self
    }

    /// Set the byte size
    pub fn with_size(mut self, byte_size: usize) -> Self {
        self.byte_size = byte_size;
        // Rough token estimate: ~4 chars per token
        self.token_estimate = byte_size / 4;
        self
    }

    /// Set source location
    pub fn with_source(mut self, file: impl Into<String>, start: usize, end: usize) -> Self {
        self.source_file = Some(file.into());
        self.source_lines = Some((start, end));
        self
    }

    /// Add symbols this chunk defines
    pub fn with_defines(mut self, symbols: Vec<String>) -> Self {
        self.symbols_defined = symbols;
        self
    }

    /// Add symbols this chunk references
    pub fn with_references(mut self, symbols: Vec<String>) -> Self {
        self.symbols_referenced = symbols;
        self
    }

    /// Add an outgoing edge (dependency)
    pub fn add_dependency(&mut self, edge_type: EdgeType, target_id: String) {
        self.outgoing_edges.push((edge_type, target_id));
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// Add an incoming edge (dependent)
    pub fn add_dependent(&mut self, edge_type: EdgeType, source_id: String) {
        self.incoming_edges.push((edge_type, source_id));
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// Get all chunks this depends on
    pub fn dependencies(&self) -> impl Iterator<Item = &String> {
        self.outgoing_edges.iter().map(|(_, id)| id)
    }

    /// Get all chunks that depend on this
    pub fn dependents(&self) -> impl Iterator<Item = &String> {
        self.incoming_edges.iter().map(|(_, id)| id)
    }

    /// Get dependencies filtered by edge type
    pub fn dependencies_of_type(&self, edge_type: EdgeType) -> Vec<&String> {
        self.outgoing_edges
            .iter()
            .filter(|(et, _)| *et == edge_type)
            .map(|(_, id)| id)
            .collect()
    }

    /// Serialize for storage
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from storage
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

impl Default for GraphNode {
    fn default() -> Self {
        Self::new("", "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = GraphNode::new("chunk:sha256:abc123", "abc123")
            .with_alias("myproject/utils/tax")
            .with_language("rust")
            .with_granularity("function")
            .with_size(1000)
            .with_defines(vec!["calculate_tax".to_string()])
            .with_references(vec!["TaxRate".to_string()]);

        assert_eq!(node.chunk_id, "chunk:sha256:abc123");
        assert_eq!(node.primary_alias, Some("myproject/utils/tax".to_string()));
        assert_eq!(node.language, "rust");
        assert_eq!(node.token_estimate, 250); // 1000 / 4
        assert!(node.symbols_defined.contains(&"calculate_tax".to_string()));
    }

    #[test]
    fn test_edge_operations() {
        let mut node = GraphNode::new("chunk:sha256:abc123", "abc123");
        
        node.add_dependency(EdgeType::Imports, "chunk:sha256:def456".to_string());
        node.add_dependency(EdgeType::TypeRef, "chunk:sha256:ghi789".to_string());
        
        assert_eq!(node.dependencies().count(), 2);
        assert_eq!(node.dependencies_of_type(EdgeType::Imports).len(), 1);
    }

    #[test]
    fn test_serialization() {
        let node = GraphNode::new("chunk:sha256:abc123", "abc123")
            .with_alias("test/node");
        
        let bytes = node.to_bytes().unwrap();
        let restored = GraphNode::from_bytes(&bytes).unwrap();
        
        assert_eq!(node.chunk_id, restored.chunk_id);
        assert_eq!(node.primary_alias, restored.primary_alias);
    }
}
