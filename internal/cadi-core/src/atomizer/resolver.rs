//! Symbol Resolver
//!
//! Resolves imports to content-addressed chunk IDs.
//! This is the core of the "Smart Atomizer" - when we see:
//!
//! ```rust,ignore
//! use crate::utils::helper;
//! ```
//!
//! We resolve `helper` to its chunk ID: `chunk:sha256:abc123...`

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{CadiError, CadiResult};
use crate::graph::GraphStore;

/// Symbol resolver for import-to-chunk mapping
pub struct SymbolResolver {
    /// Cache of resolved symbols: (file, symbol) -> chunk_id
    cache: HashMap<(PathBuf, String), String>,
    
    /// Project root for relative path resolution
    project_root: PathBuf,
    
    /// Language being resolved
    language: String,
}

/// A raw import statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawImport {
    /// The import path (e.g., "./utils", "lodash", "crate::utils")
    pub source: String,
    
    /// Symbols being imported
    pub symbols: Vec<RawSymbol>,
    
    /// Is this a default import?
    pub is_default: bool,
    
    /// Is this a namespace import (import * as)?
    pub is_namespace: bool,
    
    /// Line number where this import appears
    pub line: usize,
}

/// A raw symbol in an import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawSymbol {
    /// Original name in the source module
    pub name: String,
    
    /// Alias if renamed (import { x as y })
    pub alias: Option<String>,
}

/// A resolved import with chunk references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedImport {
    /// Original import path
    pub source_path: String,
    
    /// Imported symbols with their resolutions
    pub symbols: Vec<ImportedSymbol>,
    
    /// Line number
    pub line: usize,
}

/// A symbol resolved to its chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedSymbol {
    /// Symbol name
    pub name: String,
    
    /// Alias if renamed
    pub alias: Option<String>,
    
    /// Chunk ID where this symbol is defined
    pub chunk_id: String,
    
    /// Hash of the chunk
    pub chunk_hash: String,
    
    /// Type of the symbol (function, type, etc.)
    pub symbol_type: Option<String>,
}

impl SymbolResolver {
    /// Create a new resolver for a project
    pub fn new(project_root: impl Into<PathBuf>, language: impl Into<String>) -> Self {
        Self {
            cache: HashMap::new(),
            project_root: project_root.into(),
            language: language.into(),
        }
    }

    /// Clear the resolution cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Extract raw imports from source code
    pub fn extract_imports(&self, source: &str) -> Vec<RawImport> {
        match self.language.as_str() {
            "rust" => self.extract_rust_imports(source),
            "typescript" | "javascript" => self.extract_ts_imports(source),
            "python" => self.extract_python_imports(source),
            "c" | "cpp" => self.extract_c_imports(source),
            "csharp" => self.extract_csharp_imports(source),
            "css" => self.extract_css_imports(source),
            "glsl" => self.extract_glsl_imports(source),
            _ => Vec::new(),
        }
    }

    /// Resolve imports using a graph store
    pub fn resolve_imports(
        &mut self,
        current_file: &Path,
        imports: Vec<RawImport>,
        graph: &GraphStore,
    ) -> CadiResult<Vec<ResolvedImport>> {
        let mut resolved = Vec::new();

        for import in imports {
            let target_path = self.resolve_path(current_file, &import.source)?;
            let mut resolved_symbols = Vec::new();

            for sym in import.symbols {
                // Check cache first
                let cache_key = (target_path.clone(), sym.name.clone());
                
                if let Some(chunk_id) = self.cache.get(&cache_key) {
                    resolved_symbols.push(ImportedSymbol {
                        name: sym.name,
                        alias: sym.alias,
                        chunk_id: chunk_id.clone(),
                        chunk_hash: extract_hash(chunk_id),
                        symbol_type: None,
                    });
                    continue;
                }

                // Try to find in graph
                if let Ok(Some(chunk_id)) = graph.find_symbol(&sym.name) {
                    self.cache.insert(cache_key, chunk_id.clone());
                    resolved_symbols.push(ImportedSymbol {
                        name: sym.name,
                        alias: sym.alias,
                        chunk_id: chunk_id.clone(),
                        chunk_hash: extract_hash(&chunk_id),
                        symbol_type: None,
                    });
                } else {
                    // Symbol not found in graph - mark as unresolved
                    resolved_symbols.push(ImportedSymbol {
                        name: sym.name,
                        alias: sym.alias,
                        chunk_id: "unresolved".to_string(),
                        chunk_hash: String::new(),
                        symbol_type: None,
                    });
                }
            }

            resolved.push(ResolvedImport {
                source_path: import.source,
                symbols: resolved_symbols,
                line: import.line,
            });
        }

        Ok(resolved)
    }

