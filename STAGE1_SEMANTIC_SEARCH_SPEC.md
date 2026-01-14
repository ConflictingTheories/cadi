# Stage 1: Semantic Search + Vector Indexing
## Detailed Implementation Specification

### Overview
Enable LLMs to discover reusable atoms semantically, not just by keywords. A query like "JWT authentication for user sessions" should find the todo app's JWT middleware even if keywords don't match perfectly.

---

## 1.1 Enhanced Search Engine

### File: `internal/cadi-registry/src/search.rs`

**Current State**: Skeleton with SearchModality enum, ComponentMetadata struct, basic text indexing.

**Changes**:

#### 1. Extend SearchEngine struct
```rust
pub struct SearchEngine {
    embeddings: HashMap<String, Vec<f32>>,  // Chunk ID → embedding vector
    embedding_provider: Arc<dyn EmbeddingProvider>,  // For lazy generation
    text_index: HashMap<String, Vec<String>>,  // Chunk ID → tokenized text
    components: HashMap<String, ComponentMetadata>,
    
    // NEW: BM25 scoring
    bm25_index: BM25Index,
    
    // NEW: Semantic similarity cache
    similarity_cache: Arc<Mutex<HashMap<String, f32>>>,
}
```

#### 2. Implement BM25 Scoring
```rust
pub struct BM25Index {
    k1: f32,  // Default 1.5
    b: f32,   // Default 0.75
    corpus_stats: CorpusStats,
    documents: HashMap<String, Vec<String>>,  // Chunk ID → tokens
}

pub struct CorpusStats {
    total_docs: usize,
    avg_doc_length: f32,
    idf_cache: HashMap<String, f32>,  // Token → IDF value
}

impl BM25Index {
    pub fn new() -> Self { ... }
    
    pub fn add_document(&mut self, id: &str, tokens: Vec<String>) {
        // Update corpus stats, IDF cache
        self.documents.insert(id, tokens);
    }
    
    pub fn score(&self, query: &str, doc_id: &str) -> f32 {
        // BM25 formula: Σ IDF(qi) * (f(qi,D) * (k1 + 1)) / (f(qi,D) + k1 * (1 - b + b * |D| / avgdl))
        // where f(qi,D) is term frequency in document
    }
}
```

#### 3. Hybrid Search with Scoring
```rust
impl SearchEngine {
    pub async fn search_hybrid(
        &self,
        query: &str,
        limit: usize,
        weights: SearchWeights,  // NEW
    ) -> Vec<ScoredResult> {
        // 1. Parse query into tokens
        let query_tokens = tokenize(query);
        
        // 2. Text search (BM25)
        let text_scores: HashMap<String, f32> = self.bm25_index
            .documents
            .keys()
            .map(|id| {
                let score = self.bm25_index.score(query, id);
                (id.clone(), score)
            })
            .filter(|(_, score)| score > &0.0)
            .collect();
        
        // 3. Semantic search (embeddings)
        let query_embedding = self.embedding_provider.generate(query).await?;
        let semantic_scores: HashMap<String, f32> = self.embeddings
            .iter()
            .map(|(id, emb)| {
                let similarity = cosine_similarity(&query_embedding, emb);
                (id.clone(), similarity)
            })
            .collect();
        
        // 4. Combine scores with weights
        let mut combined_scores = HashMap::new();
        for id in self.components.keys() {
            let text_score = text_scores.get(id).copied().unwrap_or(0.0);
            let semantic_score = semantic_scores.get(id).copied().unwrap_or(0.0);
            
            let combined = weights.text * text_score 
                         + weights.semantic * semantic_score
                         + weights.quality * self.components[id].quality_score;
            
            combined_scores.insert(id.clone(), combined);
        }
        
        // 5. Sort and return top results
        let mut results: Vec<_> = combined_scores
            .into_iter()
            .map(|(id, score)| ScoredResult {
                id,
                score,
                metadata: self.components[&id].clone(),
            })
            .collect();
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        Ok(results)
    }
}

pub struct SearchWeights {
    pub text: f32,      // Default 0.3
    pub semantic: f32,  // Default 0.5
    pub quality: f32,   // Default 0.2 (usage count, test coverage)
}

pub struct ScoredResult {
    pub id: String,
    pub score: f32,
    pub metadata: ComponentMetadata,
}
```

