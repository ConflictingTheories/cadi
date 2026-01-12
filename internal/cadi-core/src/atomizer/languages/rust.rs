//! Rust-specific atomizer

use crate::atomizer::{AtomizerConfig, ExtractedAtom};
use crate::error::CadiResult;

/// Rust-specific atomizer with Tree-sitter support
pub struct RustAtomizer {
    config: AtomizerConfig,
}

impl RustAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { config }
    }

    /// Extract atoms using Tree-sitter (when feature enabled)
    #[cfg(feature = "ast-parsing")]
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use tree_sitter::{Parser, Query, QueryCursor};
        
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::language())?;
        
        let tree = parser.parse(source, None)
            .ok_or_else(|| crate::error::CadiError::AtomizerError("Parse failed".into()))?;
        
        let mut atoms = Vec::new();
        
        // Tree-sitter queries for Rust
        let query_src = r#"
            (function_item
                name: (identifier) @fn_name
            ) @function
            
            (struct_item
                name: (type_identifier) @struct_name
            ) @struct
            
            (enum_item
                name: (type_identifier) @enum_name
            ) @enum
            
            (trait_item
                name: (type_identifier) @trait_name
            ) @trait
            
            (impl_item) @impl
        "#;
        
        let query = Query::new(&tree_sitter_rust::language(), query_src)?;
        let mut cursor = QueryCursor::new();
        
        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        
        for m in matches {
            for capture in m.captures {
                let node = capture.node;
                let name_idx = query.capture_index_for_name("fn_name")
                    .or(query.capture_index_for_name("struct_name"))
                    .or(query.capture_index_for_name("enum_name"))
                    .or(query.capture_index_for_name("trait_name"));
                
                if capture.index == name_idx.unwrap_or(u32::MAX) {
                    let name = node.utf8_text(source.as_bytes()).unwrap_or("unknown");
                    // Create atom...
                }
            }
        }
        
        Ok(atoms)
    }
    
    /// Fallback extraction without Tree-sitter
    #[cfg(not(feature = "ast-parsing"))]
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use crate::atomizer::AtomExtractor;
        AtomExtractor::new("rust", self.config.clone()).extract(source)
    }
}

/// Common Rust patterns to detect
pub struct RustPatterns;

impl RustPatterns {
    /// Check if a function is a test
    pub fn is_test(attrs: &[String]) -> bool {
        attrs.iter().any(|a| a.contains("#[test]") || a.contains("#[tokio::test]"))
    }
    
    /// Check if code is behind a cfg attribute
    pub fn is_conditional(attrs: &[String]) -> bool {
        attrs.iter().any(|a| a.contains("#[cfg("))
    }
    
    /// Extract visibility from source
    pub fn parse_visibility(source: &str) -> &'static str {
        if source.contains("pub(crate)") {
            "crate"
        } else if source.contains("pub(super)") {
            "super"
        } else if source.starts_with("pub ") || source.contains(" pub ") {
            "public"
        } else {
            "private"
        }
    }
}