    /// Create link references for use in atomized code
    /// 
    /// Transforms `import { X } from './y'` into `link:sha256:abc123`
    pub fn create_link_references(&self, imports: &[ResolvedImport]) -> HashMap<String, String> {
        let mut links = HashMap::new();

        for import in imports {
            for sym in &import.symbols {
                if !sym.chunk_id.is_empty() && sym.chunk_id != "unresolved" {
                    let key = sym.alias.as_ref().unwrap_or(&sym.name).clone();
                    links.insert(key, format!("link:{}", sym.chunk_id));
                }
            }
        }

        links
    }

    // ========================================================================
    // Language-specific import extraction
    // ========================================================================

    fn extract_rust_imports(&self, source: &str) -> Vec<RawImport> {
        let mut imports = Vec::new();

        // Match use statements
        let use_regex = regex::Regex::new(
            r"(?m)^use\s+([\w:]+)(?:::\{([^}]+)\})?;"
        ).unwrap();

        for (line_idx, line) in source.lines().enumerate() {
            if let Some(cap) = use_regex.captures(line) {
                let path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                
                let symbols = if let Some(group) = cap.get(2) {
                    // use foo::{a, b, c}
                    group.as_str()
                        .split(',')
                        .map(|s| {
                            let s = s.trim();
                            if s.contains(" as ") {
                                let parts: Vec<&str> = s.split(" as ").collect();
                                RawSymbol {
                                    name: parts[0].trim().to_string(),
                                    alias: Some(parts[1].trim().to_string()),
                                }
                            } else {
                                RawSymbol {
                                    name: s.to_string(),
                                    alias: None,
                                }
                            }
                        })
                        .collect()
                } else {
                    // use foo::bar; (single import)
                    let name = path.split("::").last().unwrap_or("").to_string();
                    if name.is_empty() {
                        continue;
                    }
                    vec![RawSymbol { name, alias: None }]
                };

                imports.push(RawImport {
                    source: path.to_string(),
                    symbols,
                    is_default: false,
                    is_namespace: path.ends_with("::*"),
                    line: line_idx + 1,
                });
            }
        }

        imports
    }

