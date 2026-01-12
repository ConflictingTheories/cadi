use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for the scraper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScraperConfig {
    /// Registry URL for publishing chunks
    pub registry_url: Option<String>,

    /// Authentication token for registry
    pub auth_token: Option<String>,

    /// Namespace for published chunks
    pub namespace: Option<String>,

    /// Chunking strategy to use
    pub chunking_strategy: ChunkingStrategy,

    /// Maximum chunk size in bytes
    pub max_chunk_size: usize,

    /// Include overlapping context between chunks
    pub include_overlap: bool,

    /// Overlap size in characters
    pub overlap_size: usize,

    /// Language-specific parsing options
    pub language_options: HashMap<String, LanguageConfig>,

    /// Patterns to exclude from scraping
    pub exclude_patterns: Vec<String>,

    /// Whether to create hierarchical chunk relationships
    pub create_hierarchy: bool,

    /// Auto-extract API surfaces and functions
    pub extract_api_surface: bool,

    /// Whether to auto-detect licenses
    pub detect_licenses: bool,

    /// HTTP request timeout in seconds
    pub request_timeout: u64,

    /// Rate limit: requests per second
    pub rate_limit: f64,

    /// Local cache directory
    pub cache_dir: Option<PathBuf>,
}

/// Language-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// Minimum chunk size for semantic boundaries
    pub min_semantic_size: usize,

    /// Whether to split by function/class boundaries
    pub split_by_semantic_boundary: bool,

    /// Extract function signatures
    pub extract_functions: bool,

    /// Extract type definitions
    pub extract_types: bool,

    /// Extract class/struct definitions
    pub extract_classes: bool,
}

/// Chunking strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum ChunkingStrategy {
    /// Chunk by individual file
    #[default]
    ByFile,

    /// Chunk by semantic boundaries (functions, classes)
    Semantic,

    /// Chunk by fixed size
    FixedSize,

    /// Recursive hierarchical chunking
    Hierarchical,

    /// Chunk by line count
    ByLineCount,
}


impl Default for ScraperConfig {
    fn default() -> Self {
        let mut language_options = HashMap::new();
        
        // Default configs for common languages
        for lang in &["rust", "typescript", "python", "javascript", "go", "c"] {
            language_options.insert(
                lang.to_string(),
                LanguageConfig {
                    min_semantic_size: 100,
                    split_by_semantic_boundary: true,
                    extract_functions: true,
                    extract_types: true,
                    extract_classes: true,
                },
            );
        }

        Self {
            registry_url: None,
            auth_token: None,
            namespace: None,
            chunking_strategy: ChunkingStrategy::ByFile,
            max_chunk_size: 50 * 1024 * 1024, // 50MB default
            include_overlap: true,
            overlap_size: 500,
            language_options,
            exclude_patterns: vec![
                "**/.git".to_string(),
                "**/node_modules".to_string(),
                "**/target".to_string(),
                "**/.venv".to_string(),
                "**/dist".to_string(),
                "**/build".to_string(),
                "**/.DS_Store".to_string(),
            ],
            create_hierarchy: true,
            extract_api_surface: true,
            detect_licenses: true,
            request_timeout: 30,
            rate_limit: 10.0,
            cache_dir: None,
        }
    }
}

/// Input to the scraper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScraperInput {
    /// Local file path
    LocalPath(PathBuf),

    /// HTTP(S) URL
    Url(String),

    /// Git repository URL
    GitRepo {
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        commit: Option<String>,
    },

    /// Directory path with optional filters
    Directory {
        path: PathBuf,
        #[serde(skip_serializing_if = "Option::is_none")]
        patterns: Option<Vec<String>>,
    },
}

/// Output from the scraper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScraperOutput {
    /// Total chunks created
    pub chunk_count: usize,

    /// Total files processed
    pub file_count: usize,

    /// Total bytes processed
    pub total_bytes: u64,

    /// Chunks with their metadata
    pub chunks: Vec<ScrapedChunk>,

    /// Manifest for all chunks
    pub manifest: Option<serde_json::Value>,

    /// Errors encountered during scraping
    pub errors: Vec<String>,

    /// Time taken (milliseconds)
    pub duration_ms: u128,
}

/// A single scraped chunk with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedChunk {
    /// Chunk ID (content hash)
    pub chunk_id: String,

    /// Chunk type (source-cadi, etc)
    pub cadi_type: String,

    /// Human-readable name
    pub name: String,

    /// Description
    pub description: Option<String>,

    /// Source path or URL
    pub source: String,

    /// Content hash
    pub content_hash: String,

    /// File size
    pub size: usize,

    /// Language/format detected
    pub language: Option<String>,

    /// Detected concepts/tags
    pub concepts: Vec<String>,

    /// Detected dependencies
    pub dependencies: Vec<String>,

    /// License if detected
    pub license: Option<String>,

    /// Parent chunk ID for hierarchical relationships
    pub parent_chunk_id: Option<String>,

    /// Child chunk IDs
    pub child_chunk_ids: Vec<String>,

    /// Metadata tags
    pub tags: Vec<String>,

    /// Timestamp when scraped
    pub scraped_at: String,
}
