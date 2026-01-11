//! Types for registry operations

use serde::{Deserialize, Serialize};

/// Tier information for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageTier {
    /// Tier name (e.g., "hot", "warm", "cold")
    pub name: String,
    /// Tier priority (lower = faster access)
    pub priority: u32,
    /// Cost factor (1.0 = baseline)
    pub cost_factor: f64,
    /// Access latency in milliseconds (typical)
    pub latency_ms: u32,
}

impl StorageTier {
    pub fn hot() -> Self {
        Self {
            name: "hot".to_string(),
            priority: 0,
            cost_factor: 1.0,
            latency_ms: 10,
        }
    }

    pub fn warm() -> Self {
        Self {
            name: "warm".to_string(),
            priority: 1,
            cost_factor: 0.5,
            latency_ms: 100,
        }
    }

    pub fn cold() -> Self {
        Self {
            name: "cold".to_string(),
            priority: 2,
            cost_factor: 0.1,
            latency_ms: 1000,
        }
    }
}

/// Access pattern for a chunk
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccessPattern {
    /// Number of fetch operations
    pub fetch_count: u64,
    /// Last fetch time (RFC3339)
    pub last_fetch: Option<String>,
    /// Number of build operations
    pub build_count: u64,
    /// Last build time (RFC3339)
    pub last_build: Option<String>,
    /// Average fetch latency in ms
    pub avg_fetch_latency_ms: Option<f64>,
}

/// Replication status for a chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationStatus {
    /// Number of replicas
    pub replicas: u32,
    /// Regions where replicas exist
    pub regions: Vec<String>,
    /// Whether replication target is met
    pub target_met: bool,
}

/// Chunk location info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkLocation {
    /// Registry URL
    pub registry: String,
    /// Current storage tier
    pub tier: StorageTier,
    /// Replication status
    pub replication: ReplicationStatus,
    /// Size in bytes
    pub size_bytes: u64,
}

/// Batch fetch request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchFetchRequest {
    /// Chunk IDs to fetch
    pub chunk_ids: Vec<String>,
    /// Whether to include metadata
    #[serde(default)]
    pub include_meta: bool,
    /// Preferred tier
    pub preferred_tier: Option<String>,
}

/// Batch fetch response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchFetchResponse {
    /// Successfully fetched chunks
    pub chunks: Vec<FetchedChunk>,
    /// Failed chunk IDs
    pub failed: Vec<FetchFailure>,
}

/// A fetched chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchedChunk {
    pub chunk_id: String,
    /// Base64-encoded data
    pub data: String,
    pub size_bytes: usize,
}

/// A fetch failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchFailure {
    pub chunk_id: String,
    pub error: String,
}

/// Authentication info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    /// Token type (e.g., "Bearer")
    pub token_type: String,
    /// Access token
    pub access_token: String,
    /// Expiration time (RFC3339)
    pub expires_at: Option<String>,
    /// Scopes granted
    #[serde(default)]
    pub scopes: Vec<String>,
}

/// Publisher identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publisher {
    /// Publisher ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Verification status
    #[serde(default)]
    pub verified: bool,
    /// Public key fingerprint
    pub key_fingerprint: Option<String>,
}
