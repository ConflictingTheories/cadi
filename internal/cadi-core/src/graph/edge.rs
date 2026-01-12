//! Edge types and relationships in the dependency graph
//!
//! Edges represent relationships between chunks (atoms).

use serde::{Deserialize, Serialize};

/// Type of edge in the dependency graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    /// Direct import/use statement
    /// e.g., `use crate::utils::helper;` or `import { x } from './y'`
    Imports,

    /// Type reference (uses a type defined elsewhere)
    /// e.g., function parameter `fn foo(x: MyType)` references MyType
    TypeRef,

    /// Function/method call
    /// e.g., `helper_function()` calls helper_function
    Calls,

    /// Composition relationship (parent contains child)
    /// e.g., a module chunk is composed of function chunks
    ComposedOf,

    /// Trait/interface implementation
    /// e.g., `impl Trait for Type` creates impl edge to Trait
    Implements,

    /// Extends/inherits from
    /// e.g., `class B extends A` creates extends edge to A
    Extends,

    /// Exports symbol (reverse of imports)
    Exports,

    /// Generic/template reference
    /// e.g., `Vec<MyType>` references MyType
    GenericRef,

    /// Macro usage
    MacroUse,

    /// Test relationship (test -> code being tested)
    Tests,

    /// Documentation reference
    DocRef,
}

impl EdgeType {
    /// Is this a "strong" dependency that must be included for correctness?
    pub fn is_strong(&self) -> bool {
        matches!(
            self,
            EdgeType::Imports | EdgeType::TypeRef | EdgeType::Implements | EdgeType::Extends
        )
    }

    /// Is this a "weak" dependency that's nice to have but optional?
    pub fn is_weak(&self) -> bool {
        matches!(
            self,
            EdgeType::Calls | EdgeType::DocRef | EdgeType::Tests | EdgeType::MacroUse
        )
    }

    /// Should this edge be followed during Ghost Import expansion?
    pub fn should_auto_expand(&self) -> bool {
        matches!(
            self,
            EdgeType::Imports | EdgeType::TypeRef | EdgeType::GenericRef
        )
    }

    /// Priority for context assembly (lower = included first)
    pub fn assembly_priority(&self) -> u8 {
        match self {
            EdgeType::Imports => 1,
            EdgeType::TypeRef => 2,
            EdgeType::Implements => 3,
            EdgeType::Extends => 3,
            EdgeType::GenericRef => 4,
            EdgeType::Calls => 5,
            EdgeType::MacroUse => 6,
            EdgeType::ComposedOf => 10,
            EdgeType::Exports => 10,
            EdgeType::Tests => 20,
            EdgeType::DocRef => 20,
        }
    }
}

impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            EdgeType::Imports => "imports",
            EdgeType::TypeRef => "type_ref",
            EdgeType::Calls => "calls",
            EdgeType::ComposedOf => "composed_of",
            EdgeType::Implements => "implements",
            EdgeType::Extends => "extends",
            EdgeType::Exports => "exports",
            EdgeType::GenericRef => "generic_ref",
            EdgeType::MacroUse => "macro_use",
            EdgeType::Tests => "tests",
            EdgeType::DocRef => "doc_ref",
        };
        write!(f, "{}", s)
    }
}

/// A full edge representation with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Source chunk ID
    pub source: String,

    /// Target chunk ID
    pub target: String,

    /// Type of relationship
    pub edge_type: EdgeType,

    /// Specific symbol(s) involved in this edge
    pub symbols: Vec<String>,

    /// Weight/importance (0.0 - 1.0)
    pub weight: f32,

    /// Additional metadata
    pub metadata: Option<EdgeMetadata>,
}

/// Additional edge metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMetadata {
    /// Line number in source where this relationship exists
    pub source_line: Option<usize>,

    /// Whether this is a re-export
    pub is_reexport: bool,

    /// Conditional (e.g., cfg(feature = "x"))
    pub condition: Option<String>,
}

impl Edge {
    /// Create a new edge
    pub fn new(
        source: impl Into<String>,
        target: impl Into<String>,
        edge_type: EdgeType,
    ) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            edge_type,
            symbols: Vec::new(),
            weight: 1.0,
            metadata: None,
        }
    }

    /// Add symbols involved in this edge
    pub fn with_symbols(mut self, symbols: Vec<String>) -> Self {
        self.symbols = symbols;
        self
    }

    /// Set the weight
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: EdgeMetadata) -> Self {
        self.metadata = Some(metadata);
        self
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

/// A collection of edges for batch operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdgeSet {
    pub edges: Vec<Edge>,
}

impl EdgeSet {
    pub fn new() -> Self {
        Self { edges: Vec::new() }
    }

    pub fn add(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    pub fn filter_by_type(&self, edge_type: EdgeType) -> impl Iterator<Item = &Edge> {
        self.edges.iter().filter(move |e| e.edge_type == edge_type)
    }

    pub fn strong_edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.iter().filter(|e| e.edge_type.is_strong())
    }

    pub fn weak_edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.iter().filter(|e| e.edge_type.is_weak())
    }

    pub fn targets(&self) -> impl Iterator<Item = &String> {
        self.edges.iter().map(|e| &e.target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_creation() {
        let edge = Edge::new("chunk:a", "chunk:b", EdgeType::Imports)
            .with_symbols(vec!["foo".to_string(), "bar".to_string()])
            .with_weight(0.8);

        assert_eq!(edge.source, "chunk:a");
        assert_eq!(edge.target, "chunk:b");
        assert!(edge.edge_type.is_strong());
        assert_eq!(edge.symbols.len(), 2);
    }

    #[test]
    fn test_edge_priorities() {
        assert!(EdgeType::Imports.assembly_priority() < EdgeType::Calls.assembly_priority());
        assert!(EdgeType::TypeRef.assembly_priority() < EdgeType::Tests.assembly_priority());
    }

    #[test]
    fn test_edge_set() {
        let mut set = EdgeSet::new();
        set.add(Edge::new("a", "b", EdgeType::Imports));
        set.add(Edge::new("a", "c", EdgeType::Calls));
        set.add(Edge::new("a", "d", EdgeType::TypeRef));

        assert_eq!(set.strong_edges().count(), 2); // Imports + TypeRef
        assert_eq!(set.weak_edges().count(), 1); // Calls
    }
}
