//! Merkle DAG Graph Store
//!
//! Stores chunks and their relationships for O(1) dependency queries.
//! This is the core data structure that makes CADI efficient for agents.
//!
//! ## Architecture
//!
//! The graph store uses sled (an embedded database) to maintain:
//! - **Chunk metadata**: Core information about each atom
//! - **Forward edges**: chunk -> [things that depend on it]
//! - **Reverse edges**: chunk -> [things it depends on]
//! - **Symbol index**: symbol_name -> chunk_id for fast lookups
//!
//! ## Example
//!
//! ```rust,ignore
//! use cadi_core::graph::{GraphStore, GraphNode, EdgeType};
//!
//! let store = GraphStore::open(".cadi/graph")?;
//!
//! // Add a chunk
//! store.insert_node(GraphNode {
//!     chunk_id: "chunk:sha256:abc123...".to_string(),
//!     content_hash: "abc123...".to_string(),
//!     symbols_defined: vec!["calculate_tax".to_string()],
//!     symbols_referenced: vec!["TaxRate".to_string()],
//!     ..Default::default()
//! })?;
//!
//! // Query dependencies in O(1)
//! let deps = store.get_dependencies("chunk:sha256:abc123...")?;
//!
//! // Find who depends on this chunk
//! let dependents = store.get_dependents("chunk:sha256:abc123...")?;
//!
//! // Resolve a symbol to its defining chunk
//! let chunk = store.find_symbol("calculate_tax")?;
//! ```

pub mod store;
pub mod node;
pub mod edge;
pub mod query;

pub use store::GraphStore;
pub use node::GraphNode;
pub use edge::{Edge, EdgeType};
pub use query::{GraphQuery, QueryNode, QueryResult, TraversalDirection};

/// Re-export common error types
pub use crate::error::CadiError;
