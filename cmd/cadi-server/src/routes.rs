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
        
        // Stats
        .route("/v1/stats", get(handlers::stats))
}
