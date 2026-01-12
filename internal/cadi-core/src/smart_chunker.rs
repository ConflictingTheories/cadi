//! Smart Chunker - Intelligent code analysis and chunking
//!
//! The SmartChunker analyzes code to determine the optimal way to atomize it:
//! - Identifies natural boundaries (functions, types, modules)
//! - Detects reusability patterns
//! - Finds shared utilities that should be extracted
//! - Determines if code should be one chunk or many
//! - Handles composition detection

use crate::atomic::{
    AtomicChunk, ChunkCategory, ChunkGranularity,
    ChunkMetrics, SourceLocation,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Configuration for smart chunking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartChunkerConfig {
    /// Minimum lines for a function to be its own chunk
    #[serde(default = "default_min_function_lines")]
    pub min_function_lines: usize,

    /// Minimum lines for a file to be chunked vs kept atomic
    #[serde(default = "default_min_file_lines_to_split")]
    pub min_file_lines_to_split: usize,

    /// Maximum lines before forcing a split
    #[serde(default = "default_max_chunk_lines")]
    pub max_chunk_lines: usize,

    /// Whether to extract utility functions as separate chunks
    #[serde(default = "default_true")]
    pub extract_utilities: bool,

    /// Whether to create type-level chunks (structs, classes)
    #[serde(default = "default_true")]
    pub extract_types: bool,

    /// Whether to group related functions together
    #[serde(default = "default_true")]
    pub group_related: bool,

    /// Prefer atomic (single) chunks when possible
    #[serde(default)]
    pub prefer_atomic: bool,

    /// Namespace for generated aliases
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

fn default_min_function_lines() -> usize {
    10
}

fn default_min_file_lines_to_split() -> usize {
    50
}

fn default_max_chunk_lines() -> usize {
    500
}

fn default_true() -> bool {
    true
}

impl Default for SmartChunkerConfig {
    fn default() -> Self {
        Self {
            min_function_lines: default_min_function_lines(),
            min_file_lines_to_split: default_min_file_lines_to_split(),
            max_chunk_lines: default_max_chunk_lines(),
            extract_utilities: true,
            extract_types: true,
            group_related: true,
            prefer_atomic: false,
            namespace: None,
        }
    }
}

/// A detected code entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEntity {
    pub name: String,
    pub kind: EntityKind,
    pub start_line: usize,
    pub end_line: usize,
    pub visibility: Visibility,
    pub doc_comment: Option<String>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub calls: Vec<String>,
    pub complexity: u32,
}

/// Kind of code entity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityKind {
    Function,
    AsyncFunction,
    Method,
    Struct,
    Class,
    Trait,
    Interface,
    Enum,
    Constant,
    Module,
    Type,
    Test,
    Macro,
}

/// Visibility of entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Internal,
}

/// Analysis result for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalysis {
    pub path: PathBuf,
    pub language: String,
    pub total_lines: usize,
    pub entities: Vec<CodeEntity>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub is_entrypoint: bool,
    pub is_test: bool,
    pub is_config: bool,
    pub framework_hints: Vec<String>,
    pub category: ChunkCategory,
}

/// Result of smart chunking analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingDecision {
    pub file_path: PathBuf,
    pub strategy: ChunkingStrategy,
    pub suggested_chunks: Vec<SuggestedChunk>,
    pub reasoning: String,
}

/// How to chunk a file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkingStrategy {
    /// Keep as single atomic chunk
    Atomic,
    /// Split by entities (functions, types)
    ByEntity,
    /// Split by logical sections
    BySections,
    /// Hierarchical (file + entity chunks)
    Hierarchical,
    /// Skip this file (not useful as a chunk)
    Skip,
}

/// A suggested chunk from analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedChunk {
    pub name: String,
    pub alias: String,
    pub start_line: usize,
    pub end_line: usize,
    pub granularity: ChunkGranularity,
    pub category: ChunkCategory,
    pub concepts: Vec<String>,
    pub requires: Vec<String>,
    pub provides: Vec<String>,
}

/// The SmartChunker - intelligent code analyzer and chunker
pub struct SmartChunker {
    config: SmartChunkerConfig,
}

impl SmartChunker {
    /// Create a new SmartChunker with configuration
    pub fn new(config: SmartChunkerConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(SmartChunkerConfig::default())
    }

