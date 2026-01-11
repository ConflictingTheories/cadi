//! HTTP route handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

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
    
    store.get(&chunk_id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)
}

/// Head chunk handler (check existence)
pub async fn head_chunk(
    State(state): State<AppState>,
    Path(chunk_id): Path<String>,
) -> StatusCode {
    let store = state.store.read().await;
    
    if store.exists(&chunk_id) {
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
    // Parse chunk ID
    let _expected_hash = chunk_id.strip_prefix("chunk:sha256:")
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    // Note: We don't verify the hash here because the chunk ID may be based on
    // the logical content (file hashes) rather than the envelope bytes.
    // A full implementation would parse the envelope and verify the internal hashes.
    
    // Check size limit
    if body.len() > state.config.max_chunk_size {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    
    let mut store = state.store.write().await;
    store.store(chunk_id.clone(), body.to_vec());
    
    Ok(Json(PutResponse {
        success: true,
        chunk_id: Some(chunk_id),
        message: None,
    }))
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
    
    if store.delete(&chunk_id) {
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
    
    store.get_meta(&chunk_id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// List chunks handler
pub async fn list_chunks(
    State(state): State<AppState>,
) -> Json<Vec<crate::state::ChunkMetadata>> {
    let store = state.store.read().await;
    Json(store.list().into_iter().cloned().collect())
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
    let all_chunks: Vec<_> = store.list().into_iter().cloned().collect();
    
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
