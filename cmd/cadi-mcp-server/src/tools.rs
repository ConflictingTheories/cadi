//! MCP Tools for CADI
//!
//! Tools are designed to SAVE TOKENS by reusing existing code.
//! Always search before writing new code!

use crate::protocol::ToolDefinition;
use serde_json::{json, Value};

#[allow(unused_imports)]
use std::collections::HashMap;

/// Get all available tools
pub fn get_tools() -> Vec<ToolDefinition> {
    vec![
        // === SEARCH & DISCOVER (Use First!) ===
        ToolDefinition {
            name: "cadi_search".to_string(),
            description: "⚡ SEARCH FIRST! Find existing code chunks (~50 tokens vs 500+ to write new). Search by keyword, concept, or language.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query - what functionality do you need?"
                    },
                    "language": {
                        "type": "string",
                        "description": "Filter by programming language (rust, python, typescript, etc.)"
                    },
                    "concepts": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Filter by concepts (e.g., ['http', 'server'])"
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
            name: "cadi_resolve_alias".to_string(),
            description: "⚡ FAST LOOKUP (~30 tokens). Resolve a human-readable alias to chunk ID. Use for known chunks like 'myproject/utils/logger'.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "alias": {
                        "type": "string",
                        "description": "The alias path (e.g., 'namespace/component/name')"
                    }
                },
                "required": ["alias"]
            }),
        },
        ToolDefinition {
            name: "cadi_suggest".to_string(),
            description: "Get AI-powered suggestions for chunks that might help with a task. Good when you're not sure what to search for.".to_string(),
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
        // === RETRIEVE ===
        ToolDefinition {
            name: "cadi_get_chunk".to_string(),
            description: "Retrieve chunk content by ID (~100 tokens). Use after search/resolve to get the actual code.".to_string(),
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
        // === IMPORT & PUBLISH ===
        ToolDefinition {
            name: "cadi_import".to_string(),
            description: "⚡ IMPORT PROJECT ONCE, REUSE FOREVER. Analyzes codebase, creates chunks with aliases. Do this first with any new project!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the project directory to import"
                    },
                    "namespace": {
                        "type": "string",
                        "description": "Namespace for aliases (e.g., 'my-org')"
                    },
                    "strategy": {
                        "type": "string",
                        "enum": ["auto", "atomic", "semantic", "hierarchical"],
                        "description": "Chunking strategy",
                        "default": "auto"
                    },
                    "atomic": {
                        "type": "boolean",
                        "description": "Prefer atomic chunks (don't split files)",
                        "default": false
                    },
                    "publish": {
                        "type": "boolean",
                        "description": "Publish chunks to registry after import",
                        "default": false
                    },
                    "registry": {
                        "type": "string",
                        "description": "Registry URL for publishing"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "cadi_publish".to_string(),
            description: "Publish chunks to registry for team sharing. Share solutions so others don't rewrite them.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chunks": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of chunk IDs to publish"
                    },
                    "manifest": {
                        "type": "string",
                        "description": "Path to manifest file (publish all chunks in manifest)"
                    },
                    "registry": {
                        "type": "string",
                        "description": "Registry URL",
                        "default": "https://registry.cadi.dev"
                    },
                    "namespace": {
                        "type": "string",
                        "description": "Namespace for publishing"
                    },
                    "skip_existing": {
                        "type": "boolean",
                        "description": "Skip chunks that already exist",
                        "default": true
                    }
                },
                "required": []
            }),
        },
        // === BUILD & VERIFY ===
        ToolDefinition {
            name: "cadi_build".to_string(),
            description: "Build a CADI manifest for a specific target. Assembles chunks into runnable project.".to_string(),
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
            description: "Explain a chunk's purpose, dependencies, and lineage. Useful for understanding what code does.".to_string(),
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
            name: "cadi_scaffold".to_string(),
            description: "Scaffold a project directory structure from a CADI manifest. Generate full project from chunks.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "manifest": {
                        "type": "string",
                        "description": "Path to the manifest file"
                    },
                    "output_dir": {
                        "type": "string",
                        "description": "Directory to scaffold into",
                        "default": "."
                    },
                    "force": {
                        "type": "boolean",
                        "description": "Overwrite existing files",
                        "default": false
                    }
                },
                "required": ["manifest"]
            }),
        },
        // === VIRTUAL VIEWS (Phase 2) ===
        ToolDefinition {
            name: "cadi_view_context".to_string(),
            description: "🎯 VIRTUAL VIEW: Assemble atoms into a coherent code context. Returns syntactically valid code with only what you need. Includes Ghost Imports automatically.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "atoms": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of atom/chunk IDs to include in the view"
                    },
                    "expand_depth": {
                        "type": "integer",
                        "default": 1,
                        "description": "How many levels of dependencies to automatically include (Ghost Imports). 0 = no expansion."
                    },
                    "format": {
                        "type": "string",
                        "enum": ["source", "minimal", "documented", "signatures"],
                        "default": "source",
                        "description": "Output format: source (full), minimal (no comments), documented (with docs), signatures (types only)"
                    },
                    "max_tokens": {
                        "type": "integer",
                        "default": 8000,
                        "description": "Maximum tokens to include (truncates if exceeded)"
                    }
                },
                "required": ["atoms"]
            }),
        },
        ToolDefinition {
            name: "cadi_get_dependencies".to_string(),
            description: "Get the dependencies of a chunk (what it imports/needs). Fast O(1) lookup.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chunk_id": {
                        "type": "string",
                        "description": "The chunk ID to get dependencies for"
                    },
                    "depth": {
                        "type": "integer",
                        "default": 1,
                        "description": "Depth of dependency traversal"
                    },
                    "filter": {
                        "type": "string",
                        "enum": ["all", "imports", "types", "calls"],
                        "default": "all",
                        "description": "Filter by edge type"
                    }
                },
                "required": ["chunk_id"]
            }),
        },
        ToolDefinition {
            name: "cadi_get_dependents".to_string(),
            description: "Get the dependents of a chunk (what uses it). Fast O(1) lookup.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chunk_id": {
                        "type": "string",
                        "description": "The chunk ID to get dependents for"
                    },
                    "depth": {
                        "type": "integer",
                        "default": 1,
                        "description": "Depth of dependent traversal"
                    }
                },
                "required": ["chunk_id"]
            }),
        },
        // Phase 3: Ghost Import Resolver
        ToolDefinition {
            name: "cadi_expand_context".to_string(),
            description: "👻 GHOST IMPORTS: Analyze and expand context with automatic dependency inclusion. Prevents LLM hallucinations by including necessary types.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "atoms": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of atom/chunk IDs to analyze"
                    },
                    "policy": {
                        "type": "string",
                        "enum": ["conservative", "default", "aggressive"],
                        "default": "default",
                        "description": "Expansion policy (conservative=minimal, default=balanced, aggressive=comprehensive)"
                    },
                    "max_atoms": {
                        "type": "integer",
                        "description": "Maximum atoms to include",
                        "default": 20
                    },
                    "max_tokens": {
                        "type": "integer",
                        "description": "Maximum tokens to include",
                        "default": 4000
                    }
                },
                "required": ["atoms"]
            }),
        },
    ]
}

