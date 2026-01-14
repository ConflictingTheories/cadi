use surrealdb::{engine::local::Mem, Surreal};
use serde_json::json;

#[tokio::test]
async fn test_surreal_metadata_binding_variants() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Testing SurrealDB metadata binding variants");

    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("test").use_db("test").await?;

    // Case A: bind serde_json::Value directly
    let meta_val = json!({"name": "case-a", "description": "val"});
    let sql = r#"CREATE chunk SET id = 'case-a', metadata = $metadata_a"#;
    let res = db.query(sql).bind(("metadata_a", &meta_val)).await?;
    println!("Result A: {:?}", res);

    // Case B: bind map (serde_json::Map)
    let meta_map = meta_val.as_object().cloned().unwrap();
    let sqlb = r#"CREATE chunk SET id = 'case-b', metadata = $metadata_b"#;
    let resb = db.query(sqlb).bind(("metadata_b", &meta_map)).await?;
    println!("Result B: {:?}", resb);

    // Case C: set nested fields explicitly
    let sqlc = r#"CREATE chunk SET id = 'case-c', metadata.name = $name, metadata.description = $description"#;
    let resc = db.query(sqlc).bind(("name", "case-c")).bind(("description", "explicit")).await?;
    println!("Result C: {:?}", resc);

    // Case D: id that looks like a thing (has colon)
    let meta_d = json!({"name": "case-d", "description": "d"});
    let sqld = r#"CREATE chunk SET id = 'chunk:case-d', metadata = $metadata_d"#;
    let resd = db.query(sqld).bind(("metadata_d", &meta_d)).await?;
    println!("Result D: {:?}", resd);

    // Case E: use the full store_chunk-style SQL with many binds
    let sql_e = r#"
        CREATE chunk SET
            id = $id,
            hash = $hash,
            content = $content,
            language = $language,
            metadata = $metadata,
            concepts = $concepts,
            quality_score = $quality_score,
            test_coverage = $test_coverage,
            embedding = $embedding,
            created_at = time::now()
    "#;

    let id_e = "chunk:case-e";
    let hash_e = "chunk:case-e";
    let content_e = "console.log('hello')";
    let language_e = "typescript";
    let meta_e = json!({"name":"case-e","description":"E desc","language":"typescript","concepts":[],"quality_score":0.5,"test_coverage":0.5});
    let concepts_e: Vec<String> = vec![];
    let embedding_e: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0];

    let rese = db.query(sql_e)
        .bind(("id", id_e))
        .bind(("hash", hash_e))
        .bind(("content", content_e))
        .bind(("language", language_e))
        .bind(("metadata", meta_e.as_object().unwrap().clone()))
        .bind(("concepts", &concepts_e))
        .bind(("quality_score", 0.5f64))
        .bind(("test_coverage", 0.5f64))
        .bind(("embedding", &embedding_e))
        .await?;

    println!("Result E: {:?}", rese);

    // Read back rows
    let r = db.query("SELECT id, metadata FROM chunk ORDER BY id").await?;
    println!("Select raw: {:?}", r);

    Ok(())
}

#[tokio::test]
async fn test_registry_database_store_chunk_reproduction() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Reproducing RegistryDatabase.store_chunk binding issue");

    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("test").use_db("test").await?;

    // Initialize schema exactly like RegistryDatabase
    // let schema = r#"
    //     DEFINE TABLE chunk SCHEMAFULL;
    //     ...
    // "#;
    // db.query(schema).await?;
    // Replicate the exact store_chunk logic
    let chunk_id = "chunk:update-meta-001";
    let hash = "chunk:update-meta-001";
    let content = "console.log('hello')";
    let language = "typescript";

    let metadata = json!({
        "name": "Initial",
        "description": "Initial description",
        "language": "typescript"
        // Removed concepts, quality_score, test_coverage from metadata
    });

    // Convert metadata to a map to ensure SurrealDB receives an object
    let metadata_map = metadata.as_object().cloned().unwrap_or_default();
    println!("DEBUG: metadata_map to bind: {:?}", metadata_map);

    let sql = r#"
        CREATE chunk SET
            id = $id,
            hash = $hash,
            content = $content,
            language = $language,
            metadata = $metadata,
            concepts = $concepts,
            quality_score = $quality_score,
            test_coverage = $test_coverage,
            embedding = $embedding,
            created_at = time::now()
    "#;

    println!("DEBUG: Executing store SQL");
    let result = db.query(sql)
        .bind(("id", &chunk_id))
        .bind(("hash", &hash))
        .bind(("content", content))
        .bind(("language", "typescript"))
        .bind(("metadata", &metadata))  // Try binding the Value instead of the map
        .bind(("concepts", metadata.get("concepts").and_then(|c| c.as_array()).unwrap_or(&vec![])))
        .bind(("quality_score", metadata.get("quality_score").and_then(|q| q.as_f64()).unwrap_or(0.0)))
        .bind(("test_coverage", metadata.get("test_coverage").and_then(|t| t.as_f64()).unwrap_or(0.0)))
        .bind(("embedding", &vec![1.0f32, 2.0, 3.0, 4.0, 5.0]))
        .await
        .map_err(|e| format!("Store query failed: {}", e))?;

    println!("DEBUG: Store result: {:?}", result);

    // Read back the record
    let read_sql = r#"SELECT * FROM chunk WHERE id = $chunk_id OR hash = $chunk_id"#;
    let mut response = db.query(read_sql).bind(("chunk_id", chunk_id)).await?;
    let results: Vec<serde_json::Value> = response.take(0)?;
    println!("DEBUG: Read back results: {:?}", results);

    if let Some(record) = results.first() {
        if let Some(meta) = record.get("metadata") {
            println!("DEBUG: Stored metadata: {:?}", meta);
        } else {
            println!("DEBUG: No metadata field in record");
        }
    }

    // Try the simplest possible case
    let simple_sql = r#"CREATE chunk SET id = 'simple-test', metadata = $metadata"#;
    let simple_result = db.query(simple_sql).bind(("metadata", &metadata)).await?;
    println!("DEBUG: Simple result: {:?}", simple_result);

    let simple_read = db.query("SELECT metadata FROM chunk WHERE id = 'simple-test'").await?;
    println!("DEBUG: Simple read: {:?}", simple_read);

    Ok(())
}
