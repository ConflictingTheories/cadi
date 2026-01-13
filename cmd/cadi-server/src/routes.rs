//! Route definitions

use axum::{
    routing::{get, post, put, delete, head},
    Router,
};

use crate::handlers;
use crate::state::AppState;

/// API routes
pub fn api_routes() -> Router<AppState> {
    Router::new()
        // Health
        .route("/health", get(handlers::health))
        
        // Chunks API
        .route("/v1/chunks", get(handlers::list_chunks))
        .route("/v1/chunks/:chunk_id", get(handlers::get_chunk))
        .route("/v1/chunks/:chunk_id", head(handlers::head_chunk))
        .route("/v1/chunks/:chunk_id", put(handlers::put_chunk))
        .route("/v1/chunks/:chunk_id", delete(handlers::delete_chunk))
        .route("/v1/chunks/:chunk_id/meta", get(handlers::get_chunk_meta))
        
        // Search
        .route("/v1/search", post(handlers::search))
        // Semantic search
        .route("/v1/semantic_search", post(handlers::semantic_search))
        
        // Views (virtual view assembly)
        .route("/v1/views", post(handlers::create_view_handler))

        // Admin endpoints (add nodes/edges at runtime)
        .route("/v1/admin/nodes", post(handlers::admin_create_node))
        .route("/v1/admin/nodes/batch", post(handlers::admin_create_nodes_batch))
        .route("/v1/admin/edges", post(handlers::admin_add_edge))
        .route("/v1/admin/edges/batch", post(handlers::admin_add_edges_batch))

        // Stats
        .route("/v1/stats", get(handlers::stats))
}
