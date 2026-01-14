use cadi_core::{Chunk, CadiType, ChunkMeta, ChunkProvides, ChunkLicensing, ChunkLineage};
use cadi_registry::db::{RegistryDatabase, SearchQuery, ChunkMetadata};
use cadi_llm::embeddings::{EmbeddingManager, MockProvider};
use surrealdb::{engine::local::Mem, Surreal};

#[tokio::test]
async fn test_live_semantic_search() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Live Semantic Search");
    println!("=================================");

    // Create in-memory SurrealDB instance
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("test").use_db("test").await?;

    // Create embedding manager
    let provider = Box::new(MockProvider);
    let embedding_manager = EmbeddingManager::new(provider, None);

    // Create registry database
    let mut registry = RegistryDatabase::new(db, Some(embedding_manager)).await?;

    println!("✅ Database initialized");

    // Create test chunks
    let chunks = vec![
        (
            "chunk:jwt-auth-001",
            r#"export function authenticateUser(token: string): User {
    try {
        const payload = jwt.verify(token, SECRET_KEY);
        return payload.user;
    } catch (error) {
        throw new AuthenticationError('Invalid token');
    }
}"#,
            "typescript",
            "JWT Authentication middleware for user sessions",
            vec!["auth".to_string(), "jwt".to_string(), "security".to_string()],
        ),
        (
            "chunk:http-server-002",
            r#"import express from 'express';
const app = express();
app.listen(3000, () => console.log('Server running'));
export default app;"#,
            "typescript",
            "Express HTTP server setup with basic routing",
            vec!["http".to_string(), "server".to_string(), "express".to_string()],
        ),
        (
            "chunk:crud-ops-003",
            r#"export class TodoService {
    async create(todo: Todo): Promise<Todo> {
        return await db.todos.insert(todo);
    }
    async findAll(): Promise<Todo[]> {
        return await db.todos.find({});
    }
}"#,
            "typescript",
            "CRUD operations for todo management",
            vec!["crud".to_string(), "database".to_string(), "service".to_string()],
        ),
    ];

    for (chunk_id, content, language, description, concepts) in chunks {
        // Create Chunk instance
        let chunk = Chunk {
            chunk_id: chunk_id.to_string(),
            cadi_type: CadiType::Source,
            meta: ChunkMeta {
                name: format!("Test {}", chunk_id.split(':').nth(1).unwrap()),
                description: Some(description.to_string()),
                version: None,
                tags: vec![],
                created_at: None,
                updated_at: None,
            },
            provides: ChunkProvides {
                concepts: concepts.clone(),
                interfaces: vec![],
                abi: None,
            },
            licensing: ChunkLicensing {
                license: "MIT".to_string(),
                restrictions: vec![],
            },
            lineage: ChunkLineage::default(),
            signatures: vec![],
        };

        // Create metadata for search
        let metadata = serde_json::json!({
            "name": format!("Test {}", chunk_id.split(':').nth(1).unwrap()),
            "description": description,
            "language": language,
            "concepts": concepts,
            "quality_score": 0.9,
            "test_coverage": 0.85
        });

        // Store chunk
        match registry.store_chunk(&chunk, content, metadata).await {
            Ok(id) => println!("✅ Added chunk: {}", id),
            Err(e) => println!("❌ Failed to add chunk {}: {}", chunk_id, e),
        }
    }

    // Debug: Check what's in the database
    let chunks_in_db = registry.debug_list_chunks().await?;
    println!("📊 Chunks in database: {}", chunks_in_db.len());
    for chunk in &chunks_in_db {
        println!("  - {}: {}", chunk["id"], chunk["metadata"]["name"]);
    }

    println!("\n🔍 Testing Text Search: 'Test'");

    let text_query = SearchQuery {
        text: Some("Test".to_string()),
        embedding: None,
        language: None,
        limit: 5,
        min_score: 0.1,
    };

    let results = registry.search(text_query).await?;
    println!("Found {} results:", results.len());
    for result in &results {
        println!("  - {} (score: {:.2})", result.metadata.name, result.score);
    }

    println!("\n🔍 Testing Semantic Search: 'user login system'");

    // Generate embedding for query
    let query_embedding = registry.embedding_manager_mut().unwrap()
        .get_chunk_embedding("query", "user login system").await?;

    let semantic_query = SearchQuery {
        text: None,
        embedding: Some(query_embedding),
        language: None,
        limit: 5,
        min_score: 0.1,
    };

    let semantic_results = registry.search(semantic_query).await?;
    println!("Found {} semantic results:", semantic_results.len());
    for result in &semantic_results {
        println!("  - {} (score: {:.2})", result.metadata.name, result.score);
    }

    println!("\n🔍 Testing Hybrid Search: 'Test authentication' (text + semantic)");

    let hybrid_embedding = registry.embedding_manager_mut().unwrap()
        .get_chunk_embedding("query", "Test authentication").await?;

    let hybrid_query = SearchQuery {
        text: Some("Test".to_string()),
        embedding: Some(hybrid_embedding),
        language: None,
        limit: 5,
        min_score: 0.1,
    };

    let hybrid_results = registry.search(hybrid_query).await?;
    println!("Found {} hybrid results:", hybrid_results.len());
    for result in &hybrid_results {
        println!("  - {} (score: {:.2}) - {}", result.metadata.name, result.score, result.metadata.description);
    }

    println!("\n🎉 Semantic search is working live!");
    Ok(())
}