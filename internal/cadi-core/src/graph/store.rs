//! Sled-backed Graph Store
//!
//! The core persistent storage for the CADI dependency graph.
//! Uses sled for embedded, ACID-compliant storage with O(1) lookups.

use sled::{Db, Tree};
use std::collections::{HashSet, VecDeque};
use std::path::Path;
use std::time::Instant;

use super::{EdgeType, GraphNode, GraphQuery, QueryNode, QueryResult, TraversalDirection};
use crate::error::{CadiError, CadiResult};

/// The main graph store
pub struct GraphStore {
    /// The underlying sled database
    db: Db,

    /// Chunk metadata: chunk_id -> GraphNode (serialized)
    nodes: Tree,

    /// Forward edges: chunk_id -> [dependency chunk_ids]
    /// (things this chunk depends on)
    dependencies: Tree,

    /// Reverse edges: chunk_id -> [dependent chunk_ids]  
    /// (things that depend on this chunk)
    dependents: Tree,

    /// Symbol index: symbol_name -> chunk_id
    /// For fast "who defines this symbol?" lookups
    symbols: Tree,

    /// Alias index: alias -> chunk_id
    /// For human-readable lookups
    aliases: Tree,

    /// Chunk content storage: chunk_id -> content bytes
    content: Tree,
}

impl GraphStore {
    /// Open or create a graph store at the given path
    pub fn open(path: impl AsRef<Path>) -> CadiResult<Self> {
        let db = sled::open(path.as_ref()).map_err(|e| {
            CadiError::StorageError(format!("Failed to open graph store: {}", e))
        })?;

        Ok(Self {
            nodes: db.open_tree("nodes")?,
            dependencies: db.open_tree("dependencies")?,
            dependents: db.open_tree("dependents")?,
            symbols: db.open_tree("symbols")?,
            aliases: db.open_tree("aliases")?,
            content: db.open_tree("content")?,
            db,
        })
    }

    /// Create an in-memory graph store (for testing)
    pub fn in_memory() -> CadiResult<Self> {
        let config = sled::Config::new().temporary(true);
        let db = config.open().map_err(|e| {
            CadiError::StorageError(format!("Failed to create in-memory store: {}", e))
        })?;

        Ok(Self {
            nodes: db.open_tree("nodes")?,
            dependencies: db.open_tree("dependencies")?,
            dependents: db.open_tree("dependents")?,
            symbols: db.open_tree("symbols")?,
            aliases: db.open_tree("aliases")?,
            content: db.open_tree("content")?,
            db,
        })
    }

    // ========================================================================
    // Node Operations
    // ========================================================================

    /// Insert or update a graph node
    pub fn insert_node(&self, node: &GraphNode) -> CadiResult<()> {
        let key = node.chunk_id.as_bytes();
        let value = node.to_bytes()?;
        
        self.nodes.insert(key, value)?;

        // Index symbols
        for symbol in &node.symbols_defined {
            self.symbols.insert(symbol.as_bytes(), key)?;
        }

        // Index aliases
        for alias in &node.aliases {
            self.aliases.insert(alias.as_bytes(), key)?;
        }

        // Update edge indices
        self.update_edge_indices(&node.chunk_id, &node.outgoing_edges, &node.incoming_edges)?;

        Ok(())
    }

