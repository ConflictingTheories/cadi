//! Atom Extractor
//!
//! Extracts properly bounded code atoms from ASTs.

use serde::{Deserialize, Serialize};

use super::config::AtomizerConfig;
use super::resolver::SymbolResolver;
use crate::error::{CadiError, CadiResult};

/// Kind of code atom
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtomKind {
    /// A function or method
    Function,
    /// An async function
    AsyncFunction,
    /// A method within a class/impl
    Method,
    /// A struct (Rust) or interface (TS)
    Struct,
    /// A class
    Class,
    /// A trait (Rust) or abstract class
    Trait,
    /// An interface (TS/Java)
    Interface,
    /// An enum
    Enum,
    /// A constant or static value
    Constant,
    /// A type alias
    TypeAlias,
    /// A module or namespace
    Module,
    /// A macro
    Macro,
    /// An impl block (Rust)
    ImplBlock,
    /// A decorator/attribute (Python/Rust)
    Decorator,
    /// An import statement or require
    Import,
    /// A header file inclusion (C/C++)
    Header,
}

impl AtomKind {
    /// Is this a type definition?
    pub fn is_type(&self) -> bool {
        matches!(
            self,
            AtomKind::Struct
                | AtomKind::Class
                | AtomKind::Trait
                | AtomKind::Interface
                | AtomKind::Enum
                | AtomKind::TypeAlias
        )
    }

    /// Is this a dependency/import?
    pub fn is_dependency(&self) -> bool {
        matches!(
            self,
            AtomKind::Import | AtomKind::Header
        )
    }

    /// Is this executable code?
    pub fn is_executable(&self) -> bool {
        matches!(
            self,
            AtomKind::Function | AtomKind::AsyncFunction | AtomKind::Method
        )
    }
}

/// An extracted code atom
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedAtom {
    /// Name of the atom (function name, type name, etc.)
    pub name: String,

    /// Kind of atom
    pub kind: AtomKind,

    /// The source code of this atom
    pub source: String,

    /// Byte offset in original source
    pub start_byte: usize,
    pub end_byte: usize,

    /// Line numbers (1-indexed)
    pub start_line: usize,
    pub end_line: usize,

    /// Symbols this atom defines/exports
    pub defines: Vec<String>,

    /// Symbols this atom references/imports
    pub references: Vec<String>,

    /// Doc comment if present
    pub doc_comment: Option<String>,

    /// Visibility (public, private, etc.)
    pub visibility: Visibility,

    /// Parent atom (for nested items like methods in a class)
    pub parent: Option<String>,

    /// Decorators/attributes applied to this atom
    pub decorators: Vec<String>,
}

/// Visibility level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    /// Public (exported)
    Public,
    /// Private (not exported)
    #[default]
    Private,
    /// Package/crate-level visibility
    Internal,
    /// Protected (visible to subclasses)
    Protected,
}

/// Atom extractor for a specific language
pub struct AtomExtractor {
    #[allow(unused)]
    config: AtomizerConfig,
    language: String,
}

impl AtomExtractor {
    /// Create a new extractor for a language
    pub fn new(language: impl Into<String>, config: AtomizerConfig) -> Self {
        Self {
            config,
            language: language.into(),
        }
    }

    /// Extract atoms from source code
    /// 
    /// When the `ast-parsing` feature is enabled, this uses Tree-sitter.
    /// Otherwise, falls back to regex-based extraction.
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        #[cfg(feature = "ast-parsing")]
        {
            use crate::atomizer::languages::*;
            match self.language.as_str() {
                "rust" => return RustAtomizer::new(self.config.clone()).extract(source),
                "c" | "cpp" => return CAtomizer::new(self.config.clone()).extract(source),
                "csharp" => return CSharpAtomizer::new(self.config.clone()).extract(source),
                "css" => return CssAtomizer::new(self.config.clone()).extract(source),
                "glsl" => return GlslAtomizer::new(self.config.clone()).extract(source),
                "typescript" | "javascript" | "tsx" | "jsx" => return TypeScriptAtomizer::new(self.config.clone()).extract(source),
                "python" => return PythonAtomizer::new(self.config.clone()).extract(source),
                "html" => return HtmlAtomizer::new(self.config.clone()).extract(source),
                "go" => return GoAtomizer::new(self.config.clone()).extract(source),
                _ => {}
            }
        }