#### 4. Lazy Embedding Generation
```rust
impl SearchEngine {
    /// Ensure embedding exists, generate if missing
    pub async fn ensure_embedding(&mut self, id: &str, text: &str) -> Result<()> {
        if self.embeddings.contains_key(id) {
            return Ok(());
        }
        
        let embedding = self.embedding_provider.generate(text).await?;
        self.embeddings.insert(id.to_string(), embedding);
        Ok(())
    }
}
```

#### 5. Utility Functions
```rust
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty() && s.len() > 1)  // Filter stopwords by length
        .map(|s| s.to_string())
        .collect()
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a * norm_b)
}
```

---

## 1.2 Metadata Normalization & Indexing

### File: `internal/cadi-core/src/interface.rs` (new functions)

**Goal**: Extract rich metadata from code for search indexing.

```rust
pub struct ChunkMetadata {
    /// Human-readable name (e.g., "JWT Authenticator")
    pub name: String,
    
    /// Concise description (~200 chars)
    pub description: String,
    
    /// Programming language
    pub language: String,
    
    /// Semantic concepts/tags (e.g., ["auth", "jwt", "middleware"])
    pub concepts: Vec<String>,
    
    /// Function/class signatures (for structural search)
    pub signatures: Vec<String>,
    
    /// Dependencies (other chunk IDs)
    pub dependencies: Vec<String>,
    
    /// Module path (e.g., "utils/auth")
    pub module_path: String,
}

pub struct MetadataExtractor;

impl MetadataExtractor {
    /// Extract metadata from code
    pub fn extract(code: &str, language: &str, module_path: &str) -> Result<ChunkMetadata> {
        let name = Self::extract_name(code, language)?;
        let description = Self::extract_description(code, language)?;
        let signatures = Self::extract_signatures(code, language)?;
        let concepts = Self::infer_concepts(&description, &signatures);
        
        Ok(ChunkMetadata {
            name,
            description,
            language: language.to_string(),
            concepts,
            signatures,
            dependencies: vec![],  // Filled by dependency resolver
            module_path: module_path.to_string(),
        })
    }
    
    fn extract_name(code: &str, language: &str) -> Result<String> {
        // Parse AST, find function/class name
        // For TypeScript: function add(...) or class MyClass
        // For Python: def add(...) or class MyClass
        // For Rust: fn add(...) or impl MyStruct
    }
    
    fn extract_description(code: &str, language: &str) -> Result<String> {
        // Extract top comment block or docstring
        // Normalize to single line (~200 chars)
    }
    
    fn extract_signatures(code: &str, language: &str) -> Result<Vec<String>> {
        // Extract all function/class signatures
        // Example: ["add(x: number, y: number): number", "MyClass { ... }"]
    }
    
    fn infer_concepts(description: &str, signatures: &[String]) -> Vec<String> {
        // Use NLP or keyword extraction to infer semantic concepts
        // Example: "JWT authentication middleware" → ["auth", "jwt", "middleware", "http"]
    }
}
```

---

## 1.3 SurrealDB Integration with Vector Search

### File: `internal/cadi-registry/src/db.rs` (new module)

**Goal**: Store chunks + embeddings with MTREE vector index for <100ms queries.

