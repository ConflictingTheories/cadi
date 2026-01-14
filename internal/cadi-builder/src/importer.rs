use cadi_core::SemanticNormalizer;
use cadi_core::deduplication::DeduplicationEngine;
use cadi_core::CadiResult;
use std::sync::Mutex;
use serde_json::json;
use std::sync::Arc;

/// Minimal trait the builder expects from a search DB
#[allow(unused_variables)]
#[async_trait::async_trait]
pub trait SearchStore: Send + Sync + 'static {
    async fn store_chunk(&self, chunk_id: &str, hash: &str, metadata: serde_json::Value, embedding: Option<Vec<f32>>) -> CadiResult<String>;
}

/// Minimal trait for GraphDB operations the builder uses
#[allow(unused_variables)]
#[async_trait::async_trait]
pub trait GraphStore: Send + Sync + 'static {
    async fn create_chunk_node(&self, chunk_id: &str, metadata: serde_json::Value, semantic_hash: &str) -> CadiResult<()>;
    async fn create_edge(&self, from: &str, to: &str, edge_type: &str, props: Option<serde_json::Value>) -> CadiResult<()>;
}

pub struct ProjectImporter<S: SearchStore, G: GraphStore> {
    dedup_engine: Arc<Mutex<DeduplicationEngine>>,
    search_db: Arc<S>,
    graph_db: Arc<G>,
}

impl<S: SearchStore, G: GraphStore> ProjectImporter<S, G> {
    pub fn new(dedup_engine: Arc<Mutex<DeduplicationEngine>>, search_db: Arc<S>, graph_db: Arc<G>) -> Self {
        Self { dedup_engine, search_db, graph_db }
    }

    /// Analyze a project path and return chunk-like items (stubbed for now)
    async fn analyze_project(&self, _project_path: &std::path::Path) -> CadiResult<Vec<ChunkCandidate>> {
        // In a real implementation this would walk files and extract code chunks
        Ok(vec![])
    }

    pub async fn import_chunks(&self, chunks: Vec<ChunkCandidate>, namespace: &str) -> CadiResult<ImportReport> {
        let mut report = ImportReport::default();

        for chunk in chunks {
            let normalizer = SemanticNormalizer::new(&chunk.language)?;
            let norm = normalizer.normalize(&chunk.code)?;

            let chunk_id = format!("{}:{}", namespace, &chunk.id);

            let (is_new, equivalents) = {
                let mut engine = self.dedup_engine.lock().unwrap();
                engine.register_chunk(&chunk_id, &norm.hash)
            };

            if is_new {
                // Extract metadata (stub)
                let metadata = json!({"name": chunk.module_path, "description": "", "concepts": []});
                // Store chunk in search DB
                let _ = self.search_db.store_chunk(&chunk_id, &norm.hash, metadata.clone(), None).await?;
                self.graph_db.create_chunk_node(&chunk_id, metadata, &norm.hash).await?;
                report.new_chunks.push(ChunkInfo { id: chunk_id, hash: norm.hash.clone() });
            } else if !equivalents.is_empty() {
                // EQUIVALENT chunk found: create EQUIVALENT_TO edge
                for equivalent_id in &equivalents {
                    self.graph_db.create_edge(&chunk_id, equivalent_id, "EQUIVALENT_TO", Some(json!({"semantic_hash": norm.hash}))).await?;
                }
                report.equivalents_found.push((chunk_id.clone(), equivalents));
            }
        }

        Ok(report)
    }

    pub async fn import_and_deduplicate(&self, project_path: &std::path::Path, namespace: &str) -> CadiResult<ImportReport> {
        let chunks = self.analyze_project(project_path).await?;
        self.import_chunks(chunks, namespace).await
    }
}

#[derive(Default)]
pub struct ImportReport {
    pub new_chunks: Vec<ChunkInfo>,
    pub deduplicated: Vec<DeduplicatedInfo>,
    pub equivalents_found: Vec<(String, Vec<String>)>,
}

#[derive(Debug)]
pub struct ChunkInfo { pub id: String, pub hash: String }

#[derive(Debug)]
pub struct DeduplicatedInfo { pub original_id: String, pub replaced_by: String }

#[derive(Debug)]
pub struct ChunkCandidate { pub id: String, pub code: String, pub language: String, pub module_path: String }