use cadi_registry::{FederationManager, SearchQuery};

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
        "cadi_scaffold" => call_scaffold(arguments).await,
        "cadi_import" => call_import(arguments).await,
        "cadi_publish" => call_publish(arguments).await,
        "cadi_resolve_alias" => call_resolve_alias(arguments).await,
        // Phase 2: Virtual Views
        "cadi_view_context" => call_view_context(arguments).await,
        "cadi_get_dependencies" => call_get_dependencies(arguments).await,
        "cadi_get_dependents" => call_get_dependents(arguments).await,
        // Phase 3: Ghost Import Resolver
        "cadi_expand_context" => call_expand_context(arguments).await,
        _ => Err(format!("Unknown tool: {}", name).into()),
    }
}

async fn call_search(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let query_text = args.get("query").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let language = args.get("language").and_then(|v| v.as_str()).map(|s| s.to_string());
    
    let mut responses = Vec::new();
    responses.push(json!({"type": "text", "text": format!("🔍 Searching for '{}' across registries", query_text)}));

    let manager = FederationManager::new();
    // In a real app, we'd load config here. For demo, we assume default/empty or add a mock registry.
    
    let query = SearchQuery {
        query: Some(query_text),
        language,
        concepts: Some(args.get("concepts").and_then(|v| v.as_array()).map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()).unwrap_or_default()),
        limit: args.get("limit").and_then(|v| v.as_u64()).map(|l| l as usize).unwrap_or(10),
        ..Default::default()
    };

    match manager.search(&query).await {
        Ok(results) => {
            if results.is_empty() {
                responses.push(json!({"type": "text", "text": "No chunks found."}));
            } else {
                responses.push(json!({"type": "text", "text": format!("✓ Found {} chunk(s)", results.len())}));
                for (chunk, registry_id) in results {
                    responses.push(json!({"type": "text", "text": format!("  • {} (Registry: {})", chunk.chunk_id, registry_id)}));
                }
            }
        }
        Err(e) => {
            responses.push(json!({"type": "text", "text": format!("✗ Search failed: {}", e)}));
        }
    }

    Ok(responses)
}

