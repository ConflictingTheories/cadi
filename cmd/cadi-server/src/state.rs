//! Server state management

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address to bind to
    pub bind_address: String,
    /// Storage path for chunks
    pub storage_path: String,
    /// Maximum chunk size in bytes
    pub max_chunk_size: usize,
    /// Enable anonymous reads
    pub anonymous_read: bool,
    /// Enable anonymous writes
    pub anonymous_write: bool,
}

impl ServerConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            bind_address: std::env::var("CADI_BIND")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            storage_path: std::env::var("CADI_STORAGE")
                .unwrap_or_else(|_| "./data".to_string()),
            max_chunk_size: std::env::var("CADI_MAX_CHUNK_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100 * 1024 * 1024), // 100 MB
            anonymous_read: std::env::var("CADI_ANON_READ")
                .map(|s| s == "true" || s == "1")
                .unwrap_or(true),
            anonymous_write: std::env::var("CADI_ANON_WRITE")
                .map(|s| s == "true" || s == "1")
                .unwrap_or(false),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:8080".to_string(),
            storage_path: "./data".to_string(),
            max_chunk_size: 100 * 1024 * 1024,
            anonymous_read: true,
            anonymous_write: false,
        }
    }
}

/// In-memory chunk storage (for demo purposes)
#[derive(Default)]
pub struct ChunkStore {
    chunks: HashMap<String, Vec<u8>>,
    metadata: HashMap<String, ChunkMetadata>,
}

/// Chunk metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChunkMetadata {
    pub chunk_id: String,
    pub size: usize,
    pub created_at: String,
    pub content_type: String,
}

impl ChunkStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, chunk_id: &str) -> Option<&Vec<u8>> {
        self.chunks.get(chunk_id)
    }

    pub fn get_meta(&self, chunk_id: &str) -> Option<&ChunkMetadata> {
        self.metadata.get(chunk_id)
    }

    pub fn exists(&self, chunk_id: &str) -> bool {
        self.chunks.contains_key(chunk_id)
    }

    pub fn store(&mut self, chunk_id: String, data: Vec<u8>) {
        let size = data.len();
        let meta = ChunkMetadata {
            chunk_id: chunk_id.clone(),
            size,
            created_at: chrono::Utc::now().to_rfc3339(),
            content_type: "application/octet-stream".to_string(),
        };
        self.metadata.insert(chunk_id.clone(), meta);
        self.chunks.insert(chunk_id, data);
    }

    pub fn delete(&mut self, chunk_id: &str) -> bool {
        let existed = self.chunks.remove(chunk_id).is_some();
        self.metadata.remove(chunk_id);
        existed
    }

    pub fn list(&self) -> Vec<&ChunkMetadata> {
        self.metadata.values().collect()
    }

    pub fn stats(&self) -> StoreStats {
        StoreStats {
            chunk_count: self.chunks.len(),
            total_size: self.chunks.values().map(|v| v.len()).sum(),
        }
    }
}

/// Store statistics
#[derive(Debug, serde::Serialize)]
pub struct StoreStats {
    pub chunk_count: usize,
    pub total_size: usize,
}

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub config: ServerConfig,
    pub store: Arc<RwLock<ChunkStore>>,
}

impl AppState {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            store: Arc::new(RwLock::new(ChunkStore::new())),
        }
    }
}

// Placeholder for chrono - in real impl would use chrono crate
mod chrono {
    pub struct Utc;
    
    impl Utc {
        pub fn now() -> DateTime {
            DateTime
        }
    }
    
    pub struct DateTime;
    
    impl DateTime {
        pub fn to_rfc3339(&self) -> String {
            // Simplified ISO timestamp
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            format!("{}", now)
        }
    }
}
