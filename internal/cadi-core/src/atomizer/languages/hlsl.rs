//! HLSL-specific atomizer (fallback)

use crate::atomizer::{AtomizerConfig, ExtractedAtom};
use crate::error::CadiResult;

pub struct HlslAtomizer {
    _config: AtomizerConfig,
}

impl HlslAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { _config: config }
    }

    /// Currently fallback-only (no stable tree-sitter crate detected)
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use crate::atomizer::AtomExtractor;
        AtomExtractor::new("hlsl", self._config.clone()).extract(source)
    }
}