```rust
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use surrealdb::engine::remote::Client;

pub struct VectorSearchDB {
    client: Surreal<Client>,
}

impl VectorSearchDB {
    pub async fn new(url: &str, namespace: &str, database: &str) -> Result<Self> {
        let client = Surreal::new::<Wss>(url)
            .await?
            .use_ns(namespace)
            .use_db(database)
            .await?;
        
        Ok(Self { client })
    }
    
    /// Initialize schema with MTREE vector index
    pub async fn init_schema(&self) -> Result<()> {
        // Create chunks table with embedding field
        self.client.query(
            r#"
            DEFINE TABLE chunks SCHEMAFULL;
            
            DEFINE FIELD id ON TABLE chunks TYPE string;
            DEFINE FIELD semantic_hash ON TABLE chunks TYPE string;
            DEFINE FIELD name ON TABLE chunks TYPE string;
            DEFINE FIELD description ON TABLE chunks TYPE string;
            DEFINE FIELD language ON TABLE chunks TYPE string;
            DEFINE FIELD concepts ON TABLE chunks TYPE array<string>;
            DEFINE FIELD embedding ON TABLE chunks TYPE array<float>;
            DEFINE FIELD metadata ON TABLE chunks TYPE object;
            DEFINE FIELD created_at ON TABLE chunks TYPE datetime;
            DEFINE FIELD usage_count ON TABLE chunks TYPE number;
            DEFINE FIELD quality_score ON TABLE chunks TYPE float;
            
            -- MTREE index for vector search
            DEFINE INDEX idx_embedding_search ON TABLE chunks FIELDS embedding MTREE DIMENSION 1536;
            
            -- Text index for keyword search
            DEFINE FULLTEXT INDEX idx_name ON TABLE chunks COLUMNS name;
            DEFINE FULLTEXT INDEX idx_description ON TABLE chunks COLUMNS description;
            "#
        )
        .await?;
        
        Ok(())
    }
    
    /// Store a chunk with embedding
    pub async fn store_chunk(
        &self,
        id: &str,
        semantic_hash: &str,
        metadata: &ChunkMetadata,
        embedding: &[f32],
    ) -> Result<()> {
        let record: Vec<Thing> = self.client.create("chunks")
            .content(serde_json::json!({
                "id": id,
                "semantic_hash": semantic_hash,
                "name": metadata.name,
                "description": metadata.description,
                "language": metadata.language,
                "concepts": metadata.concepts,
                "embedding": embedding,
                "metadata": metadata,
                "created_at": chrono::Utc::now(),
                "usage_count": 0,
                "quality_score": 0.8,
            }))
            .await?;
        
        Ok(())
    }
    
    /// Vector search using MTREE index
    pub async fn vector_search(
        &self,
        query_embedding: &[f32],
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<ChunkSearchResult>> {
        let results: Vec<ChunkSearchResult> = self.client.query(
            r#"
            SELECT id, name, description, language, 
                   math::similarity::cosine(embedding, $query_embedding) AS similarity
            FROM chunks
            WHERE math::similarity::cosine(embedding, $query_embedding) > $threshold
            ORDER BY similarity DESC
            LIMIT $limit
            "#
        )
        .bind(("query_embedding", query_embedding))
        .bind(("threshold", threshold))
        .bind(("limit", limit))
        .await?
        .take(0)?;
        
        Ok(results)
    }
    
    /// Hybrid search: text + vector
    pub async fn hybrid_search(
        &self,
        query_text: &str,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<HybridSearchResult>> {
        // Combine text search (FULLTEXT) and vector search
        let results: Vec<HybridSearchResult> = self.client.query(
            r#"
            SELECT 
                id, name, description, language,
                (search::score()) AS text_score,
                math::similarity::cosine(embedding, $query_embedding) AS semantic_score,
                (search::score() * 0.3 + math::similarity::cosine(embedding, $query_embedding) * 0.7) AS combined_score
            FROM chunks
            WHERE id @@ $query_text OR concepts @@ $query_text
            ORDER BY combined_score DESC
            LIMIT $limit
            "#
        )
        .bind(("query_text", query_text))
        .bind(("query_embedding", query_embedding))
        .bind(("limit", limit))
        .await?
        .take(0)?;
        
        Ok(results)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkSearchResult {
    pub id: String,
    pub name: String,
    pub description: String,
    pub language: String,
    pub similarity: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HybridSearchResult {
    pub id: String,
    pub name: String,
    pub description: String,
    pub language: String,
    pub text_score: f32,
    pub semantic_score: f32,
    pub combined_score: f32,
}
```

---

## 1.4 MCP Tool Implementation

### File: `cmd/cadi-mcp-server/src/tools.rs` (enhance `cadi_search` handler)

**Goal**: Implement the `cadi_search` tool backend.

