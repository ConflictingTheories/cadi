//! AST Parser using Tree-sitter
//!
//! Provides language-aware parsing for supported languages.

#[cfg(feature = "ast-parsing")]
use std::collections::HashMap;

#[cfg(feature = "ast-parsing")]
use tree_sitter::{Parser, Tree};

#[cfg(feature = "ast-parsing")]
use crate::error::{CadiError, CadiResult};

#[cfg(not(feature = "ast-parsing"))]
use crate::error::CadiResult;

/// Multi-language AST parser
pub struct AstParser {
    #[cfg(feature = "ast-parsing")]
    parsers: HashMap<String, Parser>,
    
    #[cfg(not(feature = "ast-parsing"))]
    _phantom: std::marker::PhantomData<()>,
}

impl AstParser {
    /// Create a new AST parser with all supported languages
    #[cfg(feature = "ast-parsing")]
    pub fn new() -> CadiResult<Self> {
        let mut parsers = HashMap::new();
        
        // Initialize Rust parser
        {
            let mut parser = Parser::new();
            parser.set_language(&tree_sitter_rust::language())
                .map_err(|e| CadiError::AtomizerError(format!("Failed to load Rust grammar: {}", e)))?;
            parsers.insert("rust".to_string(), parser);
        }
        
        // Initialize TypeScript parser
        {
            let mut parser = Parser::new();
            parser.set_language(&tree_sitter_typescript::language_typescript())
                .map_err(|e| CadiError::AtomizerError(format!("Failed to load TypeScript grammar: {}", e)))?;
            parsers.insert("typescript".to_string(), parser);
        }
        
        // Initialize JavaScript parser
        {
            let mut parser = Parser::new();
            parser.set_language(&tree_sitter_javascript::language())
                .map_err(|e| CadiError::AtomizerError(format!("Failed to load JavaScript grammar: {}", e)))?;
            parsers.insert("javascript".to_string(), parser);
        }
        
        // Initialize Python parser
        {
            let mut parser = Parser::new();
            parser.set_language(&tree_sitter_python::language())
                .map_err(|e| CadiError::AtomizerError(format!("Failed to load Python grammar: {}", e)))?;
            parsers.insert("python".to_string(), parser);
        }
        
        Ok(Self { parsers })
    }
    
    /// Create a new AST parser (stub when feature not enabled)
    #[cfg(not(feature = "ast-parsing"))]
    pub fn new() -> CadiResult<Self> {
        Ok(Self { _phantom: std::marker::PhantomData })
    }
    
    /// Parse source code into an AST
    #[cfg(feature = "ast-parsing")]
    pub fn parse(&mut self, language: &str, source: &str) -> CadiResult<ParsedAst> {
        let parser = self.parsers.get_mut(language)
            .ok_or_else(|| CadiError::AtomizerError(format!("Unsupported language: {}", language)))?;
        
        let tree = parser.parse(source, None)
            .ok_or_else(|| CadiError::AtomizerError("Failed to parse source".to_string()))?;
        
        Ok(ParsedAst {
            tree,
            source: source.to_string(),
            language: language.to_string(),
        })
    }
    
    /// Parse source code (stub when feature not enabled)
    #[cfg(not(feature = "ast-parsing"))]
    pub fn parse(&mut self, language: &str, source: &str) -> CadiResult<ParsedAst> {
        Ok(ParsedAst {
            source: source.to_string(),
            language: language.to_string(),
        })
    }
    
    /// Check if a language is supported
    #[cfg(feature = "ast-parsing")]
    pub fn supports_language(&self, language: &str) -> bool {
        self.parsers.contains_key(language)
    }
    
    #[cfg(not(feature = "ast-parsing"))]
    pub fn supports_language(&self, _language: &str) -> bool {
        false
    }
    
    /// Get list of supported languages
    #[cfg(feature = "ast-parsing")]
    pub fn supported_languages(&self) -> Vec<&str> {
        self.parsers.keys().map(|s| s.as_str()).collect()
    }
    
    #[cfg(not(feature = "ast-parsing"))]
    pub fn supported_languages(&self) -> Vec<&str> {
        vec![]
    }
    
    /// Detect language from file extension
    pub fn detect_language(path: &std::path::Path) -> Option<String> {
        let ext = path.extension()?.to_str()?;
        match ext {
            "rs" => Some("rust".to_string()),
            "ts" | "tsx" => Some("typescript".to_string()),
            "js" | "jsx" | "mjs" | "cjs" => Some("javascript".to_string()),
            "py" | "pyi" => Some("python".to_string()),
            _ => None,
        }
    }
}

impl Default for AstParser {
    fn default() -> Self {
        Self::new().expect("Failed to create parser")
    }
}

/// A parsed AST with metadata
pub struct ParsedAst {
    #[cfg(feature = "ast-parsing")]
    pub tree: Tree,
    
    pub source: String,
    pub language: String,
}

impl ParsedAst {
    /// Get the root node
    #[cfg(feature = "ast-parsing")]
    pub fn root_node(&self) -> tree_sitter::Node<'_> {
        self.tree.root_node()
    }
    
    /// Walk the tree
    #[cfg(feature = "ast-parsing")]
    pub fn walk(&self) -> tree_sitter::TreeCursor<'_> {
        self.tree.walk()
    }
    
    /// Get source text for a byte range
    pub fn text_for_range(&self, start: usize, end: usize) -> &str {
        &self.source[start..end]
    }
    
    /// Get total lines
    pub fn line_count(&self) -> usize {
        self.source.lines().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_language_detection() {
        assert_eq!(AstParser::detect_language(std::path::Path::new("foo.rs")), Some("rust".to_string()));
        assert_eq!(AstParser::detect_language(std::path::Path::new("bar.ts")), Some("typescript".to_string()));
        assert_eq!(AstParser::detect_language(std::path::Path::new("baz.py")), Some("python".to_string()));
        assert_eq!(AstParser::detect_language(std::path::Path::new("unknown.xyz")), None);
    }
    
    #[cfg(feature = "ast-parsing")]
    #[test]
    fn test_rust_parsing() {
        let mut parser = AstParser::new().unwrap();
        
        let source = r#"
fn hello() {
    println!("Hello, world!");
}
"#;
        
        let ast = parser.parse("rust", source).unwrap();
        assert_eq!(ast.language, "rust");
        
        let root = ast.root_node();
        assert_eq!(root.kind(), "source_file");
    }
}
