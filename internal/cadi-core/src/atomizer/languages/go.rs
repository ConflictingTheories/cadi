//! Go-specific atomizer

#[cfg(feature = "ast-parsing")]
use crate::atomizer::{AtomizerConfig, ExtractedAtom, AtomKind};

#[cfg(not(feature = "ast-parsing"))]
use crate::atomizer::{AtomizerConfig, ExtractedAtom};

use crate::error::CadiResult;

/// Go atomizer
pub struct GoAtomizer {
    _config: AtomizerConfig,
}

impl GoAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { _config: config }
    }

    /// Extract atoms using Tree-sitter
    #[cfg(feature = "ast-parsing")]
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use tree_sitter::{Parser, Query, QueryCursor};
        
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_go::language())?;
        
        let tree = parser.parse(source, None)
            .ok_or_else(|| crate::error::CadiError::AtomizerError("Parse failed".into()))?;
        
        let mut atoms = Vec::new();
        
        // Tree-sitter queries for Go
        let query_src = r#"
            (function_declaration
                name: (identifier) @fn_name
            ) @function
            
            (method_declaration
                name: (field_identifier) @method_name
            ) @method
            
            (type_declaration
                (type_spec
                    name: (type_identifier) @type_name
                )
            ) @type
            
            (const_declaration
                (const_spec
                    name: (identifier) @const_name
                )
            ) @const
        "#;
        
        let query = Query::new(&tree_sitter_go::language(), query_src)?;
        let mut cursor = QueryCursor::new();
        
        // Reference query: find identifiers inside the text
        // In Go, package references are often `pkg.Func`
        let ref_query_src = r#"
            (identifier) @ref
            (type_identifier) @ref
            (field_identifier) @ref
            (package_identifier) @ref
            (selector_expression
                operand: (identifier) @pkg
                field: (field_identifier) @symbol
            ) @selection
        "#;
        let ref_query = Query::new(&tree_sitter_go::language(), ref_query_src)?;

        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

        for m in matches {
            let mut caps: std::collections::HashMap<String, tree_sitter::Node> = std::collections::HashMap::new();
            for cap in m.captures.iter() {
                let name = query.capture_names()[cap.index as usize];
                caps.insert(name.to_string(), cap.node);
            }

            // Function
            if let (Some(fn_node), Some(name_node)) = (caps.get("function"), caps.get("fn_name")) {
                let name = name_node.utf8_text(source.as_bytes()).unwrap_or("unknown").to_string();
                let start = fn_node.start_byte();
                let end = fn_node.end_byte();
                let start_point = fn_node.start_position();
                let end_point = fn_node.end_position();
                
                let visibility = if name.chars().next().unwrap_or('a').is_uppercase() {
                    crate::atomizer::extractor::Visibility::Public
                } else {
                    crate::atomizer::extractor::Visibility::Private
                };

                let params = Self::extract_references(source, *fn_node, &ref_query, &name);

                atoms.push(ExtractedAtom {
                    name: name.clone(),
                    kind: AtomKind::Function,
                    source: source[start..end].to_string(),
                    start_byte: start,
                    end_byte: end,
                    start_line: start_point.row + 1,
                    end_line: end_point.row + 1,
                    defines: vec![name],
                    references: params,
                    doc_comment: None,
                    visibility,
                    parent: None,
                    decorators: Vec::new(),
                });
            }

            // Method
            if let (Some(method_node), Some(name_node)) = (caps.get("method"), caps.get("method_name")) {
                 let name = name_node.utf8_text(source.as_bytes()).unwrap_or("unknown").to_string();
                 let start = method_node.start_byte();
                 let end = method_node.end_byte();
                 let start_point = method_node.start_position();
                 let end_point = method_node.end_position();
                 
                  let visibility = if name.chars().next().unwrap_or('a').is_uppercase() {
                    crate::atomizer::extractor::Visibility::Public
                } else {
                    crate::atomizer::extractor::Visibility::Private
                };
                 
                 let params = Self::extract_references(source, *method_node, &ref_query, &name);

                 atoms.push(ExtractedAtom {
                    name: name.clone(),
                    kind: AtomKind::Method,
                    source: source[start..end].to_string(),
                    start_byte: start,
                    end_byte: end,
                    start_line: start_point.row + 1,
                    end_line: end_point.row + 1,
                    defines: vec![name],
                    references: params,
                    doc_comment: None,
                    visibility,
                    parent: None,
                    decorators: Vec::new(),
                });
            }

            // Type (Struct/Interface)
            if let (Some(type_node), Some(name_node)) = (caps.get("type"), caps.get("type_name")) {
                let name = name_node.utf8_text(source.as_bytes()).unwrap_or("unknown").to_string();
                let start = type_node.start_byte();
                let end = type_node.end_byte();
                let start_point = type_node.start_position();
                let end_point = type_node.end_position();
                
                 let visibility = if name.chars().next().unwrap_or('a').is_uppercase() {
                    crate::atomizer::extractor::Visibility::Public
                } else {
                    crate::atomizer::extractor::Visibility::Private
                };

                let params = Self::extract_references(source, *type_node, &ref_query, &name);

                atoms.push(ExtractedAtom {
                    name: name.clone(),
                    kind: AtomKind::Struct, // Go calls them structs or interfaces, just map to Struct for now
                    source: source[start..end].to_string(),
                    start_byte: start,
                    end_byte: end,
                    start_line: start_point.row + 1,
                    end_line: end_point.row + 1,
                    defines: vec![name],
                    references: params,
                    doc_comment: None,
                    visibility,
                    parent: None,
                    decorators: Vec::new(),
                });
            }
        }
        
        Ok(atoms)
    }

    #[cfg(feature = "ast-parsing")]
    fn extract_references(source: &str, node: tree_sitter::Node, query: &tree_sitter::Query, exclude_name: &str) -> Vec<String> {
        use tree_sitter::QueryCursor;
        let mut references = Vec::new();
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(query, node, source.as_bytes());
        
        for m in matches {
             // Check if it's a selector expression (pkg.Func)
             // We want to capture 'pkg' or 'pkg.Func' as dependency
             let mut is_selector = false;
             for cap in m.captures {
                 let cap_name = query.capture_names()[cap.index as usize];
                 if cap_name == "selection" {
                     is_selector = true;
                     let text = cap.node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                      references.push(text);
                 }
             }

             if !is_selector {
                for cap in m.captures {
                     let cap_name = query.capture_names()[cap.index as usize];
                     if cap_name == "ref" {
                        let text = cap.node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                        if !text.is_empty() 
                            && text != exclude_name 
                            && text != "nil" 
                            && text != "true" 
                            && text != "false" 
                            && text != "string" 
                            && text != "int" 
                            && text != "error" {
                            references.push(text);
                        }
                     }
                }
             }
        }
        references.sort();
        references.dedup();
        references
    }
    
    /// Fallback extraction without Tree-sitter
    #[cfg(not(feature = "ast-parsing"))]
    pub fn extract(&self, _source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        // Fallback for Go is not implemented in base AtomExtractor usually, 
        // so we return empty or implement simple regex here if needed.
        // For now, assume tree-sitter is the way.
        Ok(Vec::new())
    }
}