        let result: CadiResult<Vec<ExtractedAtom>> = {
            #[cfg(feature = "ast-parsing")]
            {
                use crate::atomizer::languages::*;
                match self.language.as_str() {
                    "rust" => return RustAtomizer::new(self.config.clone()).extract(source),
                    "c" | "cpp" => return CAtomizer::new(self.config.clone()).extract(source),
                    "csharp" => return CSharpAtomizer::new(self.config.clone()).extract(source),
                    "css" => return CssAtomizer::new(self.config.clone()).extract(source),
                    "glsl" => return GlslAtomizer::new(self.config.clone()).extract(source),
                    _ => {}
                }
            }

            match self.language.as_str() {
                "rust" => self.extract_rust(source),
                "typescript" | "javascript" => self.extract_typescript(source),
                "python" => self.extract_python(source),
                "c" | "cpp" => self.extract_c(source),
                "csharp" => self.extract_csharp(source),
                "css" => self.extract_css(source),
                "glsl" => self.extract_glsl(source),
                _ => self.extract_fallback(source),
            }
        };

        // Post-processing: Extract imports as first-class atoms
        if let Ok(mut atoms) = result {
            if let Ok(imports) = self.extract_imports_as_atoms(source) {
                atoms.extend(imports);
            }
            Ok(atoms)
        } else {
            result
        }
    }

    /// Extract imports as atoms using SymbolResolver logic
    fn extract_imports_as_atoms(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        // Use a dummy path since we only need extraction logic, not resolution
        let resolver = SymbolResolver::new(std::path::PathBuf::from("/"), &self.language);
        let raw_imports = resolver.extract_imports(source);
        let mut atoms = Vec::new();
        
        // Cache source lines for quick access
        let source_lines: Vec<&str> = source.lines().collect();
        let mut byte_offset = 0;
        let mut line_offsets = Vec::new();
        for line in &source_lines {
            line_offsets.push(byte_offset);
            byte_offset += line.len() + 1; // +1 for newline
        }

        for import in raw_imports {
            if import.line == 0 || import.line > source_lines.len() {
                continue;
            }

            let line_idx = import.line - 1;
            let line_content = source_lines[line_idx];
            let start_byte = line_offsets[line_idx];
            let end_byte = start_byte + line_content.len();

            let kind = if self.language == "c" || self.language == "cpp" || self.language == "glsl" {
                AtomKind::Header
            } else {
                AtomKind::Import
            };

            // Defines: symbols imported by this statement
            let mut defines = Vec::new();
            if !import.is_namespace && !import.symbols.is_empty() {
                for sym in &import.symbols {
                    if let Some(alias) = &sym.alias {
                        defines.push(alias.clone());
                    } else {
                        defines.push(sym.name.clone());
                    }
                }
            } else if import.is_namespace {
                // For namespace imports, it defines the namespace name
                if let Some(first) = import.symbols.first() {
                    defines.push(first.name.clone());
                }
            }
            // For C headers, we might say it "defines" the header name itself as a crude proxy
            // or leave it empty. The graph will link based on the AtomKind::Header edge.
            if kind == AtomKind::Header {
                defines.push(import.source.clone());
            }

            atoms.push(ExtractedAtom {
                name: import.source.clone(),
                kind,
                source: line_content.to_string(),
                start_byte,
                end_byte,
                start_line: import.line,
                end_line: import.line,
                defines,
                references: vec![], // Imports don't strictly refer to other user code, they refer to external chunks
                doc_comment: None,
                visibility: Visibility::Private, // Imports are local unless re-exported
                parent: None,
                decorators: Vec::new(),
            });
        }

        Ok(atoms)
    }

    /// Extract atoms from Rust source
    fn extract_rust(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        let mut atoms = Vec::new();
        let _lines: Vec<&str> = source.lines().collect();

        // Regex-based extraction (fallback when tree-sitter not enabled)
        // This is a simplified version - the real implementation uses Tree-sitter
        
        let fn_regex = regex::Regex::new(
            r"(?m)^(\s*)(///.*\n)*(\s*)(?:pub(?:\([^)]*\))?\s+)?(async\s+)?fn\s+(\w+)"
        ).map_err(|e| CadiError::AtomizerError(e.to_string()))?;

        let struct_regex = regex::Regex::new(
            r"(?m)^(\s*)(///.*\n)*(\s*)(?:pub(?:\([^)]*\))?\s+)?struct\s+(\w+)"
        ).map_err(|e| CadiError::AtomizerError(e.to_string()))?;

        let _enum_regex = regex::Regex::new(
            r"(?m)^(\s*)(///.*\n)*(\s*)(?:pub(?:\([^)]*\))?\s+)?enum\s+(\w+)"
        ).map_err(|e| CadiError::AtomizerError(e.to_string()))?;

        let _trait_regex = regex::Regex::new(
            r"(?m)^(\s*)(///.*\n)*(\s*)(?:pub(?:\([^)]*\))?\s+)?trait\s+(\w+)"
        ).map_err(|e| CadiError::AtomizerError(e.to_string()))?;

        let _impl_regex = regex::Regex::new(
            r"(?m)^impl(?:<[^>]*>)?\s+(?:(\w+)\s+for\s+)?(\w+)"
        ).map_err(|e| CadiError::AtomizerError(e.to_string()))?;

        // Extract functions
        for cap in fn_regex.captures_iter(source) {
            let name = cap.get(5).map(|m| m.as_str()).unwrap_or("unknown");
            let is_async = cap.get(4).is_some();
            let is_pub = source[..cap.get(0).unwrap().start()]
                .lines()
                .last()
                .map(|l| l.contains("pub"))
                .unwrap_or(false);

            let start_byte = cap.get(0).unwrap().start();
            let end_byte = self.find_block_end(source, start_byte);
            
            let start_line = source[..start_byte].matches('\n').count() + 1;
            let end_line = source[..end_byte].matches('\n').count() + 1;

            atoms.push(ExtractedAtom {
                name: name.to_string(),
                kind: if is_async { AtomKind::AsyncFunction } else { AtomKind::Function },
                source: source[start_byte..end_byte].to_string(),
                start_byte,
                end_byte,
                start_line,
                end_line,
                defines: vec![name.to_string()],
                references: self.extract_references(&source[start_byte..end_byte]),
                doc_comment: self.extract_doc_comment(source, start_byte),
                visibility: if is_pub { Visibility::Public } else { Visibility::Private },
                parent: None,
                decorators: Vec::new(),
            });
        }

        // Extract structs
        for cap in struct_regex.captures_iter(source) {
            let name = cap.get(4).map(|m| m.as_str()).unwrap_or("unknown");
            let start_byte = cap.get(0).unwrap().start();
            let end_byte = self.find_block_end(source, start_byte);
            
            atoms.push(ExtractedAtom {
                name: name.to_string(),
                kind: AtomKind::Struct,
                source: source[start_byte..end_byte].to_string(),
                start_byte,
                end_byte,
                start_line: source[..start_byte].matches('\n').count() + 1,
                end_line: source[..end_byte].matches('\n').count() + 1,
                defines: vec![name.to_string()],
                references: self.extract_references(&source[start_byte..end_byte]),
                doc_comment: self.extract_doc_comment(source, start_byte),
                visibility: Visibility::Public, // simplified
                parent: None,
                decorators: Vec::new(),
            });
        }

        // Similar for enums, traits, impl blocks...
        // (abbreviated for clarity)

        Ok(atoms)
    }

    /// Extract atoms from TypeScript/JavaScript source
    fn extract_typescript(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        let mut atoms = Vec::new();

        let fn_regex = regex::Regex::new(
            r"(?m)^(\s*)(export\s+)?(async\s+)?function\s+(\w+)"
        ).map_err(|e| CadiError::AtomizerError(e.to_string()))?;

        let class_regex = regex::Regex::new(
            r"(?m)^(\s*)(export\s+)?class\s+(\w+)"
        ).map_err(|e| CadiError::AtomizerError(e.to_string()))?;

        let _interface_regex = regex::Regex::new(
            r"(?m)^(\s*)(export\s+)?interface\s+(\w+)"
        ).map_err(|e| CadiError::AtomizerError(e.to_string()))?;

        let const_regex = regex::Regex::new(
            r"(?m)^(\s*)(export\s+)?const\s+(\w+).*?="
        ).map_err(|e| CadiError::AtomizerError(e.to_string()))?;

        // Extract functions
        for cap in fn_regex.captures_iter(source) {
            let name = cap.get(4).map(|m| m.as_str()).unwrap_or("unknown");
            let is_async = cap.get(3).is_some();
            let is_export = cap.get(2).is_some();

            let start_byte = cap.get(0).unwrap().start();
            let end_byte = self.find_block_end(source, start_byte);

            atoms.push(ExtractedAtom {
                name: name.to_string(),
                kind: if is_async { AtomKind::AsyncFunction } else { AtomKind::Function },
                source: source[start_byte..end_byte].to_string(),
                start_byte,
                end_byte,
                start_line: source[..start_byte].matches('\n').count() + 1,
                end_line: source[..end_byte].matches('\n').count() + 1,
                defines: vec![name.to_string()],
                references: self.extract_ts_imports(source) // simplified
                    .into_iter()
                    .flat_map(|(_, syms)| syms)
                    .collect(),
                doc_comment: self.extract_jsdoc(source, start_byte),
                visibility: if is_export { Visibility::Public } else { Visibility::Private },
                parent: None,
                decorators: Vec::new(),
            });
        }

        // Classes, interfaces, etc.
        for cap in class_regex.captures_iter(source) {
            let name = cap.get(3).map(|m| m.as_str()).unwrap_or("unknown");
            let start_byte = cap.get(0).unwrap().start();
            let end_byte = self.find_block_end(source, start_byte);

            atoms.push(ExtractedAtom {
                name: name.to_string(),
                kind: AtomKind::Class,
                source: source[start_byte..end_byte].to_string(),
                start_byte,
                end_byte,
                start_line: source[..start_byte].matches('\n').count() + 1,
                end_line: source[..end_byte].matches('\n').count() + 1,
                defines: vec![name.to_string()],
                references: Vec::new(),
                doc_comment: None,
                visibility: Visibility::Public,
                parent: None,
                decorators: Vec::new(),
            });
        }

        // Constants
        for cap in const_regex.captures_iter(source) {
            let name = cap.get(3).map(|m| m.as_str()).unwrap_or("unknown");
            let is_export = cap.get(2).is_some();
            let start_byte = cap.get(0).unwrap().start();
            let end_byte = self.find_statement_end(source, start_byte);

            atoms.push(ExtractedAtom {
                name: name.to_string(),
                kind: AtomKind::Constant,
                source: source[start_byte..end_byte].to_string(),
                start_byte,
                end_byte,
                start_line: source[..start_byte].matches('\n').count() + 1,
                end_line: source[..end_byte].matches('\n').count() + 1,
                defines: vec![name.to_string()],
                references: Vec::new(),
                doc_comment: None,
                visibility: if is_export { Visibility::Public } else { Visibility::Private },
                parent: None,
                decorators: Vec::new(),
            });
        }

        Ok(atoms)
    }

    /// Extract atoms from Python source
    fn extract_python(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        let mut atoms = Vec::new();

        let fn_regex = regex::Regex::new(
            r"(?m)^(\s*)(async\s+)?def\s+(\w+)\s*\("
        ).map_err(|e| CadiError::AtomizerError(e.to_string()))?;

        let class_regex = regex::Regex::new(
            r"(?m)^(\s*)class\s+(\w+)"
        ).map_err(|e| CadiError::AtomizerError(e.to_string()))?;

        for cap in fn_regex.captures_iter(source) {
            let indent = cap.get(1).map(|m| m.as_str().len()).unwrap_or(0);
            let name = cap.get(3).map(|m| m.as_str()).unwrap_or("unknown");
            let is_async = cap.get(2).is_some();

            let start_byte = cap.get(0).unwrap().start();
            let end_byte = self.find_python_block_end(source, start_byte, indent);

            atoms.push(ExtractedAtom {
                name: name.to_string(),
                kind: if is_async { AtomKind::AsyncFunction } else { AtomKind::Function },
                source: source[start_byte..end_byte].to_string(),
                start_byte,
                end_byte,
                start_line: source[..start_byte].matches('\n').count() + 1,
                end_line: source[..end_byte].matches('\n').count() + 1,
                defines: vec![name.to_string()],
                references: Vec::new(),
                doc_comment: self.extract_python_docstring(source, start_byte),
                visibility: if name.starts_with('_') { Visibility::Private } else { Visibility::Public },
                parent: None,
                decorators: Vec::new(),
            });
        }

        for cap in class_regex.captures_iter(source) {
            let indent = cap.get(1).map(|m| m.as_str().len()).unwrap_or(0);
            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("unknown");

            let start_byte = cap.get(0).unwrap().start();
            let end_byte = self.find_python_block_end(source, start_byte, indent);

            atoms.push(ExtractedAtom {
                name: name.to_string(),
                kind: AtomKind::Class,
                source: source[start_byte..end_byte].to_string(),
                start_byte,
                end_byte,
                start_line: source[..start_byte].matches('\n').count() + 1,
                end_line: source[..end_byte].matches('\n').count() + 1,
                defines: vec![name.to_string()],
                references: Vec::new(),
                doc_comment: None,
                visibility: Visibility::Public,
                parent: None,
                decorators: Vec::new(),
            });
        }

        Ok(atoms)
    }

    /// Extract atoms from C source
    fn extract_c(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        // Simplified fallback: extract functions and structs
        let mut atoms = Vec::new();
        let fn_regex = regex::Regex::new(r"(?m)^\s*(\w+)\s+(\w+)\s*\([^)]*\)\s*\{").unwrap();
        let struct_regex = regex::Regex::new(r"(?m)^\s*struct\s+(\w+)\s*\{").unwrap();

        for cap in fn_regex.captures_iter(source) {
            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("unknown");
            let start_byte = cap.get(0).unwrap().start();
            let end_byte = self.find_block_end(source, start_byte);

            atoms.push(ExtractedAtom {
                name: name.to_string(),
                kind: AtomKind::Function,
                source: source[start_byte..end_byte].to_string(),
                start_byte,
                end_byte,
                start_line: source[..start_byte].matches('\n').count() + 1,
                end_line: source[..end_byte].matches('\n').count() + 1,
                defines: vec![name.to_string()],
                references: Vec::new(),
                doc_comment: None,
                visibility: Visibility::Public,
                parent: None,
                decorators: Vec::new(),
            });
        }

        for cap in struct_regex.captures_iter(source) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("unknown");
            let start_byte = cap.get(0).unwrap().start();
            let end_byte = self.find_block_end(source, start_byte);

            atoms.push(ExtractedAtom {
                name: name.to_string(),
                kind: AtomKind::Struct,
                source: source[start_byte..end_byte].to_string(),
                start_byte,
                end_byte,
                start_line: source[..start_byte].matches('\n').count() + 1,
                end_line: source[..end_byte].matches('\n').count() + 1,
                defines: vec![name.to_string()],
                references: Vec::new(),
                doc_comment: None,
                visibility: Visibility::Public,
                parent: None,
                decorators: Vec::new(),
            });
        }

        Ok(atoms)
    }

    /// Extract atoms from C# source
    fn extract_csharp(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        let mut atoms = Vec::new();
        let class_regex = regex::Regex::new(r"(?m)^(\s*)(?:public|private|internal|protected)?\s+class\s+(\w+)").unwrap();
        let method_regex = regex::Regex::new(r"(?m)^(\s*)(?:public|private|internal|protected)?\s+(?:\w+\s+)*(\w+)\s*\([^)]*\)\s*\{").unwrap();

        for cap in class_regex.captures_iter(source) {
            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("unknown");
            let start_byte = cap.get(0).unwrap().start();
            let end_byte = self.find_block_end(source, start_byte);

            atoms.push(ExtractedAtom {
                name: name.to_string(),
                kind: AtomKind::Class,
                source: source[start_byte..end_byte].to_string(),
                start_byte,
                end_byte,
                start_line: source[..start_byte].matches('\n').count() + 1,
                end_line: source[..end_byte].matches('\n').count() + 1,
                defines: vec![name.to_string()],
                references: Vec::new(),
                doc_comment: None,
                visibility: Visibility::Public,
                parent: None,
                decorators: Vec::new(),
            });
        }

        for cap in method_regex.captures_iter(source) {
            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("unknown");
            let start_byte = cap.get(0).unwrap().start();
            let end_byte = self.find_block_end(source, start_byte);

            atoms.push(ExtractedAtom {
                name: name.to_string(),
                kind: AtomKind::Function,
                source: source[start_byte..end_byte].to_string(),
                start_byte,
                end_byte,
                start_line: source[..start_byte].matches('\n').count() + 1,
                end_line: source[..end_byte].matches('\n').count() + 1,
                defines: vec![name.to_string()],
                references: Vec::new(),
                doc_comment: None,
                visibility: Visibility::Public,
                parent: None,
                decorators: Vec::new(),
            });
        }

        Ok(atoms)
    }

    /// Extract atoms from CSS source
    fn extract_css(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        let mut atoms = Vec::new();
        let rule_regex = regex::Regex::new(r"(?m)^([^{]+)\{").unwrap();

        for cap in rule_regex.captures_iter(source) {
            let name = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("rule");
            let start_byte = cap.get(0).unwrap().start();
            let end_byte = self.find_block_end(source, start_byte);

            atoms.push(ExtractedAtom {
                name: name.to_string(),
                kind: AtomKind::Constant, // CSS rules are roughly constants/styles
                source: source[start_byte..end_byte].to_string(),
                start_byte,
                end_byte,
                start_line: source[..start_byte].matches('\n').count() + 1,
                end_line: source[..end_byte].matches('\n').count() + 1,
                defines: Vec::new(),
                references: Vec::new(),
                doc_comment: None,
                visibility: Visibility::Public,
                parent: None,
                decorators: Vec::new(),
            });
        }
        Ok(atoms)
    }

    /// Extract atoms from GLSL source
    fn extract_glsl(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        self.extract_c(source)
    }

    /// Fallback extraction for unsupported languages
    fn extract_fallback(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        // Return the whole file as a single atom
        Ok(vec![ExtractedAtom {
            name: "module".to_string(),
            kind: AtomKind::Module,
            source: source.to_string(),
            start_byte: 0,
            end_byte: source.len(),
            start_line: 1,
            end_line: source.lines().count(),
            defines: Vec::new(),
            references: Vec::new(),
            doc_comment: None,
            visibility: Visibility::Public,
            parent: None,
            decorators: Vec::new(),
        }])
    }

    // ========================================================================
    // Helper methods
    // ========================================================================

    /// Find the end of a brace-delimited block
    fn find_block_end(&self, source: &str, start: usize) -> usize {
        let mut depth = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        let mut prev_char = ' ';

        for (i, c) in source[start..].char_indices() {
            if in_string {
                if c == string_char && prev_char != '\\' {
                    in_string = false;
                }
            } else {
                match c {
                    '"' | '\'' | '`' => {
                        in_string = true;
                        string_char = c;
                    }
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            return start + i + 1;
                        }
                    }
                    _ => {}
                }
            }
            prev_char = c;
        }

        source.len()
    }

    /// Find the end of a statement (semicolon or newline)
    fn find_statement_end(&self, source: &str, start: usize) -> usize {
        for (i, c) in source[start..].char_indices() {
            if c == ';' || c == '\n' {
                return start + i + 1;
            }
        }
        source.len()
    }

    /// Find the end of a Python block (indentation-based)
    fn find_python_block_end(&self, source: &str, start: usize, base_indent: usize) -> usize {
        let lines: Vec<&str> = source[start..].lines().collect();
        let mut end = start;
        let mut started = false;

        for line in lines {
            if line.trim().is_empty() {
                end += line.len() + 1;
                continue;
            }

            let indent = line.len() - line.trim_start().len();
            
            if !started {
                started = true;
                end += line.len() + 1;
            } else if indent > base_indent {
                end += line.len() + 1;
            } else {
                break;
            }
        }

        end.min(source.len())
    }

    /// Extract references from Rust code
    fn extract_references(&self, source: &str) -> Vec<String> {
        let mut refs = Vec::new();
        
        // Extract use statements
        let use_regex = regex::Regex::new(r"use\s+([\w:]+)").ok();
        if let Some(re) = use_regex {
            for cap in re.captures_iter(source) {
                if let Some(m) = cap.get(1) {
                    refs.push(m.as_str().to_string());
                }
            }
        }
        
        refs
    }

    /// Extract imports from TypeScript
    fn extract_ts_imports(&self, source: &str) -> Vec<(String, Vec<String>)> {
        let mut imports = Vec::new();
        
        let import_regex = regex::Regex::new(
            r#"import\s*\{([^}]+)\}\s*from\s*['"]([^'"]+)['"]"#
        ).ok();
        
        if let Some(re) = import_regex {
            for cap in re.captures_iter(source) {
                let symbols: Vec<String> = cap.get(1)
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                
                let path = cap.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
                
                imports.push((path, symbols));
            }
        }
        
        imports
    }

    /// Extract doc comment before a position
    fn extract_doc_comment(&self, source: &str, pos: usize) -> Option<String> {
        let before = &source[..pos];
        let lines: Vec<&str> = before.lines().rev().collect();
        
        let mut doc_lines = Vec::new();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("///") {
                doc_lines.push(trimmed.trim_start_matches("///").trim());
            } else if trimmed.is_empty() {
                continue;
            } else {
                break;
            }
        }
        
        if doc_lines.is_empty() {
            None
        } else {
            doc_lines.reverse();
            Some(doc_lines.join("\n"))
        }
    }

    /// Extract JSDoc comment
    fn extract_jsdoc(&self, source: &str, pos: usize) -> Option<String> {
        let before = &source[..pos];
        
        if let Some(start) = before.rfind("/**") {
            if let Some(end) = before[start..].find("*/") {
                let comment = &before[start..start + end + 2];
                return Some(comment.to_string());
            }
        }
        
        None
    }

    /// Extract Python docstring
    fn extract_python_docstring(&self, source: &str, start: usize) -> Option<String> {
        let after = &source[start..];
        
        // Find the colon ending the function definition
        if let Some(colon_pos) = after.find(':') {
            let rest = &after[colon_pos + 1..];
            let trimmed = rest.trim_start();
            
            // Check for docstring
            if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
                let quote = &trimmed[..3];
                if let Some(end) = trimmed[3..].find(quote) {
                    return Some(trimmed[3..3 + end].to_string());
                }
            }
        }
        
        None
    }
}

