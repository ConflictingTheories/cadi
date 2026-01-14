use cadi_core::graph::{ChunkNode, GraphEdgeType};
use cadi_registry::graph::GraphDB;
use cadi_builder::dependency_resolver::DependencyResolver;
use std::sync::Arc;

async fn setup_test_graph() -> GraphDB {
    let graph = GraphDB::new("mem://", "test", "test").await.unwrap();
    graph.init_schema().await.unwrap();
    graph
}

async fn setup_test_resolver() -> (Arc<GraphDB>, DependencyResolver) {
    let graph_db = Arc::new(setup_test_graph().await);
    let resolver = DependencyResolver::new(graph_db.clone());
    (graph_db, resolver)
}

async fn setup_dependency_chain(graph: &GraphDB) {
    let jwt_chunk = ChunkNode {
        id: "jwt".to_string(),
        semantic_hash: "hash3".to_string(),
        name: "JWT".to_string(),
        description: "".to_string(),
        language: "".to_string(),
        concepts: vec![],
        interface: None,
        created_at: "".to_string(),
        usage_count: 0,
        quality_score: 0.0,
    };
    let crud_chunk = ChunkNode {
        id: "crud".to_string(),
        semantic_hash: "hash2".to_string(),
        name: "CRUD".to_string(),
        description: "".to_string(),
        language: "".to_string(),
        concepts: vec![],
        interface: None,
        created_at: "".to_string(),
        usage_count: 0,
        quality_score: 0.0,
    };
    let blog_chunk = ChunkNode {
        id: "blog".to_string(),
        semantic_hash: "hash1".to_string(),
        name: "Blog".to_string(),
        description: "".to_string(),
        language: "".to_string(),
        concepts: vec![],
        interface: None,
        created_at: "".to_string(),
        usage_count: 0,
        quality_score: 0.0,
    };

    graph.create_chunk_node("jwt", &jwt_chunk).await.unwrap();
    graph.create_chunk_node("crud", &crud_chunk).await.unwrap();
    graph.create_chunk_node("blog", &blog_chunk).await.unwrap();

    graph.create_edge("crud", "jwt", GraphEdgeType::DEPENDS_ON, None).await.unwrap();
    graph.create_edge("blog", "crud", GraphEdgeType::DEPENDS_ON, None).await.unwrap();
}

async fn setup_cyclic_dependencies(graph: &GraphDB) {
    let chunk_a = ChunkNode { id: "a".to_string(), semantic_hash: "hash_a".to_string(), name: "A".to_string(), description: "".to_string(), language: "".to_string(), concepts: vec![], interface: None, created_at: "".to_string(), usage_count: 0, quality_score: 0.0 };
    let chunk_b = ChunkNode { id: "b".to_string(), semantic_hash: "hash_b".to_string(), name: "B".to_string(), description: "".to_string(), language: "".to_string(), concepts: vec![], interface: None, created_at: "".to_string(), usage_count: 0, quality_score: 0.0 };
    let chunk_c = ChunkNode { id: "c".to_string(), semantic_hash: "hash_c".to_string(), name: "C".to_string(), description: "".to_string(), language: "".to_string(), concepts: vec![], interface: None, created_at: "".to_string(), usage_count: 0, quality_score: 0.0 };

    graph.create_chunk_node("a", &chunk_a).await.unwrap();
    graph.create_chunk_node("b", &chunk_b).await.unwrap();
    graph.create_chunk_node("c", &chunk_c).await.unwrap();

    graph.create_edge("a", "b", GraphEdgeType::DEPENDS_ON, None).await.unwrap();
    graph.create_edge("b", "c", GraphEdgeType::DEPENDS_ON, None).await.unwrap();
    graph.create_edge("c", "a", GraphEdgeType::DEPENDS_ON, None).await.unwrap();
}


#[tokio::test]
async fn test_graph_creation_and_traversal() {
    let graph = setup_test_graph().await;
    
    // Create nodes
    let jwt_chunk = ChunkNode {
        id: "jwt_auth".to_string(),
        semantic_hash: "hash1".to_string(),
        name: "JWT Authenticator".to_string(),
        description: "".to_string(),
        language: "".to_string(),
        concepts: vec![],
        interface: None,
        created_at: "".to_string(),
        usage_count: 0,
        quality_score: 0.0,
    };
    
    let crud_chunk = ChunkNode {
        id: "crud_ops".to_string(),
        semantic_hash: "hash2".to_string(),
        name: "CRUD Operations".to_string(),
        description: "".to_string(),
        language: "".to_string(),
        concepts: vec![],
        interface: None,
        created_at: "".to_string(),
        usage_count: 0,
        quality_score: 0.0,
    };
    
    graph.create_chunk_node("jwt_auth", &jwt_chunk).await.unwrap();
    graph.create_chunk_node("crud_ops", &crud_chunk).await.unwrap();
    
    // Create edge: CRUD depends on JWT
    graph.create_edge(
        "crud_ops",
        "jwt_auth",
        GraphEdgeType::DEPENDS_ON,
        None,
    ).await.unwrap();
    
    // Query dependencies
    let deps = graph.get_dependencies("crud_ops").await.unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].dependency_id, "jwt_auth");
}

#[tokio::test]
async fn test_transitive_dependency_resolution() {
    let (graph_db, resolver) = setup_test_resolver().await;
    
    // Create dependency chain: blog -> crud -> jwt
    setup_dependency_chain(&graph_db).await;
    
    // Resolve all dependencies from "blog"
    let result = resolver.resolve_all_dependencies("blog").await.unwrap();
    
    assert!(result.all_dependencies.contains("crud"));
    assert!(result.all_dependencies.contains("jwt"));
}

#[tokio::test]
async fn test_cycle_detection() {
    let (graph_db, resolver) = setup_test_resolver().await;
    setup_cyclic_dependencies(&graph_db).await;
    let result = resolver.validate_composition(&["a".to_string(), "b".to_string(), "c".to_string()]).await.unwrap();
    
    assert!(!result.is_valid);
    assert!(!result.issues.is_empty());
}