//! Rehydration Engine
//!
//! The main engine for creating virtual views from atoms.

use std::collections::{HashMap, HashSet};

use super::assembler::Assembler;
use super::config::ViewConfig;
use super::view::{VirtualView, ViewFragment, InclusionReason};
use crate::error::{CadiError, CadiResult};
use crate::graph::{GraphStore, GraphQuery, EdgeType, TraversalDirection};

/// The rehydration engine
pub struct RehydrationEngine {
    graph: GraphStore,
}

impl RehydrationEngine {
    /// Create a new rehydration engine
    pub fn new(graph: GraphStore) -> Self {
        Self { graph }
    }

    /// Create a virtual view from requested atom IDs
    pub async fn create_view(
        &self,
        atom_ids: Vec<String>,
        config: ViewConfig,
    ) -> CadiResult<VirtualView> {
        let expansion_depth = config.expansion_depth;
        
        // Expand dependencies if configured
        let (all_atoms, ghost_atoms) = if expansion_depth > 0 {
            self.expand_dependencies(&atom_ids, expansion_depth).await?
        } else {
            (atom_ids.clone(), Vec::new())
        };

        // Collect atom data
        let mut atoms_with_content = Vec::new();
        for atom_id in &all_atoms {
            if let Some(node) = self.graph.get_node(atom_id)? {
                if let Some(content) = self.graph.get_content_str(atom_id)? {
                    atoms_with_content.push((node, content));
                }
            }
        }

        if atoms_with_content.is_empty() {
            return Ok(VirtualView::new("unknown"));
        }

        // Detect primary language
        let language = atoms_with_content
            .first()
            .map(|(n, _)| n.language.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Assemble the view
        let assembler = Assembler::new(config.clone());
        let result = assembler.assemble(atoms_with_content, &language);

        // Build explanation
        let mut explanation = format!(
            "Created view with {} atoms",
            all_atoms.len()
        );
        if !ghost_atoms.is_empty() {
            explanation.push_str(&format!(
                " ({} added via Ghost Import)",
                ghost_atoms.len()
            ));
        }

        Ok(VirtualView {
            source: result.source,
            atoms: all_atoms,
            ghost_atoms,
            token_estimate: result.total_tokens,
            language,
            symbol_locations: result.symbol_locations,
            fragments: result.fragments,
            truncated: result.truncated,
            explanation,
        })
    }

    /// Create a view with explicit control over ghost imports
    pub async fn create_expanded_view(
        &self,
        atom_ids: Vec<String>,
        expansion_depth: usize,
        max_tokens: usize,
    ) -> CadiResult<VirtualView> {
        let config = ViewConfig::default()
            .with_expansion(expansion_depth)
            .with_max_tokens(max_tokens);
        
        self.create_view(atom_ids, config).await
    }

    /// Expand dependencies recursively
    async fn expand_dependencies(
        &self,
        atom_ids: &[String],
        depth: usize,
    ) -> CadiResult<(Vec<String>, Vec<String>)> {
        let mut all_atoms = HashSet::new();
        let mut ghost_atoms = Vec::new();

        // Add requested atoms
        for id in atom_ids {
            all_atoms.insert(id.clone());
        }

        // BFS expansion
        let mut frontier: Vec<String> = atom_ids.to_vec();
        
        for current_depth in 0..depth {
            let mut next_frontier = Vec::new();

            for atom_id in &frontier {
                // Get dependencies
                let deps = self.graph.get_dependencies(atom_id)?;
                
                for (edge_type, dep_id) in deps {
                    // Only follow strong dependencies for ghost imports
                    if !edge_type.should_auto_expand() {
                        continue;
                    }

                    if !all_atoms.contains(&dep_id) {
                        all_atoms.insert(dep_id.clone());
                        ghost_atoms.push(dep_id.clone());
                        next_frontier.push(dep_id);
                    }
                }
            }

            frontier = next_frontier;
            
            if frontier.is_empty() {
                break;
            }
        }

        Ok((all_atoms.into_iter().collect(), ghost_atoms))
    }

    /// Get token estimate for a set of atoms
    pub fn estimate_tokens(&self, atom_ids: &[String]) -> CadiResult<usize> {
        let mut total = 0;
        for id in atom_ids {
            total += self.graph.get_token_estimate(id)?;
        }
        Ok(total)
    }

    /// Find atoms that define a specific symbol
    pub fn find_defining_atoms(&self, symbol: &str) -> CadiResult<Vec<String>> {
        match self.graph.find_symbol(symbol)? {
            Some(chunk_id) => Ok(vec![chunk_id]),
            None => Ok(Vec::new()),
        }
    }

    /// Create a view for a specific symbol with context
    pub async fn view_for_symbol(
        &self,
        symbol: &str,
        config: ViewConfig,
    ) -> CadiResult<VirtualView> {
        let defining_atoms = self.find_defining_atoms(symbol)?;
        
        if defining_atoms.is_empty() {
            return Err(CadiError::ChunkNotFound(format!(
                "No atom defines symbol: {}", symbol
            )));
        }

        self.create_view(defining_atoms, config).await
    }

    /// Create a minimal view with just type signatures
    pub async fn signatures_view(&self, atom_ids: Vec<String>) -> CadiResult<VirtualView> {
        let config = ViewConfig {
            format: super::config::ViewFormat::Signatures,
            expansion_depth: 0,
            include_docs: false,
            ..Default::default()
        };

        self.create_view(atom_ids, config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphNode;

    #[test]
    fn test_create_view() {
        let graph = GraphStore::in_memory().unwrap();
        
        // Add test nodes
        let node_a = GraphNode::new("chunk:a", "a")
            .with_alias("test/a")
            .with_language("rust")
            .with_defines(vec!["func_a".to_string()]);
        
        graph.insert_node(&node_a).unwrap();
        graph.store_content("chunk:a", b"fn func_a() {}").unwrap();
        
        let engine = RehydrationEngine::new(graph);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let view = rt.block_on(engine.create_view(
            vec!["chunk:a".to_string()],
            ViewConfig::default(),
        )).unwrap();
        
        assert!(view.source.contains("func_a"));
        assert_eq!(view.atoms.len(), 1);
    }

    #[test]
    fn test_ghost_imports() {
        let graph = GraphStore::in_memory().unwrap();
        
        // A depends on B
        let node_a = GraphNode::new("chunk:a", "a")
            .with_language("rust")
            .with_defines(vec!["use_b".to_string()]);
        let node_b = GraphNode::new("chunk:b", "b")
            .with_language("rust")
            .with_defines(vec!["helper".to_string()]);
        
        graph.insert_node(&node_a).unwrap();
        graph.insert_node(&node_b).unwrap();
        graph.store_content("chunk:a", b"fn use_b() { helper(); }").unwrap();
        graph.store_content("chunk:b", b"fn helper() {}").unwrap();
        graph.add_dependency("chunk:a", "chunk:b", EdgeType::Imports).unwrap();
        
        let engine = RehydrationEngine::new(graph);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let view = rt.block_on(engine.create_view(
            vec!["chunk:a".to_string()],
            ViewConfig::default().with_expansion(1),
        )).unwrap();
        
        // B should be included as a ghost import
        assert!(view.atoms.contains(&"chunk:b".to_string()));
        assert!(view.ghost_atoms.contains(&"chunk:b".to_string()));
    }
}