    fn extract_ts_imports(&self, source: &str) -> Vec<RawImport> {
        let mut imports = Vec::new();

        // Named imports: import { a, b } from 'path'
        let named_regex = regex::Regex::new(
            r#"import\s*\{([^}]+)\}\s*from\s*['"]([^'"]+)['"]"#
        ).unwrap();

        // Default imports: import X from 'path'
        let default_regex = regex::Regex::new(
            r#"import\s+(\w+)\s+from\s*['"]([^'"]+)['"]"#
        ).unwrap();

        // Namespace imports: import * as X from 'path'
        let namespace_regex = regex::Regex::new(
            r#"import\s*\*\s*as\s+(\w+)\s+from\s*['"]([^'"]+)['"]"#
        ).unwrap();

        for (line_idx, line) in source.lines().enumerate() {
            // Named imports
            if let Some(cap) = named_regex.captures(line) {
                let symbols_str = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let path = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                let symbols: Vec<RawSymbol> = symbols_str
                    .split(',')
                    .map(|s| {
                        let s = s.trim();
                        if s.contains(" as ") {
                            let parts: Vec<&str> = s.split(" as ").collect();
                            RawSymbol {
                                name: parts[0].trim().to_string(),
                                alias: Some(parts[1].trim().to_string()),
                            }
                        } else {
                            RawSymbol {
                                name: s.to_string(),
                                alias: None,
                            }
                        }
                    })
                    .filter(|s| !s.name.is_empty())
                    .collect();

                imports.push(RawImport {
                    source: path.to_string(),
                    symbols,
                    is_default: false,
                    is_namespace: false,
                    line: line_idx + 1,
                });
            }
            // Default imports
            else if let Some(cap) = default_regex.captures(line) {
                let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let path = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                imports.push(RawImport {
                    source: path.to_string(),
                    symbols: vec![RawSymbol {
                        name: name.to_string(),
                        alias: None,
                    }],
                    is_default: true,
                    is_namespace: false,
                    line: line_idx + 1,
                });
            }
            // Namespace imports
            else if let Some(cap) = namespace_regex.captures(line) {
                let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let path = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                imports.push(RawImport {
                    source: path.to_string(),
                    symbols: vec![RawSymbol {
                        name: name.to_string(),
                        alias: None,
                    }],
                    is_default: false,
                    is_namespace: true,
                    line: line_idx + 1,
                });
            }
        }

        imports
    }

    fn extract_python_imports(&self, source: &str) -> Vec<RawImport> {
        let mut imports = Vec::new();

        // from x import a, b, c
        let from_regex = regex::Regex::new(
            r"from\s+([\w.]+)\s+import\s+(.+)"
        ).unwrap();

        // import x, y, z
        let import_regex = regex::Regex::new(
            r"^import\s+([\w., ]+)"
        ).unwrap();

        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            
            if let Some(cap) = from_regex.captures(trimmed) {
                let path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let symbols_str = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                let symbols: Vec<RawSymbol> = symbols_str
                    .split(',')
                    .map(|s| {
                        let s = s.trim();
                        if s.contains(" as ") {
                            let parts: Vec<&str> = s.split(" as ").collect();
                            RawSymbol {
                                name: parts[0].trim().to_string(),
                                alias: Some(parts[1].trim().to_string()),
                            }
                        } else {
                            RawSymbol {
                                name: s.to_string(),
                                alias: None,
                            }
                        }
                    })
                    .filter(|s| !s.name.is_empty())
                    .collect();

                imports.push(RawImport {
                    source: path.to_string(),
                    symbols,
                    is_default: false,
                    is_namespace: false,
                    line: line_idx + 1,
                });
            } else if let Some(cap) = import_regex.captures(trimmed) {
                let modules = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                
                for module in modules.split(',') {
                    let module = module.trim();
                    let (name, alias) = if module.contains(" as ") {
                        let parts: Vec<&str> = module.split(" as ").collect();
                        (parts[0].trim().to_string(), Some(parts[1].trim().to_string()))
                    } else {
                        (module.to_string(), None)
                    };

                    imports.push(RawImport {
                        source: name.clone(),
                        symbols: vec![RawSymbol { name, alias }],
                        is_default: false,
                        is_namespace: true,
                        line: line_idx + 1,
                    });
                }
            }
        }

