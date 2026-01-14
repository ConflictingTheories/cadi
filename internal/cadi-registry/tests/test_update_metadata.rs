use cadi_core::{Chunk, CadiType, ChunkMeta, ChunkProvides, ChunkLicensing, ChunkLineage};
use cadi_registry::db::RegistryDatabase;
use cadi_llm::embeddings::MockProvider;
use cadi_llm::embeddings::EmbeddingManager;
use surrealdb::{engine::local::Mem, Surreal};

#[tokio::test]
async fn test_update_chunk_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("test").use_db("test").await?;

    let provider = Box::new(MockProvider);
    let embedding_manager = EmbeddingManager::new(provider, None);
    let mut registry = RegistryDatabase::new(db, Some(embedding_manager)).await?;

    let chunk_id = "chunk:update-meta-001".to_string();
    let chunk = Chunk {
        chunk_id: chunk_id.clone(),
        cadi_type: CadiType::Source,
        meta: ChunkMeta {
            name: "Initial".to_string(),
            description: Some("Initial description".to_string()),
            version: None,
            tags: vec![],
            created_at: None,
            updated_at: None,
        },
        provides: ChunkProvides { concepts: vec![], interfaces: vec![], abi: None },
        licensing: ChunkLicensing { license: "MIT".to_string(), restrictions: vec![] },
        lineage: ChunkLineage::default(),
        signatures: vec![],
    };

    let metadata = serde_json::json!({
        "name": "Initial",
        "description": "Initial description",
        "language": "typescript",
        "concepts": [],
        "quality_score": 0.5,
        "test_coverage": 0.5
    });

    registry.store_chunk(&chunk, "console.log('hello')", metadata).await?;

    // Update metadata
    let new_meta = serde_json::json!({
        "name": "Updated Name",
        "description": "Now with better description",
        "language": "typescript",
        "concepts": ["refactor"],
        "quality_score": 0.95,
        "test_coverage": 0.92
    });

    registry.update_chunk_metadata(&chunk_id, new_meta.clone()).await?;

    let rec = registry.get_chunk(&chunk_id).await?.expect("Chunk must exist");
    assert_eq!(rec.metadata.name, "Updated Name");
    assert_eq!(rec.metadata.description, "Now with better description");
    assert_eq!(rec.metadata.quality_score, 0.95);

    Ok(())
}
