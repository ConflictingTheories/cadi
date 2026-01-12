//! CSS-specific atomizer

use crate::atomizer::{AtomizerConfig, ExtractedAtom, AtomKind};
use crate::error::CadiResult;

/// CSS-specific atomizer with Tree-sitter support
pub struct CssAtomizer {
    _config: AtomizerConfig,
}

impl CssAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { _config: config }
    }

    /// Extract atoms using Tree-sitter (when feature enabled)
    #[cfg(feature = "ast-parsing")]
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use tree_sitter::{Parser, Query, QueryCursor};
        
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_css::language())?;
        
        let tree = parser.parse(source, None)
            .ok_or_else(|| crate::error::CadiError::AtomizerError("Parse failed".into()))?;
        
        let mut atoms = Vec::new();
        
        // Tree-sitter queries for CSS
        let query_src = r#"
            (rule_set
                (selectors) @selector
            ) @rule
            
            (media_statement) @media
            
            (keyframe_block_list) @keyframes
        "#;
        
        let query = Query::new(&tree_sitter_css::language(), query_src)?;
        let mut cursor = QueryCursor::new();
        
        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        
        for m in matches {
            let mut name = "rule".to_string();
            let mut kind = AtomKind::Constant;
            let mut atom_node = None;

            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize].as_ref();
                match capture_name {
                    "selector" => {
                        name = capture.node.utf8_text(source.as_bytes()).unwrap_or("rule").trim().to_string();
                    }
                    "rule" => {
                        kind = AtomKind::Constant;
                        atom_node = Some(capture.node);
                    }
                    "media" => {
                        name = capture.node.child(0).map(|n| n.utf8_text(source.as_bytes()).unwrap_or("@media").to_string()).unwrap_or("@media".to_string());
                        kind = AtomKind::Constant;
                        atom_node = Some(capture.node);
                    }
                    "keyframes" => {
                        name = "@keyframes".to_string();
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
        AtomExtractor::new("css", self._config.clone()).extract(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atomizer::AtomizerConfig;

    #[test]
    fn test_css_extraction() {
        let source = r#"
            .header {
                color: red;
            }

            @media (max-width: 600px) {
                body {
                    background: blue;
                }
            }
        "#;

        let atomizer = CssAtomizer::new(AtomizerConfig::default());
        let atoms = atomizer.extract(source).unwrap();

        // Should find .header and @media
        assert!(atoms.iter().any(|a| a.name == ".header"));
        assert!(atoms.iter().any(|a| a.name.contains("@media")));
    }
}
