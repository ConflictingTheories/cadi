//! C#-specific atomizer

use crate::atomizer::{AtomizerConfig, ExtractedAtom, AtomKind};
use crate::error::CadiResult;

/// C#-specific atomizer with Tree-sitter support
pub struct CSharpAtomizer {
    _config: AtomizerConfig,
}

impl CSharpAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { _config: config }
    }

    /// Extract atoms using Tree-sitter (when feature enabled)
    #[cfg(feature = "ast-parsing")]
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use tree_sitter::{Parser, Query, QueryCursor};
        
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_c_sharp::language())?;
        
        let tree = parser.parse(source, None)
            .ok_or_else(|| crate::error::CadiError::AtomizerError("Parse failed".into()))?;
        
        let mut atoms = Vec::new();
        
        // Tree-sitter queries for C#
        let query_src = r#"
            (method_declaration
                name: (identifier) @method_name
            ) @method
            
            (class_declaration
                name: (identifier) @class_name
            ) @class
            
            (interface_declaration
                name: (identifier) @interface_name
            ) @interface
            
            (struct_declaration
                name: (identifier) @struct_name
            ) @struct
            
            (enum_declaration
                name: (identifier) @enum_name
            ) @enum
            
            (namespace_declaration
                name: (_) @namespace_name
            ) @namespace
        "#;
        
        let query = Query::new(&tree_sitter_c_sharp::language(), query_src)?;
        let mut cursor = QueryCursor::new();
        
        // Query for references (identifiers) within a node
        // In C#, we might also want to capture 'generic_name' or 'qualified_name' parts
        let ref_query_src = r#"(identifier) @ref"#;
        let ref_query = Query::new(&tree_sitter_c_sharp::language(), ref_query_src)?;
        
        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        
        for m in matches {
            let mut name = "unknown".to_string();
            let mut kind = AtomKind::Method;
            let mut atom_node = None;

            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize];
                match capture_name {
                    "method_name" | "class_name" | "interface_name" | "struct_name" | "enum_name" | "namespace_name" => {
                        name = capture.node.utf8_text(source.as_bytes()).unwrap_or("unknown").to_string();
                    }
                    "method" => {
                        kind = AtomKind::Method;
                        atom_node = Some(capture.node);
                    }
                    "class" => {
                        kind = AtomKind::Class;
                        atom_node = Some(capture.node);
                    }
                    "interface" => {
                        kind = AtomKind::Interface;
                        atom_node = Some(capture.node);
                    }
                    "struct" => {
                        kind = AtomKind::Struct;
                        atom_node = Some(capture.node);
                    }
                    "enum" => {
                        kind = AtomKind::Enum;
                        atom_node = Some(capture.node);
                    }
                    "namespace" => {
                        kind = AtomKind::Module;
                        atom_node = Some(capture.node);
                    }
                    _ => {}
                }
            }

            if let Some(node) = atom_node {
                let start_byte = node.start_byte();
                let end_byte = node.end_byte();
                let start_point = node.start_position();
                let end_point = node.end_position();

                // Extract references
                let mut references = Vec::new();
                let mut ref_cursor = QueryCursor::new();
                let ref_matches = ref_cursor.matches(&ref_query, node, source.as_bytes());
                
                for rm in ref_matches {
                    for cap in rm.captures {
                        let ref_name = cap.node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                        if !ref_name.is_empty() && ref_name != name {
                            references.push(ref_name);
                        }
                    }
                }
                references.sort();
                references.dedup();

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
        AtomExtractor::new("csharp", self._config.clone()).extract(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atomizer::AtomizerConfig;

    #[test]
    fn test_csharp_extraction() {
        let source = r#"
            namespace MyNamespace {
                public class Greeter {
                    public void Greet(string name) {
                        Console.WriteLine("Hello " + name);
                    }
                }
            }
        "#;

        let atomizer = CSharpAtomizer::new(AtomizerConfig::default());
        let atoms = atomizer.extract(source).unwrap();

        // Should find class Greeter and method Greet
        assert!(atoms.iter().any(|a| a.name == "Greeter"));
        assert!(atoms.iter().any(|a| a.name == "Greet"));
    }
}
