use axum::extract::State as AxState;
use axum::Json;
use crate::state::ServerConfig;
use cmd::cadi_server::handlers::{create_view_handler, ViewRequest};

#[tokio::test]
async fn test_create_view_handler_unit() {
    // Setup in-memory graph and insert two nodes similar to integration test
    let tmp = tempfile::tempdir().unwrap();
    let config = ServerConfig {
        bind_address: "127.0.0.1:0".to_string(),
        storage_path: tmp.path().to_str().unwrap().to_string(),
        max_chunk_size: 1024 * 1024,
        anonymous_read: true,
        anonymous_write: true,
    };

    let state = cmd::cadi_server::state::AppState::new(config);

    // Create nodes in the shared graph store
    let a_content = b"pub fn helper() -> i32 { 42 }";
    let b_content = b"pub fn use_helper() -> i32 { helper() }";

    let id_a = cadi_core::hash::chunk_id_from_content(a_content);
    let id_b = cadi_core::hash::chunk_id_from_content(b_content);

    let node_a = cadi_core::graph::GraphNode::new(id_a.clone(), cadi_core::hash::sha256_bytes(a_content))
        .with_language("rust").with_defines(vec!["helper".to_string()]);
    let mut node_b = cadi_core::graph::GraphNode::new(id_b.clone(), cadi_core::hash::sha256_bytes(b_content))
        .with_language("rust");
    node_b.with_references(vec!["helper".to_string()]);

    state.graph.insert_node(&node_a).unwrap();
    state.graph.store_content(&id_a, a_content).unwrap();

    state.graph.insert_node(&node_b).unwrap();
    state.graph.store_content(&id_b, b_content).unwrap();

    // add dependency b -> a
    state.graph.add_dependency(&id_b, &id_a, cadi_core::graph::EdgeType::Imports).unwrap();

    // Build request for view
    let req = ViewRequest { atoms: vec![id_b.clone()], expansion_depth: Some(1), max_tokens: Some(1024) };

    let result = create_view_handler(AxState(state.clone()), Json(req)).await;
    assert!(result.is_ok());
    let json = result.unwrap().0;
    assert!(json.source.contains("helper"));
    assert!(json.atoms.contains(&id_a));
    assert!(json.ghost_atoms.contains(&id_a));
}
