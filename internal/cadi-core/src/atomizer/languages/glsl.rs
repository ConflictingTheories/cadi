//! GLSL-specific atomizer

use crate::atomizer::{AtomizerConfig, ExtractedAtom, AtomKind};
use crate::error::CadiResult;

/// GLSL-specific atomizer with Tree-sitter support
pub struct GlslAtomizer {
    _config: AtomizerConfig,
}

impl GlslAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { _config: config }
    }

    /// Extract atoms using Tree-sitter (when feature enabled)
    #[cfg(feature = "ast-parsing")]
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use tree_sitter::{Parser, Query, QueryCursor};
        
        let mut parser = Parser::new();
        // Note: usage of tree_sitter_glsl might vary depending on the crate version
        parser.set_language(&tree_sitter_glsl::language())?;
        
        let tree = parser.parse(source, None)
            .ok_or_else(|| crate::error::CadiError::AtomizerError("Parse failed".into()))?;
        
        let mut atoms = Vec::new();
        
        // Tree-sitter queries for GLSL
        let query_src = r#"
            (function_definition
                (function_declarator
                    (identifier) @fn_name
                )
            ) @function
            
            (struct_specifier
                (type_identifier) @struct_name
            ) @struct
            
            (translation_unit
                (declaration
                    (identifier) @global_var
                )
            ) @global
        "#;
        
        let query = Query::new(&tree_sitter_glsl::language(), query_src)?;
        let mut cursor = QueryCursor::new();
        
        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        
        for m in matches {
            let mut name = "unknown".to_string();
            let mut kind = AtomKind::Function;
            let mut atom_node = None;

            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize].as_ref();
                match capture_name {
                    "fn_name" | "struct_name" | "global_var" => {
                        name = capture.node.utf8_text(source.as_bytes()).unwrap_or("unknown").to_string();
                    }
                    "function" => {
                        kind = AtomKind::Function;
                        atom_node = Some(capture.node);
                    }
                    "struct" => {
                        kind = AtomKind::Struct;
                        atom_node = Some(capture.node);
                    }
                    "global" => {
                        kind = AtomKind::Constant;
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

                atoms.push(ExtractedAtom {
                    name,
                    kind,
                    source: source[start_byte..end_byte].to_string(),
                    start_byte,
                    end_byte,
                    start_line: start_point.row + 1,
                    end_line: end_point.row + 1,
                    defines: vec![],
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
        AtomExtractor::new("glsl", self._config.clone()).extract(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atomizer::AtomizerConfig;

    #[test]
    fn test_glsl_extraction() {
        let source = r#"
            struct Light {
                vec3 position;
                vec3 color;
            };

            void main() {
                gl_Position = vec4(0.0);
            }
        "#;

        let atomizer = GlslAtomizer::new(AtomizerConfig::default());
        let atoms = atomizer.extract(source).unwrap();

        // Should find struct Light and function main
        assert!(atoms.iter().any(|a| a.name == "Light"));
        assert!(atoms.iter().any(|a| a.name == "main"));
    }
}
