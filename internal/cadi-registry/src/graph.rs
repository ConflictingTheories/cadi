use cadi_core::graph::*;
use cadi_core::CadiError;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{Client, Wss};
use surrealdb::sql::Thing;
use surrealdb::Surreal;

pub struct GraphDB {
    client: Surreal<Client>,
}

impl GraphDB {
    pub async fn new(url: &str, namespace: &str, database: &str) -> Result<Self, CadiError> {
        let client = Surreal::new::<Wss>(url).await?;
        client.use_ns(namespace).await?;
        client.use_db(database).await?;

        Ok(Self { client })
    }

    /// Initialize graph schema
    pub async fn init_schema(&self) -> Result<(), CadiError> {
        self.client
            .query(
                r#"
            -- Chunks table (nodes)
            DEFINE TABLE chunks SCHEMAFULL;
            DEFINE FIELD id ON TABLE chunks TYPE string;
            DEFINE FIELD semantic_hash ON TABLE chunks TYPE string;
            DEFINE FIELD name ON TABLE chunks TYPE string;
            DEFINE FIELD description ON TABLE chunks TYPE string;
            DEFINE FIELD language ON TABLE chunks TYPE string;
            DEFINE FIELD interface ON TABLE chunks TYPE object;
            DEFINE FIELD created_at ON TABLE chunks TYPE datetime;
            DEFINE FIELD usage_count ON TABLE chunks TYPE number;
            DEFINE FIELD quality_score ON TABLE chunks TYPE float;
            
            -- Dependencies table (edges)
            DEFINE TABLE dependencies TYPE RELATION IN out;
            DEFINE FIELD edge_type ON TABLE dependencies TYPE string;
            DEFINE FIELD confidence ON TABLE dependencies TYPE float;
            DEFINE FIELD context ON TABLE dependencies TYPE object;
            DEFINE FIELD created_at ON TABLE dependencies TYPE datetime;
            
            -- Indexes
            DEFINE INDEX idx_chunk_hash ON TABLE chunks FIELDS semantic_hash;
            DEFINE INDEX idx_chunk_language ON TABLE chunks FIELDS language;
            "#,
            )
            .await?;

        Ok(())
    }

    /// Create a chunk node
    pub async fn create_chunk_node(
        &self,
        id: &str,
        metadata: &ChunkNode,
    ) -> Result<Option<Thing>, CadiError> {
        let record: Option<Thing> = self
            .client
            .create(("chunks", id))
            .content(serde_json::json!({
                "id": id,
                "semantic_hash": metadata.semantic_hash,
                "name": metadata.name,
                "description": metadata.description,
                "language": metadata.language,
                "interface": metadata.interface,
                "created_at": chrono::Utc::now(),
                "usage_count": 0,
                "quality_score": 0.8,
            }))
            .await?;

        Ok(record)
    }

    pub async fn get_chunk_node(&self, id: &str) -> Result<Option<ChunkNode>, CadiError> {
        let node: Option<ChunkNode> = self.client.select(("chunks", id)).await?;
        Ok(node)
    }

    /// Create an edge between two chunks
    pub async fn create_edge(
        &self,
        from_id: &str,
        to_id: &str,
        edge_type: GraphEdgeType,
        context: Option<serde_json::Value>,
    ) -> Result<Option<Thing>, CadiError> {
        let edge_name = format!("{:?}", edge_type);

        let mut record: surrealdb::Response = self
            .client
            .query(
                r#"
            RELATE $from -> dependencies -> $to SET 
                edge_type = $edge_type,
                confidence = $confidence,
                context = $context,
                created_at = now()
            "#,
            )
            .bind(("from", format!("chunks:{}", from_id)))
            .bind(("to", format!("chunks:{}", to_id)))
            .bind(("edge_type", edge_name))
            .bind(("confidence", 1.0))
            .bind(("context", context.unwrap_or(serde_json::json!({}))))
            .await?;

        Ok(record.take::<Option<Thing>>(0)?)
    }

    /// Get all direct dependencies of a chunk
    pub async fn get_dependencies(&self, chunk_id: &str) -> Result<Vec<DependencyInfo>, CadiError> {
        let mut results = self
            .client
            .query(
                r#"
            SELECT 
                type::string(out) AS dependency_id,
                edge_type,
                confidence
            FROM (
                SELECT out FROM dependencies 
                WHERE in == chunks:$chunk_id
            )
            "#,
            )
            .bind(("chunk_id", chunk_id))
            .await?;

        let deps: Vec<DependencyInfo> = results.take::<Vec<DependencyInfo>>(0)?;
        Ok(deps)
    }

