//! View configuration

use serde::{Deserialize, Serialize};

/// Configuration for creating virtual views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewConfig {
    /// Maximum tokens to include
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,

    /// Depth of automatic dependency expansion (Ghost Imports)
    #[serde(default = "default_expansion_depth")]
    pub expansion_depth: usize,

    /// Include doc comments
    #[serde(default = "default_true")]
    pub include_docs: bool,

    /// Include type definitions for referenced types
    #[serde(default = "default_true")]
    pub include_types: bool,

    /// Format for the view output
    #[serde(default)]
    pub format: ViewFormat,

    /// Add separator comments between atoms
    #[serde(default = "default_true")]
    pub add_separators: bool,

    /// Sort atoms by type (imports first, then types, then functions)
    #[serde(default = "default_true")]
    pub sort_by_type: bool,

    /// Deduplicate atoms that appear multiple times
    #[serde(default = "default_true")]
    pub deduplicate: bool,
}

fn default_max_tokens() -> usize { 8000 }
fn default_expansion_depth() -> usize { 1 }
fn default_true() -> bool { true }

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            max_tokens: default_max_tokens(),
            expansion_depth: default_expansion_depth(),
            include_docs: true,
            include_types: true,
            format: ViewFormat::Source,
            add_separators: true,
            sort_by_type: true,
            deduplicate: true,
        }
    }
}

impl ViewConfig {
    /// Create a minimal config (no expansion, compact output)
    pub fn minimal() -> Self {
        Self {
            max_tokens: 2000,
            expansion_depth: 0,
            include_docs: false,
            include_types: false,
            format: ViewFormat::Minimal,
            add_separators: false,
            sort_by_type: false,
            deduplicate: true,
        }
    }

    /// Create a config for documentation purposes
    pub fn documented() -> Self {
        Self {
            max_tokens: 16000,
            expansion_depth: 2,
            include_docs: true,
            include_types: true,
            format: ViewFormat::Documented,
            add_separators: true,
            sort_by_type: true,
            deduplicate: true,
        }
    }

    /// Set maximum tokens
    pub fn with_max_tokens(mut self, tokens: usize) -> Self {
        self.max_tokens = tokens;
        self
    }

    /// Set expansion depth
    pub fn with_expansion(mut self, depth: usize) -> Self {
        self.expansion_depth = depth;
        self
    }

    /// Disable Ghost Imports
    pub fn no_expansion(mut self) -> Self {
        self.expansion_depth = 0;
        self
    }
}

/// Output format for views
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewFormat {
    /// Full source code
    #[default]
    Source,
    /// Minimal (no comments, compact)
    Minimal,
    /// With documentation
    Documented,
    /// Type signatures only (no function bodies)
    Signatures,
    /// JSON representation
    Json,
}
