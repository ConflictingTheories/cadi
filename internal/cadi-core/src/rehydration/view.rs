//! Virtual View representation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A virtual view - atoms assembled into a coherent context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualView {
    /// The assembled source code
    pub source: String,

    /// Atoms included in this view (in order)
    pub atoms: Vec<String>,

    /// Which atoms were added automatically (Ghost Imports)
    pub ghost_atoms: Vec<String>,

    /// Total token estimate
    pub token_estimate: usize,

    /// Primary language of the view
    pub language: String,

    /// Map of symbol name -> line number in the view
    pub symbol_locations: HashMap<String, usize>,

    /// Fragments that make up this view
    pub fragments: Vec<ViewFragment>,

    /// Was the view truncated due to token limits?
    pub truncated: bool,

    /// Explanation of how the view was constructed
    pub explanation: String,
}

impl VirtualView {
    /// Create a new empty view
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            source: String::new(),
            atoms: Vec::new(),
            ghost_atoms: Vec::new(),
            token_estimate: 0,
            language: language.into(),
            symbol_locations: HashMap::new(),
            fragments: Vec::new(),
            truncated: false,
            explanation: String::new(),
        }
    }

    /// Get the number of lines in the view
    pub fn line_count(&self) -> usize {
        self.source.lines().count()
    }

    /// Get the symbols defined in this view
    pub fn defined_symbols(&self) -> Vec<&String> {
        self.symbol_locations.keys().collect()
    }

    /// Find the line number where a symbol is defined
    pub fn find_symbol(&self, name: &str) -> Option<usize> {
        self.symbol_locations.get(name).copied()
    }

    /// Get a snippet around a symbol
    pub fn snippet_for_symbol(&self, name: &str, context_lines: usize) -> Option<String> {
        let line = self.find_symbol(name)?;
        let lines: Vec<&str> = self.source.lines().collect();
        
        let start = line.saturating_sub(context_lines + 1);
        let end = (line + context_lines).min(lines.len());
        
        Some(lines[start..end].join("\n"))
    }

    /// Check if this view contains a specific atom
    pub fn contains_atom(&self, chunk_id: &str) -> bool {
        self.atoms.contains(&chunk_id.to_string())
    }

    /// Was this atom added automatically (ghost import)?
    pub fn is_ghost(&self, chunk_id: &str) -> bool {
        self.ghost_atoms.contains(&chunk_id.to_string())
    }
}

/// A fragment in a virtual view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewFragment {
    /// The chunk ID this fragment came from
    pub chunk_id: String,

    /// Alias if available
    pub alias: Option<String>,

    /// Start line in the assembled view
    pub start_line: usize,

    /// End line in the assembled view
    pub end_line: usize,

    /// Token count for this fragment
    pub token_count: usize,

    /// Why this fragment was included
    pub inclusion_reason: InclusionReason,

    /// Symbols defined in this fragment
    pub defines: Vec<String>,
}

/// Why a fragment was included in a view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InclusionReason {
    /// Explicitly requested
    Requested,
    /// Imported by a requested atom
    Imported,
    /// Type referenced by a requested atom
    TypeDependency,
    /// Required for context (Ghost Import)
    GhostImport,
}

impl ViewFragment {
    /// Create a new fragment
    pub fn new(chunk_id: impl Into<String>, reason: InclusionReason) -> Self {
        Self {
            chunk_id: chunk_id.into(),
            alias: None,
            start_line: 0,
            end_line: 0,
            token_count: 0,
            inclusion_reason: reason,
            defines: Vec::new(),
        }
    }

    /// With alias
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }

    /// With line range
    pub fn with_lines(mut self, start: usize, end: usize) -> Self {
        self.start_line = start;
        self.end_line = end;
        self
    }

    /// With token count
    pub fn with_tokens(mut self, tokens: usize) -> Self {
        self.token_count = tokens;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_creation() {
        let mut view = VirtualView::new("rust");
        view.source = "fn hello() {}\n\nfn world() {}".to_string();
        view.atoms = vec!["chunk:a".to_string(), "chunk:b".to_string()];
        view.symbol_locations.insert("hello".to_string(), 1);
        view.symbol_locations.insert("world".to_string(), 3);

        assert_eq!(view.line_count(), 3);
        assert_eq!(view.find_symbol("hello"), Some(1));
        assert!(view.contains_atom("chunk:a"));
    }

    #[test]
    fn test_snippet_extraction() {
        let mut view = VirtualView::new("rust");
        view.source = "line 1\nline 2\nfn target() {}\nline 4\nline 5".to_string();
        view.symbol_locations.insert("target".to_string(), 3);

        let snippet = view.snippet_for_symbol("target", 1).unwrap();
        assert!(snippet.contains("line 2"));
        assert!(snippet.contains("fn target"));
        assert!(snippet.contains("line 4"));
    }
}
