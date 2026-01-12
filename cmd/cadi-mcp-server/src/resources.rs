//! MCP Resources for CADI
//!
//! Resources provide context and documentation to AI agents.
//! The usage guide explains how to use CADI to save tokens.

use crate::protocol::ResourceDefinition;
use serde_json::{json, Value};

/// Get all available resources
pub fn get_resources() -> Vec<ResourceDefinition> {
    vec![
        ResourceDefinition {
            uri: "cadi://guide".to_string(),
            name: "CADI Usage Guide".to_string(),
            description: "⚡ READ THIS FIRST - How to use CADI to save tokens and reuse code".to_string(),
            mime_type: "text/markdown".to_string(),
        },
        ResourceDefinition {
            uri: "cadi://aliases".to_string(),
            name: "Available Aliases".to_string(),
            description: "List of all cached chunk aliases - use instead of reading files".to_string(),
            mime_type: "application/json".to_string(),
        },
        ResourceDefinition {
            uri: "cadi://config".to_string(),
            name: "CADI Configuration".to_string(),
            description: "Current CADI configuration settings".to_string(),
            mime_type: "application/json".to_string(),
        },
        ResourceDefinition {
            uri: "cadi://cache/stats".to_string(),
            name: "Cache Statistics".to_string(),
            description: "Local cache usage statistics".to_string(),
            mime_type: "application/json".to_string(),
        },
        ResourceDefinition {
            uri: "cadi://registries".to_string(),
            name: "Registry Federation".to_string(),
            description: "Configured registry endpoints".to_string(),
            mime_type: "application/json".to_string(),
        },
        ResourceDefinition {
            uri: "cadi://trust/policy".to_string(),
            name: "Trust Policy".to_string(),
            description: "Current trust policy configuration".to_string(),
            mime_type: "application/json".to_string(),
        },
    ]
}

/// Read a resource by URI
pub async fn read_resource(
    uri: &str,
) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    match uri {
        "cadi://guide" => read_usage_guide().await,
        "cadi://aliases" => read_aliases().await,
        "cadi://config" => read_config().await,
        "cadi://cache/stats" => read_cache_stats().await,
        "cadi://registries" => read_registries().await,
        "cadi://trust/policy" => read_trust_policy().await,
        _ => {
            // Try to parse as chunk URI
            if uri.starts_with("cadi://chunk/") {
                let chunk_id = uri.strip_prefix("cadi://chunk/").unwrap();
                read_chunk(chunk_id).await
            } else {
                Err(format!("Unknown resource: {}", uri).into())
            }
        }
    }
}

async fn read_usage_guide() -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let guide = r#"# CADI Usage Guide for AI Agents

## ⚡ TOKEN SAVING PRIORITY

CADI (Content-Addressed Development Interface) helps you **save tokens** by:
1. **Reusing existing code chunks** instead of regenerating code
2. **Using compact aliases** instead of full file paths
3. **Searching before coding** to find existing solutions

## 🎯 CADI-FIRST WORKFLOW (Recommended)

Before writing ANY code, follow this workflow:

### Step 1: Search CADI First
```
cadi_search(query: "what you need")
```
This searches all registries for existing chunks. **Costs ~50 tokens vs 500+ to write code.**

### Step 2: Check Local Aliases
```
cadi_resolve_alias(alias: "namespace/component")
```
If you've imported projects before, chunks are cached with aliases.

### Step 3: Import Existing Projects
```
cadi_import(path: "./project", namespace: "myproject")
```
Analyzes and chunks a codebase. **Do this once, reuse forever.**

### Step 4: Only Write New Code If Needed
If CADI has no solution, then write new code. But **import it after** so it's reusable:
```
cadi_import(path: "./new-code", publish: true)
```

## 📊 TOKEN COST COMPARISON

| Action | Tokens | Notes |
|--------|--------|-------|
| cadi_search | ~50 | Check for existing solution |
| cadi_resolve_alias | ~30 | Get chunk by alias |
| cadi_get_chunk | ~100 | Retrieve full chunk |
| Writing new code | 500-5000 | Expensive! Avoid if possible |
| Reading large files | 1000+ | Use CADI chunks instead |

