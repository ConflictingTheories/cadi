use crate::types::ScraperConfig;
use crate::error::Result;

/// Load configuration from a YAML file
pub fn load_config(path: &str) -> Result<ScraperConfig> {
    let content = std::fs::read_to_string(path)?;
    let config = serde_yaml::from_str(&content)?;
    Ok(config)
}

/// Load configuration from environment variables
pub fn load_from_env() -> ScraperConfig {
    let mut config = ScraperConfig::default();

    if let Ok(url) = std::env::var("CADI_REGISTRY_URL") {
        config.registry_url = Some(url);
    }

    if let Ok(token) = std::env::var("CADI_AUTH_TOKEN") {
        config.auth_token = Some(token);
    }

    if let Ok(namespace) = std::env::var("CADI_NAMESPACE") {
        config.namespace = Some(namespace);
    }

    if let Ok(strategy) = std::env::var("CADI_CHUNKING_STRATEGY") {
        config.chunking_strategy = match strategy.as_str() {
            "semantic" => crate::types::ChunkingStrategy::Semantic,
            "fixed-size" => crate::types::ChunkingStrategy::FixedSize,
            "hierarchical" => crate::types::ChunkingStrategy::Hierarchical,
            "by-line-count" => crate::types::ChunkingStrategy::ByLineCount,
            _ => crate::types::ChunkingStrategy::ByFile,
        };
    }

    config
}

/// Save configuration to a YAML file
pub fn save_config(config: &ScraperConfig, path: &str) -> Result<()> {
    let content = serde_yaml::to_string(config)?;
    std::fs::write(path, content)?;
    Ok(())
}
