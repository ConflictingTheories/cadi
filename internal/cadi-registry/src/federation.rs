//! Registry federation support for CADI

use cadi_core::{CadiError, CadiResult};
use std::collections::HashMap;

/// A federated registry configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FederatedRegistry {
    /// Registry identifier
    pub id: String,
    /// Registry URL
    pub url: String,
    /// Priority (lower = higher priority)
    pub priority: u32,
    /// Trust level
    pub trust_level: TrustLevel,
    /// Whether this registry is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Regions this registry serves
    #[serde(default)]
    pub regions: Vec<String>,
    /// Capabilities of this registry
    #[serde(default)]
    pub capabilities: RegistryCapabilities,
}

fn default_true() -> bool {
    true
}

/// Trust level for a registry
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrustLevel {
    /// Fully trusted - accept all chunks
    Full,
    /// Verified - require signatures
    Verified,
    /// Limited - only fetch, don't execute
    Limited,
    /// Untrusted - for read-only browsing
    Untrusted,
}

impl Default for TrustLevel {
    fn default() -> Self {
        Self::Verified
    }
}

/// Capabilities of a registry
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RegistryCapabilities {
    /// Supports push/publish
    #[serde(default)]
    pub push: bool,
    /// Supports search
    #[serde(default)]
    pub search: bool,
    /// Supports streaming
    #[serde(default)]
    pub streaming: bool,
    /// Supports signature verification
    #[serde(default)]
    pub signatures: bool,
    /// Maximum chunk size in bytes
    pub max_chunk_size: Option<u64>,
}

/// Federation manager for multiple registries
pub struct FederationManager {
    registries: Vec<FederatedRegistry>,
    clients: HashMap<String, super::RegistryClient>,
}

impl FederationManager {
    /// Create a new federation manager
    pub fn new() -> Self {
        Self {
            registries: Vec::new(),
            clients: HashMap::new(),
        }
    }

    /// Add a registry to the federation
    pub fn add_registry(&mut self, registry: FederatedRegistry) -> CadiResult<()> {
        let config = super::RegistryConfig {
            url: registry.url.clone(),
            ..Default::default()
        };
        
        let client = super::RegistryClient::new(config)?;
        self.clients.insert(registry.id.clone(), client);
        self.registries.push(registry);
        
        // Sort by priority
        self.registries.sort_by_key(|r| r.priority);
        
        Ok(())
    }

    /// Remove a registry from the federation
    pub fn remove_registry(&mut self, id: &str) -> bool {
        if let Some(pos) = self.registries.iter().position(|r| r.id == id) {
            self.registries.remove(pos);
            self.clients.remove(id);
            true
        } else {
            false
        }
    }

    /// Get all registered registries
    pub fn registries(&self) -> &[FederatedRegistry] {
        &self.registries
    }

    /// Fetch a chunk from the federation
    /// 
    /// Tries registries in priority order until one succeeds
    pub async fn fetch_chunk(&self, chunk_id: &str) -> CadiResult<(Vec<u8>, String)> {
        let mut last_error = None;
        
        for registry in &self.registries {
            if !registry.enabled {
                continue;
            }
            
            if let Some(client) = self.clients.get(&registry.id) {
                match client.fetch_chunk(chunk_id).await {
                    Ok(data) => {
                        return Ok((data, registry.id.clone()));
                    }
                    Err(e) => {
                        tracing::debug!("Failed to fetch from {}: {}", registry.id, e);
                        last_error = Some(e);
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| CadiError::ChunkNotFound(chunk_id.to_string())))
    }

    /// Check if a chunk exists in any registry
    pub async fn chunk_exists(&self, chunk_id: &str) -> CadiResult<Option<String>> {
        for registry in &self.registries {
            if !registry.enabled {
                continue;
            }
            
            if let Some(client) = self.clients.get(&registry.id) {
                if client.chunk_exists(chunk_id).await? {
                    return Ok(Some(registry.id.clone()));
                }
            }
        }
        
        Ok(None)
    }

    /// Find the best registry for publishing
    pub fn best_push_registry(&self) -> Option<&FederatedRegistry> {
        self.registries.iter()
            .filter(|r| r.enabled && r.capabilities.push)
            .min_by_key(|r| r.priority)
    }

    /// Publish a chunk to the best available registry
    pub async fn publish_chunk(&self, chunk_id: &str, data: &[u8]) -> CadiResult<String> {
        let registry = self.best_push_registry()
            .ok_or_else(|| CadiError::RegistryError("No push-capable registry available".to_string()))?;
        
        let client = self.clients.get(&registry.id)
            .ok_or_else(|| CadiError::RegistryError("Registry client not found".to_string()))?;
        
        client.publish_chunk(chunk_id, data).await?;
        
        Ok(registry.id.clone())
    }

    /// Search across all registries
    pub async fn search(&self, query: &super::SearchQuery) -> CadiResult<Vec<(super::ChunkSummary, String)>> {
        let mut results = Vec::new();
        
        for registry in &self.registries {
            if !registry.enabled || !registry.capabilities.search {
                continue;
            }
            
            if let Some(client) = self.clients.get(&registry.id) {
                match client.search(query).await {
                    Ok(search_result) => {
                        for chunk in search_result.chunks {
                            results.push((chunk, registry.id.clone()));
                        }
                    }
                    Err(e) => {
                        tracing::debug!("Search failed on {}: {}", registry.id, e);
                    }
                }
            }
        }
        
        // Deduplicate by chunk_id, keeping first occurrence (highest priority)
        let mut seen = std::collections::HashSet::new();
        results.retain(|(chunk, _)| seen.insert(chunk.chunk_id.clone()));
        
        Ok(results)
    }
}

impl Default for FederationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Load federation config from file
pub fn load_federation_config(path: &std::path::Path) -> CadiResult<Vec<FederatedRegistry>> {
    let content = std::fs::read_to_string(path)?;
    let registries: Vec<FederatedRegistry> = serde_json::from_str(&content)?;
    Ok(registries)
}

/// Save federation config to file
pub fn save_federation_config(path: &std::path::Path, registries: &[FederatedRegistry]) -> CadiResult<()> {
    let content = serde_json::to_string_pretty(registries)?;
    std::fs::write(path, content)?;
    Ok(())
}