```rust
pub async fn handle_cadi_search(
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    let query: String = params["query"]
        .as_str()
        .ok_or("Missing query")?
        .to_string();
    
    let language: Option<String> = params["language"].as_str().map(|s| s.to_string());
    let limit: usize = params["limit"].as_u64().unwrap_or(10) as usize;
    
    // Get database connection
    let db = get_vector_search_db().await?;
    
    // Generate query embedding
    let embedding_provider = get_embedding_provider();
    let query_embedding = embedding_provider.generate(&query).await?;
    
    // Search
    let results = db.hybrid_search(&query, &query_embedding, limit).await?;
    
    // Format for LLM (small response)
    let formatted_results: Vec<serde_json::Value> = results
        .iter()
        .map(|r| json!({
            "id": r.id,
            "name": r.name,
            "description": r.description,
            "language": r.language,
            "relevance_score": r.combined_score,
        }))
        .collect();
    
    Ok(json!({
        "results": formatted_results,
        "total": results.len(),
        "query": query,
        "note": "Use cadi_get_chunk(id) to retrieve full code"
    }))
}
```

---

## 1.5 Integration with Import Pipeline

### File: `internal/cadi-builder/src/importer.rs` (new code)

**Goal**: When importing a project, automatically index chunks for search.

```rust
pub struct ProjectImporter {
    search_db: Arc<VectorSearchDB>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
}

impl ProjectImporter {
    pub async fn import_and_index(
        &self,
        project_path: &Path,
        namespace: &str,
    ) -> Result<Vec<String>> {
        // 1. Analyze project, extract chunks
        let chunks = self.analyze_project(project_path).await?;
        
        let mut chunk_ids = Vec::new();
        
        for chunk in chunks {
            // 2. Extract metadata
            let metadata = MetadataExtractor::extract(
                &chunk.code,
                &chunk.language,
                &chunk.module_path,
            )?;
            
            // 3. Generate embedding
            let embedding_text = format!(
                "{} {} {}",
                metadata.name,
                metadata.description,
                metadata.signatures.join(" ")
            );
            let embedding = self.embedding_provider.generate(&embedding_text).await?;
            
            // 4. Store in vector DB
            let chunk_id = self.compute_chunk_id(&chunk);
            self.search_db.store_chunk(
                &chunk_id,
                &chunk.semantic_hash,
                &metadata,
                &embedding,
            ).await?;
            
            chunk_ids.push(chunk_id);
        }
        
        Ok(chunk_ids)
    }
}
```

---

## Testing

### File: `tests/semantic_search_test.rs`

```rust
#[tokio::test]
async fn test_semantic_search_jwt_auth() {
    let db = setup_test_db().await;
    let embedding_provider = MockProvider::default();
    
    // Register JWT chunk from todo app
    db.store_chunk(
        "chunk:sha256:jwt123",
        "hash123",
        &ChunkMetadata {
            name: "JWT Authenticator".into(),
            description: "Token-based authentication with JWT for user sessions".into(),
            language: "typescript".into(),
            concepts: vec!["auth".into(), "jwt".into(), "middleware".into()],
            signatures: vec!["authenticate(req, res)".into()],
            dependencies: vec![],
            module_path: "utils/auth".into(),
        },
        &vec![0.1; 1536],  // Mock embedding
    ).await.unwrap();
    
    // Search for similar concept
    let query = "user authentication with tokens";
    let embedding = embedding_provider.generate(query).await.unwrap();
    
    let results = db.hybrid_search(query, &embedding, 5).await.unwrap();
    
    assert!(!results.is_empty());
    assert_eq!(results[0].id, "chunk:sha256:jwt123");
}
```

---

## Deployment & Configuration

### Environment Variables
```env
# Embedding service
EMBEDDING_PROVIDER=openai  # or "mock" for testing
OPENAI_API_KEY=sk-...

# SurrealDB
SURREALDB_URL=ws://localhost:8000
SURREALDB_NS=cadi
SURREALDB_DB=chunks
```

### CI/CD Changes
- Add tests to `cargo test --all-features`
- Ensure embeddings are generated during import
- Validate vector index on startup

---

## Success Criteria

✅ Semantic search returns relevant results even with different keywords  
✅ <100ms query latency for 10K+ chunks  
✅ Hybrid scoring balances text + semantic relevance  
✅ Metadata extraction works across TypeScript, Python, Rust  
✅ MCP tool `cadi_search` returns <200 tokens  
✅ Forward project demo: "JWT auth" search finds todo app's chunk  

---

## Next: Stage 2 (Semantic Hashing)
Once Stage 1 is complete, implement semantic hashing to ensure identical code across projects gets deduplicated automatically.
