//! HLSL-specific atomizer (fallback)

use crate::atomizer::{AtomizerConfig, ExtractedAtom};
use crate::error::CadiResult;

pub struct HlslAtomizer {
    config: AtomizerConfig,
}

impl HlslAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { config }
    }

    /// Currently fallback-only (no stable tree-sitter crate detected)
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use crate::atomizer::AtomExtractor;
        AtomExtractor::new("hlsl", self.config.clone()).extract(source)
    }
}
