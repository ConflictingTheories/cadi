//! HTML atomizer

use crate::atomizer::{AtomizerConfig, ExtractedAtom};
use crate::error::CadiResult;

/// HTML atomizer - extracts meaningful fragments and embedded scripts/styles
pub struct HtmlAtomizer {
    config: AtomizerConfig,
}

impl HtmlAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { config }
    }

    /// Extract atoms using Tree-sitter
    #[cfg(feature = "ast-parsing")]
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use tree_sitter::{Parser, Query, QueryCursor};
        use crate::atomizer::AtomKind;

        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_html::language())?;
        
        let tree = parser.parse(source, None)
            .ok_or_else(|| crate::error::CadiError::AtomizerError("Parse failed".into()))?;
        
        let mut atoms = Vec::new();
        
        // Tree-sitter queries for HTML
        // We look for script tags and link tags
        let _query_src = r#"
            (script_element
                (start_tag 
                    (attribute
                        (attribute_name) @attr_name
                        (quoted_attribute_value (attribute_value) @src_val)
                    )
                )
            ) @script

            (link_element
                 (start_tag 
                    (attribute
                        (attribute_name) @attr_name
                        (quoted_attribute_value (attribute_value) @href_val)
                    )
                )
            ) @link
            
            (style_element) @style
            (script_element) @inline_script
        "#;
        // Note: The above query is a simplification. HTML attributes are unordered.
        // A more robust manual traversal is often safer than complex queries for attributes.
        // But let's try a simpler approach capturing the whole element and parsing attributes manually if needed.
        
        let simplified_query = r#"
            (script_element) @script
            (style_element) @style
            (element) @element
        "#;

        let query = Query::new(&tree_sitter_html::language(), simplified_query)?;
        let mut cursor = QueryCursor::new();
        
        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

        for m in matches {
            let mut kind = AtomKind::Module; // Default
            let mut name = "unknown".to_string();
            let mut atom_node = None;
            let mut references = Vec::new();
            
            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize];
                match capture_name {
                    "script" => {
                        atom_node = Some(capture.node);
                        kind = AtomKind::Function; // Closest mapping for script block
                        name = "script".to_string();
                        
                        // Check for src attribute
                        let node_text = capture.node.utf8_text(source.as_bytes()).unwrap_or("");
                        if let Some(src_idx) = node_text.find("src=") {
                             // Crude attribute parsing
                             let after_src = &node_text[src_idx+4..];
                             let quote = after_src.chars().next().unwrap_or('"');
                             if quote == '"' || quote == '\'' {
                                 if let Some(end) = after_src[1..].find(quote) {
                                     let path = after_src[1..1+end].to_string();
                                     references.push(path);
                                     kind = AtomKind::Import; // It's an external script
                                     name = "script_import".to_string();
                                 }
                             }
                        }
                    }
                    "style" => {
                        atom_node = Some(capture.node);
                        kind = AtomKind::Constant;
                        name = "style".to_string();
                    }
                    "element" => {
                         // Check for link
                         let node_text = capture.node.utf8_text(source.as_bytes()).unwrap_or("");
                         if node_text.starts_with("<link") {
                            atom_node = Some(capture.node);
                             // Check for rel="stylesheet"
                            if node_text.contains("stylesheet") {
                                 kind = AtomKind::Import; // CSS Import
                                 name = "style_import".to_string();
                                 
                                 if let Some(href_idx) = node_text.find("href=") {
                                     let after_href = &node_text[href_idx+5..];
                                     let quote = after_href.chars().next().unwrap_or('"');
                                     if quote == '"' || quote == '\'' {
                                         if let Some(end) = after_href[1..].find(quote) {
                                             let path = after_href[1..1+end].to_string();
                                             references.push(path);
                                         }
                                     }
                                }
                            } else {
                                continue; 
                            }
                         } else {
                             continue;
                         }
                    }
                    _ => {}
                }
            }

            if let Some(node) = atom_node {
                let start_byte = node.start_byte();
                let end_byte = node.end_byte();
                let start_point = node.start_position();
                let end_point = node.end_position();

                atoms.push(ExtractedAtom {
                    name,
                    kind,
                    source: source[start_byte..end_byte].to_string(),
                    start_byte,
                    end_byte,
                    start_line: start_point.row + 1,
                    end_line: end_point.row + 1,
                    defines: vec![],
                    references,
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
        AtomExtractor::new("html", self.config.clone()).extract(source)
    }
}
