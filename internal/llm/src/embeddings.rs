use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::store::EmbeddingStore;

/// An embedding vector
pub type Embedding = Vec<f32>;

/// Interface for generating embeddings
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate an embedding for a piece of text
    async fn generate(&self, text: &str) -> Result<Embedding>;
    
    /// Generate embeddings for multiple pieces of text
    async fn generate_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.generate(text).await?);
        }
        Ok(results)
    }
}


/// OpenAI embedding provider
pub struct OpenAiProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            model: model.unwrap_or_else(|| "text-embedding-3-small".to_string()),
            client: reqwest::Client::new(),
        }
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        MockProvider
    }
}

impl std::fmt::Debug for MockProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockProvider").finish()
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAiProvider {
    async fn generate(&self, text: &str) -> Result<Embedding> {
        let response = self.client.post("https://api.openai.com/v1/embeddings")
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({
                "input": text,
                "model": self.model,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error: serde_json::Value = response.json().await?;
            return Err(anyhow!("OpenAI error: {}", error));
        }

        let result: OpenAiResponse = response.json().await?;
        Ok(result.data[0].embedding.clone())
    }
}

#[derive(Deserialize)]
struct OpenAiResponse {
    data: Vec<OpenAiData>,
}

#[derive(Deserialize)]
struct OpenAiData {
    embedding: Vec<f32>,
}

/// Mock embedding provider for testing and dev
pub struct MockProvider;

#[async_trait]
impl EmbeddingProvider for MockProvider {
    async fn generate(&self, text: &str) -> Result<Embedding> {
        // Deterministic mock embedding based on text length
        let len = text.len() as f32;
        Ok(vec![len, len * 2.0, len / 2.0, 0.0, 1.0])
    }
}

/// Manager for chunk semantic search
pub struct EmbeddingManager {
    provider: Box<dyn EmbeddingProvider>,
    cache: HashMap<String, Embedding>,
    store_path: Option<PathBuf>,
    store: EmbeddingStore,
}

impl EmbeddingManager {
    pub fn new(provider: Box<dyn EmbeddingProvider>, store_path: Option<PathBuf>) -> Self {
        let store = if let Some(ref p) = store_path {
            EmbeddingStore::load(p).unwrap_or_else(|_| EmbeddingStore::new())
        } else {
            EmbeddingStore::new()
        };

        Self {
            provider,
            cache: store.embeddings.clone(),
            store_path,
            store,
        }
    }

    /// Get or generate an embedding for a chunk
    pub async fn get_chunk_embedding(&mut self, chunk_id: &str, content: &str) -> Result<Embedding> {
        if let Some(emb) = self.cache.get(chunk_id) {
            return Ok(emb.clone());
        }

        if let Some(emb) = self.store.get(chunk_id) {
            self.cache.insert(chunk_id.to_string(), emb.clone());
            return Ok(emb);
        }

        let emb = self.provider.generate(content).await?;
        self.cache.insert(chunk_id.to_string(), emb.clone());
        self.store.insert(chunk_id.to_string(), emb.clone());

        if let Some(ref p) = self.store_path {
            let _ = self.store.save(p);
        }

        Ok(emb)
    }

    /// Perform a semantic search
    pub async fn search(&self, query: &str, candidates: &HashMap<String, Embedding>, limit: usize) -> Result<Vec<(String, f32)>> {
        let query_emb = self.provider.generate(query).await?;
        
        let mut results: Vec<_> = candidates.iter()
            .map(|(id, emb)| {
                let score = cosine_similarity(&query_emb, emb);
                (id.clone(), score)
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(limit);
        
        Ok(results)
    }
}

impl EmbeddingManager {
    /// Save persistent embedding store to disk (best-effort)
    pub fn save_store(&self) -> Result<()> {
        if let Some(ref p) = self.store_path {
            self.store.save(p)?;
        }
        Ok(())
    }

    /// Return a clone of the configured store path
    pub fn store_path_clone(&self) -> Option<PathBuf> {
        self.store_path.clone()
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a * norm_b)
}
