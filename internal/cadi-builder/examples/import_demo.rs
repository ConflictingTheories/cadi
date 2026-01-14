use cadi_builder::ProjectImporter;
use cadi_core::deduplication::DeduplicationEngine;
use std::sync::{Arc, Mutex};
use serde_json::json;

struct ConsoleSearch;

#[async_trait::async_trait]
impl cadi_builder::SearchStore for ConsoleSearch {
    async fn store_chunk(&self, chunk_id: &str, _hash: &str, metadata: serde_json::Value, _embedding: Option<Vec<f32>>) -> cadi_core::CadiResult<String> {
        println!("[search] stored chunk {} metadata {:?}", chunk_id, metadata);
        Ok(chunk_id.to_string())
    }
}

struct ConsoleGraph;

#[async_trait::async_trait]
impl cadi_builder::GraphStore for ConsoleGraph {
    async fn create_chunk_node(&self, chunk_id: &str, metadata: serde_json::Value, semantic_hash: &str) -> cadi_core::CadiResult<()> {
        println!("[graph] create node {} hash {} metadata {:?}", chunk_id, semantic_hash, metadata);
        Ok(())
    }

    async fn create_edge(&self, from: &str, to: &str, edge_type: &str, props: Option<serde_json::Value>) -> cadi_core::CadiResult<()> {
        println!("[graph] edge {} -> {} ({}) props {:?}", from, to, edge_type, props);
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let dedup = Arc::new(Mutex::new(DeduplicationEngine::new()));
    let search = Arc::new(ConsoleSearch);
    let graph = Arc::new(ConsoleGraph);

    let importer = ProjectImporter::new(dedup, search, graph);

    let chunk1 = cadi_builder::ChunkCandidate { id: "c1".to_string(), code: "function add(x, y) { return x + y; }".to_string(), language: "typescript".to_string(), module_path: "mod1".to_string() };
    let chunk2 = cadi_builder::ChunkCandidate { id: "c2".to_string(), code: "function  add  ( a , b ) { return a + b; }".to_string(), language: "typescript".to_string(), module_path: "mod2".to_string() };

    let report = importer.import_chunks(vec![chunk1, chunk2], "demo").await.unwrap();

    println!("Import report: new={}, equivalents={}", report.new_chunks.len(), report.equivalents_found.len());
}