    /// Get a node by chunk ID
    pub fn get_node(&self, chunk_id: &str) -> CadiResult<Option<GraphNode>> {
        match self.nodes.get(chunk_id.as_bytes())? {
            Some(bytes) => Ok(Some(GraphNode::from_bytes(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Check if a node exists
    pub fn node_exists(&self, chunk_id: &str) -> CadiResult<bool> {
        Ok(self.nodes.contains_key(chunk_id.as_bytes())?)
    }

    /// Delete a node and its edges
    pub fn delete_node(&self, chunk_id: &str) -> CadiResult<bool> {
        // Get the node first to clean up indices
        if let Some(node) = self.get_node(chunk_id)? {
            // Remove from symbol index
            for symbol in &node.symbols_defined {
                self.symbols.remove(symbol.as_bytes())?;
            }

            // Remove from alias index
            for alias in &node.aliases {
                self.aliases.remove(alias.as_bytes())?;
            }

            // Remove edges
            self.dependencies.remove(chunk_id.as_bytes())?;
            self.dependents.remove(chunk_id.as_bytes())?;

            // Remove content
            self.content.remove(chunk_id.as_bytes())?;

            // Remove node
            self.nodes.remove(chunk_id.as_bytes())?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    // ========================================================================
    // Content Operations
    // ========================================================================

    /// Store chunk content
    pub fn store_content(&self, chunk_id: &str, content: &[u8]) -> CadiResult<()> {
        self.content.insert(chunk_id.as_bytes(), content)?;
        Ok(())
    }

    /// Get chunk content
    pub fn get_content(&self, chunk_id: &str) -> CadiResult<Option<Vec<u8>>> {
        Ok(self.content.get(chunk_id.as_bytes())?.map(|v| v.to_vec()))
    }

    /// Get content as string
    pub fn get_content_str(&self, chunk_id: &str) -> CadiResult<Option<String>> {
        match self.get_content(chunk_id)? {
            Some(bytes) => Ok(Some(String::from_utf8_lossy(&bytes).to_string())),
            None => Ok(None),
        }
    }

    // ========================================================================
    // Edge Operations
    // ========================================================================

    /// Add a dependency edge: source depends on target
    pub fn add_dependency(&self, source: &str, target: &str, edge_type: EdgeType) -> CadiResult<()> {
        // Add to forward index (source -> dependencies)
        self.add_to_edge_list(&self.dependencies, source, target, edge_type)?;
        
        // Add to reverse index (target -> dependents)
        self.add_to_edge_list(&self.dependents, target, source, edge_type)?;

        Ok(())
    }

    /// Get all dependencies of a chunk (things it needs)
    pub fn get_dependencies(&self, chunk_id: &str) -> CadiResult<Vec<(EdgeType, String)>> {
        self.get_edge_list(&self.dependencies, chunk_id)
    }

    /// Get all dependents of a chunk (things that need it)
    pub fn get_dependents(&self, chunk_id: &str) -> CadiResult<Vec<(EdgeType, String)>> {
        self.get_edge_list(&self.dependents, chunk_id)
    }

    /// Get dependencies filtered by edge type
    pub fn get_dependencies_of_type(
        &self,
        chunk_id: &str,
        edge_type: EdgeType,
    ) -> CadiResult<Vec<String>> {
        Ok(self
            .get_dependencies(chunk_id)?
            .into_iter()
            .filter(|(et, _)| *et == edge_type)
            .map(|(_, id)| id)
            .collect())
    }

    // ========================================================================
    // Symbol & Alias Lookups
    // ========================================================================

    /// Find the chunk that defines a symbol
    pub fn find_symbol(&self, symbol: &str) -> CadiResult<Option<String>> {
        match self.symbols.get(symbol.as_bytes())? {
            Some(bytes) => Ok(Some(String::from_utf8_lossy(&bytes).to_string())),
            None => Ok(None),
        }
    }

    /// Resolve an alias to chunk ID
    pub fn resolve_alias(&self, alias: &str) -> CadiResult<Option<String>> {
        match self.aliases.get(alias.as_bytes())? {
            Some(bytes) => Ok(Some(String::from_utf8_lossy(&bytes).to_string())),
            None => Ok(None),
        }
    }

    /// Get all symbols defined by a chunk
    pub fn get_symbols_for_chunk(&self, chunk_id: &str) -> CadiResult<Vec<String>> {
        if let Some(node) = self.get_node(chunk_id)? {
            Ok(node.symbols_defined)
        } else {
            Ok(Vec::new())
        }
    }

    // ========================================================================
    // Graph Queries
    // ========================================================================

    /// Execute a graph query
    pub fn query(&self, query: &GraphQuery) -> CadiResult<QueryResult> {
        let start = Instant::now();
        let mut visited = HashSet::new();
        let mut result_nodes = Vec::new();
        let mut queue = VecDeque::new();

        // Initialize queue with start nodes
        for chunk_id in &query.start_nodes {
            if query.include_start {
                if let Some(node) = self.get_node(chunk_id)? {
                    result_nodes.push(QueryNode {
                        chunk_id: chunk_id.clone(),
                        alias: node.primary_alias,
                        depth: 0,
                        reached_via: None,
                        parent: None,
                        token_estimate: node.token_estimate,
                    });
                }
            }
            visited.insert(chunk_id.clone());
            queue.push_back((chunk_id.clone(), 0, None, None));
        }

        // BFS traversal
        while let Some((current_id, depth, _reached_via, _parent)) = queue.pop_front() {
            if depth >= query.max_depth {
                continue;
            }
            if result_nodes.len() >= query.max_results {
                break;
            }

            // Get edges based on direction
            let edges = match query.direction {
                TraversalDirection::Outgoing => self.get_dependencies(&current_id)?,
                TraversalDirection::Incoming => self.get_dependents(&current_id)?,
                TraversalDirection::Both => {
                    let mut all = self.get_dependencies(&current_id)?;
                    all.extend(self.get_dependents(&current_id)?);
                    all
                }
            };

            for (edge_type, target_id) in edges {
                if visited.contains(&target_id) {
                    continue;
                }
                if !query.should_follow_edge(edge_type) {
                    continue;
                }

                visited.insert(target_id.clone());

                // Get node details
                if let Some(node) = self.get_node(&target_id)? {
                    // Apply filters
                    if let Some(ref lang) = query.language_filter {
                        if &node.language != lang {
                            continue;
                        }
                    }
                    if let Some(ref gran) = query.granularity_filter {
                        if &node.granularity != gran {
                            continue;
                        }
                    }

                    result_nodes.push(QueryNode {
                        chunk_id: target_id.clone(),
                        alias: node.primary_alias,
                        depth: depth + 1,
                        reached_via: Some(edge_type),
                        parent: Some(current_id.clone()),
                        token_estimate: node.token_estimate,
                    });

                    queue.push_back((
                        target_id,
                        depth + 1,
                        Some(edge_type),
                        Some(current_id.clone()),
                    ));
                }
            }
        }

        Ok(QueryResult {
            nodes: result_nodes,
            nodes_visited: visited.len(),
            truncated: visited.len() > query.max_results,
            execution_time_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Get token estimate for a chunk
    pub fn get_token_estimate(&self, chunk_id: &str) -> CadiResult<usize> {
        if let Some(node) = self.get_node(chunk_id)? {
            Ok(node.token_estimate)
        } else {
            Ok(0)
        }
    }

    // ========================================================================
    // Statistics
    // ========================================================================

    /// Get store statistics
    pub fn stats(&self) -> CadiResult<GraphStoreStats> {
        Ok(GraphStoreStats {
            node_count: self.nodes.len(),
            symbol_count: self.symbols.len(),
            alias_count: self.aliases.len(),
            dependency_entries: self.dependencies.len(),
            dependent_entries: self.dependents.len(),
            content_entries: self.content.len(),
            db_size_bytes: self.db.size_on_disk()?,
        })
    }

    /// List all nodes in the graph
    pub fn list_nodes(&self) -> CadiResult<Vec<GraphNode>> {
        let mut nodes = Vec::new();
        
        for result in self.nodes.iter() {
            match result {
                Ok((_key_bytes, value_bytes)) => {
                    if let Ok(node) = GraphNode::from_bytes(&value_bytes) {
                        nodes.push(node);
                    }
                }
                Err(e) => return Err(CadiError::StorageError(format!("Failed to iterate nodes: {}", e))),
            }
        }
        
        Ok(nodes)
    }

    /// List all edges in the graph
    pub fn list_edges(&self) -> CadiResult<Vec<(String, String, EdgeType)>> {
        let mut edges = Vec::new();
        
        for result in self.dependencies.iter() {
            match result {
                Ok((key_bytes, value_bytes)) => {
                    if let Ok(from_chunk) = std::str::from_utf8(&key_bytes) {
                        if let Ok(edge_list) = serde_json::from_slice::<Vec<(EdgeType, String)>>(&value_bytes) {
                            for (edge_type, to_chunk) in edge_list {
                                edges.push((from_chunk.to_string(), to_chunk, edge_type));
                            }
                        }
                    }
                }
                Err(e) => return Err(CadiError::StorageError(format!("Failed to iterate edges: {}", e))),
            }
        }
        
        Ok(edges)
    }

    /// Flush all pending writes to disk
    pub fn flush(&self) -> CadiResult<()> {
        self.db.flush()?;
        Ok(())
    }

    // ========================================================================
    // Private Helpers
    // ========================================================================

    fn update_edge_indices(
        &self,
        chunk_id: &str,
        outgoing: &[(EdgeType, String)],
        _incoming: &[(EdgeType, String)],
    ) -> CadiResult<()> {
        // Store outgoing edges
        if !outgoing.is_empty() {
            let serialized = serde_json::to_vec(outgoing)?;
            self.dependencies.insert(chunk_id.as_bytes(), serialized)?;
        }

        // Update reverse indices for each outgoing edge
        for (edge_type, target) in outgoing {
            self.add_to_edge_list(&self.dependents, target, chunk_id, *edge_type)?;
        }

        Ok(())
    }

    fn add_to_edge_list(
        &self,
        tree: &Tree,
        key: &str,
        value: &str,
        edge_type: EdgeType,
    ) -> CadiResult<()> {
        let mut edges = self.get_edge_list(tree, key)?;
        
        // Avoid duplicates
        if !edges.iter().any(|(et, v)| et == &edge_type && v == value) {
            edges.push((edge_type, value.to_string()));
            let serialized = serde_json::to_vec(&edges)?;
            tree.insert(key.as_bytes(), serialized)?;
        }
        
        Ok(())
    }

    fn get_edge_list(&self, tree: &Tree, key: &str) -> CadiResult<Vec<(EdgeType, String)>> {
        match tree.get(key.as_bytes())? {
            Some(bytes) => Ok(serde_json::from_slice(&bytes)?),
            None => Ok(Vec::new()),
        }
    }
}

/// Statistics about the graph store
#[derive(Debug, Clone)]
pub struct GraphStoreStats {
    pub node_count: usize,
    pub symbol_count: usize,
    pub alias_count: usize,
    pub dependency_entries: usize,
    pub dependent_entries: usize,
    pub content_entries: usize,
    pub db_size_bytes: u64,
}

impl std::fmt::Display for GraphStoreStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GraphStore: {} nodes, {} symbols, {} aliases, {} deps, {} size",
            self.node_count,
            self.symbol_count,
            self.alias_count,
            self.dependency_entries,
            human_bytes(self.db_size_bytes)
        )
    }
}

fn human_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

// Implement From for sled errors
impl From<sled::Error> for CadiError {
    fn from(e: sled::Error) -> Self {
        CadiError::StorageError(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_creation() {
        let store = GraphStore::in_memory().unwrap();
        let stats = store.stats().unwrap();
        assert_eq!(stats.node_count, 0);
    }

    #[test]
    fn test_node_crud() {
        let store = GraphStore::in_memory().unwrap();

        let node = GraphNode::new("chunk:sha256:abc123", "abc123")
            .with_alias("test/node")
            .with_language("rust")
            .with_size(1000)
            .with_defines(vec!["my_function".to_string()]);

        // Insert
        store.insert_node(&node).unwrap();
        assert!(store.node_exists("chunk:sha256:abc123").unwrap());

        // Get
        let retrieved = store.get_node("chunk:sha256:abc123").unwrap().unwrap();
        assert_eq!(retrieved.chunk_id, "chunk:sha256:abc123");
        assert_eq!(retrieved.primary_alias, Some("test/node".to_string()));

        // Symbol lookup
        let found = store.find_symbol("my_function").unwrap();
        assert_eq!(found, Some("chunk:sha256:abc123".to_string()));

        // Alias lookup
        let found = store.resolve_alias("test/node").unwrap();
        assert_eq!(found, Some("chunk:sha256:abc123".to_string()));

        // Delete
        assert!(store.delete_node("chunk:sha256:abc123").unwrap());
        assert!(!store.node_exists("chunk:sha256:abc123").unwrap());
    }

    #[test]
    fn test_content_storage() {
        let store = GraphStore::in_memory().unwrap();

        let content = b"fn hello() { println!(\"Hello\"); }";
        store.store_content("chunk:abc", content).unwrap();

        let retrieved = store.get_content("chunk:abc").unwrap().unwrap();
        assert_eq!(retrieved, content);

        let as_str = store.get_content_str("chunk:abc").unwrap().unwrap();
        assert!(as_str.contains("hello"));
    }

    #[test]
    fn test_dependency_edges() {
        let store = GraphStore::in_memory().unwrap();

        // Create nodes
        let node_a = GraphNode::new("chunk:a", "a").with_defines(vec!["A".to_string()]);
        let node_b = GraphNode::new("chunk:b", "b").with_defines(vec!["B".to_string()]);
        let node_c = GraphNode::new("chunk:c", "c").with_defines(vec!["C".to_string()]);

        store.insert_node(&node_a).unwrap();
        store.insert_node(&node_b).unwrap();
        store.insert_node(&node_c).unwrap();

        // A depends on B (imports)
        // A depends on C (type ref)
        store.add_dependency("chunk:a", "chunk:b", EdgeType::Imports).unwrap();
        store.add_dependency("chunk:a", "chunk:c", EdgeType::TypeRef).unwrap();

        // Check dependencies of A
        let deps = store.get_dependencies("chunk:a").unwrap();
        assert_eq!(deps.len(), 2);

        // Check dependents of B
        let deps = store.get_dependents("chunk:b").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].1, "chunk:a");

        // Filter by type
        let import_deps = store.get_dependencies_of_type("chunk:a", EdgeType::Imports).unwrap();
        assert_eq!(import_deps.len(), 1);
        assert_eq!(import_deps[0], "chunk:b");
    }

    #[test]
    fn test_graph_query() {
        let store = GraphStore::in_memory().unwrap();

        // Create a dependency chain: A -> B -> C
        for (id, alias) in [("a", "root"), ("b", "mid"), ("c", "leaf")] {
            let node = GraphNode::new(format!("chunk:{}", id), id)
                .with_alias(alias)
                .with_size(100);
            store.insert_node(&node).unwrap();
        }

        store.add_dependency("chunk:a", "chunk:b", EdgeType::Imports).unwrap();
        store.add_dependency("chunk:b", "chunk:c", EdgeType::Imports).unwrap();

        // Query dependencies of A with depth 2
        let query = GraphQuery::dependencies("chunk:a").with_depth(2);
        let result = store.query(&query).unwrap();

        assert_eq!(result.nodes.len(), 3); // A, B, C
        assert!(result.nodes.iter().any(|n| n.chunk_id == "chunk:a"));
        assert!(result.nodes.iter().any(|n| n.chunk_id == "chunk:b"));
        assert!(result.nodes.iter().any(|n| n.chunk_id == "chunk:c"));
    }
}
