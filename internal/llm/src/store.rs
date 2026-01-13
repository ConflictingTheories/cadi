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
}
