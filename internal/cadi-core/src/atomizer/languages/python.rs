//! Python-specific atomizer

use crate::atomizer::{AtomizerConfig, ExtractedAtom};
use crate::error::CadiResult;

/// Python atomizer
pub struct PythonAtomizer {
    config: AtomizerConfig,
}

impl PythonAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { config }
    }

    /// Extract atoms from Python
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use crate::atomizer::AtomExtractor;
        AtomExtractor::new("python", self.config.clone()).extract(source)
    }
}

/// Common Python patterns
pub struct PythonPatterns;

impl PythonPatterns {
    /// Check if a function is a test
    pub fn is_test(name: &str, decorators: &[String]) -> bool {
        name.starts_with("test_") 
            || decorators.iter().any(|d| d.contains("@pytest") || d.contains("@test"))
    }
    
    /// Check if this is a private function/class
    pub fn is_private(name: &str) -> bool {
        name.starts_with('_') && !name.starts_with("__")
    }
    
    /// Check if this is a dunder method
    pub fn is_dunder(name: &str) -> bool {
        name.starts_with("__") && name.ends_with("__")
    }
    
    /// Check if this is a dataclass
    pub fn is_dataclass(decorators: &[String]) -> bool {
        decorators.iter().any(|d| d.contains("@dataclass"))
    }
    
    /// Extract decorator names
    pub fn extract_decorators(source: &str) -> Vec<String> {
        let mut decorators = Vec::new();
        
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('@') {
                decorators.push(trimmed.to_string());
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }
        }
        
        decorators
    }
}