impl ExtractedAtom {
    /// Get the number of lines in this atom
    pub fn line_count(&self) -> usize {
        self.end_line - self.start_line + 1
    }

    /// Estimate token count
    pub fn token_estimate(&self) -> usize {
        self.source.len() / 4
    }

    /// Is this a public/exported symbol?
    pub fn is_public(&self) -> bool {
        self.visibility == Visibility::Public
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_extraction() {
        let source = r#"
/// A simple greeting function
pub fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn private_helper() {
    // do something
}
"#;

        let extractor = AtomExtractor::new("rust", AtomizerConfig::default());
        let atoms = extractor.extract(source).unwrap();

        assert!(!atoms.is_empty());
        assert!(atoms.iter().any(|a| a.name == "hello"));
    }

    #[test]
    fn test_typescript_extraction() {
        let source = r#"
export function greet(name: string): string {
    return `Hello, ${name}!`;
}

export class Greeter {
    greet(name: string) {
        return `Hello, ${name}`;
    }
}
"#;

        let extractor = AtomExtractor::new("typescript", AtomizerConfig::default());
        let atoms = extractor.extract(source).unwrap();

        assert!(!atoms.is_empty());
    }

    #[test]
    fn test_python_extraction() {
        let source = r#"
def hello(name):
    """Say hello to someone."""
    print(f"Hello, {name}!")

class Greeter:
    def greet(self, name):
        return f"Hello, {name}"
"#;

        let extractor = AtomExtractor::new("python", AtomizerConfig::default());
        let atoms = extractor.extract(source).unwrap();

        assert!(!atoms.is_empty());
    }
}
