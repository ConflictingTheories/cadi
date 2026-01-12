//! Graph query system
//!
//! Provides efficient queries over the chunk dependency graph.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::{EdgeType, GraphNode};

/// A query against the graph
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphQuery {
    /// Starting chunk IDs
    pub start_nodes: Vec<String>,

    /// Direction of traversal
    pub direction: TraversalDirection,

    /// Maximum depth to traverse
    pub max_depth: usize,

    /// Edge types to follow
    pub edge_types: Option<Vec<EdgeType>>,

    /// Maximum nodes to return
    pub max_results: usize,

    /// Filter by language
    pub language_filter: Option<String>,

    /// Filter by granularity
    pub granularity_filter: Option<String>,

    /// Include the starting nodes in results
    pub include_start: bool,
}

/// Direction of graph traversal
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraversalDirection {
    /// Follow outgoing edges (dependencies)
    #[default]
    Outgoing,
    /// Follow incoming edges (dependents)
    Incoming,
    /// Follow both directions
    Both,
}

/// Result of a graph query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Nodes found by the query
    pub nodes: Vec<QueryNode>,

    /// Total nodes visited
    pub nodes_visited: usize,

    /// Was the query truncated due to limits?
    pub truncated: bool,

    /// Query execution time in milliseconds
    pub execution_time_ms: u64,
}

/// A node in query results with traversal metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryNode {
    /// The chunk ID
    pub chunk_id: String,

    /// Primary alias if available
    pub alias: Option<String>,

    /// Depth from starting node
    pub depth: usize,

    /// How this node was reached
    pub reached_via: Option<EdgeType>,

    /// Parent node in traversal
    pub parent: Option<String>,

    /// Token estimate for budgeting
    pub token_estimate: usize,
}

impl GraphQuery {
    /// Create a new query starting from given chunks
    pub fn new(start_nodes: Vec<String>) -> Self {
        Self {
            start_nodes,
            direction: TraversalDirection::Outgoing,
            max_depth: 3,
            edge_types: None,
            max_results: 100,
            language_filter: None,
            granularity_filter: None,
            include_start: true,
        }
    }

    /// Query dependencies (things this chunk needs)
    pub fn dependencies(chunk_id: impl Into<String>) -> Self {
        Self::new(vec![chunk_id.into()]).with_direction(TraversalDirection::Outgoing)
    }

    /// Query dependents (things that need this chunk)
    pub fn dependents(chunk_id: impl Into<String>) -> Self {
        Self::new(vec![chunk_id.into()]).with_direction(TraversalDirection::Incoming)
    }

    /// Set traversal direction
    pub fn with_direction(mut self, direction: TraversalDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set maximum depth
    pub fn with_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Filter by edge types
    pub fn with_edge_types(mut self, types: Vec<EdgeType>) -> Self {
        self.edge_types = Some(types);
        self
    }

    /// Only follow strong dependencies
    pub fn strong_only(self) -> Self {
        self.with_edge_types(vec![
            EdgeType::Imports,
            EdgeType::TypeRef,
            EdgeType::Implements,
            EdgeType::Extends,
        ])
    }

    /// Set maximum results
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.max_results = limit;
        self
    }

    /// Filter by language
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language_filter = Some(language.into());
        self
    }

    /// Whether to include start nodes
    pub fn include_start(mut self, include: bool) -> Self {
        self.include_start = include;
        self
    }

    /// Check if an edge type should be followed
    pub fn should_follow_edge(&self, edge_type: EdgeType) -> bool {
        match &self.edge_types {
            Some(types) => types.contains(&edge_type),
            None => true,
        }
    }
}

impl QueryResult {
    /// Create an empty result
    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            nodes_visited: 0,
            truncated: false,
            execution_time_ms: 0,
        }
    }

    /// Get all chunk IDs in the result
    pub fn chunk_ids(&self) -> Vec<&String> {
        self.nodes.iter().map(|n| &n.chunk_id).collect()
    }

    /// Get total estimated tokens
    pub fn total_tokens(&self) -> usize {
        self.nodes.iter().map(|n| n.token_estimate).sum()
    }

    /// Get nodes at a specific depth
    pub fn at_depth(&self, depth: usize) -> impl Iterator<Item = &QueryNode> {
        self.nodes.iter().filter(move |n| n.depth == depth)
    }

    /// Get the traversal path to a specific node
    pub fn path_to(&self, chunk_id: &str) -> Vec<&QueryNode> {
        let mut path = Vec::new();
        let mut current = self.nodes.iter().find(|n| n.chunk_id == chunk_id);

        while let Some(node) = current {
            path.push(node);
            current = node
                .parent
                .as_ref()
                .and_then(|p| self.nodes.iter().find(|n| &n.chunk_id == p));
        }

        path.reverse();
        path
    }
}

/// Builder for complex queries
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    query: GraphQuery,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            query: GraphQuery::default(),
        }
    }

    pub fn start_from(mut self, chunks: Vec<String>) -> Self {
        self.query.start_nodes = chunks;
        self
    }

    pub fn find_dependencies(self) -> Self {
        Self {
            query: self.query.with_direction(TraversalDirection::Outgoing),
        }
    }

    pub fn find_dependents(self) -> Self {
        Self {
            query: self.query.with_direction(TraversalDirection::Incoming),
        }
    }

    pub fn depth(mut self, d: usize) -> Self {
        self.query.max_depth = d;
        self
    }

    pub fn limit(mut self, l: usize) -> Self {
        self.query.max_results = l;
        self
    }

    pub fn only_strong(self) -> Self {
        Self {
            query: self.query.strong_only(),
        }
    }

    pub fn build(self) -> GraphQuery {
        self.query
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_creation() {
        let query = GraphQuery::dependencies("chunk:abc123")
            .with_depth(2)
            .strong_only();

        assert_eq!(query.start_nodes.len(), 1);
        assert_eq!(query.direction, TraversalDirection::Outgoing);
        assert_eq!(query.max_depth, 2);
        assert!(query.should_follow_edge(EdgeType::Imports));
        assert!(!query.should_follow_edge(EdgeType::Calls));
    }

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::new()
            .start_from(vec!["chunk:a".to_string(), "chunk:b".to_string()])
            .find_dependencies()
            .depth(3)
            .only_strong()
            .limit(50)
            .build();

        assert_eq!(query.start_nodes.len(), 2);
        assert_eq!(query.max_depth, 3);
        assert_eq!(query.max_results, 50);
    }

    #[test]
    fn test_result_operations() {
        let result = QueryResult {
            nodes: vec![
                QueryNode {
                    chunk_id: "a".to_string(),
                    alias: Some("root".to_string()),
                    depth: 0,
                    reached_via: None,
                    parent: None,
                    token_estimate: 100,
                },
                QueryNode {
                    chunk_id: "b".to_string(),
                    alias: None,
                    depth: 1,
                    reached_via: Some(EdgeType::Imports),
                    parent: Some("a".to_string()),
                    token_estimate: 200,
                },
            ],
            nodes_visited: 2,
            truncated: false,
            execution_time_ms: 5,
        };

        assert_eq!(result.total_tokens(), 300);
        assert_eq!(result.at_depth(1).count(), 1);
        assert_eq!(result.path_to("b").len(), 2);
    }
}
