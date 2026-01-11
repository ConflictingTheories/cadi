use axum::{
    routing::{get, post},
    extract::{Path, State},
    Json, Router,
};
use std::sync::{Arc, RwLock};
use todo_core::{Todo, TodoService};
use tower_http::cors::CorsLayer;

type SharedState = Arc<RwLock<TodoService>>;

#[tokio::main]
async fn main() {
    let service = TodoService::new();
    let state = Arc::new(RwLock::new(service));

    let app = Router::new()
        .route("/todos", get(list_todos).post(add_todo))
        .route("/todos/:id/complete", post(complete_todo))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Todo Server running on http://localhost:8080");
    axum::serve(listener, app).await.unwrap();
}

async fn list_todos(State(state): State<SharedState>) -> Json<Vec<Todo>> {
    let service = state.read().unwrap();
    Json(service.list().into_iter().cloned().collect())
}

async fn add_todo(
    State(state): State<SharedState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<Todo> {
    let mut service = state.write().unwrap();
    let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled").to_string();
    let description = payload.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    
    Json(service.add(title, description))
}

async fn complete_todo(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Json<bool> {
    let mut service = state.write().unwrap();
    Json(service.complete(&id).is_some())
}