    /// Analyze a file and determine chunking strategy
    pub fn analyze_file(&self, path: &Path, content: &str) -> FileAnalysis {
        let language = self.detect_language(path);
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let entities = self.extract_entities(content, &language);
        let imports = self.extract_imports(content, &language);
        let exports = self.extract_exports(content, &language, &entities);
        let is_entrypoint = self.is_entrypoint(path, content, &language);
        let is_test = self.is_test_file(path, content, &language);
        let is_config = self.is_config_file(path);
        let framework_hints = self.detect_frameworks(content, &language);
        let category = self.categorize_file(path, &entities, is_test, is_config);

        FileAnalysis {
            path: path.to_path_buf(),
            language,
            total_lines,
            entities,
            imports,
            exports,
            is_entrypoint,
            is_test,
            is_config,
            framework_hints,
            category,
        }
    }

    /// Decide how to chunk a file based on analysis
    pub fn decide_chunking(&self, analysis: &FileAnalysis) -> ChunkingDecision {
        let strategy;
        let reasoning;
        let mut suggested_chunks = Vec::new();

        // Very small files -> atomic
        if analysis.total_lines < self.config.min_file_lines_to_split {
            strategy = ChunkingStrategy::Atomic;
            reasoning = format!(
                "File has {} lines (< {}), keeping atomic",
                analysis.total_lines, self.config.min_file_lines_to_split
            );
            suggested_chunks.push(self.create_file_chunk(analysis));
        }
        // Config files -> atomic
        else if analysis.is_config {
            strategy = ChunkingStrategy::Atomic;
            reasoning = "Configuration file, keeping atomic".to_string();
            suggested_chunks.push(self.create_file_chunk(analysis));
        }
        // Single large entity -> atomic
        else if analysis.entities.len() == 1 {
            strategy = ChunkingStrategy::Atomic;
            reasoning = "Single entity file, keeping atomic".to_string();
            suggested_chunks.push(self.create_file_chunk(analysis));
        }
        // Has clear entity structure -> split by entity
        else if analysis.entities.len() > 1 && self.has_clear_structure(&analysis.entities) {
            strategy = ChunkingStrategy::ByEntity;
            reasoning = format!(
                "Found {} distinct entities, splitting by entity",
                analysis.entities.len()
            );
            suggested_chunks = self.create_entity_chunks(analysis);
        }
        // Large file with mixed content -> hierarchical
        else if analysis.total_lines > self.config.max_chunk_lines {
            strategy = ChunkingStrategy::Hierarchical;
            reasoning = format!(
                "Large file ({} lines), creating hierarchical chunks",
                analysis.total_lines
            );
            // Parent chunk
            suggested_chunks.push(self.create_file_chunk(analysis));
            // Child chunks
            suggested_chunks.extend(self.create_entity_chunks(analysis));
        }
        // Default: atomic if prefer_atomic, otherwise by entity
        else if self.config.prefer_atomic {
            strategy = ChunkingStrategy::Atomic;
            reasoning = "Preferring atomic chunks".to_string();
            suggested_chunks.push(self.create_file_chunk(analysis));
        } else {
            strategy = ChunkingStrategy::ByEntity;
            reasoning = "Splitting by code entities".to_string();
            suggested_chunks = self.create_entity_chunks(analysis);
        }

        ChunkingDecision {
            file_path: analysis.path.clone(),
            strategy,
            suggested_chunks,
            reasoning,
        }
    }

    /// Generate atomic chunks from content
    pub fn generate_chunks(
        &self,
        path: &Path,
        content: &str,
        decision: &ChunkingDecision,
    ) -> Vec<AtomicChunk> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for suggested in &decision.suggested_chunks {
            let chunk_content = if suggested.start_line == 0 && suggested.end_line >= lines.len() {
                content.to_string()
            } else {
                let start = suggested.start_line.saturating_sub(1);
                let end = suggested.end_line.min(lines.len());
                lines[start..end].join("\n")
            };

            let content_hash = compute_hash(&chunk_content);
            let chunk_id = format!("chunk:sha256:{}", content_hash);

            let analysis = self.analyze_file(path, content);

            let mut chunk = AtomicChunk::new(
                chunk_id,
                suggested.name.clone(),
                analysis.language.clone(),
                content_hash,
                chunk_content.len(),
            )
            .with_alias(&suggested.alias)
            .with_granularity(suggested.granularity)
            .with_categories(vec![suggested.category.clone()])
            .with_concepts(suggested.concepts.clone());

            chunk.provides = suggested.provides.clone();
            chunk.requires = suggested.requires.clone();
            chunk.sources = vec![SourceLocation {
                file: path.to_string_lossy().to_string(),
                start_line: Some(suggested.start_line),
                end_line: Some(suggested.end_line),
                start_col: None,
                end_col: None,
            }];
            chunk.metrics = ChunkMetrics {
                loc: suggested.end_line - suggested.start_line + 1,
                ..Default::default()
            };

            chunks.push(chunk);
        }

