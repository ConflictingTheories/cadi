//! JSX atomizer (React / web)

use crate::atomizer::{AtomizerConfig, ExtractedAtom};
use crate::error::CadiResult;

/// JSX atomizer (uses JS extractor semantics)
pub struct JSXAtomizer {
    config: AtomizerConfig,
}

impl JSXAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { config }
    }

    /// Extract atoms from JSX/JS files
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use crate::atomizer::AtomExtractor;
        // Use javascript extractor for JSX
        AtomExtractor::new("javascript", self.config.clone()).extract(source)
    }
}
