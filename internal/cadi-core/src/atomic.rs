//! Atomic Chunk System
//!
//! This module provides the AtomicChunk type - a smart, reusable code unit with:
//! - Human-readable aliases for easy reference
//! - Automatic categorization and tagging
//! - Platform/architecture constraints
//! - Composition support (chunks made of other chunks)
//! - Granularity levels for different reuse patterns

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Granularity level of an atomic chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ChunkGranularity {
    /// Single function, method, or small utility
    Function,
    /// A class, struct, trait, or interface
    Type,
    /// A complete module or file
    #[default]
    Module,
    /// A package or crate (multiple modules)
    Package,
    /// An entire project or workspace
    Project,
}


/// Category of code chunk
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ChunkCategory {
    /// Core logic and business rules
    #[default]
    Logic,
    /// Data types and structures
    Data,
    /// Utility and helper functions
    Utility,
    /// API and interface definitions
    Api,
    /// Configuration and constants
    Config,
    /// Tests and test utilities
    Test,
    /// Documentation
    Docs,
    /// Build and tooling scripts
    Build,
    /// UI/Frontend components
    Ui,
    /// Backend/Server code
    Backend,
    /// Database and persistence
    Database,
    /// Custom category
    Custom(String),
}


/// Platform constraint for a chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConstraint {
    /// Target OS (any, linux, macos, windows, etc.)
    #[serde(default = "default_any")]
    pub os: String,
    /// Target architecture (any, x86_64, aarch64, wasm32, etc.)
    #[serde(default = "default_any")]
    pub arch: String,
    /// Required runtime/environment (node, browser, native, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    /// Minimum version requirements
    #[serde(default)]
    pub min_versions: HashMap<String, String>,
}

fn default_any() -> String {
    "any".to_string()
}

impl Default for PlatformConstraint {
    fn default() -> Self {
        Self {
            os: "any".to_string(),
            arch: "any".to_string(),
            runtime: None,
            min_versions: HashMap::new(),
        }
    }
}

/// An alias for a chunk - human readable path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkAlias {
    /// The alias path (e.g., "utils/string-helpers")
    pub path: String,
    /// Namespace/scope (e.g., "my-org")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    /// Whether this is the primary/canonical alias
    #[serde(default)]
    pub primary: bool,
    /// When this alias was registered
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registered_at: Option<String>,
}

impl ChunkAlias {
    /// Create a new primary alias
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            namespace: None,
            primary: true,
            registered_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    /// With namespace
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Full qualified path
    pub fn full_path(&self) -> String {
        match &self.namespace {
            Some(ns) => format!("{}/{}", ns, self.path),
            None => self.path.clone(),
        }
    }
}

/// Reference to another chunk (for composition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkReference {
    /// The chunk ID being referenced
    pub chunk_id: String,
    /// Optional alias for easier reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// Whether this is required (vs optional)
    #[serde(default = "default_true")]
    pub required: bool,
    /// Specific items imported from this chunk
    #[serde(default)]
    pub imports: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// Composition information - how this chunk relates to others
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkComposition {
    /// Chunks that this chunk is composed of (dependencies)
    #[serde(default)]
    pub composed_of: Vec<ChunkReference>,
    /// Chunks that compose/include this chunk
    #[serde(default)]
    pub composed_by: Vec<String>,
    /// Whether this chunk is atomic (not composed of other CADI chunks)
    #[serde(default = "default_true")]
    pub is_atomic: bool,
    /// If composed, the strategy used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub composition_strategy: Option<String>,
}

impl Default for ChunkComposition {
    fn default() -> Self {
        Self {
            composed_of: Vec::new(),
            composed_by: Vec::new(),
            is_atomic: true,
            composition_strategy: None,
        }
    }
}

/// Quality and reusability metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChunkMetrics {
    /// Lines of code
    #[serde(default)]
    pub loc: usize,
    /// Cyclomatic complexity estimate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<f32>,
    /// Reusability score (0-1, higher = more reusable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reusability_score: Option<f32>,
    /// Number of public exports
    #[serde(default)]
    pub export_count: usize,
    /// Number of dependencies
    #[serde(default)]
    pub dependency_count: usize,
    /// Coupling score (0-1, lower = better)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coupling: Option<f32>,
}

/// Source location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Original file path (relative)
    pub file: String,
    /// Start line (1-indexed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<usize>,
    /// End line (1-indexed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,
    /// Start column
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_col: Option<usize>,
    /// End column
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_col: Option<usize>,
}

/// An atomic chunk - the fundamental unit of reusable code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicChunk {
    /// Content-addressed chunk ID (chunk:sha256:...)
    pub chunk_id: String,

    /// Human-readable aliases for this chunk
    #[serde(default)]
    pub aliases: Vec<ChunkAlias>,

    /// Name of the chunk
    pub name: String,

    /// Description of what this chunk does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Primary language
    pub language: String,

    /// Granularity level
    #[serde(default)]
    pub granularity: ChunkGranularity,

    /// Categories this chunk belongs to
    #[serde(default)]
    pub categories: Vec<ChunkCategory>,

    /// Tags for search and discovery
    #[serde(default)]
    pub tags: Vec<String>,

    /// Concepts this chunk implements
    #[serde(default)]
    pub concepts: Vec<String>,

    /// Interfaces/APIs provided
    #[serde(default)]
    pub provides: Vec<String>,

    /// Interfaces/APIs required
    #[serde(default)]
    pub requires: Vec<String>,

    /// Platform constraints
    #[serde(default)]
    pub platform: PlatformConstraint,

    /// Composition information
    #[serde(default)]
    pub composition: ChunkComposition,

    /// Quality metrics
    #[serde(default)]
    pub metrics: ChunkMetrics,

    /// Source location(s)
    #[serde(default)]
    pub sources: Vec<SourceLocation>,

    /// Content hash (sha256)
    pub content_hash: String,

    /// Size in bytes
    pub size: usize,

    /// License
    #[serde(default = "default_license")]
    pub license: String,

    /// Creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// Version if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

