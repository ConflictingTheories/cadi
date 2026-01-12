//! TypeScript/JavaScript-specific atomizer

use crate::atomizer::{AtomizerConfig, ExtractedAtom};
use crate::error::CadiResult;

/// TypeScript/JavaScript atomizer
pub struct TypeScriptAtomizer {
    _config: AtomizerConfig,
}

impl TypeScriptAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { _config: config }
    }

    /// Extract atoms using Tree-sitter
    #[cfg(feature = "ast-parsing")]
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use tree_sitter::{Parser, Query, QueryCursor};
        use crate::atomizer::AtomKind;
        
        let mut parser = Parser::new();
        // Use TSX grammar as it handles both TS and TSX
        parser.set_language(&tree_sitter_typescript::language_tsx())?;
        
        let tree = parser.parse(source, None)
            .ok_or_else(|| crate::error::CadiError::AtomizerError("Parse failed".into()))?;
        
        let mut atoms = Vec::new();
        
        // Tree-sitter queries for TypeScript/TSX
        let query_src = r#"
            (function_declaration
                name: (identifier) @fn_name
            ) @function
            
            (class_declaration
                name: (type_identifier) @class_name
            ) @class
            
            (interface_declaration
                name: (type_identifier) @interface_name
            ) @interface
            
            (enum_declaration
                name: (identifier) @enum_name
            ) @enum
            
            (variable_declarator
                name: (identifier) @var_name
                value: [(arrow_function) (function_expression)] @arrow_fn
            ) @variable_fn
        "#;
        
        let query = Query::new(&tree_sitter_typescript::language_tsx(), query_src)?;
        let mut cursor = QueryCursor::new();
        
        // Reference query
        let ref_query_src = r#"
            (identifier) @ref
            (type_identifier) @ref
        "#;
        let ref_query = Query::new(&tree_sitter_typescript::language_tsx(), ref_query_src)?;

        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

        for m in matches {
            let mut caps: std::collections::HashMap<String, tree_sitter::Node> = std::collections::HashMap::new();
            for cap in m.captures.iter() {
                let name = query.capture_names()[cap.index as usize];
                caps.insert(name.to_string(), cap.node);
            }

            // Function Declaration
            if let (Some(fn_node), Some(name_node)) = (caps.get("function"), caps.get("fn_name")) {
                let name = name_node.utf8_text(source.as_bytes()).unwrap_or("unknown").to_string();
                let start = fn_node.start_byte();
                let end = fn_node.end_byte();
                let start_point = fn_node.start_position();
                let end_point = fn_node.end_position();
                
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
                    visibility: crate::atomizer::extractor::Visibility::Public,
                    parent: None,
                    decorators: Vec::new(),
                });
            }

            // Arrow Function / Variable Function
            if let (Some(var_node), Some(name_node), Some(fn_val)) = (caps.get("variable_fn"), caps.get("var_name"), caps.get("arrow_fn")) {
                 let name = name_node.utf8_text(source.as_bytes()).unwrap_or("unknown").to_string();
                 // We want the whole declarations
                 let start = var_node.start_byte();
                 let end = var_node.end_byte();
                 let start_point = var_node.start_position();
                 let end_point = var_node.end_position();
                 
                 let params = Self::extract_references(source, *fn_val, &ref_query, &name);

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
                    visibility: crate::atomizer::extractor::Visibility::Public,
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

                let params = Self::extract_references(source, *class_node, &ref_query, &name);

                atoms.push(ExtractedAtom {
                    name: name.clone(),
                    kind: AtomKind::Class,
                    source: source[start..end].to_string(),
                    start_byte: start,
                    end_byte: end,
                    start_line: start_point.row + 1,
                    end_line: end_point.row + 1,
                    defines: vec![name],
                    references: params,
                    doc_comment: None,
                    visibility: crate::atomizer::extractor::Visibility::Public,
                    parent: None,
                    decorators: Vec::new(),
                });
            }

            // Interface
            if let (Some(interface_node), Some(name_node)) = (caps.get("interface"), caps.get("interface_name")) {
                let name = name_node.utf8_text(source.as_bytes()).unwrap_or("unknown").to_string();
                let start = interface_node.start_byte();
                let end = interface_node.end_byte();
                let start_point = interface_node.start_position();
                let end_point = interface_node.end_position();

                let params = Self::extract_references(source, *interface_node, &ref_query, &name);

                atoms.push(ExtractedAtom {
                    name: name.clone(),
                    kind: AtomKind::Interface,
                    source: source[start..end].to_string(),
                    start_byte: start,
                    end_byte: end,
                    start_line: start_point.row + 1,
                    end_line: end_point.row + 1,
                    defines: vec![name],
                    references: params,
                    doc_comment: None,
                    visibility: crate::atomizer::extractor::Visibility::Public,
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
            for cap in m.captures {
                let name = cap.node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                if !name.is_empty() 
                && name != exclude_name 
                && name != "this" 
                && name != "undefined" 
                && name != "null"
                && name != "true"
                && name != "false" {
                    references.push(name);
                }
            }
        }
        references.sort();
        references.dedup();
        references
    }

    /// Fallback extraction without Tree-sitter
    #[cfg(not(feature = "ast-parsing"))]
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use crate::atomizer::AtomExtractor;
        AtomExtractor::new("typescript", self._config.clone()).extract(source)
    }
}

