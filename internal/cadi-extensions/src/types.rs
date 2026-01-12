//! Core types for CADI extensions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::traits::Extension;

/// Unique identifier for an extension
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExtensionId(pub Uuid);

impl ExtensionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Metadata for an extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionMetadata {
    pub id: ExtensionId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: String,
    pub extension_type: ExtensionType,
}

/// Type of extension
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtensionType {
    Atomizer,
    BuildBackend,
    Registry,
    McpTool,
    Ui,
}

impl std::fmt::Display for ExtensionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtensionType::Atomizer => write!(f, "atomizer"),
            ExtensionType::BuildBackend => write!(f, "build-backend"),
            ExtensionType::Registry => write!(f, "registry"),
            ExtensionType::McpTool => write!(f, "mcp-tool"),
            ExtensionType::Ui => write!(f, "ui"),
        }
    }
}

/// Context passed to extensions during initialization
pub struct ExtensionContext {
    pub config: HashMap<String, serde_json::Value>,
    pub registry: Box<dyn crate::traits::RegistryExtension>,
}

/// Status of an extension
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionStatus {
    Unloaded,
    Loading,
    Loaded,
    Initializing,
    Active,
    Error,
    Disabled,
}

/// Information about a loaded extension
pub struct LoadedExtension {
    pub metadata: ExtensionMetadata,
    pub status: ExtensionStatus,
    pub instance: Box<dyn Extension>,
}

/// Search query for finding extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionQuery {
    pub name: Option<String>,
    pub extension_type: Option<ExtensionType>,
    pub tags: Vec<String>,
    pub min_version: Option<String>,
}

/// Result of searching for extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionSearchResult {
    pub extensions: Vec<ExtensionMetadata>,
    pub total_count: usize,
    pub page: usize,
    pub page_size: usize,
}

/// Error types for extension operations
#[derive(Debug, thiserror::Error)]
pub enum ExtensionError {
    #[error("Extension not found: {0}")]
    NotFound(String),

    #[error("Extension already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid extension manifest: {0}")]
    InvalidManifest(String),

    #[error("Extension loading failed: {0}")]
    LoadFailed(String),

    #[error("Extension initialization failed: {0}")]
    InitFailed(String),

    #[error("Extension type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        expected: ExtensionType,
        actual: ExtensionType,
    },

    #[error("Version requirement not satisfied: {0}")]
    VersionMismatch(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Library loading error: {0}")]
    LibLoading(#[from] libloading::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, ExtensionError>;