//! CADI Registry Database Layer
//!
//! Provides SurrealDB integration for:
//! - Vector search with MTREE indexing
//! - Chunk storage and retrieval
//! - Metadata normalization and indexing

use cadi_core::{CadiError, CadiResult, Chunk};
use cadi_llm::embeddings::EmbeddingManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

/// Database schema for chunks with vector search
#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkRecord {
    pub id: String,
    pub hash: String,
    pub content: String,
    pub language: String,
    pub metadata: ChunkMetadata,
    pub embedding: Option<Vec<f32>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub usage_count: u64,
}

/// Normalized metadata for search indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub name: String,
    pub description: String,
    pub language: String,
    pub concepts: Vec<String>,
    pub dependencies: Vec<String>,
    pub function_signatures: Vec<String>,
    pub quality_score: f64,
    pub test_coverage: f64,
}

/// Search query structure
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub embedding: Option<Vec<f32>>,
    pub language: Option<String>,
    pub limit: usize,
    pub min_score: f64,
}

/// Search result from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbSearchResult {
    pub chunk_id: String,
    pub score: f64,
    pub metadata: ChunkMetadata,
}

/// Registry database manager
pub struct RegistryDatabase {
    db: Surreal<Db>,
    embedding_manager: Option<EmbeddingManager>,
}

impl RegistryDatabase {
    /// Create a new registry database
    pub async fn new(db: Surreal<Db>, embedding_manager: Option<EmbeddingManager>) -> CadiResult<Self> {
        // Initialize schema
        Self::init_schema(&db).await?;

        Ok(Self { db, embedding_manager })
    }

