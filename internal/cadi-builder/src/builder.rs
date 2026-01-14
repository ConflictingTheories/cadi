use cadi_core::SemanticNormalizer;
use cadi_core::deduplication::DeduplicationEngine;
use cadi_core::CadiResult;
use std::sync::Arc;
use std::sync::Mutex;

pub struct BuildPlanner {
    dedup_engine: Arc<Mutex<DeduplicationEngine>>,
}

impl BuildPlanner {
    pub fn new(dedup_engine: Arc<Mutex<DeduplicationEngine>>) -> Self {
        Self { dedup_engine }
    }

    /// When building, check if generated code has semantic equivalents
    pub async fn check_for_equivalents(&self, generated_code: &str, language: &str, _purpose: &str) -> CadiResult<Vec<EquivalentChunk>> {
        let normalizer = SemanticNormalizer::new(language)?;
        let norm = normalizer.normalize(generated_code)?;
        let list = {
            let engine = self.dedup_engine.lock().unwrap();
            engine.find_equivalents(&norm.hash)
        };

        Ok(list.into_iter().map(|id| EquivalentChunk {
            id,
            hash: norm.hash.clone(),
            message: "Semantically equivalent to existing chunk. Consider reusing instead of generating.".to_string(),
        }).collect())
    }
}

pub struct EquivalentChunk {
    pub id: String,
    pub hash: String,
    pub message: String,
}
