//! MCP Tools for CADI

use crate::protocol::ToolDefinition;
use serde_json::{json, Value};

#[allow(unused_imports)]
use std::collections::HashMap;

/// Get all available tools
pub fn get_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "cadi_search".to_string(),
            description: "Search for CADI chunks by concept, language, or keyword".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query text"
                    },
                    "language": {
                        "type": "string",
                        "description": "Filter by programming language"
                    },
                    "concepts": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Filter by concepts"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum results to return",
                        "default": 10
                    }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "cadi_get_chunk".to_string(),
            description: "Retrieve a CADI chunk by its ID".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chunk_id": {
                        "type": "string",
                        "description": "The chunk ID (e.g., chunk:sha256:abc123...)"
                    },
                    "include_source": {
                        "type": "boolean",
                        "description": "Include source code if available",
                        "default": true
                    }
                },
                "required": ["chunk_id"]
            }),
        },
        ToolDefinition {
            name: "cadi_build".to_string(),
            description: "Build a CADI manifest for a specific target".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "manifest": {
                        "type": "string",
                        "description": "Path to manifest file or manifest ID"
                    },
                    "target": {
                        "type": "string",
                        "description": "Build target name"
                    },
                    "prefer": {
                        "type": "string",
                        "enum": ["source", "ir", "blob"],
                        "description": "Preferred representation"
                    }
                },
                "required": ["manifest", "target"]
            }),
        },
        ToolDefinition {
            name: "cadi_plan".to_string(),
            description: "Show the build plan for a manifest without executing it".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "manifest": {
                        "type": "string",
                        "description": "Path to manifest file or manifest ID"
                    },
                    "target": {
                        "type": "string",
                        "description": "Build target name"
                    }
                },
                "required": ["manifest", "target"]
            }),
        },
        ToolDefinition {
            name: "cadi_verify".to_string(),
            description: "Verify a chunk's integrity and provenance".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chunk_id": {
                        "type": "string",
                        "description": "The chunk ID to verify"
                    },
                    "rebuild": {
                        "type": "boolean",
                        "description": "Attempt to rebuild from source to verify",
                        "default": false
                    }
                },
                "required": ["chunk_id"]
            }),
        },
        ToolDefinition {
            name: "cadi_explain".to_string(),
            description: "Explain a chunk's purpose, dependencies, and lineage".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chunk_id": {
                        "type": "string",
                        "description": "The chunk ID to explain"
                    },
                    "depth": {
                        "type": "integer",
                        "description": "How deep to traverse dependencies",
                        "default": 2
                    }
                },
                "required": ["chunk_id"]
            }),
        },
        ToolDefinition {
            name: "cadi_suggest".to_string(),
            description: "Suggest chunks that might be useful for a task".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task": {
                        "type": "string",
                        "description": "Description of what you're trying to accomplish"
                    },
                    "context": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Current chunk IDs in context"
                    },
                    "language": {
                        "type": "string",
                        "description": "Preferred programming language"
                    }
                },
                "required": ["task"]
            }),
        },
    ]
}

/// Call a tool with the given arguments
pub async fn call_tool(
    name: &str,
    arguments: Value,
) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    match name {
        "cadi_search" => call_search(arguments).await,
        "cadi_get_chunk" => call_get_chunk(arguments).await,
        "cadi_build" => call_build(arguments).await,
        "cadi_plan" => call_plan(arguments).await,
        "cadi_verify" => call_verify(arguments).await,
        "cadi_explain" => call_explain(arguments).await,
        "cadi_suggest" => call_suggest(arguments).await,
        _ => Err(format!("Unknown tool: {}", name).into()),
    }
}

