//! WGSL-specific atomizer

use crate::atomizer::{AtomizerConfig, ExtractedAtom};
use crate::error::CadiResult;

pub struct WgslAtomizer {
    config: AtomizerConfig,
}

impl WgslAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { config }
    }

    /// Fallback extraction only: WGSL tree-sitter crate depends on an older tree-sitter version,
    /// so use the generic extractor for now to avoid dependency conflicts.
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use crate::atomizer::AtomExtractor;
        AtomExtractor::new("wgsl", self.config.clone()).extract(source)
    }
}
