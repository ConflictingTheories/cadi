//! Batch Importer for GraphStore
//!
//! Handles the import of AtomicChunks into the GraphStore, ensuring that
//! dependencies (requires) are correctly resolved to edges.

use crate::atomic::{AtomicChunk, AliasRegistry};
use crate::graph::{GraphStore, GraphNode, EdgeType};
use crate::error::CadiResult;

pub struct BatchImporter<'a> {
    store: &'a GraphStore,
}

impl<'a> BatchImporter<'a> {
    pub fn new(store: &'a GraphStore) -> Self {
        Self { store }
    }

    /// Import atomic chunks and create resolved edges
    pub fn import(&self, chunks: Vec<AtomicChunk>, registry: &AliasRegistry) -> CadiResult<()> {
        // 1. First pass: Insert all nodes
        for chunk in &chunks {
            let mut node = GraphNode::new(&chunk.chunk_id, &chunk.content_hash)
                .with_language(&chunk.language)
                .with_granularity(format!("{:?}", chunk.granularity)) // Debug fmt for now
                .with_size(chunk.size)
                .with_defines(chunk.provides.clone())
                .with_references(chunk.requires.clone());

            // Add aliases
            for alias in &chunk.aliases {
                node = node.with_alias(alias.full_path());
            }

            self.store.insert_node(&node)?;
        }

        // 2. Second pass: Create strong edges from 'requires'
        for chunk in &chunks {
            let source_id = &chunk.chunk_id;
            
            for required_name in &chunk.requires {
                // Try to resolve the requirement to a chunk ID
                // 1. Check if it's an alias in registry
                if let Some(target_id) = registry.resolve(required_name) {
                    self.store.add_dependency(source_id, target_id, EdgeType::Imports)?;
                    continue;
                }

                // 2. Fallback: Search the store for a node defining this symbol
                // This is slower but handles implicit internal symbols
                if let Ok(Some(target_id)) = self.store.find_symbol(required_name) {
                     // Avoid self-dependency
                     if &target_id != source_id {
                        self.store.add_dependency(source_id, &target_id, EdgeType::Imports)?;
                     }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atomic::ChunkGranularity;

    #[test]
    fn test_batch_import_wiring() {
        let store = GraphStore::in_memory().unwrap();
        let importer = BatchImporter::new(&store);
        let mut registry = AliasRegistry::new();

        // Chunk A: Depends on "helper"
        let chunk_a = AtomicChunk::new(
            "chunk:a".to_string(),
            "main".to_string(),
            "rust".to_string(),
            "hash_a".to_string(),
            100,
        ).with_granularity(ChunkGranularity::Function);
        // Manually add requirement since we aren't using SmartChunker here
        let mut chunk_a = chunk_a;
        chunk_a.requires = vec!["helper".to_string()];

        // Chunk B: Provides "helper"
        let chunk_b = AtomicChunk::new(
            "chunk:b".to_string(),
            "helper".to_string(),
            "rust".to_string(),
            "hash_b".to_string(),
            50,
        )
        .with_granularity(ChunkGranularity::Function);
        
        let mut chunk_b = chunk_b;
        chunk_b.provides = vec!["helper".to_string()]; // Export symbol
        
        // Register alias for B as "helper" (simulating an import alias)
        registry.register("helper", "chunk:b");

        // Import
        importer.import(vec![chunk_a.clone(), chunk_b.clone()], &registry).unwrap();

        // Verify Edge
        let deps = store.get_dependencies("chunk:a").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].1, "chunk:b");
        assert_eq!(deps[0].0, EdgeType::Imports);
    }
}
