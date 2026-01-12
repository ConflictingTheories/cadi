//! JSX atomizer (React / web)

use crate::atomizer::{AtomizerConfig, ExtractedAtom, AtomKind};
use crate::error::CadiResult;

/// JSX atomizer (uses JS extractor semantics)
pub struct JSXAtomizer {
    _config: AtomizerConfig,
}

impl JSXAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { _config: config }
    }

    /// Extract atoms from JSX/JS files using Tree-sitter when available
    #[cfg(feature = "ast-parsing")]
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use tree_sitter::{Parser, Query, QueryCursor};

        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_javascript::language())?;
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| crate::error::CadiError::AtomizerError("Parse failed".into()))?;

        let query_src = r#"
            (function_declaration
                name: (identifier) @fn_name
            ) @function

            (lexical_declaration
                (variable_declarator
                    name: (identifier) @var_name
                    value: (arrow_function) @arrow_fn
                )
            ) @var_fn

            (export_statement (function_declaration name: (identifier) @exported_fn_name)) @exported_function

            (class_declaration
                name: (identifier) @class_name
            ) @class

            (jsx_element) @jsx
        "#;

        let query = Query::new(&tree_sitter_javascript::language(), query_src)?;
        let mut cursor = QueryCursor::new();

        let mut atoms = Vec::new();

        for m in cursor.matches(&query, tree.root_node(), source.as_bytes()) {
            let mut name = "unknown".to_string();
            let mut kind = AtomKind::Function;
            let mut atom_node = None;

            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize].as_ref();
                match capture_name {
                    "fn_name" | "var_name" | "exported_fn_name" | "class_name" => {
                        name = capture.node.utf8_text(source.as_bytes()).unwrap_or("unknown").to_string();
                    }
                    "function" | "var_fn" | "exported_function" => {
                        kind = AtomKind::Function;
                        atom_node = Some(capture.node);
                    }
                    "class" => {
                        kind = AtomKind::Class;
                        atom_node = Some(capture.node);
                    }
                    "jsx" => {
                        // If we see a JSX element outside of other captures, record a small fragment
                        let node = capture.node;
                        let start = node.start_byte();
                        let end = node.end_byte();

                        atoms.push(ExtractedAtom {
                            name: "jsx_fragment".to_string(),
                            kind: AtomKind::Module,
                            source: source[start..end].to_string(),
                            start_byte: start,
                            end_byte: end,
                            start_line: node.start_position().row + 1,
                            end_line: node.end_position().row + 1,
                            defines: Vec::new(),
                            references: Vec::new(),
                            doc_comment: None,
                            visibility: crate::atomizer::extractor::Visibility::Public,
                            parent: None,
                            decorators: Vec::new(),
                        });
                    }
                    _ => {}
                }
            }

            if let Some(node) = atom_node {
                let start = node.start_byte();
                let end = node.end_byte();
                let start_point = node.start_position();
                let end_point = node.end_position();

                atoms.push(ExtractedAtom {
                    name: name.clone(),
                    kind: kind.clone(),
                    source: source[start..end].to_string(),
                    start_byte: start,
                    end_byte: end,
                    start_line: start_point.row + 1,
                    end_line: end_point.row + 1,
                    defines: vec![name.clone()],
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
        // Use javascript extractor for JSX
        AtomExtractor::new("javascript", self.config.clone()).extract(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atomizer::AtomizerConfig;

    #[test]
    fn test_jsx_extraction_component() {
        let source = r#"
            export function Hello() { return <div>Hello</div> }
            const X = () => <span>Hi</span>;
            class C extends React.Component { render() { return <p/> } }
        "#;

        let atomizer = JSXAtomizer::new(AtomizerConfig::default());
        let atoms = atomizer.extract(source).unwrap();

        assert!(atoms.iter().any(|a| a.name == "Hello"));
        assert!(atoms.iter().any(|a| a.name == "X"));
        assert!(atoms.iter().any(|a| a.name == "C"));
    }
}
