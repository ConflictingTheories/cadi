use cadi_builder::ProjectImporter;
use cadi_core::SemanticNormalizer;
use cadi_core::deduplication::DeduplicationEngine;
use std::sync::{Arc, Mutex};
use cadi_builder::{SearchStore, GraphStore};
use cadi_core::CadiResult;
use serde_json::json;

struct MockSearch {
    pub stored: Mutex<Vec<String>>,
}

#[async_trait::async_trait]
impl SearchStore for MockSearch {
    async fn store_chunk(&self, chunk_id: &str, _hash: &str, _metadata: serde_json::Value, _embedding: Option<Vec<f32>>) -> CadiResult<String> {
        let mut s = self.stored.lock().unwrap();
        s.push(chunk_id.to_string());
        Ok(chunk_id.to_string())
    }
}

struct MockGraph {
    pub nodes: Mutex<Vec<String>>,
    pub edges: Mutex<Vec<(String,String,String)>>,
}

#[async_trait::async_trait]
impl GraphStore for MockGraph {
    async fn create_chunk_node(&self, chunk_id: &str, _metadata: serde_json::Value, _semantic_hash: &str) -> CadiResult<()> {
        let mut n = self.nodes.lock().unwrap();
        n.push(chunk_id.to_string());
        Ok(())
    }
    async fn create_edge(&self, from: &str, to: &str, edge_type: &str, _props: Option<serde_json::Value>) -> CadiResult<()> {
        let mut e = self.edges.lock().unwrap();
        e.push((from.to_string(), to.to_string(), edge_type.to_string()));
        Ok(())
    }
}

#[tokio::test]
async fn test_import_smoke_creates_nodes_and_edges() {
    let dedup = Arc::new(Mutex::new(DeduplicationEngine::new()));
    let search = Arc::new(MockSearch { stored: Mutex::new(vec![]) });
    let graph = Arc::new(MockGraph { nodes: Mutex::new(vec![]), edges: Mutex::new(vec![]) });

    let importer = ProjectImporter::new(dedup.clone(), search.clone(), graph.clone());

    let chunk1 = cadi_builder::ChunkCandidate { id: "c1".to_string(), code: "function add(x, y) { return x + y; }".to_string(), language: "typescript".to_string(), module_path: "mod1".to_string() };
    let chunk2 = cadi_builder::ChunkCandidate { id: "c2".to_string(), code: "function  add  ( a , b ) { return a + b; }".to_string(), language: "typescript".to_string(), module_path: "mod2".to_string() };

    let report = importer.import_chunks(vec![chunk1, chunk2], "demo").await.unwrap();

    // First chunk should be new and stored
    assert_eq!(report.new_chunks.len(), 1);
    assert_eq!(report.new_chunks[0].id, "demo:c1");

    // Second chunk should be deduplicated and create an equivalence edge
    assert_eq!(report.equivalents_found.len(), 1);
    assert_eq!(report.equivalents_found[0].0, "demo:c2");
    let equivalents = &report.equivalents_found[0].1;
    assert!(equivalents.contains(&"demo:c1".to_string()));

    // Search store should have one stored chunk
    let s = search.stored.lock().unwrap();
    assert_eq!(s.len(), 1);
    assert_eq!(s[0], "demo:c1");

    // Graph nodes should include demo:c1
    let nodes = graph.nodes.lock().unwrap();
    assert!(nodes.contains(&"demo:c1".to_string()));

    // Graph edges should include an edge for the equivalence
    let edges = graph.edges.lock().unwrap();
    assert!(edges.iter().any(|(from,to,et)| from=="demo:c2" && to=="demo:c1" && et=="EQUIVALENT_TO"));
}
