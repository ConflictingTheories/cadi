//! Extension manifest handling

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

use crate::types::*;

/// Extension manifest (extension.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub extension: ExtensionInfo,
    pub dependencies: Option<HashMap<String, String>>,
    pub config: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: String,
    #[serde(rename = "type")]
    pub extension_type: String,
}

impl ExtensionManifest {
    /// Load manifest from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    /// Parse manifest from string
    pub fn from_str(content: &str) -> Result<Self> {
        toml::from_str(content).map_err(Into::into)
    }

    /// Convert to ExtensionMetadata
    pub fn to_metadata(&self) -> Result<ExtensionMetadata> {
        let extension_type = match self.extension.extension_type.as_str() {
            "atomizer" => ExtensionType::Atomizer,
            "build-backend" => ExtensionType::BuildBackend,
            "registry" => ExtensionType::Registry,
            "mcp-tool" => ExtensionType::McpTool,
            "ui" => ExtensionType::Ui,
            other => return Err(ExtensionError::InvalidManifest(
                format!("Unknown extension type: {}", other)
            )),
        };

        Ok(ExtensionMetadata {
            id: ExtensionId::new(),
            name: self.extension.name.clone(),
            version: self.extension.version.clone(),
            description: self.extension.description.clone(),
            author: self.extension.author.clone(),
            homepage: self.extension.homepage.clone(),
            repository: self.extension.repository.clone(),
            license: self.extension.license.clone(),
            extension_type,
        })
    }
}

/// Validate an extension manifest
pub fn validate_manifest(manifest: &ExtensionManifest) -> Result<()> {
    // Validate required fields
    if manifest.extension.name.is_empty() {
        return Err(ExtensionError::InvalidManifest("Extension name cannot be empty".into()));
    }

    if manifest.extension.description.is_empty() {
        return Err(ExtensionError::InvalidManifest("Extension description cannot be empty".into()));
    }

    // Basic version validation (could be improved)
    if manifest.extension.version.is_empty() {
        return Err(ExtensionError::InvalidManifest("Extension version cannot be empty".into()));
    }

    // Validate extension type
    match manifest.extension.extension_type.as_str() {
        "atomizer" | "build-backend" | "registry" | "mcp-tool" | "ui" => {}
        other => return Err(ExtensionError::InvalidManifest(
            format!("Unknown extension type: {}", other)
        )),
    }

    Ok(())
}