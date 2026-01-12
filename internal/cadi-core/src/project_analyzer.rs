//! Project Analyzer - Whole-project analysis for intelligent chunking
//!
//! This module analyzes entire projects to:
//! - Identify the project structure and layout
//! - Find shared utilities and common patterns
//! - Detect composition relationships between files
//! - Determine optimal chunking for maximum reuse

use crate::atomic::{
    AliasRegistry, AtomicChunk, ChunkCategory, ChunkComposition,
    ChunkGranularity, ChunkReference,
};
use crate::smart_chunker::{ChunkingStrategy, FileAnalysis, SmartChunker, SmartChunkerConfig};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Project type detection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    C,
    Cpp,
    Java,
    Mixed,
    Unknown,
}

impl ProjectType {
    pub fn primary_language(&self) -> &str {
        match self {
            ProjectType::Rust => "rust",
            ProjectType::TypeScript => "typescript",
            ProjectType::JavaScript => "javascript",
            ProjectType::Python => "python",
            ProjectType::Go => "go",
            ProjectType::C => "c",
            ProjectType::Cpp => "cpp",
            ProjectType::Java => "java",
            ProjectType::Mixed => "mixed",
            ProjectType::Unknown => "unknown",
        }
    }
}

/// Configuration for project analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalyzerConfig {
    /// Directories to ignore
    #[serde(default = "default_ignore_dirs")]
    pub ignore_dirs: Vec<String>,

    /// File patterns to ignore
    #[serde(default = "default_ignore_patterns")]
    pub ignore_patterns: Vec<String>,

    /// Maximum file size to process (bytes)
    #[serde(default = "default_max_file_size")]
    pub max_file_size: usize,

    /// Smart chunker config
    #[serde(default)]
    pub chunker_config: SmartChunkerConfig,

    /// Whether to create composition chunks
    #[serde(default = "default_true")]
    pub detect_compositions: bool,

    /// Whether to merge small related files
    #[serde(default = "default_true")]
    pub merge_small_files: bool,

    /// Minimum files for a composition chunk
    #[serde(default = "default_min_composition_files")]
    pub min_composition_files: usize,

    /// Namespace for aliases
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

fn default_ignore_dirs() -> Vec<String> {
    vec![
        ".git".to_string(),
        "node_modules".to_string(),
        "target".to_string(),
        "__pycache__".to_string(),
        ".venv".to_string(),
        "venv".to_string(),
        "dist".to_string(),
        "build".to_string(),
        ".next".to_string(),
        ".cache".to_string(),
        "coverage".to_string(),
    ]
}

fn default_ignore_patterns() -> Vec<String> {
    vec![
        "*.lock".to_string(),
        "*.log".to_string(),
        ".DS_Store".to_string(),
        "*.min.js".to_string(),
        "*.min.css".to_string(),
        "*.map".to_string(),
    ]
}

fn default_max_file_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

fn default_true() -> bool {
    true
}

fn default_min_composition_files() -> usize {
    2
}

impl Default for ProjectAnalyzerConfig {
    fn default() -> Self {
        Self {
            ignore_dirs: default_ignore_dirs(),
            ignore_patterns: default_ignore_patterns(),
            max_file_size: default_max_file_size(),
            chunker_config: SmartChunkerConfig::default(),
            detect_compositions: true,
            merge_small_files: true,
            min_composition_files: 2,
            namespace: None,
        }
    }
}

/// Result of project analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    /// Root path of project
    pub root: PathBuf,

    /// Project name
    pub name: String,

    /// Detected project type
    pub project_type: ProjectType,

    /// Primary language
    pub primary_language: String,

    /// All languages found
    pub languages: Vec<String>,

    /// Total files analyzed
    pub total_files: usize,

    /// Total lines of code
    pub total_lines: usize,

    /// File analyses
    pub files: Vec<FileAnalysis>,

    /// Detected entry points
    pub entrypoints: Vec<PathBuf>,

    /// Detected modules/packages
    pub modules: Vec<ModuleInfo>,

    /// Shared utilities (files imported by many)
    pub shared_utilities: Vec<PathBuf>,

    /// Suggested compositions
    pub compositions: Vec<CompositionSuggestion>,
}

