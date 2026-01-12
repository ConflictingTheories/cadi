//! Rust-specific atomizer

use crate::atomizer::{AtomizerConfig, ExtractedAtom, AtomKind};
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
            // Build a map of capture name -> node for this match
            let mut caps: std::collections::HashMap<String, tree_sitter::Node> = std::collections::HashMap::new();
            for cap in m.captures.iter() {
                let name = query.capture_names()[cap.index as usize].clone();
                caps.insert(name.to_string(), cap.node);
            }

            // Function
            if let (Some(fn_node), Some(name_node)) = (caps.get("function"), caps.get("fn_name")) {
                let name = name_node.utf8_text(source.as_bytes()).unwrap_or("unknown").to_string();
                let start = fn_node.start_byte();
                let end = fn_node.end_byte();
                let start_point = fn_node.start_position();
                let end_point = fn_node.end_position();

                atoms.push(ExtractedAtom {
                    name: name.clone(),
                    kind: AtomKind::Function,
                    source: source[start..end].to_string(),
                    start_byte: start,
                    end_byte: end,
                    start_line: start_point.row + 1,
                    end_line: end_point.row + 1,
                    defines: vec![name],
                    references: Vec::new(),
                    doc_comment: None,
                    visibility: crate::atomizer::extractor::Visibility::Public,
                    parent: None,
                    decorators: Vec::new(),
                });
            }

            // Struct
            if let (Some(struct_node), Some(name_node)) = (caps.get("struct"), caps.get("struct_name")) {
                let name = name_node.utf8_text(source.as_bytes()).unwrap_or("unknown").to_string();
                let start = struct_node.start_byte();
                let end = struct_node.end_byte();
                let start_point = struct_node.start_position();
                let end_point = struct_node.end_position();

                atoms.push(ExtractedAtom {
                    name: name.clone(),
                    kind: AtomKind::Struct,
                    source: source[start..end].to_string(),
                    start_byte: start,
                    end_byte: end,
                    start_line: start_point.row + 1,
                    end_line: end_point.row + 1,
                    defines: vec![name],
                    references: Vec::new(),
                    doc_comment: None,
                    visibility: crate::atomizer::extractor::Visibility::Public,
                    parent: None,
                    decorators: Vec::new(),
                });
            }

            // Enum
            if let (Some(enum_node), Some(name_node)) = (caps.get("enum"), caps.get("enum_name")) {
                let name = name_node.utf8_text(source.as_bytes()).unwrap_or("unknown").to_string();
                let start = enum_node.start_byte();
                let end = enum_node.end_byte();
                let start_point = enum_node.start_position();
                let end_point = enum_node.end_position();

                atoms.push(ExtractedAtom {
                    name: name.clone(),
                    kind: AtomKind::Enum,
                    source: source[start..end].to_string(),
                    start_byte: start,
                    end_byte: end,
                    start_line: start_point.row + 1,
                    end_line: end_point.row + 1,
                    defines: vec![name],
                    references: Vec::new(),
                    doc_comment: None,
                    visibility: crate::atomizer::extractor::Visibility::Public,
                    parent: None,
                    decorators: Vec::new(),
                });
            }

            // Trait
            if let (Some(trait_node), Some(name_node)) = (caps.get("trait"), caps.get("trait_name")) {
                let name = name_node.utf8_text(source.as_bytes()).unwrap_or("unknown").to_string();
                let start = trait_node.start_byte();
                let end = trait_node.end_byte();
                let start_point = trait_node.start_position();
                let end_point = trait_node.end_position();

                atoms.push(ExtractedAtom {
                    name: name.clone(),
                    kind: AtomKind::Trait,
                    source: source[start..end].to_string(),
                    start_byte: start,
                    end_byte: end,
                    start_line: start_point.row + 1,
                    end_line: end_point.row + 1,
                    defines: vec![name],
                    references: Vec::new(),
                    doc_comment: None,
                    visibility: crate::atomizer::extractor::Visibility::Public,
                    parent: None,
                    decorators: Vec::new(),
                });
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