        imports
    }

    fn extract_c_imports(&self, source: &str) -> Vec<RawImport> {
        let mut imports = Vec::new();
        let include_regex = regex::Regex::new(r#"#include\s+["<]([^">]+)[">]"#).unwrap();

        for (line_idx, line) in source.lines().enumerate() {
            if let Some(cap) = include_regex.captures(line) {
                let path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                imports.push(RawImport {
                    source: path.to_string(),
                    symbols: Vec::new(), // C includes don't have explicit symbols
                    is_default: false,
                    is_namespace: true,
                    line: line_idx + 1,
                });
            }
        }
        imports
    }

    fn extract_csharp_imports(&self, source: &str) -> Vec<RawImport> {
        let mut imports = Vec::new();
        let using_regex = regex::Regex::new(r#"using\s+([\w.]+);"#).unwrap();

        for (line_idx, line) in source.lines().enumerate() {
            if let Some(cap) = using_regex.captures(line) {
                let path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                imports.push(RawImport {
                    source: path.to_string(),
                    symbols: Vec::new(),
                    is_default: false,
                    is_namespace: true,
                    line: line_idx + 1,
                });
            }
        }
        imports
    }

    fn extract_css_imports(&self, source: &str) -> Vec<RawImport> {
        let mut imports = Vec::new();
        let import_regex = regex::Regex::new(r#"@import\s+['"]([^'"]+)['"]"#).unwrap();

        for (line_idx, line) in source.lines().enumerate() {
            if let Some(cap) = import_regex.captures(line) {
                let path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                imports.push(RawImport {
                    source: path.to_string(),
                    symbols: Vec::new(),
                    is_default: false,
                    is_namespace: true,
                    line: line_idx + 1,
                });
            }
        }
        imports
    }

    fn extract_glsl_imports(&self, source: &str) -> Vec<RawImport> {
        // GLSL often uses #include if extended, or we can look for vendor-specific includes
        self.extract_c_imports(source)
    }

    // ========================================================================
    // Path resolution
    // ========================================================================

    fn resolve_path(&self, current_file: &Path, import_path: &str) -> CadiResult<PathBuf> {
        match self.language.as_str() {
            "rust" => self.resolve_rust_path(current_file, import_path),
            "typescript" | "javascript" => self.resolve_ts_path(current_file, import_path),
            "python" => self.resolve_python_path(current_file, import_path),
            "c" | "cpp" | "glsl" => self.resolve_c_path(current_file, import_path),
            "csharp" => self.resolve_csharp_path(current_file, import_path),
            "css" => self.resolve_css_path(current_file, import_path),
            _ => Ok(PathBuf::from(import_path)),
        }
    }

    fn resolve_rust_path(&self, current_file: &Path, import_path: &str) -> CadiResult<PathBuf> {
        // Handle crate::, super::, self::
        if import_path.starts_with("crate::") {
            let relative = import_path.strip_prefix("crate::").unwrap();
            let parts: Vec<&str> = relative.split("::").collect();
            
            let mut path = self.project_root.join("src");
            for part in &parts[..parts.len().saturating_sub(1)] {
                path = path.join(part);
            }
            path = path.with_extension("rs");
            
            Ok(path)
        } else if import_path.starts_with("super::") {
            let relative = import_path.strip_prefix("super::").unwrap();
            let parent = current_file.parent().and_then(|p| p.parent()).unwrap_or(&self.project_root);
            Ok(parent.join(relative.replace("::", "/")).with_extension("rs"))
        } else {
            // External crate - just return the path as-is
            Ok(PathBuf::from(import_path.replace("::", "/")))
        }
    }

    fn resolve_ts_path(&self, current_file: &Path, import_path: &str) -> CadiResult<PathBuf> {
        if import_path.starts_with('.') {
            // Relative import
            let parent = current_file.parent().unwrap_or(&self.project_root);
            let resolved = parent.join(import_path);
            
            // Try various extensions
            for ext in &["ts", "tsx", "js", "jsx", "index.ts", "index.js"] {
                let with_ext = resolved.with_extension(ext);
                if with_ext.exists() {
                    return Ok(with_ext);
                }
            }
            
            Ok(resolved)
        } else if import_path.starts_with('@') || import_path.starts_with("~") {
            // Aliased import - would need tsconfig.json to resolve
            Ok(PathBuf::from(import_path))
        } else {
            // node_modules import
            Ok(self.project_root.join("node_modules").join(import_path))
        }
    }

    fn resolve_python_path(&self, current_file: &Path, import_path: &str) -> CadiResult<PathBuf> {
        if import_path.starts_with('.') {
            // Relative import
            let dots = import_path.chars().take_while(|c| *c == '.').count();
            let rest = &import_path[dots..];
            
            let mut base = current_file.to_path_buf();
            for _ in 0..=dots {
                base = base.parent().unwrap_or(&self.project_root).to_path_buf();
            }
            
            Ok(base.join(rest.replace('.', "/")).with_extension("py"))
        } else {
            // Absolute import
            Ok(self.project_root.join(import_path.replace('.', "/")).with_extension("py"))
        }
    }

    fn resolve_c_path(&self, current_file: &Path, import_path: &str) -> CadiResult<PathBuf> {
        let parent = current_file.parent().unwrap_or(&self.project_root);
        let resolved = parent.join(import_path);
        
        if resolved.exists() {
            Ok(resolved)
        } else {
            // Check common include directories if not found relatively
            for dir in &["include", "src"] {
                let candidate = self.project_root.join(dir).join(import_path);
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
            Ok(resolved)
        }
    }

    fn resolve_csharp_path(&self, _current_file: &Path, import_path: &str) -> CadiResult<PathBuf> {
        // C# namespaces don't map directly to paths always, but we'll try
        Ok(self.project_root.join(import_path.replace('.', "/")).with_extension("cs"))
    }

    fn resolve_css_path(&self, current_file: &Path, import_path: &str) -> CadiResult<PathBuf> {
        let parent = current_file.parent().unwrap_or(&self.project_root);
        Ok(parent.join(import_path))
    }
}

/// Extract hash from chunk ID
fn extract_hash(chunk_id: &str) -> String {
    chunk_id
        .split(':')
        .last()
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_import_extraction() {
        let source = r#"
use std::collections::HashMap;
use crate::utils::{helper, Logger as Log};
use super::parent_module;
"#;

        let resolver = SymbolResolver::new("/project", "rust");
        let imports = resolver.extract_imports(source);

        assert_eq!(imports.len(), 3);
        assert_eq!(imports[0].source, "std::collections::HashMap");
        assert_eq!(imports[1].symbols.len(), 2);
        assert_eq!(imports[1].symbols[1].alias, Some("Log".to_string()));
    }

    #[test]
    fn test_ts_import_extraction() {
        let source = r#"
import { foo, bar as baz } from './utils';
import React from 'react';
import * as lodash from 'lodash';
"#;

        let resolver = SymbolResolver::new("/project", "typescript");
        let imports = resolver.extract_imports(source);

        assert_eq!(imports.len(), 3);
        assert!(!imports[0].is_default);
        assert!(imports[1].is_default);
        assert!(imports[2].is_namespace);
    }

    #[test]
    fn test_python_import_extraction() {
        let source = r#"
from os import path, getcwd as cwd
import json, yaml
from .utils import helper
"#;

        let resolver = SymbolResolver::new("/project", "python");
        let imports = resolver.extract_imports(source);

        assert_eq!(imports.len(), 4); // from os, import json, import yaml, from .utils
    }

    #[test]
    fn test_c_import_extraction() {
        let source = r#"
#include <stdio.h>
#include "my_header.h"
"#;
        let resolver = SymbolResolver::new("/project", "c");
        let imports = resolver.extract_imports(source);
        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].source, "stdio.h");
        assert_eq!(imports[1].source, "my_header.h");
    }

    #[test]
    fn test_csharp_import_extraction() {
        let source = r#"
using System;
using System.Collections.Generic;
"#;
        let resolver = SymbolResolver::new("/project", "csharp");
        let imports = resolver.extract_imports(source);
        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].source, "System");
        assert_eq!(imports[1].source, "System.Collections.Generic");
    }

    #[test]
    fn test_css_import_extraction() {
        let source = r#"
@import "base.css";
@import 'themes/dark.css';
"#;
        let resolver = SymbolResolver::new("/project", "css");
        let imports = resolver.extract_imports(source);
        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].source, "base.css");
        assert_eq!(imports[1].source, "themes/dark.css");
    }
}
