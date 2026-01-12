//! Python-specific atomizer

use crate::atomizer::{AtomizerConfig, ExtractedAtom};
use crate::error::CadiResult;

/// Python atomizer
pub struct PythonAtomizer {
    _config: AtomizerConfig,
}

impl PythonAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { _config: config }
    }

    /// Extract atoms using Tree-sitter
    #[cfg(feature = "ast-parsing")]
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use tree_sitter::{Parser, Query, QueryCursor};
        use crate::atomizer::AtomKind;
        
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::language())?;
        
        let tree = parser.parse(source, None)
            .ok_or_else(|| crate::error::CadiError::AtomizerError("Parse failed".into()))?;
        
        let mut atoms = Vec::new();
        
        // Tree-sitter queries for Python
        let query_src = r#"
            (function_definition
                name: (identifier) @fn_name
            ) @function
            
            (class_definition
                name: (identifier) @class_name
            ) @class
            
            (decorated_definition
                (function_definition
                    name: (identifier) @fn_name
                ) @function
            )
            
            (decorated_definition
                (class_definition
                    name: (identifier) @class_name
                ) @class
            )
        "#;
        
        let query = Query::new(&tree_sitter_python::language(), query_src)?;
        let mut cursor = QueryCursor::new();
        
        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

        // Reference query
        let ref_query_src = r#"
            (identifier) @ref
            (attribute attribute: (identifier) @ref)
            (call function: (identifier) @ref)
        "#;
        let ref_query = Query::new(&tree_sitter_python::language(), ref_query_src)?;

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

                // Extract references
                let mut references = Vec::new();
                let mut ref_cursor = QueryCursor::new();
                let ref_matches = ref_cursor.matches(&ref_query, *fn_node, source.as_bytes());
                
                for rm in ref_matches {
                    for cap in rm.captures {
                        let ref_name = cap.node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                        if !ref_name.is_empty() && ref_name != name && ref_name != "self" && ref_name != "cls" {
                            references.push(ref_name);
                        }
                    }
                }
                references.sort();
                references.dedup();

                atoms.push(ExtractedAtom {
                    name: name.clone(),
                    kind: AtomKind::Function,
                    source: source[start..end].to_string(),
                    start_byte: start,
                    end_byte: end,
                    start_line: start_point.row + 1,
                    end_line: end_point.row + 1,
                    defines: vec![name],
                    references,
                    doc_comment: None,
                    visibility: crate::atomizer::extractor::Visibility::Public, // Python is public by default unless _
                    parent: None,
                    decorators: Vec::new(),
                });
            }

            // Class
            if let (Some(class_node), Some(name_node)) = (caps.get("class"), caps.get("class_name")) {
                let name = name_node.utf8_text(source.as_bytes()).unwrap_or("unknown").to_string();
                let start = class_node.start_byte();
                let end = class_node.end_byte();
                let start_point = class_node.start_position();
                let end_point = class_node.end_position();

                // Extract references
                let mut references = Vec::new();
                let mut ref_cursor = QueryCursor::new();
                let ref_matches = ref_cursor.matches(&ref_query, *class_node, source.as_bytes());
                
                for rm in ref_matches {
                    for cap in rm.captures {
                         let ref_name = cap.node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                        if !ref_name.is_empty() && ref_name != name && ref_name != "self" {
                             references.push(ref_name);
                        }
                    }
                }
                references.sort();
                references.dedup();

                atoms.push(ExtractedAtom {
                    name: name.clone(),
                    kind: AtomKind::Class,
                    source: source[start..end].to_string(),
                    start_byte: start,
                    end_byte: end,
                    start_line: start_point.row + 1,
                    end_line: end_point.row + 1,
                    defines: vec![name],
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
        AtomExtractor::new("python", self._config.clone()).extract(source)
    }
}

