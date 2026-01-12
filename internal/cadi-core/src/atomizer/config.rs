//! Atomizer configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the atomizer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomizerConfig {
    /// Minimum lines for a function to be its own atom
    #[serde(default = "default_min_function_lines")]
    pub min_function_lines: usize,

    /// Minimum lines for a file to be split into atoms
    #[serde(default = "default_min_file_lines")]
    pub min_file_lines_to_split: usize,

    /// Maximum lines per atom before forcing a split
    #[serde(default = "default_max_atom_lines")]
    pub max_atom_lines: usize,

    /// Whether to extract doc comments as part of the atom
    #[serde(default = "default_true")]
    pub include_doc_comments: bool,

    /// Whether to include type definitions referenced by functions
    #[serde(default = "default_true")]
    pub include_type_context: bool,

    /// Depth of dependency resolution
    #[serde(default = "default_resolution_depth")]
    pub resolution_depth: usize,

    /// Language-specific configurations
    #[serde(default)]
    pub languages: HashMap<String, LanguageConfig>,

    /// Namespace for generated aliases
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

fn default_min_function_lines() -> usize { 5 }
fn default_min_file_lines() -> usize { 30 }
fn default_max_atom_lines() -> usize { 300 }
fn default_resolution_depth() -> usize { 2 }
fn default_true() -> bool { true }

impl Default for AtomizerConfig {
    fn default() -> Self {
        let mut languages = HashMap::new();
        // Common web & systems languages enabled by default
        languages.insert("tsx".to_string(), LanguageConfig { extensions: vec!["tsx".to_string()], enabled: true, ..Default::default() });
        languages.insert("jsx".to_string(), LanguageConfig { extensions: vec!["jsx".to_string()], enabled: true, ..Default::default() });
        languages.insert("html".to_string(), LanguageConfig { extensions: vec!["html".to_string(), "htm".to_string()], enabled: true, ..Default::default() });
        languages.insert("css".to_string(), LanguageConfig { extensions: vec!["css".to_string()], enabled: true, ..Default::default() });
        languages.insert("c".to_string(), LanguageConfig { extensions: vec!["c".to_string()], enabled: true, ..Default::default() });
        languages.insert("cpp".to_string(), LanguageConfig { extensions: vec!["cpp".to_string(), "cc".to_string(), "cxx".to_string()], enabled: true, ..Default::default() });
        languages.insert("csharp".to_string(), LanguageConfig { extensions: vec!["cs".to_string()], enabled: true, ..Default::default() });
        languages.insert("glsl".to_string(), LanguageConfig { extensions: vec!["glsl".to_string()], enabled: true, ..Default::default() });
        languages.insert("wgsl".to_string(), LanguageConfig { extensions: vec!["wgsl".to_string()], enabled: true, ..Default::default() });

        Self {
            min_function_lines: default_min_function_lines(),
            min_file_lines_to_split: default_min_file_lines(),
            max_atom_lines: default_max_atom_lines(),
            include_doc_comments: true,
            include_type_context: true,
            resolution_depth: default_resolution_depth(),
            languages,
            namespace: None,
        }
    }
}

impl AtomizerConfig {
    /// Create a config optimized for minimal atoms
    pub fn minimal() -> Self {
        Self {
            min_function_lines: 1,
            min_file_lines_to_split: 10,
            max_atom_lines: 100,
            ..Default::default()
        }
    }

    /// Create a config for coarse-grained atoms
    pub fn coarse() -> Self {
        Self {
            min_function_lines: 20,
            min_file_lines_to_split: 100,
            max_atom_lines: 500,
            ..Default::default()
        }
    }

    /// Set the namespace
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }
}

/// Language-specific atomizer configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// File extensions for this language
    #[serde(default)]
    pub extensions: Vec<String>,

    /// Whether to treat this language specially
    #[serde(default)]
    pub enabled: bool,

    /// Custom atom boundaries (e.g., specific decorators in Python)
    #[serde(default)]
    pub custom_boundaries: Vec<String>,

    /// Symbols to always include in context
    #[serde(default)]
    pub always_include: Vec<String>,

    /// Patterns to ignore
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
}

impl LanguageConfig {
    pub fn rust() -> Self {
        Self {
            extensions: vec!["rs".to_string()],
            enabled: true,
            custom_boundaries: vec![
                "#[test]".to_string(),
                "#[cfg(test)]".to_string(),
            ],
            always_include: vec![],
            ignore_patterns: vec![],
        }
    }

    pub fn typescript() -> Self {
        Self {
            extensions: vec![
                "ts".to_string(),
                "tsx".to_string(),
                "js".to_string(),
                "jsx".to_string(),
            ],
            enabled: true,
            custom_boundaries: vec![
                "export default".to_string(),
                "export function".to_string(),
                "export class".to_string(),
            ],
            always_include: vec![],
            ignore_patterns: vec![],
        }
    }

    pub fn python() -> Self {
        Self {
            extensions: vec!["py".to_string(), "pyi".to_string()],
            enabled: true,
            custom_boundaries: vec![
                "def ".to_string(),
                "class ".to_string(),
                "@".to_string(), // decorators
            ],
            always_include: vec![],
            ignore_patterns: vec!["__pycache__".to_string()],
        }
    }

    pub fn tsx() -> Self {
        Self {
            extensions: vec!["tsx".to_string()],
            enabled: true,
            ..Default::default()
        }
    }

    pub fn jsx() -> Self {
        Self {
            extensions: vec!["jsx".to_string()],
            enabled: true,
            ..Default::default()
        }
    }

    pub fn html() -> Self {
        Self {
            extensions: vec!["html".to_string(), "htm".to_string()],
            enabled: true,
            ..Default::default()
        }
    }

    pub fn css() -> Self {
        Self {
            extensions: vec!["css".to_string()],
            enabled: true,
            ..Default::default()
        }
    }

    pub fn c() -> Self {
        Self {
            extensions: vec!["c".to_string()],
            enabled: true,
            ..Default::default()
        }
    }

    pub fn cpp() -> Self {
        Self {
            extensions: vec!["cpp".to_string(), "cc".to_string(), "cxx".to_string()],
            enabled: true,
            ..Default::default()
        }
    }

    pub fn csharp() -> Self {
        Self {
            extensions: vec!["cs".to_string()],
            enabled: true,
            ..Default::default()
        }
    }

    pub fn glsl() -> Self {
        Self {
            extensions: vec!["glsl".to_string()],
            enabled: true,
            ..Default::default()
        }
    }

    pub fn wgsl() -> Self {
        Self {
            extensions: vec!["wgsl".to_string()],
            enabled: true,
            ..Default::default()
        }
    }
}