/// Information about a module/package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    pub path: PathBuf,
    pub files: Vec<PathBuf>,
    pub is_entrypoint: bool,
    pub exports: Vec<String>,
    pub category: ChunkCategory,
}

/// Suggestion for composing multiple chunks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionSuggestion {
    pub name: String,
    pub description: String,
    pub files: Vec<PathBuf>,
    pub category: ChunkCategory,
    pub reason: String,
}

/// Import result containing all chunks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// Project analysis
    pub analysis: ProjectAnalysis,

    /// All atomic chunks created
    pub chunks: Vec<AtomicChunk>,

    /// Alias registry
    pub alias_registry: AliasRegistry,

    /// Composition chunks (chunks made of other chunks)
    pub compositions: Vec<AtomicChunk>,

    /// Import summary
    pub summary: ImportSummary,
}

/// Summary of import operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSummary {
    pub project_name: String,
    pub project_type: String,
    pub total_files: usize,
    pub total_lines: usize,
    pub atomic_chunks: usize,
    pub composition_chunks: usize,
    pub skipped_files: usize,
    pub categories: HashMap<String, usize>,
    pub aliases_created: usize,
    pub duration_ms: u128,
}

/// The Project Analyzer
pub struct ProjectAnalyzer {
    config: ProjectAnalyzerConfig,
    chunker: SmartChunker,
}

impl ProjectAnalyzer {
    /// Create a new analyzer with configuration
    pub fn new(config: ProjectAnalyzerConfig) -> Self {
        let chunker = SmartChunker::new(config.chunker_config.clone());
        Self { config, chunker }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(ProjectAnalyzerConfig::default())
    }