    /// Initialize database schema
    async fn init_schema(db: &Surreal<Db>) -> CadiResult<()> {
        // Define chunk table with vector indexing
        let schema = r#"
            DEFINE TABLE chunk SCHEMAFULL;

            DEFINE FIELD id ON chunk;
            DEFINE FIELD hash ON chunk TYPE string;
            DEFINE FIELD content ON chunk TYPE string;
            DEFINE FIELD language ON chunk TYPE string;
            DEFINE FIELD name ON chunk TYPE string;
            DEFINE FIELD description ON chunk TYPE string;
            DEFINE FIELD concepts ON chunk TYPE array;
            DEFINE FIELD quality_score ON chunk TYPE float;
            DEFINE FIELD test_coverage ON chunk TYPE float;
            DEFINE FIELD embedding ON chunk TYPE array;
            DEFINE FIELD created_at ON chunk TYPE string DEFAULT time::now();
            DEFINE FIELD usage_count ON chunk TYPE int DEFAULT 0;

            -- Vector index for semantic search
            DEFINE INDEX embedding_mtree ON chunk FIELDS embedding MTREE DIMENSION 5 DIST EUCLIDEAN;

            -- Text indexes for metadata search
            DEFINE INDEX metadata_name ON chunk FIELDS metadata.name;
            DEFINE INDEX metadata_concepts ON chunk FIELDS metadata.concepts;
            DEFINE INDEX metadata_language ON chunk FIELDS language;
        "#;

        db.query(schema).await.map_err(|e| CadiError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Store a chunk with metadata and embedding
    pub async fn store_chunk(&mut self, chunk: &Chunk, content: &str, metadata: serde_json::Value) -> CadiResult<String> {
        let chunk_id = chunk.chunk_id.clone();
        let hash = chunk.chunk_id.strip_prefix("chunk:sha256:").unwrap_or(&chunk.chunk_id).to_string();

        // Generate embedding if manager available
        let embedding = if let Some(ref mut manager) = self.embedding_manager {
            // Create content for embedding from metadata
            let name = metadata.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let description = metadata.get("description").and_then(|d| d.as_str()).unwrap_or("");
            let concepts = metadata.get("concepts")
                .and_then(|c| c.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(" "))
                .unwrap_or_default();
            let content_for_embedding = format!("{} {} {}", name, description, concepts);
            match manager.get_chunk_embedding(&chunk_id, &content_for_embedding).await {
                Ok(embedding) => Some(embedding),
                Err(e) => return Err(CadiError::RegistryError(format!("Embedding generation failed: {}", e))),
            }
        } else {
            None
        };

        // Store in database
        let sql = r#"
            CREATE chunk SET
                id = $id,
                hash = $hash,
                content = $content,
                language = $language,
                name = $name,
                description = $description,
                concepts = $concepts,
                quality_score = $quality_score,
                test_coverage = $test_coverage,
                embedding = $embedding,
                created_at = time::now()
        "#;

        println!("DEBUG: Storing chunk {} with SQL", chunk_id);
        let result = self.db.query(sql)
            .bind(("id", &chunk_id))
            .bind(("hash", &hash))
            .bind(("content", content))
            .bind(("language", "typescript"))
            .bind(("name", metadata.get("name").and_then(|n| n.as_str()).unwrap_or("")))
            .bind(("description", metadata.get("description").and_then(|d| d.as_str()).unwrap_or("")))
            .bind(("concepts", metadata.get("concepts").and_then(|c| c.as_array()).unwrap_or(&vec![])))
            .bind(("quality_score", metadata.get("quality_score").and_then(|q| q.as_f64()).unwrap_or(0.0)))
            .bind(("test_coverage", metadata.get("test_coverage").and_then(|t| t.as_f64()).unwrap_or(0.0)))
            .bind(("embedding", &embedding))
            .await
            .map_err(|e| CadiError::DatabaseError(format!("Store query failed: {}", e)))?;

        println!("DEBUG: Store result: {:?}", result);
        Ok(chunk_id)
    }

    /// Retrieve a chunk by ID
    pub async fn get_chunk(&self, chunk_id: &str) -> CadiResult<Option<ChunkRecord>> {
        let sql = r#"
            SELECT * FROM chunk WHERE id = $chunk_id
        "#;

        let mut response = self.db.query(sql)
            .bind(("chunk_id", chunk_id))
            .await
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;

        let results: Vec<ChunkRecord> = response.take(0)
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;

        Ok(results.into_iter().next())
    }

    /// Perform hybrid search
    pub async fn search(&self, query: SearchQuery) -> CadiResult<Vec<DbSearchResult>> {
        let mut text_results = Vec::new();
        let mut semantic_results = Vec::new();

        // Text-based search
        if let Some(text) = &query.text {
            text_results = self.text_search(text, query.limit * 2).await?;
        }

        // Semantic search
        if let Some(embedding) = &query.embedding {
            semantic_results = self.semantic_search(embedding, query.limit * 2).await?;
        }

        // Combine results with weighted scoring
        let mut combined = HashMap::new();
        
        // Add text results
        for result in text_results {
            combined.entry(result.chunk_id.clone())
                .or_insert_with(|| DbSearchResult {
                    chunk_id: result.chunk_id.clone(),
                    score: 0.0,
                    metadata: result.metadata.clone(),
                })
                .score += result.score * 0.3; // 30% weight for text
        }

        // Add semantic results
        for result in semantic_results {
            let entry = combined.entry(result.chunk_id.clone())
                .or_insert_with(|| DbSearchResult {
                    chunk_id: result.chunk_id.clone(),
                    score: 0.0,
                    metadata: result.metadata.clone(),
                });
            entry.score += result.score * 0.5; // 50% weight for semantic
        }

        // Add quality/usage boost to all results
        for result in combined.values_mut() {
            let quality_boost = (result.metadata.quality_score * 0.5 + 
                               result.metadata.test_coverage.min(1.0) * 0.5) * 0.2;
            result.score += quality_boost;
        }

        let mut final_results: Vec<_> = combined.into_values().collect();

        // Filter by minimum score and language
        final_results.retain(|r| r.score >= query.min_score);
        if let Some(lang) = &query.language {
            final_results.retain(|r| r.metadata.language == *lang);
        }

        // Sort by score and limit
        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        final_results.truncate(query.limit);

        Ok(final_results)
    }

    /// Text-based search using BM25-like scoring
    async fn text_search(&self, query: &str, limit: usize) -> CadiResult<Vec<DbSearchResult>> {
        let sql = r#"
            SELECT
                id,
                name,
                description,
                concepts,
                quality_score,
                test_coverage,
                (
                    string::similarity::fuzzy(name, $query) * 0.4 +
                    string::similarity::fuzzy(description, $query) * 0.6
                ) AS score
            FROM chunk
            WHERE score > 0.0
            ORDER BY score DESC
            LIMIT $limit
        "#;

        let mut response = self.db.query(sql)
            .bind(("query", query))
            .bind(("limit", limit))
            .await
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;

        let results: Vec<serde_json::Value> = response.take(0)
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;

        let search_results = results.into_iter()
            .filter_map(|row| {
                // Extract id from the complex structure
                let chunk_id = row.get("id")?
                    .get("id")?
                    .get("String")?
                    .as_str()?
                    .to_string();
                
                let score = row.get("score")?.as_f64()?;
                
                let name = row.get("name")?.as_str()?.to_string();
                let description = row.get("description")?.as_str()?.to_string();
                let concepts = row.get("concepts")?
                    .as_array()?
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                let quality_score = row.get("quality_score")?.as_f64()?;
                let test_coverage = row.get("test_coverage")?.as_f64()?;
                
                let metadata = ChunkMetadata {
                    name,
                    description,
                    language: "typescript".to_string(), // TODO: store language separately
                    concepts,
                    dependencies: vec![],
                    function_signatures: vec![],
                    quality_score,
                    test_coverage,
                };

                Some(DbSearchResult {
                    chunk_id,
                    score,
                    metadata,
                })
            })
            .collect();

        Ok(search_results)
    }

    /// Semantic search using vector similarity
    async fn semantic_search(&self, query_embedding: &[f32], limit: usize) -> CadiResult<Vec<DbSearchResult>> {
        let sql = r#"
            SELECT
                id,
                name,
                description,
                concepts,
                quality_score,
                test_coverage,
                vector::similarity::cosine(embedding, $query_embedding) AS score
            FROM chunk
            WHERE embedding IS NOT NULL
            ORDER BY score DESC
            LIMIT $limit
        "#;

        let mut response = self.db.query(sql)
            .bind(("query_embedding", query_embedding))
            .bind(("limit", limit))
            .await
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;

        let results: Vec<serde_json::Value> = response.take(0)
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;

        let search_results = results.into_iter()
            .filter_map(|row| {
                // Extract id from the complex structure
                let chunk_id = row.get("id")?
                    .get("id")?
                    .get("String")?
                    .as_str()?
                    .to_string();
                
                let score = row.get("score")?.as_f64()?;
                
                let name = row.get("name")?.as_str()?.to_string();
                let description = row.get("description")?.as_str()?.to_string();
                let concepts = row.get("concepts")?
                    .as_array()?
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                let quality_score = row.get("quality_score")?.as_f64()?;
                let test_coverage = row.get("test_coverage")?.as_f64()?;
                
                let metadata = ChunkMetadata {
                    name,
                    description,
                    language: "typescript".to_string(), // TODO: store language separately
                    concepts,
                    dependencies: vec![],
                    function_signatures: vec![],
                    quality_score,
                    test_coverage,
                };

                Some(DbSearchResult {
                    chunk_id,
                    score,
                    metadata,
                })
            })
            .collect();

        Ok(search_results)
    }

    /// Increment usage count for a chunk
    pub async fn increment_usage(&self, chunk_id: &str) -> CadiResult<()> {
        let sql = r#"
            UPDATE chunk SET usage_count += 1 WHERE id = $chunk_id
        "#;

        self.db.query(sql)
            .bind(("chunk_id", chunk_id))
            .await
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get embedding manager (for testing)
    pub fn embedding_manager_mut(&mut self) -> Option<&mut EmbeddingManager> {
        self.embedding_manager.as_mut()
    }

    /// Debug method to list chunks (for testing)
    pub async fn debug_list_chunks(&self) -> CadiResult<Vec<serde_json::Value>> {
        let sql = "SELECT id, metadata.name, metadata.description FROM chunk";
        let mut response = self.db.query(sql).await
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;
        let chunks: Vec<serde_json::Value> = response.take(0)
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;
        Ok(chunks)
    }
}

/// Registry statistics
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_chunks: u64,
    pub total_usage: u64,
}