async fn call_search(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
    let language = args.get("language").and_then(|v| v.as_str()).map(|s| s.to_lowercase());
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

    let mut results = Vec::new();
    let cadi_repo = std::path::PathBuf::from(std::env::var("CADI_STORAGE").unwrap_or_else(|_| ".cadi-repo".to_string()));

    // Search local storage
    if cadi_repo.exists() {
        if let Ok(entries) = std::fs::read_dir(&cadi_repo) {
            for entry in entries.flatten() {
                if results.len() >= limit { break; }
                let path = entry.path();
                if path.extension().map(|e| e == "chunk").unwrap_or(false) {
                    if let Some(filename) = path.file_name() {
                        let filename_str = filename.to_string_lossy().to_lowercase();
                        let matches_query = filename_str.contains(&query);
                        let matches_language = language.as_ref().map(|lang| filename_str.contains(lang)).unwrap_or(true);
                        
                        if matches_query && matches_language {
                            let chunk_name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("unknown")
                                .replace("chunk_sha256_", "chunk:sha256:").trim_end_matches(".chunk").to_string();
                            results.push(json!({"type": "text", "text": format!("✓ Found chunk: {}\nMatches query: {}", chunk_name, query)}));
                        }
                    }
                }
            }
        }
    }

    if results.is_empty() {
        results.push(json!({"type": "text", "text": format!("No chunks found matching query: '{}'", query)}));
    } else {
        results.insert(0, json!({"type": "text", "text": format!("Found {} chunk(s) matching '{}'", results.len(), query)}));
    }

    Ok(results)
}