fn default_license() -> String {
    "MIT".to_string()
}

impl AtomicChunk {
    /// Create a new atomic chunk
    pub fn new(
        chunk_id: String,
        name: String,
        language: String,
        content_hash: String,
        size: usize,
    ) -> Self {
        Self {
            chunk_id,
            aliases: Vec::new(),
            name,
            description: None,
            language,
            granularity: ChunkGranularity::default(),
            categories: Vec::new(),
            tags: Vec::new(),
            concepts: Vec::new(),
            provides: Vec::new(),
            requires: Vec::new(),
            platform: PlatformConstraint::default(),
            composition: ChunkComposition::default(),
            metrics: ChunkMetrics::default(),
            sources: Vec::new(),
            content_hash,
            size,
            license: "MIT".to_string(),
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            version: None,
        }
    }

    /// Add a primary alias
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(ChunkAlias::new(alias));
        self
    }

    /// Add multiple aliases
    pub fn with_aliases(mut self, aliases: Vec<String>) -> Self {
        for (i, alias) in aliases.into_iter().enumerate() {
            let mut a = ChunkAlias::new(alias);
            a.primary = i == 0;
            self.aliases.push(a);
        }
        self
    }

    /// Set categories
    pub fn with_categories(mut self, categories: Vec<ChunkCategory>) -> Self {
        self.categories = categories;
        self
    }

    /// Set granularity
    pub fn with_granularity(mut self, granularity: ChunkGranularity) -> Self {
        self.granularity = granularity;
        self
    }

    /// Add concepts
    pub fn with_concepts(mut self, concepts: Vec<String>) -> Self {
        self.concepts = concepts;
        self
    }

    /// Mark as composed of other chunks
    pub fn composed_of(mut self, chunks: Vec<ChunkReference>) -> Self {
        let is_empty = chunks.is_empty();
        self.composition.composed_of = chunks;
        self.composition.is_atomic = is_empty;
        self
    }

    /// Get the primary alias, if any
    pub fn primary_alias(&self) -> Option<&ChunkAlias> {
        self.aliases.iter().find(|a| a.primary)
    }

    /// Get the display name (alias or chunk name)
    pub fn display_name(&self) -> String {
        self.primary_alias()
            .map(|a| a.full_path())
            .unwrap_or_else(|| self.name.clone())
    }

    /// Check if chunk is atomic (not composed of other chunks)
    pub fn is_atomic(&self) -> bool {
        self.composition.is_atomic && self.composition.composed_of.is_empty()
    }
}

/// Alias registry for tracking used aliases
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AliasRegistry {
    /// Map of alias path -> chunk_id
    pub aliases: HashMap<String, String>,
    /// Map of chunk_id -> list of aliases
    pub chunks: HashMap<String, Vec<String>>,
    /// Reserved aliases that cannot be used
    #[serde(default)]
    pub reserved: Vec<String>,
}

impl AliasRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if an alias is available
    pub fn is_available(&self, alias: &str) -> bool {
        !self.aliases.contains_key(alias) && !self.reserved.contains(&alias.to_string())
    }

    /// Register an alias for a chunk
    pub fn register(&mut self, alias: impl Into<String>, chunk_id: impl Into<String>) -> bool {
        let alias = alias.into();
        let chunk_id = chunk_id.into();

        if !self.is_available(&alias) {
            return false;
        }

        self.aliases.insert(alias.clone(), chunk_id.clone());
        self.chunks
            .entry(chunk_id)
            .or_default()
            .push(alias);

        true
    }

    /// Resolve an alias to a chunk ID
    pub fn resolve(&self, alias: &str) -> Option<&String> {
        self.aliases.get(alias)
    }

    /// Get all aliases for a chunk
    pub fn get_aliases(&self, chunk_id: &str) -> Option<&Vec<String>> {
        self.chunks.get(chunk_id)
    }

    /// Generate a unique alias from a base name
    pub fn generate_unique(&self, base: &str) -> String {
        if self.is_available(base) {
            return base.to_string();
        }

        let mut counter = 1;
        loop {
            let candidate = format!("{}-{}", base, counter);
            if self.is_available(&candidate) {
                return candidate;
            }
            counter += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_alias() {
        let alias = ChunkAlias::new("utils/string-helpers")
            .with_namespace("my-org");
        
        assert_eq!(alias.full_path(), "my-org/utils/string-helpers");
        assert!(alias.primary);
    }

    #[test]
    fn test_alias_registry() {
        let mut registry = AliasRegistry::new();
        
        assert!(registry.is_available("test/chunk"));
        assert!(registry.register("test/chunk", "chunk:sha256:abc123"));
        assert!(!registry.is_available("test/chunk"));
        
        let resolved = registry.resolve("test/chunk");
        assert_eq!(resolved, Some(&"chunk:sha256:abc123".to_string()));
    }

    #[test]
    fn test_atomic_chunk() {
        let chunk = AtomicChunk::new(
            "chunk:sha256:abc123".to_string(),
            "string-helpers".to_string(),
            "rust".to_string(),
            "abc123".to_string(),
            1024,
        )
        .with_alias("utils/string-helpers")
        .with_granularity(ChunkGranularity::Module)
        .with_categories(vec![ChunkCategory::Utility]);

        // Ensure composition defaults to atomic
        assert!(ChunkComposition::default().is_atomic);

        assert!(chunk.is_atomic());
        assert_eq!(chunk.display_name(), "utils/string-helpers");
    }
}