        chunks
    }

    // ========================================================================
    // Private helper methods
    // ========================================================================

    fn detect_language(&self, path: &Path) -> String {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "rs" => "rust".to_string(),
            "ts" | "tsx" => "typescript".to_string(),
            "js" | "jsx" | "mjs" | "cjs" => "javascript".to_string(),
            "py" | "pyi" => "python".to_string(),
            "go" => "go".to_string(),
            "c" | "h" => "c".to_string(),
            "cpp" | "cc" | "cxx" | "hpp" | "hh" => "cpp".to_string(),
            "java" => "java".to_string(),
            "rb" => "ruby".to_string(),
            "swift" => "swift".to_string(),
            "kt" | "kts" => "kotlin".to_string(),
            "cs" => "csharp".to_string(),
            "php" => "php".to_string(),
            "scala" => "scala".to_string(),
            "zig" => "zig".to_string(),
            "md" | "markdown" => "markdown".to_string(),
            "json" => "json".to_string(),
            "yaml" | "yml" => "yaml".to_string(),
            "toml" => "toml".to_string(),
            "html" | "htm" => "html".to_string(),
            "css" => "css".to_string(),
            "scss" | "sass" => "scss".to_string(),
            "sql" => "sql".to_string(),
            "sh" | "bash" | "zsh" => "shell".to_string(),
            "dockerfile" => "dockerfile".to_string(),
            _ => "unknown".to_string(),
        }
    }

    fn extract_entities(&self, content: &str, language: &str) -> Vec<CodeEntity> {
        let mut entities = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        match language {
            "rust" => self.extract_rust_entities(&lines, &mut entities),
            "typescript" | "javascript" => self.extract_ts_entities(&lines, &mut entities),
            "python" => self.extract_python_entities(&lines, &mut entities),
            "go" => self.extract_go_entities(&lines, &mut entities),
            _ => {}
        }

        entities
    }

    fn extract_rust_entities(&self, lines: &[&str], entities: &mut Vec<CodeEntity>) {
        let mut current_entity: Option<(String, EntityKind, usize, Visibility, Option<String>)> =
            None;
        let mut brace_depth = 0;
        let mut doc_comment = String::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Collect doc comments
            if trimmed.starts_with("///") || trimmed.starts_with("//!") {
                doc_comment.push_str(trimmed.trim_start_matches('/').trim());
                doc_comment.push('\n');
                continue;
            }

            // Detect entity starts
            let (is_pub, rest) = if trimmed.starts_with("pub ") {
                (true, &trimmed[4..])
            } else {
                (false, trimmed)
            };

            let visibility = if is_pub {
                Visibility::Public
            } else {
                Visibility::Private
            };

            if rest.starts_with("fn ")
                || rest.starts_with("async fn ")
                || rest.starts_with("const fn ")
                || rest.starts_with("unsafe fn ")
            {
                let is_async = rest.starts_with("async");
                if let Some(name) = self.extract_rust_fn_name(rest) {
                    if current_entity.is_some() {
                        self.close_entity(current_entity.take(), i, entities);
                    }
                    let kind = if is_async {
                        EntityKind::AsyncFunction
                    } else {
                        EntityKind::Function
                    };
                    let doc = if doc_comment.is_empty() {
                        None
                    } else {
                        Some(doc_comment.trim().to_string())
                    };
                    current_entity = Some((name, kind, i + 1, visibility, doc));
                    brace_depth = 0;
                }
            } else if rest.starts_with("struct ") {
                if let Some(name) = self.extract_rust_type_name(rest, "struct ") {
                    if current_entity.is_some() {
                        self.close_entity(current_entity.take(), i, entities);
                    }
                    let doc = if doc_comment.is_empty() {
                        None
                    } else {
                        Some(doc_comment.trim().to_string())
                    };
                    current_entity = Some((name, EntityKind::Struct, i + 1, visibility, doc));
                    brace_depth = 0;
                }
            } else if rest.starts_with("enum ") {
                if let Some(name) = self.extract_rust_type_name(rest, "enum ") {
                    if current_entity.is_some() {
                        self.close_entity(current_entity.take(), i, entities);
                    }
                    let doc = if doc_comment.is_empty() {
                        None
                    } else {
                        Some(doc_comment.trim().to_string())
                    };
                    current_entity = Some((name, EntityKind::Enum, i + 1, visibility, doc));
                    brace_depth = 0;
                }
            } else if rest.starts_with("trait ") {
                if let Some(name) = self.extract_rust_type_name(rest, "trait ") {
                    if current_entity.is_some() {
                        self.close_entity(current_entity.take(), i, entities);
                    }
                    let doc = if doc_comment.is_empty() {
                        None
                    } else {
                        Some(doc_comment.trim().to_string())
                    };
                    current_entity = Some((name, EntityKind::Trait, i + 1, visibility, doc));
                    brace_depth = 0;
                }
            } else if rest.starts_with("impl ") || rest.starts_with("impl<") {
                if let Some(name) = self.extract_impl_name(rest) {
                    if current_entity.is_some() {
                        self.close_entity(current_entity.take(), i, entities);
                    }
                    let doc = if doc_comment.is_empty() {
                        None
                    } else {
                        Some(doc_comment.trim().to_string())
                    };
                    current_entity =
                        Some((format!("impl_{}", name), EntityKind::Module, i + 1, visibility, doc));
                    brace_depth = 0;
                }
            }

            // Track braces for entity end
            brace_depth += trimmed.matches('{').count() as i32;
            brace_depth -= trimmed.matches('}').count() as i32;

            if brace_depth <= 0 && current_entity.is_some() {
                self.close_entity(current_entity.take(), i + 1, entities);
            }

            // Clear doc comment if not immediately followed by entity
            if !trimmed.starts_with("///")
                && !trimmed.starts_with("//!")
                && !trimmed.starts_with("#[")
                && !trimmed.is_empty()
            {
                doc_comment.clear();
            }
        }

        // Close any remaining entity
        if let Some(entity) = current_entity {
            self.close_entity(Some(entity), lines.len(), entities);
        }
    }

    fn extract_rust_fn_name(&self, line: &str) -> Option<String> {
        let rest = line
            .trim_start_matches("async ")
            .trim_start_matches("const ")
            .trim_start_matches("unsafe ")
            .trim_start_matches("fn ");
        let name_end = rest.find('(').or_else(|| rest.find('<'))?;
        Some(rest[..name_end].trim().to_string())
    }

    fn extract_rust_type_name(&self, line: &str, prefix: &str) -> Option<String> {
        let rest = line.trim_start_matches(prefix);
        let name_end = rest
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(rest.len());
        if name_end > 0 {
            Some(rest[..name_end].to_string())
        } else {
            None
        }
    }

    fn extract_impl_name(&self, line: &str) -> Option<String> {
        // Handle "impl Trait for Type" and "impl Type"
        let rest = line.trim_start_matches("impl").trim_start_matches('<');
        // Skip generic params
        let rest = if let Some(idx) = rest.find('>') {
            &rest[idx + 1..]
        } else {
            rest
        };
        let rest = rest.trim();
        
        if let Some(idx) = rest.find(" for ") {
            // impl Trait for Type
            let type_name = rest[idx + 5..].split_whitespace().next()?;
            Some(type_name.to_string())
        } else {
            // impl Type
            let type_name = rest.split_whitespace().next()?;
            Some(type_name.to_string())
        }
    }

    fn extract_ts_entities(&self, lines: &[&str], entities: &mut Vec<CodeEntity>) {
        let mut current_entity: Option<(String, EntityKind, usize, Visibility, Option<String>)> =
            None;
        let mut brace_depth = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Detect entity starts
            let is_export = trimmed.starts_with("export ");
            let rest = if is_export {
                &trimmed[7..]
            } else {
                trimmed
            };

            let visibility = if is_export {
                Visibility::Public
            } else {
                Visibility::Private
            };

            if rest.starts_with("function ")
                || rest.starts_with("async function ")
                || (rest.starts_with("const ") && rest.contains("=>"))
            {
                let is_async = rest.contains("async");
                if let Some(name) = self.extract_ts_fn_name(rest) {
                    if current_entity.is_some() {
                        self.close_entity(current_entity.take(), i, entities);
                    }
                    let kind = if is_async {
                        EntityKind::AsyncFunction
                    } else {
                        EntityKind::Function
                    };
                    current_entity = Some((name, kind, i + 1, visibility, None));
                    brace_depth = 0;
                }
            } else if rest.starts_with("class ") {
                if let Some(name) = self.extract_ts_class_name(rest) {
                    if current_entity.is_some() {
                        self.close_entity(current_entity.take(), i, entities);
                    }
                    current_entity = Some((name, EntityKind::Class, i + 1, visibility, None));
                    brace_depth = 0;
                }
            } else if rest.starts_with("interface ") {
                if let Some(name) = self.extract_ts_interface_name(rest) {
                    if current_entity.is_some() {
                        self.close_entity(current_entity.take(), i, entities);
                    }
                    current_entity = Some((name, EntityKind::Interface, i + 1, visibility, None));
                    brace_depth = 0;
                }
            } else if rest.starts_with("type ") {
                if let Some(name) = self.extract_ts_type_name(rest) {
                    if current_entity.is_some() {
                        self.close_entity(current_entity.take(), i, entities);
                    }
                    current_entity = Some((name, EntityKind::Type, i + 1, visibility, None));
                    brace_depth = 0;
                }
            } else if rest.starts_with("enum ") {
                if let Some(name) = self.extract_ts_enum_name(rest) {
                    if current_entity.is_some() {
                        self.close_entity(current_entity.take(), i, entities);
                    }
                    current_entity = Some((name, EntityKind::Enum, i + 1, visibility, None));
                    brace_depth = 0;
                }
            }

            // Track braces
            brace_depth += trimmed.matches('{').count() as i32;
            brace_depth -= trimmed.matches('}').count() as i32;

            if brace_depth <= 0 && current_entity.is_some() && trimmed.contains('}') {
                self.close_entity(current_entity.take(), i + 1, entities);
            }
        }

        if let Some(entity) = current_entity {
            self.close_entity(Some(entity), lines.len(), entities);
        }
    }

    fn extract_ts_fn_name(&self, line: &str) -> Option<String> {
        if line.starts_with("const ") {
            // Arrow function: const name = ...
            let rest = line.trim_start_matches("const ");
            let name_end = rest.find(|c: char| !c.is_alphanumeric() && c != '_')?;
            return Some(rest[..name_end].to_string());
        }
        let rest = line
            .trim_start_matches("async ")
            .trim_start_matches("function ");
        let name_end = rest.find('(')?;
        Some(rest[..name_end].trim().to_string())
    }

    fn extract_ts_class_name(&self, line: &str) -> Option<String> {
        let rest = line.trim_start_matches("class ");
        let name_end = rest.find(|c: char| !c.is_alphanumeric() && c != '_')?;
        Some(rest[..name_end].to_string())
    }

    fn extract_ts_interface_name(&self, line: &str) -> Option<String> {
        let rest = line.trim_start_matches("interface ");
        let name_end = rest.find(|c: char| !c.is_alphanumeric() && c != '_')?;
        Some(rest[..name_end].to_string())
    }

    fn extract_ts_type_name(&self, line: &str) -> Option<String> {
        let rest = line.trim_start_matches("type ");
        let name_end = rest.find(|c: char| !c.is_alphanumeric() && c != '_')?;
        Some(rest[..name_end].to_string())
    }

    fn extract_ts_enum_name(&self, line: &str) -> Option<String> {
        let rest = line.trim_start_matches("enum ");
        let name_end = rest.find(|c: char| !c.is_alphanumeric() && c != '_')?;
        Some(rest[..name_end].to_string())
    }

    fn extract_python_entities(&self, lines: &[&str], entities: &mut Vec<CodeEntity>) {
        let mut current_entity: Option<(String, EntityKind, usize, usize, Visibility)> = None;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let indent = line.len() - line.trim_start().len();

            // Top-level definitions only (no indentation)
            if indent == 0 {
                // Close previous if exists
                if let Some((name, kind, start, _, vis)) = current_entity.take() {
                    entities.push(CodeEntity {
                        name,
                        kind,
                        start_line: start,
                        end_line: i,
                        visibility: vis,
                        doc_comment: None,
                        imports: Vec::new(),
                        exports: Vec::new(),
                        calls: Vec::new(),
                        complexity: 1,
                    });
                }

                if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
                    let is_async = trimmed.starts_with("async");
                    if let Some(name) = self.extract_python_fn_name(trimmed) {
                        let visibility = if name.starts_with('_') {
                            Visibility::Private
                        } else {
                            Visibility::Public
                        };
                        let kind = if is_async {
                            EntityKind::AsyncFunction
                        } else {
                            EntityKind::Function
                        };
                        current_entity = Some((name, kind, i + 1, indent, visibility));
                    }
                } else if trimmed.starts_with("class ") {
                    if let Some(name) = self.extract_python_class_name(trimmed) {
                        let visibility = if name.starts_with('_') {
                            Visibility::Private
                        } else {
                            Visibility::Public
                        };
                        current_entity = Some((name, EntityKind::Class, i + 1, indent, visibility));
                    }
                }
            }
        }

        // Close final entity
        if let Some((name, kind, start, _, vis)) = current_entity {
            entities.push(CodeEntity {
                name,
                kind,
                start_line: start,
                end_line: lines.len(),
                visibility: vis,
                doc_comment: None,
                imports: Vec::new(),
                exports: Vec::new(),
                calls: Vec::new(),
                complexity: 1,
            });
        }
    }

    fn extract_python_fn_name(&self, line: &str) -> Option<String> {
        let rest = line
            .trim_start_matches("async ")
            .trim_start_matches("def ");
        let name_end = rest.find('(')?;
        Some(rest[..name_end].trim().to_string())
    }

    fn extract_python_class_name(&self, line: &str) -> Option<String> {
        let rest = line.trim_start_matches("class ");
        let name_end = rest.find(['(', ':'])?;
        Some(rest[..name_end].trim().to_string())
    }

    fn extract_go_entities(&self, lines: &[&str], entities: &mut Vec<CodeEntity>) {
        let mut current_entity: Option<(String, EntityKind, usize, Visibility)> = None;
        let mut brace_depth = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("func ") {
                if current_entity.is_some() {
                    self.close_entity_simple(current_entity.take(), i, entities);
                }
                if let Some(name) = self.extract_go_fn_name(trimmed) {
                    let visibility = if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                    {
                        Visibility::Public
                    } else {
                        Visibility::Private
                    };
                    current_entity = Some((name, EntityKind::Function, i + 1, visibility));
                    brace_depth = 0;
                }
            } else if trimmed.starts_with("type ") && trimmed.contains("struct") {
                if current_entity.is_some() {
                    self.close_entity_simple(current_entity.take(), i, entities);
                }
                if let Some(name) = self.extract_go_type_name(trimmed) {
                    let visibility = if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                    {
                        Visibility::Public
                    } else {
                        Visibility::Private
                    };
                    current_entity = Some((name, EntityKind::Struct, i + 1, visibility));
                    brace_depth = 0;
                }
            } else if trimmed.starts_with("type ") && trimmed.contains("interface") {
                if current_entity.is_some() {
                    self.close_entity_simple(current_entity.take(), i, entities);
                }
                if let Some(name) = self.extract_go_type_name(trimmed) {
                    let visibility = if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                    {
                        Visibility::Public
                    } else {
                        Visibility::Private
                    };
                    current_entity = Some((name, EntityKind::Interface, i + 1, visibility));
                    brace_depth = 0;
                }
            }

            brace_depth += trimmed.matches('{').count() as i32;
            brace_depth -= trimmed.matches('}').count() as i32;

            if brace_depth <= 0 && current_entity.is_some() && trimmed.contains('}') {
                self.close_entity_simple(current_entity.take(), i + 1, entities);
            }
        }

        if let Some(entity) = current_entity {
            self.close_entity_simple(Some(entity), lines.len(), entities);
        }
    }

    fn extract_go_fn_name(&self, line: &str) -> Option<String> {
        let rest = line.trim_start_matches("func ");
        // Handle method receivers: (r *Receiver)
        let rest = if rest.starts_with('(') {
            if let Some(idx) = rest.find(')') {
                &rest[idx + 1..]
            } else {
                rest
            }
        } else {
            rest
        };
        let rest = rest.trim();
        let name_end = rest.find('(')?;
        Some(rest[..name_end].trim().to_string())
    }

    fn extract_go_type_name(&self, line: &str) -> Option<String> {
        let rest = line.trim_start_matches("type ");
        let name_end = rest.find(|c: char| !c.is_alphanumeric() && c != '_')?;
        Some(rest[..name_end].to_string())
    }

    fn close_entity(
        &self,
        entity: Option<(String, EntityKind, usize, Visibility, Option<String>)>,
        end_line: usize,
        entities: &mut Vec<CodeEntity>,
    ) {
        if let Some((name, kind, start, visibility, doc)) = entity {
            entities.push(CodeEntity {
                name,
                kind,
                start_line: start,
                end_line,
                visibility,
                doc_comment: doc,
                imports: Vec::new(),
                exports: Vec::new(),
                calls: Vec::new(),
                complexity: 1,
            });
        }
    }

    fn close_entity_simple(
        &self,
        entity: Option<(String, EntityKind, usize, Visibility)>,
        end_line: usize,
        entities: &mut Vec<CodeEntity>,
    ) {
        if let Some((name, kind, start, visibility)) = entity {
            entities.push(CodeEntity {
                name,
                kind,
                start_line: start,
                end_line,
                visibility,
                doc_comment: None,
                imports: Vec::new(),
                exports: Vec::new(),
                calls: Vec::new(),
                complexity: 1,
            });
        }
    }

    fn extract_imports(&self, content: &str, language: &str) -> Vec<String> {
        let mut imports = Vec::new();

        match language {
            "rust" => {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("use ") {
                        let import = trimmed.trim_start_matches("use ").trim_end_matches(';');
                        imports.push(import.to_string());
                    }
                }
            }
            "typescript" | "javascript" => {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("import ") {
                        imports.push(trimmed.to_string());
                    }
                }
            }
            "python" => {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                        imports.push(trimmed.to_string());
                    }
                }
            }
            "go" => {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("import ") || trimmed.starts_with("\"") {
                        imports.push(trimmed.to_string());
                    }
                }
            }
            _ => {}
        }

        imports
    }

    fn extract_exports(&self, _content: &str, _language: &str, entities: &[CodeEntity]) -> Vec<String> {
        entities
            .iter()
            .filter(|e| e.visibility == Visibility::Public)
            .map(|e| e.name.clone())
            .collect()
    }

    fn is_entrypoint(&self, path: &Path, content: &str, language: &str) -> bool {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        match language {
            "rust" => file_name == "main.rs" || content.contains("fn main()"),
            "typescript" | "javascript" => {
                file_name == "index.ts"
                    || file_name == "index.tsx"
                    || file_name == "index.js"
                    || file_name == "main.ts"
            }
            "python" => {
                file_name == "__main__.py"
                    || file_name == "main.py"
                    || content.contains("if __name__")
            }
            "go" => file_name == "main.go" || content.contains("func main()"),
            _ => false,
        }
    }

    fn is_test_file(&self, path: &Path, content: &str, language: &str) -> bool {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let path_str = path.to_string_lossy();

        match language {
            "rust" => {
                path_str.contains("/tests/")
                    || content.contains("#[test]")
                    || content.contains("#[cfg(test)]")
            }
            "typescript" | "javascript" => {
                file_name.contains(".test.")
                    || file_name.contains(".spec.")
                    || path_str.contains("__tests__")
            }
            "python" => {
                file_name.starts_with("test_")
                    || file_name.ends_with("_test.py")
                    || path_str.contains("/tests/")
            }
            "go" => file_name.ends_with("_test.go"),
            _ => false,
        }
    }

    fn is_config_file(&self, path: &Path) -> bool {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        matches!(
            file_name,
            "Cargo.toml"
                | "package.json"
                | "tsconfig.json"
                | "pyproject.toml"
                | "setup.py"
                | "go.mod"
                | "CMakeLists.txt"
                | "Makefile"
                | ".eslintrc.json"
                | ".prettierrc"
                | "webpack.config.js"
                | "vite.config.ts"
                | "tailwind.config.js"
        ) || matches!(ext, "toml" | "yaml" | "yml" | "json")
            && (file_name.contains("config") || file_name.contains("settings"))
    }

    fn detect_frameworks(&self, content: &str, language: &str) -> Vec<String> {
        let mut frameworks = Vec::new();

        match language {
            "typescript" | "javascript" => {
                if content.contains("react") || content.contains("React") {
                    frameworks.push("react".to_string());
                }
                if content.contains("@angular") {
                    frameworks.push("angular".to_string());
                }
                if content.contains("vue") {
                    frameworks.push("vue".to_string());
                }
                if content.contains("next") {
                    frameworks.push("nextjs".to_string());
                }
                if content.contains("express") {
                    frameworks.push("express".to_string());
                }
            }
            "rust" => {
                if content.contains("actix") {
                    frameworks.push("actix".to_string());
                }
                if content.contains("tokio") {
                    frameworks.push("tokio".to_string());
                }
                if content.contains("axum") {
                    frameworks.push("axum".to_string());
                }
                if content.contains("serde") {
                    frameworks.push("serde".to_string());
                }
            }
            "python" => {
                if content.contains("django") {
                    frameworks.push("django".to_string());
                }
                if content.contains("flask") {
                    frameworks.push("flask".to_string());
                }
                if content.contains("fastapi") {
                    frameworks.push("fastapi".to_string());
                }
            }
            _ => {}
        }

        frameworks
    }

    fn categorize_file(
        &self,
        path: &Path,
        entities: &[CodeEntity],
        is_test: bool,
        is_config: bool,
    ) -> ChunkCategory {
        if is_test {
            return ChunkCategory::Test;
        }
        if is_config {
            return ChunkCategory::Config;
        }

        let path_str = path.to_string_lossy().to_lowercase();
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Path-based categorization
        if path_str.contains("/api/") || path_str.contains("/routes/") {
            return ChunkCategory::Api;
        }
        if path_str.contains("/ui/")
            || path_str.contains("/components/")
            || path_str.contains("/views/")
        {
            return ChunkCategory::Ui;
        }
        if path_str.contains("/utils/") || path_str.contains("/helpers/") {
            return ChunkCategory::Utility;
        }
        if path_str.contains("/models/") || path_str.contains("/types/") {
            return ChunkCategory::Data;
        }
        if path_str.contains("/db/") || path_str.contains("/database/") {
            return ChunkCategory::Database;
        }

        // Name-based categorization
        if file_name.contains("util") || file_name.contains("helper") {
            return ChunkCategory::Utility;
        }
        if file_name.contains("type") || file_name.contains("model") {
            return ChunkCategory::Data;
        }

        // Entity-based categorization
        let type_count = entities
            .iter()
            .filter(|e| {
                matches!(
                    e.kind,
                    EntityKind::Struct
                        | EntityKind::Class
                        | EntityKind::Interface
                        | EntityKind::Enum
                        | EntityKind::Type
                )
            })
            .count();

        if type_count > entities.len() / 2 {
            return ChunkCategory::Data;
        }

        ChunkCategory::Logic
    }

    fn has_clear_structure(&self, entities: &[CodeEntity]) -> bool {
        // Has multiple significant entities
        let significant = entities
            .iter()
            .filter(|e| e.end_line - e.start_line >= self.config.min_function_lines)
            .count();

        significant >= 2
    }

    fn create_file_chunk(&self, analysis: &FileAnalysis) -> SuggestedChunk {
        let file_name = analysis
            .path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let alias = self.generate_alias(file_name, &analysis.path);

        SuggestedChunk {
            name: file_name.to_string(),
            alias,
            start_line: 1,
            end_line: analysis.total_lines,
            granularity: ChunkGranularity::Module,
            category: analysis.category.clone(),
            concepts: analysis.exports.clone(),
            requires: analysis.imports.clone(),
            provides: analysis.exports.clone(),
        }
    }

    fn create_entity_chunks(&self, analysis: &FileAnalysis) -> Vec<SuggestedChunk> {
        let file_stem = analysis
            .path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        analysis
            .entities
            .iter()
            .filter(|e| e.end_line - e.start_line >= self.config.min_function_lines)
            .map(|entity| {
                let granularity = match entity.kind {
                    EntityKind::Function | EntityKind::AsyncFunction | EntityKind::Method => {
                        ChunkGranularity::Function
                    }
                    EntityKind::Struct
                    | EntityKind::Class
                    | EntityKind::Trait
                    | EntityKind::Interface
                    | EntityKind::Enum => ChunkGranularity::Type,
                    EntityKind::Module => ChunkGranularity::Module,
                    _ => ChunkGranularity::Function,
                };

                let alias = format!("{}/{}", file_stem, to_kebab_case(&entity.name));

                SuggestedChunk {
                    name: entity.name.clone(),
                    alias,
                    start_line: entity.start_line,
                    end_line: entity.end_line,
                    granularity,
                    category: analysis.category.clone(),
                    concepts: vec![entity.name.clone()],
                    requires: entity.imports.clone(),
                    provides: if entity.visibility == Visibility::Public {
                        vec![entity.name.clone()]
                    } else {
                        Vec::new()
                    },
                }
            })
            .collect()
    }

    fn generate_alias(&self, name: &str, path: &Path) -> String {
        let parent = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if parent.is_empty() || parent == "src" {
            to_kebab_case(name)
        } else {
            format!("{}/{}", to_kebab_case(parent), to_kebab_case(name))
        }
    }
}

