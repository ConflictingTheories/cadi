use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingStore {
    pub embeddings: HashMap<String, Vec<f32>>,
}

impl EmbeddingStore {
    pub fn new() -> Self {
        Self { embeddings: HashMap::new() }
    }

    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let s = fs::read_to_string(path)?;
            let store: EmbeddingStore = serde_json::from_str(&s)?;
            Ok(store)
        } else {
            Ok(Self::new())
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let s = serde_json::to_string_pretty(self)?;
        fs::write(path, s)?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<Vec<f32>> {
        self.embeddings.get(key).cloned()
    }

    pub fn insert(&mut self, key: String, emb: Vec<f32>) {
        self.embeddings.insert(key, emb);
    }

    /// Compute cosine similarity between two vectors
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    /// Search for similar embeddings
    pub fn search(&self, query_embedding: &[f32], limit: usize) -> Vec<(String, f32)> {
        let mut results: Vec<(String, f32)> = self.embeddings.iter()
            .map(|(key, emb)| (key.clone(), Self::cosine_similarity(query_embedding, emb)))
            .collect();
        
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        results.into_iter().take(limit).collect()
    }
}
