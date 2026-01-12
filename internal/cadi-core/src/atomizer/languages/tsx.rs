//! TSX atomizer (TypeScript + JSX)

use crate::atomizer::{AtomizerConfig, ExtractedAtom};
use crate::error::CadiResult;

/// TSX atomizer (uses TypeScript extractor semantics)
pub struct TSXAtomizer {
    config: AtomizerConfig,
}

impl TSXAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { config }
    }

    /// Extract atoms from TSX files
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use crate::atomizer::AtomExtractor;
        // Use typescript extractor for TSX
        AtomExtractor::new("typescript", self.config.clone()).extract(source)
    }
}