/// Convert string to kebab-case
fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('-');
        }
        result.push(c.to_ascii_lowercase());
    }
    result
        .replace(['_', ' '], "-")
        .replace("--", "-")
}

/// Compute SHA256 hash
fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        let chunker = SmartChunker::default();
        assert_eq!(
            chunker.detect_language(Path::new("test.rs")),
            "rust"
        );
        assert_eq!(
            chunker.detect_language(Path::new("test.ts")),
            "typescript"
        );
        assert_eq!(
            chunker.detect_language(Path::new("test.py")),
            "python"
        );
    }

    #[test]
    fn test_kebab_case() {
        assert_eq!(to_kebab_case("HelloWorld"), "hello-world");
        assert_eq!(to_kebab_case("hello_world"), "hello-world");
        assert_eq!(to_kebab_case("my-file"), "my-file");
    }

    #[test]
    fn test_rust_entity_extraction() {
        let chunker = SmartChunker::default();
        let content = r#"
pub fn hello_world() {
    println!("Hello!");
}

struct MyStruct {
    field: i32,
}

impl MyStruct {
    fn new() -> Self {
        Self { field: 0 }
    }
}
"#;
        let lines: Vec<&str> = content.lines().collect();
        let mut entities = Vec::new();
        chunker.extract_rust_entities(&lines, &mut entities);

        assert!(entities.len() >= 2);
    }
}