    /// Analyze an entire project
    pub fn analyze_project(&self, root: &Path) -> std::io::Result<ProjectAnalysis> {
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string();

        // Collect all files
        let files = self.collect_files(root)?;
        let total_files = files.len();

        // Analyze each file
        let mut file_analyses = Vec::new();
        let mut total_lines = 0;
        let mut language_counts: HashMap<String, usize> = HashMap::new();
        let mut entrypoints = Vec::new();

        for file_path in &files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                let analysis = self.chunker.analyze_file(file_path, &content);
                total_lines += analysis.total_lines;

                *language_counts.entry(analysis.language.clone()).or_insert(0) += 1;

                if analysis.is_entrypoint {
                    entrypoints.push(file_path.clone());
                }

                file_analyses.push(analysis);
            }
        }

        // Determine project type and primary language
        let (project_type, primary_language) = self.detect_project_type(root, &language_counts);

        let languages: Vec<String> = language_counts.keys().cloned().collect();

        // Detect modules
        let modules = self.detect_modules(root, &file_analyses);

        // Find shared utilities
        let shared_utilities = self.find_shared_utilities(&file_analyses);

        // Detect composition opportunities
        let compositions = if self.config.detect_compositions {
            self.detect_compositions(&file_analyses, &modules)
        } else {
            Vec::new()
        };

        Ok(ProjectAnalysis {
            root: root.to_path_buf(),
            name,
            project_type,
            primary_language,
            languages,
            total_files,
            total_lines,
            files: file_analyses,
            entrypoints,
            modules,
            shared_utilities,
            compositions,
        })
    }

    /// Import a project - analyze and create all chunks
    pub fn import_project(&self, root: &Path) -> std::io::Result<ImportResult> {
        let start = std::time::Instant::now();

        // Analyze project
        let analysis = self.analyze_project(root)?;

        // Create chunks
        let mut chunks = Vec::new();
        let mut alias_registry = AliasRegistry::new();
        let mut skipped_files = 0;
        let mut categories: HashMap<String, usize> = HashMap::new();

        for file_analysis in &analysis.files {
            if let Ok(content) = std::fs::read_to_string(&file_analysis.path) {
                let decision = self.chunker.decide_chunking(file_analysis);

                if decision.strategy == ChunkingStrategy::Skip {
                    skipped_files += 1;
                    continue;
                }

                let _relative_path = file_analysis
                    .path
                    .clone();

                let file_chunks =
                    self.chunker
                        .generate_chunks(&file_analysis.path, &content, &decision);

                for mut chunk in file_chunks {
                    // Add namespace if configured
                    if let Some(ref ns) = self.config.namespace {
                        for alias in &mut chunk.aliases {
                            alias.namespace = Some(ns.clone());
                        }
                    }

                    // Register aliases
                    for alias in &chunk.aliases {
                        let alias_path = alias.full_path();
                        let unique_alias = alias_registry.generate_unique(&alias_path);
                        alias_registry.register(&unique_alias, &chunk.chunk_id);
                    }

                    // Count categories
                    for cat in &chunk.categories {
                        let cat_str = format!("{:?}", cat);
                        *categories.entry(cat_str).or_insert(0) += 1;
                    }

                    chunks.push(chunk);
                }
            }
        }

        // Create composition chunks
        let mut compositions = Vec::new();
        for suggestion in &analysis.compositions {
            if let Some(comp_chunk) =
                self.create_composition_chunk(suggestion, &chunks, &mut alias_registry)
            {
                compositions.push(comp_chunk);
            }
        }

        let duration_ms = start.elapsed().as_millis();

        let summary = ImportSummary {
            project_name: analysis.name.clone(),
            project_type: format!("{:?}", analysis.project_type),
            total_files: analysis.total_files,
            total_lines: analysis.total_lines,
            atomic_chunks: chunks.len(),
            composition_chunks: compositions.len(),
            skipped_files,
            categories,
            aliases_created: alias_registry.aliases.len(),
            duration_ms,
        };

        Ok(ImportResult {
            analysis,
            chunks,
            alias_registry,
            compositions,
            summary,
        })
    }

    // ========================================================================
    // Private helpers
    // ========================================================================

    fn collect_files(&self, root: &Path) -> std::io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.collect_files_recursive(root, &mut files)?;
        Ok(files)
    }

    fn collect_files_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Skip ignored directories
                if self.config.ignore_dirs.contains(&dir_name.to_string()) {
                    continue;
                }
                if dir_name.starts_with('.') {
                    continue;
                }

                self.collect_files_recursive(&path, files)?;
            } else if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Check against ignore patterns
                let should_ignore = self.config.ignore_patterns.iter().any(|pattern| {
                    if pattern.starts_with('*') {
                        file_name.ends_with(&pattern[1..])
                    } else {
                        file_name == pattern
                    }
                });

                if should_ignore {
                    continue;
                }

                // Check file size
                if let Ok(metadata) = path.metadata() {
                    if metadata.len() as usize > self.config.max_file_size {
                        continue;
                    }
                }

                // Check if it's a source file
                if self.is_source_file(&path) {
                    files.push(path);
                }
            }
        }

        Ok(())
    }

    fn is_source_file(&self, path: &Path) -> bool {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        matches!(
            ext,
            "rs"
                | "ts"
                | "tsx"
                | "js"
                | "jsx"
                | "mjs"
                | "cjs"
                | "py"
                | "pyi"
                | "go"
                | "c"
                | "h"
                | "cpp"
                | "hpp"
                | "cc"
                | "cxx"
                | "java"
                | "kt"
                | "swift"
                | "rb"
                | "php"
                | "scala"
                | "cs"
                | "zig"
                | "md"
                | "json"
                | "yaml"
                | "yml"
                | "toml"
                | "html"
                | "css"
                | "scss"
                | "sql"
                | "sh"
        )
    }

    fn detect_project_type(
        &self,
        root: &Path,
        language_counts: &HashMap<String, usize>,
    ) -> (ProjectType, String) {
        // Check for project markers
        if root.join("Cargo.toml").exists() {
            return (ProjectType::Rust, "rust".to_string());
        }
        if root.join("tsconfig.json").exists() {
            return (ProjectType::TypeScript, "typescript".to_string());
        }
        if root.join("package.json").exists() {
            // Could be TS or JS
            if language_counts.get("typescript").unwrap_or(&0)
                > language_counts.get("javascript").unwrap_or(&0)
            {
                return (ProjectType::TypeScript, "typescript".to_string());
            }
            return (ProjectType::JavaScript, "javascript".to_string());
        }
        if root.join("pyproject.toml").exists() || root.join("setup.py").exists() {
            return (ProjectType::Python, "python".to_string());
        }
        if root.join("go.mod").exists() {
            return (ProjectType::Go, "go".to_string());
        }
        if root.join("CMakeLists.txt").exists() || root.join("Makefile").exists() {
            let cpp_count = language_counts.get("cpp").unwrap_or(&0);
            let c_count = language_counts.get("c").unwrap_or(&0);
            if cpp_count > c_count {
                return (ProjectType::Cpp, "cpp".to_string());
            }
            return (ProjectType::C, "c".to_string());
        }
        if root.join("pom.xml").exists() || root.join("build.gradle").exists() {
            return (ProjectType::Java, "java".to_string());
        }

        // Fallback to most common language
        let primary = language_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(lang, _)| lang.clone())
            .unwrap_or_else(|| "unknown".to_string());

        if language_counts.len() > 2 {
            (ProjectType::Mixed, primary)
        } else {
            (ProjectType::Unknown, primary)
        }
    }

    fn detect_modules(&self, root: &Path, files: &[FileAnalysis]) -> Vec<ModuleInfo> {
        let mut modules: HashMap<PathBuf, ModuleInfo> = HashMap::new();

        for file in files {
            // Get the parent directory as the module
            let module_path = file
                .path
                .parent()
                .unwrap_or(&file.path)
                .strip_prefix(root)
                .unwrap_or(file.path.parent().unwrap_or(&file.path));

            let module_name = module_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("root")
                .to_string();

            let entry = modules
                .entry(module_path.to_path_buf())
                .or_insert_with(|| ModuleInfo {
                    name: module_name,
                    path: module_path.to_path_buf(),
                    files: Vec::new(),
                    is_entrypoint: false,
                    exports: Vec::new(),
                    category: ChunkCategory::Logic,
                });

            entry.files.push(file.path.clone());
            entry.exports.extend(file.exports.clone());

            if file.is_entrypoint {
                entry.is_entrypoint = true;
            }
        }

        modules.into_values().collect()
    }

    fn find_shared_utilities(&self, files: &[FileAnalysis]) -> Vec<PathBuf> {
        // Find files that are imported by many others
        let mut import_counts: HashMap<String, usize> = HashMap::new();

        for file in files {
            for import in &file.imports {
                *import_counts.entry(import.clone()).or_insert(0) += 1;
            }
        }

        // Files imported by 3+ others are considered shared
        let shared_imports: HashSet<String> = import_counts
            .into_iter()
            .filter(|(_, count)| *count >= 3)
            .map(|(import, _)| import)
            .collect();

        // Match imports to actual files (simplified)
        files
            .iter()
            .filter(|f| {
                let file_stem = f.path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
                shared_imports.iter().any(|imp| imp.contains(file_stem))
            })
            .map(|f| f.path.clone())
            .collect()
    }

    fn detect_compositions(
        &self,
        files: &[FileAnalysis],
        modules: &[ModuleInfo],
    ) -> Vec<CompositionSuggestion> {
        let mut suggestions = Vec::new();

        // Suggest module-level compositions
        for module in modules {
            if module.files.len() >= self.config.min_composition_files {
                suggestions.push(CompositionSuggestion {
                    name: format!("{}-module", module.name),
                    description: format!("Complete {} module", module.name),
                    files: module.files.clone(),
                    category: module.category.clone(),
                    reason: format!(
                        "Module contains {} related files",
                        module.files.len()
                    ),
                });
            }
        }

        // Suggest test compositions
        let test_files: Vec<PathBuf> = files
            .iter()
            .filter(|f| f.is_test)
            .map(|f| f.path.clone())
            .collect();

        if test_files.len() >= 2 {
            suggestions.push(CompositionSuggestion {
                name: "test-suite".to_string(),
                description: "Complete test suite".to_string(),
                files: test_files,
                category: ChunkCategory::Test,
                reason: "Grouped all test files together".to_string(),
            });
        }

        // Suggest utility compositions
        let util_files: Vec<PathBuf> = files
            .iter()
            .filter(|f| matches!(f.category, ChunkCategory::Utility))
            .map(|f| f.path.clone())
            .collect();

        if util_files.len() >= 2 {
            suggestions.push(CompositionSuggestion {
                name: "utilities".to_string(),
                description: "Shared utility functions".to_string(),
                files: util_files,
                category: ChunkCategory::Utility,
                reason: "Grouped utility files together".to_string(),
            });
        }

        suggestions
    }

    fn create_composition_chunk(
        &self,
        suggestion: &CompositionSuggestion,
        chunks: &[AtomicChunk],
        registry: &mut AliasRegistry,
    ) -> Option<AtomicChunk> {
        // Find chunks for the files in this composition
        let mut component_chunks: Vec<&AtomicChunk> = Vec::new();

        for file in &suggestion.files {
            let file_str = file.to_string_lossy();
            for chunk in chunks {
                if chunk.sources.iter().any(|s| file_str.contains(&s.file)) {
                    component_chunks.push(chunk);
                }
            }
        }

        if component_chunks.len() < self.config.min_composition_files {
            return None;
        }

        // Create composition content hash from component hashes
        let mut hasher = Sha256::new();
        for chunk in &component_chunks {
            hasher.update(chunk.chunk_id.as_bytes());
        }
        let content_hash = hex::encode(hasher.finalize());
        let chunk_id = format!("chunk:sha256:{}", content_hash);

        // Create references to component chunks
        let composed_of: Vec<ChunkReference> = component_chunks
            .iter()
            .map(|c| ChunkReference {
                chunk_id: c.chunk_id.clone(),
                alias: c.primary_alias().map(|a| a.full_path()),
                required: true,
                imports: c.provides.clone(),
            })
            .collect();

        let total_size: usize = component_chunks.iter().map(|c| c.size).sum();

        let alias = registry.generate_unique(&suggestion.name);
        registry.register(&alias, &chunk_id);

        let mut chunk = AtomicChunk::new(
            chunk_id,
            suggestion.name.clone(),
            "composition".to_string(),
            content_hash,
            total_size,
        )
        .with_alias(&alias)
        .with_granularity(ChunkGranularity::Package)
        .with_categories(vec![suggestion.category.clone()]);

        chunk.description = Some(suggestion.description.clone());
        chunk.composition = ChunkComposition {
            composed_of,
            composed_by: Vec::new(),
            is_atomic: false,
            composition_strategy: Some("aggregate".to_string()),
        };

        // Aggregate concepts from components
        chunk.concepts = component_chunks
            .iter()
            .flat_map(|c| c.concepts.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        chunk.provides = component_chunks
            .iter()
            .flat_map(|c| c.provides.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        Some(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_type_detection() {
        let analyzer = ProjectAnalyzer::default();
        let mut counts = HashMap::new();
        counts.insert("rust".to_string(), 10);
        counts.insert("toml".to_string(), 2);

        // Without project files, uses language counts
        let (project_type, lang) =
            analyzer.detect_project_type(Path::new("/fake/path"), &counts);
        assert_eq!(lang, "rust");
    }

    #[test]
    fn test_is_source_file() {
        let analyzer = ProjectAnalyzer::default();
        assert!(analyzer.is_source_file(Path::new("test.rs")));
        assert!(analyzer.is_source_file(Path::new("test.ts")));
        assert!(analyzer.is_source_file(Path::new("test.py")));
        assert!(!analyzer.is_source_file(Path::new("test.exe")));
        assert!(!analyzer.is_source_file(Path::new("test.bin")));
    }
}
