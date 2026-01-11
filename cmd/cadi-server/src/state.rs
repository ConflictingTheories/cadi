//! Server state management

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
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

/// File-based chunk storage
#[derive(Clone)]
pub struct ChunkStore {
    storage_path: PathBuf,
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
    pub fn new(storage_path: PathBuf) -> std::io::Result<Self> {
        // Create storage directory if it doesn't exist
        fs::create_dir_all(&storage_path)?;
        
        // Load existing metadata if available
        let metadata_path = storage_path.join("metadata.json");
        let metadata = if metadata_path.exists() {
            let data = fs::read_to_string(&metadata_path)?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            HashMap::new()
        };
        
        Ok(Self {
            storage_path,
            metadata,
        })
    }

    pub async fn get(&self, chunk_id: &str) -> Option<Vec<u8>> {
        let chunk_path = self.chunk_path(chunk_id);
        match fs::read(&chunk_path) {
            Ok(data) => Some(data),
            Err(_) => None,
        }
    }

    pub async fn get_meta(&self, chunk_id: &str) -> Option<ChunkMetadata> {
        self.metadata.get(chunk_id).cloned()
    }

    pub async fn exists(&self, chunk_id: &str) -> bool {
        let chunk_path = self.chunk_path(chunk_id);
        chunk_path.exists()
    }

    pub async fn store(&mut self, chunk_id: String, data: Vec<u8>) -> std::io::Result<()> {
        let chunk_path = self.chunk_path(&chunk_id);
        
        // Write chunk data
        fs::write(&chunk_path, &data)?;
        
        // Update metadata
        let size = data.len();
        let meta = ChunkMetadata {
            chunk_id: chunk_id.clone(),
            size,
            created_at: chrono::Utc::now().to_rfc3339(),
            content_type: "application/octet-stream".to_string(),
        };
        
        self.metadata.insert(chunk_id, meta);
        
        // Save metadata to disk
        self.save_metadata().await?;
        
        Ok(())
    }

    pub async fn delete(&mut self, chunk_id: &str) -> bool {
        let chunk_path = self.chunk_path(chunk_id);
        let existed = chunk_path.exists();
        
        if existed {
            let _ = fs::remove_file(&chunk_path);
            self.metadata.remove(chunk_id);
            let _ = self.save_metadata().await;
        }
        
        existed
    }

    pub async fn list(&self) -> Vec<ChunkMetadata> {
        self.metadata.values().cloned().collect()
    }

    pub fn stats(&self) -> StoreStats {
        // For simplicity, we'll calculate stats from filesystem
        // In a real implementation, we'd cache this
        let mut chunk_count = 0;
        let mut total_size = 0;
        
        if let Ok(entries) = fs::read_dir(&self.storage_path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() && entry.file_name() != "metadata.json" {
                        chunk_count += 1;
                        total_size += metadata.len() as usize;
                    }
                }
            }
        }
        
        StoreStats {
            chunk_count,
            total_size,
        }
    }

    fn chunk_path(&self, chunk_id: &str) -> PathBuf {
        // Use chunk ID as filename, but sanitize it
        let safe_name = chunk_id.replace(":", "_").replace("/", "_");
        self.storage_path.join(format!("{}.chunk", safe_name))
    }

    async fn save_metadata(&self) -> std::io::Result<()> {
        let metadata_path = self.storage_path.join("metadata.json");
        let data = serde_json::to_string_pretty(&self.metadata)?;
        fs::write(metadata_path, data)?;
        Ok(())
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
        let store = ChunkStore::new(config.storage_path.clone().into())
            .expect("Failed to initialize chunk store");
        Self {
            config,
            store: Arc::new(RwLock::new(store)),
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
