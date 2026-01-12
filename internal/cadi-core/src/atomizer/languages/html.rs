//! HTML atomizer

use crate::atomizer::{AtomizerConfig, ExtractedAtom};
use crate::error::CadiResult;

/// HTML atomizer - extracts meaningful fragments and embedded scripts/styles
pub struct HtmlAtomizer {
    config: AtomizerConfig,
}

impl HtmlAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { config }
    }

    /// Extract atoms from HTML
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use crate::atomizer::AtomExtractor;
        AtomExtractor::new("html", self.config.clone()).extract(source)
    }
}
