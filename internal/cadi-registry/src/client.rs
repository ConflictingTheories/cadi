//! Registry client for CADI

use cadi_core::{CadiError, CadiResult, Chunk, Manifest};
use std::time::Duration;

/// Registry client configuration
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// Registry URL
    pub url: String,
    /// Authentication token
    pub token: Option<String>,
    /// Request timeout
    pub timeout: Duration,
    /// Whether to verify TLS certificates
    pub verify_tls: bool,
    /// Maximum concurrent requests
    pub max_concurrent: usize,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            url: "https://registry.cadi.dev".to_string(),
            token: None,
            timeout: Duration::from_secs(30),
            verify_tls: true,
            max_concurrent: 4,
        }
    }
}

/// Registry client
pub struct RegistryClient {
    config: RegistryConfig,
    http: reqwest::Client,
}

impl RegistryClient {
    /// Create a new registry client
    pub fn new(config: RegistryConfig) -> CadiResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .danger_accept_invalid_certs(!config.verify_tls)
            .build()
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;
        
        Ok(Self { config, http })
    }

    /// Create a client with default configuration
    pub fn default_client() -> CadiResult<Self> {
        Self::new(RegistryConfig::default())
    }

    /// Check if a chunk exists in the registry
    pub async fn chunk_exists(&self, chunk_id: &str) -> CadiResult<bool> {
        let url = format!("{}/v1/chunks/{}", self.config.url, chunk_id);
        
        let mut request = self.http.head(&url);
        if let Some(ref token) = self.config.token {
            request = request.bearer_auth(token);
        }
        
        let response = request.send().await
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;
        
        Ok(response.status().is_success())
    }

    /// Fetch a chunk from the registry
    pub async fn fetch_chunk(&self, chunk_id: &str) -> CadiResult<Vec<u8>> {
        let url = format!("{}/v1/chunks/{}", self.config.url, chunk_id);
        
        let mut request = self.http.get(&url);
        if let Some(ref token) = self.config.token {
            request = request.bearer_auth(token);
        }
        
        let response = request.send().await
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;
        
        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(CadiError::ChunkNotFound(chunk_id.to_string()));
            }
            return Err(CadiError::RegistryError(
                format!("HTTP {}: {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown"))
            ));
        }
        
        let bytes = response.bytes().await
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;
        
        Ok(bytes.to_vec())
    }

    /// Fetch chunk metadata
    pub async fn fetch_chunk_meta(&self, chunk_id: &str) -> CadiResult<Chunk> {
        let url = format!("{}/v1/chunks/{}/meta", self.config.url, chunk_id);
        
        let mut request = self.http.get(&url);
        if let Some(ref token) = self.config.token {
            request = request.bearer_auth(token);
        }
        
        let response = request.send().await
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;
        
        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(CadiError::ChunkNotFound(chunk_id.to_string()));
            }
            return Err(CadiError::RegistryError(
                format!("HTTP {}", response.status())
            ));
        }
        
        let chunk: Chunk = response.json().await
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;
        
        Ok(chunk)
    }

    /// Publish a chunk to the registry
    pub async fn publish_chunk(&self, chunk_id: &str, data: &[u8]) -> CadiResult<PublishResult> {
        let url = format!("{}/v1/chunks/{}", self.config.url, chunk_id);
        
        let mut request = self.http
            .put(&url)
            .body(data.to_vec());
        
        if let Some(ref token) = self.config.token {
            request = request.bearer_auth(token);
        }
        
        let response = request.send().await
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(CadiError::RegistryError(
                format!("HTTP {}: {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown"))
            ));
        }
        
        let result: PublishResult = response.json().await
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;
        
        Ok(result)
    }

    /// Search for chunks
    pub async fn search(&self, query: &SearchQuery) -> CadiResult<RegistrySearchResult> {
        let url = format!("{}/v1/search", self.config.url);
        
        let mut request = self.http
            .post(&url)
            .json(query);
        
        if let Some(ref token) = self.config.token {
            request = request.bearer_auth(token);
        }
        
        let response = request.send().await
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(CadiError::RegistryError(
                format!("HTTP {}", response.status())
            ));
        }
        
        let result: RegistrySearchResult = response.json().await
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;
        
        Ok(result)
    }

    /// Perform a semantic search against the registry
    pub async fn semantic_search(&self, query: &str, limit: usize) -> CadiResult<Vec<(ChunkSummary, f32)>> {
        let url = format!("{}/v1/semantic_search", self.config.url);
        let body = serde_json::json!({ "query": query, "limit": limit });

        let mut request = self.http.post(&url).json(&body);
        if let Some(ref token) = self.config.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CadiError::RegistryError(format!("HTTP {}", response.status())));
        }

        let hits: Vec<serde_json::Value> = response.json().await
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;

        // Map json results to (ChunkSummary, score) where ChunkSummary is partial
        let mut out = Vec::new();
        for v in hits {
            if let (Some(chunk_id), Some(score)) = (v.get("chunk_id"), v.get("score")) {
                let cs = ChunkSummary {
                    chunk_id: chunk_id.as_str().unwrap_or_default().to_string(),
                    name: "".to_string(),
                    cadi_type: "".to_string(),
                    concepts: vec![],
                    description: None,
                };
                out.push((cs, score.as_f64().unwrap_or(0.0) as f32));
            }
        }

        Ok(out)
    }

    /// Fetch a manifest
    pub async fn fetch_manifest(&self, manifest_id: &str) -> CadiResult<Manifest> {
        let url = format!("{}/v1/manifests/{}", self.config.url, manifest_id);
        
        let mut request = self.http.get(&url);
        if let Some(ref token) = self.config.token {
            request = request.bearer_auth(token);
        }
        
        let response = request.send().await
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;
        
        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(CadiError::ManifestNotFound(manifest_id.to_string()));
            }
            return Err(CadiError::RegistryError(
                format!("HTTP {}", response.status())
            ));
        }
        
        let manifest: Manifest = response.json().await
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;
        
        Ok(manifest)
    }

    /// Publish a manifest
    pub async fn publish_manifest(&self, manifest: &Manifest) -> CadiResult<PublishResult> {
        let url = format!("{}/v1/manifests/{}", self.config.url, manifest.manifest_id);
        
        let mut request = self.http
            .put(&url)
            .json(manifest);
        
        if let Some(ref token) = self.config.token {
            request = request.bearer_auth(token);
        }
        
        let response = request.send().await
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(CadiError::RegistryError(
                format!("HTTP {}", response.status())
            ));
        }
        
        let result: PublishResult = response.json().await
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;
        
        Ok(result)
    }

    /// Get registry health status
    pub async fn health(&self) -> CadiResult<HealthStatus> {
        let url = format!("{}/health", self.config.url);
        
        let response = self.http.get(&url).send().await
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Ok(HealthStatus {
                healthy: false,
                version: None,
                message: Some(format!("HTTP {}", response.status())),
            });
        }
        
        let status: HealthStatus = response.json().await
            .map_err(|e| CadiError::RegistryError(e.to_string()))?;
        
        Ok(status)
    }
}

/// Result of a publish operation
#[derive(Debug, serde::Deserialize)]
pub struct PublishResult {
    pub success: bool,
    pub chunk_id: Option<String>,
    pub message: Option<String>,
}

/// Search query
#[derive(Debug, serde::Serialize)]
pub struct SearchQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concepts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cadi_type: Option<String>,
    #[serde(default)]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            query: None,
            concepts: None,
            language: None,
            cadi_type: None,
            limit: 20,
            offset: 0,
        }
    }
}

/// Registry search result
#[derive(Debug, serde::Deserialize)]
pub struct RegistrySearchResult {
    pub chunks: Vec<ChunkSummary>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
}

/// Summary of a chunk in search results
#[derive(Debug, serde::Deserialize)]
pub struct ChunkSummary {
    pub chunk_id: String,
    pub name: String,
    pub cadi_type: String,
    #[serde(default)]
    pub concepts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Health status of the registry
#[derive(Debug, serde::Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub version: Option<String>,
    pub message: Option<String>,
}
