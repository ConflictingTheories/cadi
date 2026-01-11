//! MCP Resources for CADI

use crate::protocol::ResourceDefinition;
use serde_json::{json, Value};

/// Get all available resources
pub fn get_resources() -> Vec<ResourceDefinition> {
    vec![
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

async fn read_config() -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    // Placeholder - would load actual config
    let config = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "registry": {
            "primary": "https://registry.cadi.dev",
            "cache_enabled": true
        },
        "build": {
            "parallel_jobs": 4,
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
    // Placeholder - would read actual stats
    let stats = json!({
        "entries": 0,
        "total_size_bytes": 0,
        "hit_rate": 0.0,
        "last_gc": null
    });
    
    Ok(vec![json!({
        "uri": "cadi://cache/stats",
        "mimeType": "application/json",
        "text": serde_json::to_string_pretty(&stats)?
    })])
}

async fn read_registries() -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    // Placeholder - would read actual registry config
    let registries = json!([
        {
            "id": "primary",
            "url": "https://registry.cadi.dev",
            "priority": 0,
            "trust_level": "verified",
            "enabled": true
        }
    ]);
    
    Ok(vec![json!({
        "uri": "cadi://registries",
        "mimeType": "application/json",
        "text": serde_json::to_string_pretty(&registries)?
    })])
}

async fn read_trust_policy() -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    // Placeholder - would read actual trust policy
    let policy = json!({
        "default_action": "verify",
        "trusted_publishers": [],
        "required_attestations": [],
        "allow_unsigned": false
    });
    
    Ok(vec![json!({
        "uri": "cadi://trust/policy",
        "mimeType": "application/json",
        "text": serde_json::to_string_pretty(&policy)?
    })])
}

async fn read_chunk(chunk_id: &str) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    // Placeholder - would fetch actual chunk
    Ok(vec![json!({
        "uri": format!("cadi://chunk/{}", chunk_id),
        "mimeType": "application/json",
        "text": format!("Chunk {} not found (placeholder implementation)", chunk_id)
    })])
}
