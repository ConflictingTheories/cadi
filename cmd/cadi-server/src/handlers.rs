//! HTTP route handlers

use axum::{
    extract::{Path, State, Query},
    http::{HeaderMap, StatusCode},
    body::Bytes,
    Json,
};
use serde::{Deserialize, Serialize};
use cadi_core::Chunk;

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
    body: Bytes,
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
    
    // Try to store in registry database first
    // For now, we'll store a simple chunk with the data as content
    // In a real implementation, we'd parse the chunk format
    let content = String::from_utf8_lossy(&body);
    let chunk = Chunk {
        chunk_id: chunk_id.clone(),
        cadi_type: cadi_core::CadiType::Source,
        meta: cadi_core::ChunkMeta {
            name: chunk_id.clone(),
            description: Some(format!("Chunk {}", chunk_id)),
            version: None,
            tags: vec![],
            created_at: None,
            updated_at: None,
        },
        provides: cadi_core::ChunkProvides {
            concepts: vec![],
            interfaces: vec![],
            abi: None,
        },
        licensing: cadi_core::ChunkLicensing {
            license: "MIT".to_string(),
            restrictions: vec![],
        },
        lineage: cadi_core::ChunkLineage::default(),
        signatures: vec![],
    };
    
    // Create metadata for search
    let metadata = serde_json::json!({
        "name": chunk_id,
        "description": format!("Chunk {}", chunk_id),
        "language": "unknown",
        "concepts": [],
        "quality_score": 0.9,
        "test_coverage": 0.85
    });
    
    match state.registry_db.write().await.store_chunk(&chunk, &content, metadata).await {
        Ok(_) => Ok(Json(PutResponse {
            success: true,
            chunk_id: Some(chunk_id),
            message: None,
        })),
        Err(e) => {
            // Fallback to file store
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
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<Vec<crate::state::ChunkMetadata>> {
    // Try registry database first
    match state.registry_db.read().await.debug_list_chunks().await {
        Ok(chunks) => {
            let filtered: Vec<_> = if let Some(q) = params.get("name") {
                chunks.into_iter()
                    .filter(|c| {
                        // Check both id and metadata.name for the query
                        let id_match = c.get("id")
                            .and_then(|i| i.as_str())
                            .map(|s| s.contains(q))
                            .unwrap_or(false);
                        let name_match = c.get("name")
                            .and_then(|n| n.as_str())
                            .map(|s| s.contains(q))
                            .unwrap_or(false);
                        id_match || name_match
                    })
                    .map(|c| {
                        let id = c.get("id").and_then(|i| i.as_str()).unwrap_or("unknown");
                        let name = c.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                        let description = c.get("description").and_then(|d| d.as_str()).unwrap_or("");
                        crate::state::ChunkMetadata {
                            chunk_id: format!("{}:{}", id, name),
                            size: description.len(),
                            created_at: chrono::Utc::now().to_rfc3339(),
                            content_type: "application/json".to_string(),
                        }
                    })
                    .collect()
            } else {
                chunks.into_iter()
                    .map(|c| {
                        let id = c.get("id").and_then(|i| i.as_str()).unwrap_or("unknown");
                        let name = c.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                        let description = c.get("description").and_then(|d| d.as_str()).unwrap_or("");
                        crate::state::ChunkMetadata {
                            chunk_id: format!("{}:{}", id, name),
                            size: description.len(),
                            created_at: chrono::Utc::now().to_rfc3339(),
                            content_type: "application/json".to_string(),
                        }
                    })
                    .collect()
            };
            Json(filtered)
        }
        Err(_) => {
            // Fallback to file store
            let store = state.store.read().await;
            let all_chunks = store.list().await;
            
            let filtered: Vec<_> = if let Some(q) = params.get("name") {
                all_chunks.into_iter()
                    .filter(|c| c.chunk_id.contains(q))
                    .collect()
            } else {
                all_chunks
            };
            
            Json(filtered)
        }
    }
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

    // Generate embedding for the query
    let provider = Box::new(cadi_llm::embeddings::MockProvider::default());
    let mut emb_manager = cadi_llm::embeddings::EmbeddingManager::new(provider, None);
    let embedding = match emb_manager.get_chunk_embedding("query", &req.query).await {
        Ok(emb) => emb,
        Err(_) => return Json(vec![]),
    };

    // Use registry database for semantic search
    let query = cadi_registry::db::SearchQuery {
        text: None,
        embedding: Some(embedding.clone()),
        language: None,
        limit,
        min_score: 0.0,
    };

    match state.registry_db.read().await.search(query).await {
        Ok(results) => {
            Json(results.into_iter().map(|r| SemanticSearchHit {
                chunk_id: r.chunk_id,
                score: r.score as f32,
            }).collect())
        }
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

/// Internal: check whether a request is authorized for admin operations
fn is_authorized(state: &AppState, headers: &HeaderMap) -> bool {
    // Allow if anonymous writes are enabled (dev/test convenience)
    if state.config.anonymous_write {
        return true;
    }

    // If admin token is configured, accept either Authorization: Bearer <token>
    // or X-Admin-Token: <token>
    if let Some(ref token) = state.config.admin_token {
        if let Some(hv) = headers.get("authorization") {
            if let Ok(s) = hv.to_str() {
                if let Some(rest) = s.strip_prefix("Bearer ") {
                    return rest == token;
                }
            }
        }
        if let Some(hv) = headers.get("x-admin-token") {
            if let Ok(s) = hv.to_str() {
                return s == token;
            }
        }
    }

    false
}

/// Helper: create a node from JSON payload
fn create_node_from_payload(state: &AppState, payload: &serde_json::Value) -> Result<String, StatusCode> {
    let content = payload.get("content").and_then(|v| v.as_str()).ok_or(StatusCode::BAD_REQUEST)?;

    let chunk_id = if let Some(val) = payload.get("chunk_id").and_then(|v| v.as_str()) {
        val.to_string()
    } else {
        cadi_core::hash::chunk_id_from_content(content.as_bytes())
    };

    if let Some(provided) = payload.get("chunk_id").and_then(|v| v.as_str()) {
        if !cadi_core::hash::verify_chunk_content(provided, content.as_bytes()) {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let content_hash = cadi_core::hash::parse_chunk_id(&chunk_id).ok_or(StatusCode::BAD_REQUEST)?;

    let mut node = cadi_core::graph::GraphNode::new(chunk_id.clone(), content_hash)
        .with_language(payload.get("language").and_then(|v| v.as_str()).unwrap_or("unknown"))
        .with_size(content.as_bytes().len());

    if let Some(defs) = payload.get("defines").and_then(|v| v.as_array()) {
        let defs_str: Vec<String> = defs.iter().filter_map(|d| d.as_str().map(|s| s.to_string())).collect();
        node = node.with_defines(defs_str);
    }

    if let Some(refs) = payload.get("references").and_then(|v| v.as_array()) {
        let refs_str: Vec<String> = refs.iter().filter_map(|d| d.as_str().map(|s| s.to_string())).collect();
        node = node.with_references(refs_str);
    }

    if let Some(alias) = payload.get("alias").and_then(|v| v.as_str()) {
        node = node.with_alias(alias);
    }

    state.graph.store_content(&chunk_id, content.as_bytes()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.graph.insert_node(&node).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(chunk_id)
}

/// Admin: list all graph nodes
pub async fn admin_list_nodes(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    if !is_authorized(&state, &headers) {
        return Err(StatusCode::FORBIDDEN);
    }

    match state.graph.list_nodes() {
        Ok(nodes) => {
            let node_json: Vec<serde_json::Value> = nodes.into_iter().map(|node| {
                serde_json::json!({
                    "chunk_id": node.chunk_id,
                    "language": node.language,
                    "size": node.byte_size,
                    "defines": node.symbols_defined,
                    "references": node.symbols_referenced
                })
            }).collect();
            Ok(Json(node_json))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Admin: list all graph edges
pub async fn admin_list_edges(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    if !is_authorized(&state, &headers) {
        return Err(StatusCode::FORBIDDEN);
    }

    match state.graph.list_edges() {
        Ok(edges) => {
            let edge_json: Vec<serde_json::Value> = edges.into_iter().map(|(from, to, edge_type)| {
                serde_json::json!({
                    "from": from,
                    "to": to,
                    "edge_type": edge_type
                })
            }).collect();
            Ok(Json(edge_json))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Admin: create a graph node at runtime (for tests and ingestion)
pub async fn admin_create_node(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    if !is_authorized(&state, &headers) {
        return Err(StatusCode::FORBIDDEN);
    }

    let _ = create_node_from_payload(&state, &payload)?;

    Ok(StatusCode::CREATED)
}

/// Admin: create multiple nodes in a batch
pub async fn admin_create_nodes_batch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<Vec<serde_json::Value>>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    if !is_authorized(&state, &headers) {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut results: Vec<serde_json::Value> = Vec::new();
    for item in payload.iter() {
        match create_node_from_payload(&state, item) {
            Ok(id) => results.push(serde_json::json!({"chunk_id": id, "status": 201})),
            Err(code) => results.push(serde_json::json!({"status": code.as_u16()})),
        }
    }

    Ok(Json(results))
}

/// Helper: add an edge from payload
fn add_edge_from_payload(state: &AppState, payload: &serde_json::Value) -> Result<(), StatusCode> {
    let source = payload.get("source").and_then(|v| v.as_str()).ok_or(StatusCode::BAD_REQUEST)?;
    let target = payload.get("target").and_then(|v| v.as_str()).ok_or(StatusCode::BAD_REQUEST)?;
    let edge_type = payload.get("edge_type").and_then(|v| v.as_str()).unwrap_or("imports");

    let et = match edge_type {
        "imports" => cadi_core::graph::EdgeType::Imports,
        "type_ref" | "type-ref" => cadi_core::graph::EdgeType::TypeRef,
        "calls" => cadi_core::graph::EdgeType::Calls,
        "composed_of" => cadi_core::graph::EdgeType::ComposedOf,
        "implements" => cadi_core::graph::EdgeType::Implements,
        "extends" => cadi_core::graph::EdgeType::Extends,
        "exports" => cadi_core::graph::EdgeType::Exports,
        "generic_ref" => cadi_core::graph::EdgeType::GenericRef,
        "macro_use" => cadi_core::graph::EdgeType::MacroUse,
        "tests" => cadi_core::graph::EdgeType::Tests,
        "doc_ref" => cadi_core::graph::EdgeType::DocRef,
        _ => cadi_core::graph::EdgeType::Imports,
    };

    state.graph.add_dependency(source, target, et).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Admin: add an edge between two nodes
pub async fn admin_add_edge(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    if !is_authorized(&state, &headers) {
        return Err(StatusCode::FORBIDDEN);
    }

    add_edge_from_payload(&state, &payload)?;

    Ok(StatusCode::CREATED)
}

/// Admin: add multiple edges in a batch
pub async fn admin_add_edges_batch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<Vec<serde_json::Value>>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    if !is_authorized(&state, &headers) {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut results: Vec<serde_json::Value> = Vec::new();
    for item in payload.iter() {
        match add_edge_from_payload(&state, item) {
            Ok(_) => results.push(serde_json::json!({"status": 201})),
            Err(code) => results.push(serde_json::json!({"status": code.as_u16()})),
        }
    }

    Ok(Json(results))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ServerConfig;
    use axum::extract::State as AxState;
    use axum::http::HeaderValue;

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
            admin_token: None,
        };

        let state = AppState::new(config.clone());

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

    #[tokio::test]
    async fn test_admin_create_node_and_edge_handler() {
        let tmp = tempfile::tempdir().unwrap();
        let config = ServerConfig {
            bind_address: "127.0.0.1:0".to_string(),
            storage_path: tmp.path().to_str().unwrap().to_string(),
            max_chunk_size: 1024 * 1024,
            anonymous_read: true,
            anonymous_write: true,
            admin_token: None,
        };

        let state = AppState::new(config.clone());

        // Prepare node A
        let a_content = "pub fn helper() -> i32 { 42 }".to_string();
        let id_a = cadi_core::hash::chunk_id_from_content(a_content.as_bytes());
        let new_node = serde_json::json!({
            "chunk_id": id_a,
            "content": a_content,
            "language": "rust",
            "defines": ["helper"]
        });

        // Call admin_create_node (dev allows anonymous writes in this test)
        let payload = serde_json::to_vec(&new_node).unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_str("Bearer dev").unwrap());
        let res = admin_create_node(AxState(state.clone()), headers, axum::Json(new_node.clone())).await;
        assert!(res.is_ok());

        // Prepare node B referencing helper
        let b_content = "pub fn use_helper() -> i32 { helper() }".to_string();
        let id_b = cadi_core::hash::chunk_id_from_content(b_content.as_bytes());
        let new_node_b = serde_json::json!({
            "chunk_id": id_b,
            "content": b_content,
            "language": "rust",
            "references": ["helper"]
        });
        let payload_b = serde_json::to_vec(&new_node_b).unwrap();
        let mut headers_b = HeaderMap::new();
        headers_b.insert("authorization", HeaderValue::from_str("Bearer dev").unwrap());
        let resb = admin_create_node(AxState(state.clone()), headers_b, axum::Json(new_node_b)).await;
        assert!(resb.is_ok());

        // Add edge B -> A
        let edge_body = serde_json::json!({ "source": id_b, "target": id_a, "edge_type": "imports" });
        let edge_payload = serde_json::to_vec(&edge_body).unwrap();
        let mut headers_e = HeaderMap::new();
        headers_e.insert("authorization", HeaderValue::from_str("Bearer dev").unwrap());
        let edge_res = admin_add_edge(AxState(state.clone()), headers_e, axum::Json(edge_body)).await;
        assert!(edge_res.is_ok());

        // Now create a view for B and ensure A appears as ghost import
        let view_req = ViewRequest { atoms: vec![id_b.clone()], expansion_depth: Some(1), max_tokens: Some(1024) };
        let view = create_view_handler(AxState(state.clone()), axum::Json(view_req)).await.expect("view failed");
        let json = view.0;
        assert!(json.atoms.contains(&id_a));
        assert!(json.ghost_atoms.contains(&id_a));
    }

    #[tokio::test]
    async fn test_admin_auth_required() {
        let tmp = tempfile::tempdir().unwrap();
        let mut config = ServerConfig::default();
        config.storage_path = tmp.path().to_str().unwrap().to_string();
        config.anonymous_read = true;
        config.anonymous_write = false;
        config.admin_token = Some("secret-token".to_string());

        let state = AppState::new(config.clone());

        // Attempt to call admin endpoint without header
        let a_content = "pub fn helper() -> i32 { 42 }".to_string();
        let id_a = cadi_core::hash::chunk_id_from_content(a_content.as_bytes());
        let new_node = serde_json::json!({
            "chunk_id": id_a,
            "content": a_content,
            "language": "rust",
            "defines": ["helper"]
        });

        let mut headers_none = HeaderMap::new();
        let res = admin_create_node(AxState(state.clone()), headers_none, axum::Json(new_node.clone())).await;
        assert!(res.is_err());

        // Call with Authorization header
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_str("Bearer secret-token").unwrap());
        let res_ok = admin_create_node(AxState(state.clone()), headers, axum::Json(new_node)).await;
        assert!(res_ok.is_ok());
    }
}

