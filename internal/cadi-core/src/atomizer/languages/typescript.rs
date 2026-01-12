//! TypeScript/JavaScript-specific atomizer

use crate::atomizer::{AtomizerConfig, ExtractedAtom};
use crate::error::CadiResult;

/// TypeScript/JavaScript atomizer
pub struct TypeScriptAtomizer {
    config: AtomizerConfig,
}

impl TypeScriptAtomizer {
    pub fn new(config: AtomizerConfig) -> Self {
        Self { config }
    }

    /// Extract atoms from TypeScript/JavaScript
    pub fn extract(&self, source: &str) -> CadiResult<Vec<ExtractedAtom>> {
        use crate::atomizer::AtomExtractor;
        AtomExtractor::new("typescript", self.config.clone()).extract(source)
    }
}

/// Common TypeScript patterns
pub struct TypeScriptPatterns;

impl TypeScriptPatterns {
    /// Check if an export is a React component
    pub fn is_react_component(source: &str) -> bool {
        source.contains("React.FC") 
            || source.contains("React.Component")
            || source.contains("useState")
            || source.contains("useEffect")
    }
    
    /// Check if this is a type-only export
    pub fn is_type_export(source: &str) -> bool {
        source.contains("export type") || source.contains("export interface")
    }
    
    /// Extract JSX component name
    pub fn extract_component_name(source: &str) -> Option<String> {
        // Match: export const Name = () => or export function Name
        let patterns = [
            r"export\s+const\s+(\w+)\s*=",
            r"export\s+function\s+(\w+)",
            r"export\s+default\s+function\s+(\w+)",
        ];
        
        for pattern in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(cap) = re.captures(source) {
                    return cap.get(1).map(|m| m.as_str().to_string());
                }
            }
        }
        
        None
    }
}
