//! TSX atomizer (TypeScript + JSX)

#[cfg(feature = "ast-parsing")]
use crate::atomizer::{AtomizerConfig, ExtractedAtom, AtomKind};

#[cfg(not(feature = "ast-parsing"))]
use crate::atomizer::{AtomizerConfig, ExtractedAtom};

use crate::error::CadiResult;

/// TSX atomizer (uses TypeScript extractor semantics)
pub struct TSXAtomizer {
    _config: AtomizerConfig,
}

impl TSXAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { _config: config }
    }

    /// Extract atoms from TSX files using Tree-sitter when available
    #[cfg(feature = "ast-parsing")]
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use tree_sitter::{Parser, Query, QueryCursor};

        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_typescript::language_tsx())?;
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| crate::error::CadiError::AtomizerError("Parse failed".into()))?;

        let query_src = r#"
            (function_declaration) @function

            (variable_declarator
                value: (arrow_function) @arrow_fn
            ) @var_fn

            (export_statement (function_declaration) ) @exported_function

            (class_declaration) @class
        "#;

        let query = Query::new(&tree_sitter_typescript::language_tsx(), query_src)?;
        let mut cursor = QueryCursor::new();

        let mut atoms = Vec::new();

        for m in cursor.matches(&query, tree.root_node(), source.as_bytes()) {
            let mut name = "unknown".to_string();
            let mut kind = AtomKind::Function;
            let mut atom_node: Option<tree_sitter::Node> = None;

            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize];
                match capture_name {
                    "function" => {
                        atom_node = Some(capture.node);
                    }
                    "var_fn" => {
                        // The capture for var_fn may be the full lexical_declaration; prefer the inner variable_declarator
                        let mut chosen = capture.node;
                        for child in capture.node.children(&mut capture.node.walk()) {
                            if child.kind() == "variable_declarator" {
                                chosen = child;
                                break;
                            }
                        }
                        atom_node = Some(chosen);
                    }
                    "exported_function" => {
                        atom_node = Some(capture.node);
                    }
                    "class" => {
                        atom_node = Some(capture.node);
                        kind = AtomKind::Class;
                    }
                    _ => {}
                }
            }

            if let Some(node) = atom_node {
                // derive name by scanning descendants for an identifier
                // BFS search descendants for identifier/type_identifier
                let mut queue = std::collections::VecDeque::new();
                queue.push_back(node);
                while let Some(curr) = queue.pop_front() {
                    for child in curr.children(&mut curr.walk()) {
                        if child.kind() == "identifier" || child.kind() == "type_identifier" {
                            name = child.utf8_text(source.as_bytes()).unwrap_or("unknown").to_string();
                            queue.clear();
                            break;
                        }
                        queue.push_back(child);
                    }
                }

                let start = node.start_byte();
                let end = node.end_byte();
                let start_point = node.start_position();
                let end_point = node.end_position();

                atoms.push(ExtractedAtom {
                    name: name.clone(),
                    kind,
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

        // If the tree-sitter based extraction produced nothing, fall back to regex extractor
        if atoms.is_empty() {
            use crate::atomizer::AtomExtractor;
                return AtomExtractor::new("typescript", self._config.clone()).extract(source);
        }

        Ok(atoms)
    }

    /// Fallback extraction without Tree-sitter
    #[cfg(not(feature = "ast-parsing"))]
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use crate::atomizer::AtomExtractor;
        // Use typescript extractor for TSX
           AtomExtractor::new("typescript", self._config.clone()).extract(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atomizer::AtomizerConfig;

    #[test]
    fn test_tsx_extraction_component() {
        let source = r#"
            export function Hello(): JSX.Element { return <div>Hello</div> }
            const X: React.FC = () => <span>Hi</span>;
            class C { render() { return <p/> } }
        "#;

        let atomizer = TSXAtomizer::new(AtomizerConfig::default());
        let atoms = atomizer.extract(source).unwrap();

        eprintln!("TSX atoms: {}", atoms.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", "));

        assert!(atoms.iter().any(|a| a.name == "Hello"));
        assert!(atoms.iter().any(|a| a.name == "X"));
        assert!(atoms.iter().any(|a| a.name == "C"));
    }
}