## 🔧 TOOL QUICK REFERENCE

### Search & Discover
- `cadi_search` - Find chunks by keyword/concept
- `cadi_suggest` - Get AI suggestions for a task
- `cadi_resolve_alias` - Look up chunk by alias

### Import & Publish  
- `cadi_import` - Import any codebase (analyze, chunk, alias)
- `cadi_publish` - Publish chunks to registry

### Build & Verify
- `cadi_build` - Build from manifest
- `cadi_plan` - Show build plan
- `cadi_verify` - Verify chunk integrity
- `cadi_explain` - Explain what a chunk does

### Retrieve
- `cadi_get_chunk` - Get chunk content by ID
- `cadi_scaffold` - Generate project from manifest

## 💡 BEST PRACTICES

1. **Always search before coding** - Someone may have solved this
2. **Import projects early** - Makes code reusable via aliases
3. **Use aliases, not file paths** - `myproject/utils/logger` vs full path
4. **Publish useful code** - Share solutions to the registry
5. **Prefer small chunks** - Atomic, reusable units

## 🚫 ANTI-PATTERNS (Waste Tokens)

- ❌ Writing code without searching CADI first
- ❌ Reading entire files when you need one function
- ❌ Re-implementing common utilities
- ❌ Not importing projects you're working on

## 📦 CHUNK GRANULARITY

CADI chunks code at these levels:
- **Function** - Individual functions (most reusable)
- **Type** - Structs, classes, enums
- **Module** - Logical groupings
- **Package** - Full libraries
- **Project** - Entire applications

Prefer smaller chunks for maximum reuse.
"#;

    Ok(vec![json!({
        "uri": "cadi://guide",
        "mimeType": "text/markdown",
        "text": guide
    })])
}

async fn read_aliases() -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("dev.cadi.cadi")
        .join("chunks");
    
    let registry_file = cache_dir.join("aliases.json");
    
    if let Ok(content) = std::fs::read_to_string(&registry_file) {
        if let Ok(registry) = serde_json::from_str::<Value>(&content) {
            if let Some(aliases) = registry.get("aliases").and_then(|a| a.as_object()) {
                let alias_list: Vec<_> = aliases.keys().cloned().collect();
                let summary = json!({
                    "total_aliases": alias_list.len(),
                    "aliases": alias_list,
                    "usage": "Use cadi_resolve_alias(alias) to get chunk ID, then cadi_get_chunk to retrieve"
                });
                return Ok(vec![json!({
                    "uri": "cadi://aliases",
                    "mimeType": "application/json",
                    "text": serde_json::to_string_pretty(&summary)?
                })]);
            }
        }
    }
    
    Ok(vec![json!({
        "uri": "cadi://aliases",
        "mimeType": "application/json",
        "text": json!({"total_aliases": 0, "aliases": [], "hint": "Run cadi_import to populate aliases"}).to_string()
    })])
}

async fn read_config() -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let registry_url = std::env::var("CADI_REGISTRY").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let storage = std::env::var("CADI_STORAGE").unwrap_or_else(|_| ".cadi-repo".to_string());
    
    let config = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "registry": {
            "primary": registry_url,
            "cache_enabled": true,
            "storage_path": storage
        },
        "build": {
            "parallel_jobs": num_cpus::get(),
            "prefer_cached": true
        },
        "security": {
            "verify_signatures": true,
            "require_attestations": false
        }
    });
    
    Ok(vec![json!({
        "uri": "cadi://config",
        "mimeType": "application/json",
        "text": serde_json::to_string_pretty(&config)?
    })])
}