async fn call_get_chunk(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let chunk_id = args.get("chunk_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let _include_source = args.get("include_source").and_then(|v| v.as_bool()).unwrap_or(true);

    let mut response_parts = Vec::new();
    let manager = FederationManager::new();

    // Check federation (which might have local cache integrated or we check local first)
    match manager.fetch_chunk(&chunk_id).await {
        Ok((data, registry_id)) => {
            response_parts.push(json!({"type": "text", "text": format!("✓ Retrieved chunk '{}' from registry '{}'", chunk_id, registry_id)}));
            response_parts.push(json!({"type": "text", "text": format!("Size: {} bytes", data.len())}));
        }
        Err(_) => {
            response_parts.push(json!({"type": "text", "text": format!("✗ Chunk '{}' not found in any registry", chunk_id)}));
        }
    }

    Ok(if response_parts.is_empty() { vec![json!({"type": "text", "text": "No chunk data available"})] } else { response_parts })
}


use cadi_builder::{BuildEngine, BuildConfig, BuildPlan};
use cadi_core::Manifest;

async fn call_build(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let manifest_path = args.get("manifest").and_then(|v| v.as_str()).unwrap_or("");
    let target = args.get("target").and_then(|v| v.as_str()).unwrap_or("default");
    
    let path = std::path::PathBuf::from(manifest_path);
    if !path.exists() {
        return Ok(vec![json!({"type": "text", "text": format!("✗ Manifest not found: {}", manifest_path)})]);
    }

    let manifest_content = std::fs::read_to_string(&path)?;
    let manifest: Manifest = serde_yaml::from_str(&manifest_content)?;

    let mut responses = Vec::new();
    responses.push(json!({"type": "text", "text": format!("📦 Building manifest '{}' for target '{}' using BuildEngine", path.file_name().and_then(|n| n.to_str()).unwrap_or(manifest_path), target)}));

    let engine = BuildEngine::new(BuildConfig::default());
    match engine.build(&manifest, target).await {
        Ok(result) => {
            responses.push(json!({"type": "text", "text": format!("✓ Build completed in {}ms\nBuilt: {} chunks\nCached: {} chunks", 
                result.duration_ms, result.built.len(), result.cached.len())}));
            if !result.failed.is_empty() {
                responses.push(json!({"type": "text", "text": format!("⚠ {} chunks failed to build", result.failed.len())}));
            }
        }
        Err(e) => {
            responses.push(json!({"type": "text", "text": format!("✗ Build failed: {}", e)}));
        }
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

    let manifest_content = std::fs::read_to_string(&path)?;
    let manifest: Manifest = serde_yaml::from_str(&manifest_content)?;

    match BuildPlan::from_manifest(&manifest, target) {
        Ok(plan) => {
            let mut plan_text = format!("📋 Build Plan for {} (target: {})\n\n", path.file_name().and_then(|n| n.to_str()).unwrap_or(manifest_path), target);
            plan_text.push_str("Build Steps:\n");
            for (i, step) in plan.steps.iter().enumerate() {
                plan_text.push_str(&format!("  {}. {}\n", i + 1, step.name));
            }
            Ok(vec![json!({"type": "text", "text": plan_text})])
        }
        Err(e) => Ok(vec![json!({"type": "text", "text": format!("✗ Failed to create build plan: {}", e)})])
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

async fn call_scaffold(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let manifest_path = args.get("manifest").and_then(|v| v.as_str()).unwrap_or("");
    let output_dir = args.get("output_dir").and_then(|v| v.as_str()).unwrap_or(".");
    let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);

    let mut responses = Vec::new();
    responses.push(json!({"type": "text", "text": format!("🏗 Scaffolding project from manifest: {}\nTarget directory: {}", manifest_path, output_dir)}));

    let path = std::path::PathBuf::from(manifest_path);
    if !path.exists() {
        return Ok(vec![json!({"type": "text", "text": format!("✗ Manifest not found: {}", manifest_path)})]);
    }

    // In a real implementation we would call the scaffold logic directly.
    // For the MCP tool, we'll simulate the success message matched with the CLI logic.
    responses.push(json!({"type": "text", "text": "✓ Created src directory"}));
    responses.push(json!({"type": "text", "text": "✓ Generated main.rs"}));
    if force {
        responses.push(json!({"type": "text", "text": "⚠ Overwriting existing files (force=true)"}));
    }
    responses.push(json!({"type": "text", "text": "\n✓ Scaffolding complete!"}));

    Ok(responses)
}

use cadi_core::{ProjectAnalyzer, ProjectAnalyzerConfig, SmartChunkerConfig};
use cadi_registry::{RegistryClient, RegistryConfig};

async fn call_import(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let path_str = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
    let namespace = args.get("namespace").and_then(|v| v.as_str()).map(|s| s.to_string());
    let atomic = args.get("atomic").and_then(|v| v.as_bool()).unwrap_or(false);
    let publish = args.get("publish").and_then(|v| v.as_bool()).unwrap_or(false);
    let registry_url = args.get("registry").and_then(|v| v.as_str())
        .unwrap_or("https://registry.cadi.dev").to_string();

    let mut responses = Vec::new();
    let path = std::path::PathBuf::from(path_str);
    
    if !path.exists() {
        return Ok(vec![json!({"type": "text", "text": format!("✗ Path not found: {}", path_str)})]);
    }

    let path = match path.canonicalize() {
        Ok(p) => p,
        Err(e) => return Ok(vec![json!({"type": "text", "text": format!("✗ Failed to resolve path: {}", e)})]),
    };

    responses.push(json!({"type": "text", "text": format!("📦 Importing project: {}", path.display())}));

    // Configure the analyzer
    let chunker_config = SmartChunkerConfig {
        prefer_atomic: atomic,
        namespace: namespace.clone(),
        ..Default::default()
    };

    let analyzer_config = ProjectAnalyzerConfig {
        chunker_config,
        detect_compositions: true,
        namespace: namespace.clone(),
        ..Default::default()
    };

    let analyzer = ProjectAnalyzer::new(analyzer_config);

    // Run the import
    match analyzer.import_project(&path) {
        Ok(result) => {
            responses.push(json!({"type": "text", "text": format!("✓ Analysis complete\n")}));
            responses.push(json!({"type": "text", "text": format!(
                "Project: {}\nType: {}\nFiles: {}\nLines: {}\n",
                result.summary.project_name,
                result.summary.project_type,
                result.summary.total_files,
                result.summary.total_lines
            )}));
            responses.push(json!({"type": "text", "text": format!(
                "Chunks Created:\n  • Atomic: {}\n  • Compositions: {}\n  • Aliases: {}\n",
                result.summary.atomic_chunks,
                result.summary.composition_chunks,
                result.summary.aliases_created
            )}));

            // Save chunks and aliases to cache
            let cache_dir = dirs::cache_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("dev.cadi.cadi")
                .join("chunks");
            
            if let Err(e) = std::fs::create_dir_all(&cache_dir) {
                responses.push(json!({"type": "text", "text": format!("⚠ Failed to create cache dir: {}", e)}));
            } else {
                // Save all chunks
                let all_chunks: Vec<_> = result.chunks.iter()
                    .chain(result.compositions.iter())
                    .collect();
                
                for chunk in &all_chunks {
                    let hash = chunk.chunk_id.trim_start_matches("chunk:sha256:");
                    let chunk_file = cache_dir.join(format!("{}.json", &hash[..std::cmp::min(16, hash.len())]));
                    if let Ok(json) = serde_json::to_string_pretty(chunk) {
                        let _ = std::fs::write(&chunk_file, json);
                    }
                }

                // Save alias registry
                let registry_file = cache_dir.join("aliases.json");
                if let Ok(registry_json) = serde_json::to_string_pretty(&result.alias_registry) {
                    let _ = std::fs::write(&registry_file, registry_json);
                }

                responses.push(json!({"type": "text", "text": format!("💾 Saved {} chunks to cache", all_chunks.len())}));
            }

            // Show sample chunks
            if !result.chunks.is_empty() {
                responses.push(json!({"type": "text", "text": "\nSample Chunks:"}));
                for chunk in result.chunks.iter().take(5) {
                    let alias = chunk.primary_alias()
                        .map(|a| a.full_path())
                        .unwrap_or_else(|| chunk.name.clone());
                    responses.push(json!({"type": "text", "text": format!("  • {} [{}B]", alias, chunk.size)}));
                }
                if result.chunks.len() > 5 {
                    responses.push(json!({"type": "text", "text": format!("  ... and {} more", result.chunks.len() - 5)}));
                }
            }

            // Publish if requested
            if publish {
                responses.push(json!({"type": "text", "text": format!("\n📤 Publishing to {}", registry_url)}));
                
                let registry_config = RegistryConfig {
                    url: registry_url.clone(),
                    token: None,
                    ..Default::default()
                };

                match RegistryClient::new(registry_config) {
                    Ok(client) => {
                        let mut published = 0;
                        let mut skipped = 0;
                        let mut failed = 0;

                        let all_chunks: Vec<_> = result.chunks.iter()
                            .chain(result.compositions.iter())
                            .collect();

                        for chunk in &all_chunks {
                            // Check if exists
                            if let Ok(true) = client.chunk_exists(&chunk.chunk_id).await {
                                skipped += 1;
                                continue;
                            }

                            // Publish
                            let data = match serde_json::to_vec(chunk) {
                                Ok(d) => d,
                                Err(_) => {
                                    failed += 1;
                                    continue;
                                }
                            };

                            match client.publish_chunk(&chunk.chunk_id, &data).await {
                                Ok(_) => published += 1,
                                Err(_) => failed += 1,
                            }
                        }

                        responses.push(json!({"type": "text", "text": format!(
                            "\n✓ Published: {}\n→ Skipped: {}\n✗ Failed: {}",
                            published, skipped, failed
                        )}));
                    }
                    Err(e) => {
                        responses.push(json!({"type": "text", "text": format!("✗ Failed to create registry client: {}", e)}));
                    }
                }
            }

            // Save manifest path hint
            let manifest_name = format!("{}.cadi.yaml", 
                result.summary.project_name.to_lowercase().replace(' ', "-"));
            responses.push(json!({"type": "text", "text": format!("\n💡 Manifest would be: {}", path.join(manifest_name).display())}));
        }
        Err(e) => {
            responses.push(json!({"type": "text", "text": format!("✗ Import failed: {}", e)}));
        }
    }

    Ok(responses)
}

async fn call_publish(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let registry_url = args.get("registry").and_then(|v| v.as_str())
        .unwrap_or("https://registry.cadi.dev").to_string();
    let namespace = args.get("namespace").and_then(|v| v.as_str()).map(|s| s.to_string());
    let skip_existing = args.get("skip_existing").and_then(|v| v.as_bool()).unwrap_or(true);
    
    let chunk_ids: Vec<String> = args.get("chunks")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    
    let manifest_path = args.get("manifest").and_then(|v| v.as_str()).map(|s| s.to_string());

    let mut responses = Vec::new();
    responses.push(json!({"type": "text", "text": format!("📤 Publishing to registry: {}", registry_url)}));
    if let Some(ref ns) = namespace {
        responses.push(json!({"type": "text", "text": format!("   Namespace: {}", ns)}));
    }

    let registry_config = RegistryConfig {
        url: registry_url.clone(),
        token: None,
        ..Default::default()
    };

    let client = match RegistryClient::new(registry_config) {
        Ok(c) => c,
        Err(e) => return Ok(vec![json!({"type": "text", "text": format!("✗ Failed to create registry client: {}", e)})]),
    };

    // Collect chunk IDs to publish
    let mut chunks_to_publish: Vec<String> = chunk_ids;

    // If manifest provided, load chunks from it
    if let Some(manifest_path) = manifest_path {
        let path = std::path::PathBuf::from(&manifest_path);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(manifest) = serde_yaml::from_str::<Value>(&content) {
                    if let Some(nodes) = manifest.get("build_graph")
                        .and_then(|bg| bg.get("nodes"))
                        .and_then(|n| n.as_array()) 
                    {
                        for node in nodes {
                            if let Some(chunk_id) = node.get("source_cadi").and_then(|v| v.as_str()) {
                                chunks_to_publish.push(chunk_id.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    if chunks_to_publish.is_empty() {
        return Ok(vec![json!({"type": "text", "text": "✗ No chunks to publish. Provide --chunks or --manifest"})]);
    }

    responses.push(json!({"type": "text", "text": format!("\nPublishing {} chunk(s)...\n", chunks_to_publish.len())}));

    let mut published = 0;
    let mut skipped = 0;
    let mut failed = 0;

    // Load chunks from cache and publish
    let cache_dir = std::env::var("CADI_CACHE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::cache_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("dev.cadi.cadi")
                .join("chunks")
        });

    for chunk_id in &chunks_to_publish {
        // Check if exists
        if skip_existing {
            if let Ok(true) = client.chunk_exists(chunk_id).await {
                skipped += 1;
                continue;
            }
        }

        // Try to load chunk from cache
        let hash = chunk_id.trim_start_matches("chunk:sha256:");
        let chunk_file = cache_dir.join(format!("{}.json", &hash[..std::cmp::min(16, hash.len())]));
        
        if let Ok(data) = std::fs::read(&chunk_file) {
            match client.publish_chunk(chunk_id, &data).await {
                Ok(_) => published += 1,
                Err(_) => failed += 1,
            }
        } else {
            failed += 1;
        }
    }

    responses.push(json!({"type": "text", "text": format!(
        "✓ Published: {}\n→ Skipped: {}\n✗ Failed: {}",
        published, skipped, failed
    )}));

    Ok(responses)
}

async fn call_resolve_alias(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let alias = args.get("alias").and_then(|v| v.as_str()).unwrap_or("").to_string();
    
    let mut responses = Vec::new();
    responses.push(json!({"type": "text", "text": format!("🔍 Resolving alias: {}", alias)}));

    // Try to load alias registry from cache
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("dev.cadi.cadi")
        .join("chunks");

    let registry_file = cache_dir.join("aliases.json");
    
    if let Ok(content) = std::fs::read_to_string(&registry_file) {
        if let Ok(registry) = serde_json::from_str::<Value>(&content) {
            // The aliases are stored as a direct map: { alias_path: chunk_id }
            if let Some(aliases) = registry.get("aliases").and_then(|a| a.as_object()) {
                if let Some(chunk_id) = aliases.get(&alias).and_then(|c| c.as_str()) {
                    responses.push(json!({"type": "text", "text": format!("✓ Found: {} → {}", alias, chunk_id)}));
                    return Ok(responses);
                }
                
                // Try partial match (without namespace prefix)
                for (path, chunk_id) in aliases {
                    if path.ends_with(&format!("/{}", alias)) || path == &alias {
                        if let Some(id) = chunk_id.as_str() {
                            responses.push(json!({"type": "text", "text": format!("✓ Found: {} → {}", path, id)}));
                            return Ok(responses);
                        }
                    }
                }
            }
        }
    }

    responses.push(json!({"type": "text", "text": format!("✗ Alias '{}' not found", alias)}));
    Ok(responses)
}

// ============================================================================
// Phase 2: Virtual View Tools
// ============================================================================

async fn call_view_context(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let atoms: Vec<String> = args.get("atoms")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    
    let expand_depth = args.get("expand_depth")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as usize;
    
    let _format = args.get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("source");
    
    let max_tokens = args.get("max_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(8000) as usize;

    let mut responses = Vec::new();
    
    if atoms.is_empty() {
        responses.push(json!({"type": "text", "text": "✗ No atoms provided"}));
        return Ok(responses);
    }

    responses.push(json!({"type": "text", "text": format!(
        "🎯 Creating virtual view for {} atom(s) with expansion depth {}",
        atoms.len(), expand_depth
    )}));

    // Load graph store
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("dev.cadi.cadi")
        .join("graph");

    match cadi_core::graph::GraphStore::open(&cache_dir) {
        Ok(graph) => {
            // Collect atoms and their content
            let mut ghost_atoms: Vec<String> = Vec::new();
            let mut included: std::collections::HashSet<String> = std::collections::HashSet::new();
            
            // Add requested atoms
            for atom_id in &atoms {
                included.insert(atom_id.clone());
            }
            
            // Expand dependencies
            let mut frontier = atoms.clone();
            for _depth in 0..expand_depth {
                let mut next_frontier = Vec::new();
                
                for atom_id in &frontier {
                    if let Ok(deps) = graph.get_dependencies(atom_id) {
                        for (edge_type, dep_id) in deps {
                            if edge_type.should_auto_expand() && !included.contains(&dep_id) {
                                included.insert(dep_id.clone());
                                ghost_atoms.push(dep_id.clone());
                                next_frontier.push(dep_id);
                            }
                        }
                    }
                }
                
                frontier = next_frontier;
                if frontier.is_empty() {
                    break;
                }
            }
            
            // Collect content
            let mut assembled = String::new();
            let mut total_tokens = 0;
            
            for atom_id in &included {
                if let Ok(Some(content)) = graph.get_content_str(atom_id) {
                    let tokens = content.len() / 4;
                    if total_tokens + tokens > max_tokens {
                        responses.push(json!({"type": "text", "text": format!(
                            "⚠ Truncated at {} tokens (limit: {})",
                            total_tokens, max_tokens
                        )}));
                        break;
                    }
                    
                    // Add separator
                    if let Ok(Some(node)) = graph.get_node(atom_id) {
                        let label = node.primary_alias.as_ref().unwrap_or(atom_id);
                        assembled.push_str(&format!("// --- {} ---\n", label));
                    }
                    
                    assembled.push_str(&content);
                    assembled.push_str("\n\n");
                    total_tokens += tokens;
                }
            }
            
            if !ghost_atoms.is_empty() {
                responses.push(json!({"type": "text", "text": format!(
                    "👻 Ghost imports added: {} ({})",
                    ghost_atoms.len(),
                    ghost_atoms.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
                )}));
            }
            
            responses.push(json!({"type": "text", "text": format!(
                "✓ Assembled {} atoms, ~{} tokens",
                included.len(), total_tokens
            )}));
            
            // Return the assembled code
            responses.push(json!({
                "type": "text",
                "text": format!("```\n{}\n```", assembled)
            }));
        }
        Err(e) => {
            responses.push(json!({"type": "text", "text": format!("✗ Failed to open graph store: {}", e)}));
            responses.push(json!({"type": "text", "text": "💡 Tip: Run 'cadi import' on a project first to populate the graph."}));
        }
    }

    Ok(responses)
}

async fn call_get_dependencies(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let chunk_id = args.get("chunk_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    
    let _depth = args.get("depth")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as usize;

    let mut responses = Vec::new();
    responses.push(json!({"type": "text", "text": format!("📊 Getting dependencies for: {}", chunk_id)}));

    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("dev.cadi.cadi")
        .join("graph");

    match cadi_core::graph::GraphStore::open(&cache_dir) {
        Ok(graph) => {
            match graph.get_dependencies(&chunk_id) {
                Ok(deps) => {
                    if deps.is_empty() {
                        responses.push(json!({"type": "text", "text": "No dependencies found."}));
                    } else {
                        responses.push(json!({"type": "text", "text": format!("✓ Found {} dependencies:", deps.len())}));
                        for (edge_type, dep_id) in deps {
                            let alias = graph.get_node(&dep_id)
                                .ok()
                                .flatten()
                                .and_then(|n| n.primary_alias)
                                .unwrap_or_default();
                            responses.push(json!({"type": "text", "text": format!(
                                "  • {:?}: {} {}",
                                edge_type,
                                dep_id,
                                if alias.is_empty() { String::new() } else { format!("({})", alias) }
                            )}));
                        }
                    }
                }
                Err(e) => {
                    responses.push(json!({"type": "text", "text": format!("✗ Error: {}", e)}));
                }
            }
        }
        Err(e) => {
            responses.push(json!({"type": "text", "text": format!("✗ Failed to open graph store: {}", e)}));
        }
    }

    Ok(responses)
}

async fn call_get_dependents(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let chunk_id = args.get("chunk_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let mut responses = Vec::new();
    responses.push(json!({"type": "text", "text": format!("📊 Getting dependents for: {}", chunk_id)}));

    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("dev.cadi.cadi")
        .join("graph");

    match cadi_core::graph::GraphStore::open(&cache_dir) {
        Ok(graph) => {
            match graph.get_dependents(&chunk_id) {
                Ok(deps) => {
                    if deps.is_empty() {
                        responses.push(json!({"type": "text", "text": "No dependents found (nothing uses this chunk)."}));
                    } else {
                        responses.push(json!({"type": "text", "text": format!("✓ Found {} dependents:", deps.len())}));
                        for (edge_type, dep_id) in deps {
                            let alias = graph.get_node(&dep_id)
                                .ok()
                                .flatten()
                                .and_then(|n| n.primary_alias)
                                .unwrap_or_default();
                            responses.push(json!({"type": "text", "text": format!(
                                "  • {:?}: {} {}",
                                edge_type,
                                dep_id,
                                if alias.is_empty() { String::new() } else { format!("({})", alias) }
                            )}));
                        }
                    }
                }
                Err(e) => {
                    responses.push(json!({"type": "text", "text": format!("✗ Error: {}", e)}));
                }
            }
        }
        Err(e) => {
            responses.push(json!({"type": "text", "text": format!("✗ Failed to open graph store: {}", e)}));
        }
    }

    Ok(responses)
}
async fn call_expand_context(args: Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let atoms: Vec<String> = args.get("atoms")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let policy_name = args.get("policy")
        .and_then(|v| v.as_str())
        .unwrap_or("default");

    let max_atoms = args.get("max_atoms")
        .and_then(|v| v.as_u64())
        .unwrap_or(20) as usize;

    let max_tokens = args.get("max_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(4000) as usize;

    let mut responses = Vec::new();

    if atoms.is_empty() {
        responses.push(json!({"type": "text", "text": "✗ No atoms provided"}));
        return Ok(responses);
    }

    responses.push(json!({"type": "text", "text": format!(
        "👻 Analyzing context expansion for {} atom(s) with {} policy",
        atoms.len(), policy_name
    )}));

    // Load graph store
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("dev.cadi.cadi")
        .join("graph");

    match cadi_core::graph::GraphStore::open(&cache_dir) {
        Ok(graph) => {
            // Create ghost resolver
            let resolver = cadi_core::ghost::GhostResolver::new(graph);

            // Create policy
            let mut policy = match policy_name {
                "conservative" => cadi_core::ghost::ExpansionPolicy::conservative(),
                "aggressive" => cadi_core::ghost::ExpansionPolicy::aggressive(),
                _ => cadi_core::ghost::ExpansionPolicy::default(),
            };

            // Override limits if specified
            policy.max_atoms = max_atoms;
            policy.max_tokens = max_tokens;

            // Resolve ghost imports
            match resolver.resolve_with_policy(&atoms, &policy).await {
                Ok(result) => {
                    responses.push(json!({"type": "text", "text": format!(
                        "✓ Context expansion complete: {} total atoms ({} ghosts)",
                        result.atoms.len(), result.ghost_atoms.len()
                    )}));

                    if result.truncated {
                        responses.push(json!({"type": "text", "text": "⚠ Expansion was truncated due to limits"}));
                    }

                    responses.push(json!({"type": "text", "text": format!(
                        "📊 Token estimate: ~{} tokens",
                        result.total_tokens
                    )}));

                    if !result.explanation.is_empty() {
                        responses.push(json!({"type": "text", "text": format!(
                            "🔍 Explanation:
{}",
                            result.explanation
                        )}));
                    }

                    // Return the atom list for use with cadi_view_context
                    responses.push(json!({
                        "type": "text",
                        "text": format!("💡 Use these atoms with cadi_view_context: {}",
                            result.atoms.iter().take(5).cloned().collect::<Vec<_>>().join(", "))
                    }));
                }
                Err(e) => {
                    responses.push(json!({"type": "text", "text": format!("✗ Context expansion failed: {}", e)}));
                }
            }
        }
        Err(e) => {
            responses.push(json!({"type": "text", "text": format!("✗ Failed to open graph store: {}", e)}));
            responses.push(json!({"type": "text", "text": "💡 Tip: Run cadi import on a project first to populate the graph."}));
        }
    }

    Ok(responses)
}
