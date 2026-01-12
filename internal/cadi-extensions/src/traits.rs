//! Core traits for CADI extensions

use async_trait::async_trait;
use cadi_core::{AtomicChunk, atomizer::ResolvedImport};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::*;

/// Simple search query for registry operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub limit: Option<usize>,
}

/// Build artifact result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildArtifact {
    pub id: String,
    pub content_type: String,
    pub data: Vec<u8>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Base trait that all extensions must implement
#[async_trait]
pub trait Extension: Send + Sync {
    /// Get metadata about this extension
    fn metadata(&self) -> ExtensionMetadata;

    /// Initialize the extension with context
    async fn initialize(&mut self, context: &ExtensionContext) -> Result<()>;

    /// Shutdown the extension
    async fn shutdown(&mut self) -> Result<()>;
}

/// Extension for atomizing new programming languages
#[async_trait]
pub trait AtomizerExtension: Extension {
    /// The programming language this atomizer handles
    fn language(&self) -> &str;

    /// Extract atomic chunks from source code
    async fn extract_atoms(&self, source: &str) -> Result<Vec<AtomicChunk>>;

    /// Resolve imports in source code
    async fn resolve_imports(&self, source: &str) -> Result<Vec<ResolvedImport>>;
}

/// Extension for custom build backends
#[async_trait]
pub trait BuildBackendExtension: Extension {
    /// List supported target formats
    fn target_formats(&self) -> Vec<&str>;

    /// Build a chunk for a specific target
    async fn build(&self, chunk: &AtomicChunk, target: &str) -> Result<BuildArtifact>;
}

/// Extension for custom registry backends
#[async_trait]
pub trait RegistryExtension: Extension {
    /// Store a chunk in the registry
    async fn store(&self, chunk: &AtomicChunk) -> Result<String>;

    /// Retrieve a chunk by ID
    async fn retrieve(&self, id: &str) -> Result<Option<AtomicChunk>>;

    /// Search for chunks
    async fn search(&self, query: &SearchQuery) -> Result<Vec<AtomicChunk>>;
}

/// Extension for MCP (Model Context Protocol) tools
#[async_trait]
pub trait McpToolExtension: Extension {
    /// Get available tools
    fn tools(&self) -> Vec<McpToolDefinition>;

    /// Call a tool with arguments
    async fn call_tool(&self, name: &str, args: Value) -> Result<Vec<Value>>;
}

/// Definition of an MCP tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Value,
}

/// Extension for UI integrations
#[async_trait]
pub trait UiExtension: Extension {
    /// Get available UI views
    fn views(&self) -> Vec<UiViewDefinition>;

    /// Get available commands
    fn commands(&self) -> Vec<UiCommandDefinition>;

    /// Handle a UI event
    async fn handle_event(&self, event: UiEvent) -> Result<UiResponse>;
}

/// Definition of a UI view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiViewDefinition {
    pub id: String,
    pub title: String,
    pub icon: Option<String>,
    pub component: String, // HTML/React component name
}

/// Definition of a UI command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiCommandDefinition {
    pub id: String,
    pub title: String,
    pub keybinding: Option<String>,
}

/// UI event from the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiEvent {
    pub event_type: String,
    pub payload: Value,
}

/// Response to a UI event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiResponse {
    pub actions: Vec<UiAction>,
}

/// Action to perform in the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiAction {
    UpdateView { view_id: String, data: Value },
    ShowNotification { message: String, level: String },
    OpenUrl { url: String },
    ExecuteCommand { command: String, args: Vec<String> },
}