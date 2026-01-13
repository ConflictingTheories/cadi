//! HTTP route handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use cadi_llm::embeddings::Embedding;

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Health check handler
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Get chunk handler
pub async fn get_chunk(
    State(state): State<AppState>,
    Path(chunk_id): Path<String>,
) -> Result<Vec<u8>, StatusCode> {
    let store = state.store.read().await;
    
    store.get(&chunk_id).await
        .ok_or(StatusCode::NOT_FOUND)
}

/// Head chunk handler (check existence)
pub async fn head_chunk(
    State(state): State<AppState>,
    Path(chunk_id): Path<String>,
) -> StatusCode {
    let store = state.store.read().await;
    
    if store.exists(&chunk_id).await {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

/// Put chunk handler
pub async fn put_chunk(
    State(state): State<AppState>,
    Path(chunk_id): Path<String>,
    body: axum::body::Bytes,
) -> Result<Json<PutResponse>, StatusCode> {
    // Verify hash matches
    if !cadi_core::hash::verify_chunk_content(&chunk_id, &body) {
        return Ok(Json(PutResponse {
            success: false,
            chunk_id: Some(chunk_id),
            message: Some("Hash mismatch".to_string()),
        }));
    }
    
    // Check size limit
    if body.len() > state.config.max_chunk_size {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    
    let mut store = state.store.write().await;
    match store.store(chunk_id.clone(), body.to_vec()).await {
        Ok(_) => Ok(Json(PutResponse {
            success: true,
            chunk_id: Some(chunk_id),
            message: None,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Put response
#[derive(Serialize)]
pub struct PutResponse {
    pub success: bool,
    pub chunk_id: Option<String>,
    pub message: Option<String>,
}

/// Delete chunk handler
pub async fn delete_chunk(
    State(state): State<AppState>,
    Path(chunk_id): Path<String>,
) -> StatusCode {
    let mut store = state.store.write().await;
    
    if store.delete(&chunk_id).await {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

/// Get chunk metadata handler
pub async fn get_chunk_meta(
    State(state): State<AppState>,
    Path(chunk_id): Path<String>,
) -> Result<Json<crate::state::ChunkMetadata>, StatusCode> {
    let store = state.store.read().await;
    
    store.get_meta(&chunk_id).await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// List chunks handler
pub async fn list_chunks(
    State(state): State<AppState>,
) -> Json<Vec<crate::state::ChunkMetadata>> {
    let store = state.store.read().await;
    Json(store.list().await)
}

/// Stats handler
pub async fn stats(
    State(state): State<AppState>,
) -> Json<crate::state::StoreStats> {
    let store = state.store.read().await;
    Json(store.stats())
}

/// Search query
#[derive(Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
}

/// Search response
#[derive(Serialize)]
pub struct SearchResponse {
    pub chunks: Vec<crate::state::ChunkMetadata>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
}

/// Search handler
pub async fn search(
    State(state): State<AppState>,
    Json(query): Json<SearchQuery>,
) -> Json<SearchResponse> {
    let store = state.store.read().await;
    let all_chunks = store.list().await;
    
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    
    // Simple substring search
    let filtered: Vec<_> = if let Some(ref q) = query.query {
        all_chunks.into_iter()
            .filter(|c| c.chunk_id.contains(q))
            .collect()
    } else {
        all_chunks
    };
    
    let total = filtered.len();
    let chunks: Vec<_> = filtered.into_iter()
        .skip(offset)
        .take(limit)
        .collect();
    
    Json(SearchResponse {
        chunks,
        total,
        offset,
        limit,
    })
}

/// Semantic search request
#[derive(Deserialize)]
pub struct SemanticSearchRequest {
    pub query: String,
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Semantic search response
#[derive(Serialize)]
pub struct SemanticSearchHit {
    pub chunk_id: String,
    pub score: f32,
}

pub async fn semantic_search(
    State(state): State<AppState>,
    Json(req): Json<SemanticSearchRequest>,
) -> Json<Vec<SemanticSearchHit>> {
    let limit = req.limit.unwrap_or(10);

    // Build candidates: for now, use chunk_id + metadata as textual content
    let store = state.store.read().await;
    let chunks = store.list().await;

    let mut candidates: std::collections::HashMap<String, cadi_llm::embeddings::Embedding> = std::collections::HashMap::new();

    for ch in &chunks {
        // Create a simple textual summary for embedding
        let summary = format!("{} size:{} type:{}", ch.chunk_id, ch.size, ch.content_type);
        // Acquire embedding manager and compute/get embedding
        let mut emb_mgr = state.embedding_manager.lock().await;
        match emb_mgr.get_chunk_embedding(&ch.chunk_id, &summary).await {
            Ok(emb) => {
                candidates.insert(ch.chunk_id.clone(), emb);
            }
            Err(e) => {
                tracing::debug!("Embedding failed for {}: {}", ch.chunk_id, e);
            }
        }
    }

    // Now compute search scores
    let mut emb_mgr = state.embedding_manager.lock().await;
    let results = emb_mgr.search(&req.query, &candidates, limit).await;

    // Persist current embedding store (best-effort)
    let _ = emb_mgr.save_store();

    match results {
        Ok(vec) => Json(vec.into_iter().map(|(id, score)| SemanticSearchHit { chunk_id: id, score }).collect()),
        Err(_) => Json(vec![]),
    }
}

/// Request for creating a virtual view
#[derive(Deserialize)]
pub struct ViewRequest {
    pub atoms: Vec<String>,
    #[serde(default)]
    pub expansion_depth: Option<usize>,
    #[serde(default)]
    pub max_tokens: Option<usize>,
}

/// Response for virtual view
#[derive(Serialize)]
pub struct ViewResponse {
    pub source: String,
    pub atoms: Vec<String>,
    pub ghost_atoms: Vec<String>,
    pub language: String,
    pub token_estimate: usize,
    pub explanation: String,
    pub truncated: bool,
}

/// Handler: create a virtual view from a list of atom/chunk IDs
pub async fn create_view_handler(
    State(state): State<AppState>,
    Json(req): Json<ViewRequest>,
) -> Result<Json<ViewResponse>, StatusCode> {
    let engine = cadi_core::rehydration::RehydrationEngine::new_arc(state.graph.clone());

    let view_res = if let Some(depth) = req.expansion_depth {
        engine.create_expanded_view(req.atoms.clone(), depth, req.max_tokens.unwrap_or(1024)).await
    } else {
        engine.create_view(req.atoms.clone(), cadi_core::rehydration::config::ViewConfig::default()).await
    };

    match view_res {
        Ok(v) => Ok(Json(ViewResponse {
            source: v.source,
            atoms: v.atoms,
            ghost_atoms: v.ghost_atoms,
            language: v.language,
            token_estimate: v.token_estimate,
            explanation: v.explanation,
            truncated: v.truncated,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ServerConfig;
    use axum::extract::State as AxState;

    #[tokio::test]
    async fn test_semantic_search_handler() {
        // Setup a temporary server config and state
        let tmp = tempfile::tempdir().unwrap();
        let config = ServerConfig {
            bind_address: "127.0.0.1:0".to_string(),
            storage_path: tmp.path().to_str().unwrap().to_string(),
            max_chunk_size: 1024 * 1024,
            anonymous_read: true,
            anonymous_write: true,
        };

        let state = AppState::new(config);

        // Store a chunk
        let chunk_id = "chunk:sha256:testchunk".to_string();
        let data = b"hello semantic".to_vec();
        {
            let mut s = state.store.write().await;
            s.store(chunk_id.clone(), data).await.unwrap();
        }

        // Call handler
        let req = SemanticSearchRequest { query: "hello".to_string(), limit: Some(10) };
        let res = semantic_search(AxState(state.clone()), axum::Json(req)).await;
        assert!(!res.0.is_empty(), "Expected at least one search hit");
        assert_eq!(res.0[0].chunk_id, chunk_id);
    }
}