async fn read_cache_stats() -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let cadi_storage = std::env::var("CADI_STORAGE").unwrap_or_else(|_| ".cadi-repo".to_string());
    let cadi_repo = std::path::PathBuf::from(&cadi_storage);
    
    let (entries, total_size_bytes) = if cadi_repo.exists() {
        let mut count = 0;
        let mut size = 0u64;
        if let Ok(dir_entries) = std::fs::read_dir(&cadi_repo) {
            for entry in dir_entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() && entry.path().extension().map(|e| e == "chunk").unwrap_or(false) {
                        count += 1;
                        size += metadata.len();
                    }
                }
            }
        }
        (count, size)
    } else {
        (0, 0)
    };
    
    let stats = json!({
        "entries": entries,
        "total_size_bytes": total_size_bytes,
        "hit_rate": 0.0,
        "last_gc": null,
        "storage_path": cadi_storage
    });
    
    Ok(vec![json!({
        "uri": "cadi://cache/stats",
        "mimeType": "application/json",
        "text": serde_json::to_string_pretty(&stats)?
    })])
}

async fn read_registries() -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let cadi_registry = std::env::var("CADI_REGISTRY").unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let registries = json!({
        "registries": [
            {
                "url": cadi_registry.clone(),
                "name": "primary",
                "priority": 100,
                "federation": true,
                "storage_path": ".cadi-repo"
            }
        ],
        "federation_enabled": true,
        "primary_registry": cadi_registry
    });
    
    Ok(vec![json!({
        "uri": "cadi://registry/list",
        "mimeType": "application/json",
        "text": serde_json::to_string_pretty(&registries)?
    })])
}

async fn read_trust_policy() -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let cadi_storage = std::env::var("CADI_STORAGE").unwrap_or_else(|_| ".cadi-repo".to_string());
    let trust_file = std::path::PathBuf::from(&cadi_storage).join("trust-policy.json");
    
    let policy = if trust_file.exists() {
        std::fs::read_to_string(&trust_file)
            .ok()
            .and_then(|content| serde_json::from_str::<Value>(&content).ok())
            .unwrap_or_else(|| json!({
                "version": "1.0.0",
                "trusted_sources": [],
                "signing_keys": [],
                "verification_required": false
            }))
    } else {
        json!({
            "version": "1.0.0",
            "trusted_sources": [],
            "signing_keys": [],
            "verification_required": false
        })
    };
    
    Ok(vec![json!({
        "uri": "cadi://trust/policy",
        "mimeType": "application/json",
        "text": serde_json::to_string_pretty(&policy)?
    })])
}

async fn read_chunk(chunk_id: &str) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let cadi_storage = std::env::var("CADI_STORAGE").unwrap_or_else(|_| ".cadi-repo".to_string());
    let cadi_repo = std::path::PathBuf::from(&cadi_storage);
    
    // Read metadata to validate chunk exists
    if let Ok(metadata_content) = std::fs::read_to_string(cadi_repo.join("metadata.json")) {
        if let Ok(metadata) = serde_json::from_str::<serde_json::Map<String, Value>>(&metadata_content) {
            if let Some(chunk_meta) = metadata.get(chunk_id) {
                let safe_name = chunk_id.replace(':', "_").replace('/', "_");
                let chunk_path = cadi_repo.join(format!("chunk_sha256_{}.chunk", safe_name));
                
                if let Ok(chunk_content) = std::fs::read_to_string(&chunk_path) {
                    let preview = chunk_content.chars().take(500).collect::<String>();
                    let chunk_info = json!({
                        "chunk_id": chunk_id,
                        "metadata": chunk_meta,
                        "content_preview": preview,
                        "path": chunk_path.to_string_lossy(),
                        "size": chunk_meta.get("size").and_then(|v| v.as_u64()).unwrap_or(0)
                    });
                    
                    return Ok(vec![json!({
                        "uri": format!("cadi://chunk/{}", chunk_id),
                        "mimeType": "application/json",
                        "text": serde_json::to_string_pretty(&chunk_info)?
                    })]);
                }
            }
        }
    }
    
    // Chunk not found - return error response
    Ok(vec![json!({
        "uri": format!("cadi://chunk/{}", chunk_id),
        "mimeType": "application/json",
        "text": serde_json::json!({"error": format!("Chunk {} not found", chunk_id)}).to_string()
    })])
}