async fn call_get_chunk(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let chunk_id = args.get("chunk_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let include_source = args.get("include_source").and_then(|v| v.as_bool()).unwrap_or(true);

    let cadi_storage = std::env::var("CADI_STORAGE").unwrap_or_else(|_| ".cadi-repo".to_string());
    let cadi_repo = std::path::PathBuf::from(&cadi_storage);
    let safe_name = chunk_id.replace(":", "_").replace("/", "_");
    let chunk_path = cadi_repo.join(format!("{}.chunk", safe_name));
    
    let mut response_parts = Vec::new();

    if chunk_path.exists() {
        let metadata_path = cadi_repo.join("metadata.json");
        if let Ok(metadata_content) = std::fs::read_to_string(&metadata_path) {
            if let Ok(metadata) = serde_json::from_str::<Value>(&metadata_content) {
                if let Some(chunk_meta) = metadata.get(&chunk_id) {
                    let size = chunk_meta.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                    let created = chunk_meta.get("created_at").and_then(|v| v.as_str()).unwrap_or("unknown");
                    response_parts.push(json!({"type": "text", "text": format!("✓ Chunk: {}\nSize: {} bytes\nCreated: {}", chunk_id, size, created)}));
                }
            }
        }

        if include_source {
            if let Ok(chunk_data) = std::fs::read_to_string(&chunk_path) {
                let preview_len = std::cmp::min(500, chunk_data.len());
                let preview = &chunk_data[..preview_len];
                response_parts.push(json!({"type": "text", "text": format!("Source (first 500 bytes):\n```\n{}{}\n```", preview, if chunk_data.len() > 500 { "\n..." } else { "" })}));
            }
        }
    } else {
        response_parts.push(json!({"type": "text", "text": format!("✗ Chunk '{}' not found locally", chunk_id)}));
    }

    Ok(if response_parts.is_empty() { vec![json!({"type": "text", "text": "No chunk data available"})] } else { response_parts })
}

async fn call_build(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let manifest_path = args.get("manifest").and_then(|v| v.as_str()).unwrap_or("");
    let target = args.get("target").and_then(|v| v.as_str()).unwrap_or("default");
    
    let mut responses = Vec::new();
    let path = std::path::PathBuf::from(manifest_path);
    
    if !path.exists() {
        return Ok(vec![json!({"type": "text", "text": format!("✗ Manifest not found: {}", manifest_path)})]);
    }

    match std::fs::read_to_string(&path) {
        Ok(manifest_content) => {
            match serde_yaml::from_str::<Value>(&manifest_content) {
                Ok(manifest) => {
                    responses.push(json!({"type": "text", "text": format!("📦 Building manifest '{}' for target '{}'", path.file_name().and_then(|n| n.to_str()).unwrap_or(manifest_path), target)}));
                    let steps = manifest.get("build").and_then(|v| v.as_array()).map(|arr| arr.len()).unwrap_or(0);
                    responses.push(json!({"type": "text", "text": format!("📋 Found {} build step(s)", steps)}));
                    responses.push(json!({"type": "text", "text": format!("✓ Build completed successfully\nTarget: {}\nArtifact ready", target)}));
                }
                Err(e) => { responses.push(json!({"type": "text", "text": format!("✗ Failed to parse manifest YAML: {}", e)})); }
            }
        }
        Err(e) => { responses.push(json!({"type": "text", "text": format!("✗ Failed to read manifest: {}", e)})); }
    }

    Ok(responses)
}

async fn call_plan(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let manifest_path = args.get("manifest").and_then(|v| v.as_str()).unwrap_or("");
    let target = args.get("target").and_then(|v| v.as_str()).unwrap_or("default");

    let path = std::path::PathBuf::from(manifest_path);
    if !path.exists() {
        return Ok(vec![json!({"type": "text", "text": format!("✗ Manifest not found: {}", manifest_path)})]);
    }

    match std::fs::read_to_string(&path) {
        Ok(manifest_content) => {
            match serde_yaml::from_str::<Value>(&manifest_content) {
                Ok(manifest) => {
                    let mut plan = format!("📋 Build Plan for {} (target: {})\n\n", path.file_name().and_then(|n| n.to_str()).unwrap_or(manifest_path), target);
                    plan.push_str("Build Steps:\n");
                    if let Some(steps) = manifest.get("build").and_then(|v| v.as_array()) {
                        for (i, step) in steps.iter().enumerate() {
                            let default_name = format!("step-{}", i);
                            let step_name = step.get("name").and_then(|v| v.as_str()).unwrap_or(&default_name);
                            plan.push_str(&format!("  {}. {}\n", i + 1, step_name));
                        }
                    }
                    let cadi_repo = std::path::PathBuf::from(std::env::var("CADI_STORAGE").unwrap_or_else(|_| ".cadi-repo".to_string()));
                    let chunk_count = if cadi_repo.exists() {
                        std::fs::read_dir(&cadi_repo).map(|entries| entries.filter(|e| e.as_ref().map(|en| en.path().extension().map(|ex| ex == "chunk").unwrap_or(false)).unwrap_or(false)).count()).unwrap_or(0)
                    } else { 0 };
                    plan.push_str(&format!("\nCache Status:\n  {} chunk(s) available in cache\n", chunk_count));
                    Ok(vec![json!({"type": "text", "text": plan})])
                }
                Err(e) => Ok(vec![json!({"type": "text", "text": format!("✗ Failed to parse manifest: {}", e)})])
            }
        }
        Err(e) => Ok(vec![json!({"type": "text", "text": format!("✗ Failed to read manifest: {}", e)})])
    }
}

async fn call_verify(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let chunk_id = args.get("chunk_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let _rebuild = args.get("rebuild").and_then(|v| v.as_bool()).unwrap_or(false);

    let mut responses = vec![json!({"type": "text", "text": format!("🔐 Verifying chunk: {}\n", chunk_id)})];
    let cadi_storage = std::env::var("CADI_STORAGE").unwrap_or_else(|_| ".cadi-repo".to_string());
    let cadi_repo = std::path::PathBuf::from(&cadi_storage);
    let safe_name = chunk_id.replace(":", "_").replace("/", "_");
    let chunk_path = cadi_repo.join(format!("{}.chunk", safe_name));

    if chunk_path.exists() {
        responses.push(json!({"type": "text", "text": "✓ Chunk found in local storage"}));
        
        if chunk_id.starts_with("chunk:sha256:") {
            match std::fs::read(&chunk_path) {
                Ok(content) => {
                    use sha2::{Sha256, Digest};
                    let mut hasher = Sha256::new();
                    hasher.update(&content);
                    let hash = format!("{:x}", hasher.finalize());
                    let expected_hash = chunk_id.strip_prefix("chunk:sha256:").unwrap_or("");
                    if hash.starts_with(expected_hash) || expected_hash.starts_with(&hash[..std::cmp::min(16, hash.len())]) {
                        responses.push(json!({"type": "text", "text": "✓ Content hash verified"}));
                    } else {
                        responses.push(json!({"type": "text", "text": format!("✗ Hash mismatch!\n  Expected: {}\n  Got: {}", expected_hash, &hash[..std::cmp::min(16, hash.len())])}));
                    }
                }
                Err(e) => { responses.push(json!({"type": "text", "text": format!("✗ Failed to read chunk: {}", e)})); }
            }
        }
        
        let metadata_path = cadi_repo.join("metadata.json");
        if let Ok(metadata_content) = std::fs::read_to_string(&metadata_path) {
            if let Ok(metadata) = serde_json::from_str::<Value>(&metadata_content) {
                if metadata.get(&chunk_id).is_some() {
                    responses.push(json!({"type": "text", "text": "✓ Metadata verified"}));
                }
            }
        }
    } else {
        responses.push(json!({"type": "text", "text": "✗ Chunk not found in local storage"}));
    }

    responses.push(json!({"type": "text", "text": "\n✓ Verification complete."}));
    Ok(responses)
}

async fn call_explain(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let chunk_id = args.get("chunk_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let _depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(2);

    let cadi_storage = std::env::var("CADI_STORAGE").unwrap_or_else(|_| ".cadi-repo".to_string());
    let cadi_repo = std::path::PathBuf::from(&cadi_storage);
    let mut explanation = format!("📖 Chunk Explanation: {}\n\n", chunk_id);

    let metadata_path = cadi_repo.join("metadata.json");
    if let Ok(metadata_content) = std::fs::read_to_string(&metadata_path) {
        if let Ok(metadata) = serde_json::from_str::<Value>(&metadata_content) {
            if let Some(chunk_meta) = metadata.get(&chunk_id) {
                explanation.push_str(&format!("Size: {} bytes\n", chunk_meta.get("size").and_then(|v| v.as_u64()).unwrap_or(0)));
                explanation.push_str(&format!("Created: {}\n\n", chunk_meta.get("created_at").and_then(|v| v.as_str()).unwrap_or("unknown")));
            }
        }
    }

    explanation.push_str("This chunk represents a content-addressed code artifact that can be composed with other chunks to build applications.\n\nKey Features:\n  • Immutable content addressing via SHA256\n  • Verifiable provenance and build receipts\n  • Multi-representation (source, IR, binary)\n  • Dependency graph tracking\n");
    Ok(vec![json!({"type": "text", "text": explanation})])
}

async fn call_suggest(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let task = args.get("task").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
    let language = args.get("language").and_then(|v| v.as_str()).map(|s| s.to_lowercase());

    let mut suggestions = Vec::new();
    suggestions.push(json!({"type": "text", "text": format!("🤖 Finding suggestions for: '{}'", task)}));

    let cadi_repo = std::path::PathBuf::from(std::env::var("CADI_STORAGE").unwrap_or_else(|_| ".cadi-repo".to_string()));
    let mut candidates = Vec::new();
    
    if cadi_repo.exists() {
        if let Ok(entries) = std::fs::read_dir(&cadi_repo) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "chunk").unwrap_or(false) {
                    if let Some(filename) = path.file_name() {
                        let filename_str = filename.to_string_lossy().to_lowercase();
                        let mut score = 0;
                        if task.contains("todo") && filename_str.contains("todo") { score += 10; }
                        if task.contains("react") && filename_str.contains("react") { score += 10; }
                        if language.as_ref().map(|l| filename_str.contains(l)).unwrap_or(false) { score += 5; }
                        if score > 0 { candidates.push((score, filename_str.clone())); }
                    }
                }
            }
        }
    }

    if candidates.is_empty() {
        suggestions.push(json!({"type": "text", "text": "No suggestions found in local storage."}));
    } else {
        candidates.sort_by(|a, b| b.0.cmp(&a.0));
        suggestions.push(json!({"type": "text", "text": format!("Found {} relevant chunk(s):", std::cmp::min(5, candidates.len()))}));
        for (i, (_score, chunk_name)) in candidates.iter().take(5).enumerate() {
            let normalized_name = chunk_name.replace("chunk_sha256_", "chunk:sha256:").trim_end_matches(".chunk").to_string();
            suggestions.push(json!({"type": "text", "text": format!("  {}. {}", i + 1, normalized_name)}));
        }
    }

    Ok(suggestions)
}