    /// Get transitive closure of dependencies
    pub async fn get_transitive_dependencies(
        &self,
        chunk_id: &str,
        max_depth: usize,
    ) -> Result<TransitiveDependencies, CadiError> {
        let mut results = self
            .client
            .query(
                r#"
            SELECT 
                id,
                depth
            FROM (
                SELECT ->dependencies->chunks AS dep FROM chunks:$chunk_id 
                LIMIT $depth
            ) FLATTEN(dep)
            "#,
            )
            .bind(("chunk_id", chunk_id))
            .bind(("depth", max_depth))
            .await?;

        let results: Vec<TransitiveResult> = results.take::<Vec<TransitiveResult>>(0)?;

        let mut direct = Vec::new();
        let mut transitive = Vec::new();
        let mut all = Vec::new();

        for result in results {
            all.push(result.id.clone());
            if result.depth == 1 {
                direct.push(result.id);
            } else {
                transitive.push(result.id);
            }
        }

        Ok(TransitiveDependencies {
            direct,
            transitive,
            all,
        })
    }

    /// Find shortest path between two chunks
    pub async fn find_path(
        &self,
        from_id: &str,
        to_id: &str,
    ) -> Result<Option<DependencyPath>, CadiError> {
        // Use GRAPH traversal
        let mut results = self
            .client
            .query(
                r#"
            SELECT * FROM (
                SELECT ->dependencies->(chunks)* AS path FROM chunks:$from 
                WHERE id == chunks:$to
            ) LIMIT 1
            "#,
            )
            .bind(("from", from_id))
            .bind(("to", to_id))
            .await?;

        let results: Vec<PathResult> = results.take::<Vec<PathResult>>(0)?;

        if results.is_empty() {
            return Ok(None);
        }

        // Construct DependencyPath from results
        // (Implementation depends on path format)
        Ok(None) // Placeholder
    }

    /// Find all chunks that implement a specific interface
    pub async fn find_implementing_chunks(&self, interface_name: &str) -> Result<Vec<ChunkNode>, CadiError> {
        let mut results = self
            .client
            .query(
                r#"
            SELECT * FROM chunks 
            WHERE interface.name == $interface_name
            "#,
            )
            .bind(("interface_name", interface_name))
            .await?;
        let chunks: Vec<ChunkNode> = results.take::<Vec<ChunkNode>>(0)?;
        Ok(chunks)
    }

    /// Find semantic equivalents using graph
    pub async fn find_equivalents(&self, chunk_id: &str) -> Result<Vec<EquivalentChunkInfo>, CadiError> {
        let mut results = self
            .client
            .query(
                r#"
            SELECT 
                type::string(->dependencies->(chunks WHERE edge_type == 'EQUIVALENT_TO')) AS equivalent_id,
                semantic_hash,
                name,
                language
            FROM chunks:$chunk_id
            "#,
            )
            .bind(("chunk_id", chunk_id))
            .await?;
        let equivs: Vec<EquivalentChunkInfo> = results.take(0)?;
        Ok(equivs)
    }

    /// Get chunks that can be composed (find SATISFIES relationships)
    pub async fn find_composable_chunks(&self, requirement: &str) -> Result<Vec<ComposableChunk>, CadiError> {
        let mut results = self
            .client
            .query(
                r#"
            SELECT 
                id,
                name,
                description,
                language,
                interface
            FROM chunks
            WHERE description @@ $requirement 
               OR concepts @@ $requirement
            ORDER BY quality_score DESC
            LIMIT 20
            "#,
            )
            .bind(("requirement", requirement))
            .await?;

        let chunks: Vec<ComposableChunk> = results.take::<Vec<ComposableChunk>>(0)?;
        Ok(chunks)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub dependency_id: String,
    pub edge_type: String,
    pub confidence: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransitiveResult {
    pub id: String,
    pub depth: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathResult {
    pub path: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EquivalentChunkInfo {
    pub equivalent_id: String,
    pub semantic_hash: String,
    pub name: String,
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComposableChunk {
    pub id: String,
    pub name: String,
    pub description: String,
    pub language: String,
    pub interface: Option<ChunkInterface>,
}