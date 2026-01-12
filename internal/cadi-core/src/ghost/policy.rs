use crate::graph::EdgeType;

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

impl ExpansionPolicy {
    /// Conservative policy for minimal context
    pub fn conservative() -> Self {
        Self {
            max_depth: 1,
            max_atoms: 10,
            max_tokens: 2000,
            follow_edges: vec![EdgeType::Imports],
            always_include_types: false,
            include_signatures: false,
        }
    }

    /// Aggressive policy for comprehensive context
    pub fn aggressive() -> Self {
        Self {
            max_depth: 3,
            max_atoms: 50,
            max_tokens: 8000,
            follow_edges: vec![
                EdgeType::Imports,
                EdgeType::TypeRef,
                EdgeType::Calls,
            ],
            always_include_types: true,
            include_signatures: true,
        }
    }
}