//! Code Assembler
//!
//! Assembles atoms into syntactically valid code.

use super::config::{ViewConfig, ViewFormat};
use super::view::{ViewFragment, InclusionReason};
use crate::graph::GraphNode;

/// Assembler for creating virtual views from atoms
pub struct Assembler {
    config: ViewConfig,
}

impl Assembler {
    pub fn new(config: ViewConfig) -> Self {
        Self { config }
    }

    /// Assemble atoms into a single coherent source
    pub fn assemble(
        &self,
        atoms: Vec<(GraphNode, String)>,
        language: &str,
    ) -> AssemblyResult {
        let mut output = String::new();
        let mut fragments = Vec::new();
        let mut symbol_locations = std::collections::HashMap::new();
        let mut current_line = 1;
        let mut total_tokens = 0;

        // Sort atoms by priority if configured
        let sorted_atoms = if self.config.sort_by_type {
            self.sort_by_type(atoms)
        } else {
            atoms
        };

        for (node, content) in sorted_atoms {
            // Check token limit
            let atom_tokens = content.len() / 4;
            if total_tokens + atom_tokens > self.config.max_tokens {
                break;
            }

            // Add separator if configured
            if self.config.add_separators && !output.is_empty() {
                let separator = self.create_separator(&node, language);
                output.push_str(&separator);
                current_line += separator.lines().count();
            }

            // Track symbol locations
            for symbol in &node.symbols_defined {
                symbol_locations.insert(symbol.clone(), current_line);
            }

            // Add content
            let formatted_content = self.format_content(&content, language);
            let content_lines = formatted_content.lines().count();

            // Create fragment
            fragments.push(ViewFragment {
                chunk_id: node.chunk_id.clone(),
                alias: node.primary_alias.clone(),
                start_line: current_line,
                end_line: current_line + content_lines - 1,
                token_count: atom_tokens,
                inclusion_reason: InclusionReason::Requested,
                defines: node.symbols_defined.clone(),
            });

            output.push_str(&formatted_content);
            if !formatted_content.ends_with('\n') {
                output.push('\n');
            }
            output.push('\n');

            current_line += content_lines + 1;
            total_tokens += atom_tokens;
        }

        AssemblyResult {
            source: output,
            fragments,
            symbol_locations,
            total_tokens,
            truncated: total_tokens >= self.config.max_tokens,
        }
    }

    /// Sort atoms by type priority
    fn sort_by_type(&self, mut atoms: Vec<(GraphNode, String)>) -> Vec<(GraphNode, String)> {
        atoms.sort_by_key(|(node, _)| {
            match node.granularity.as_str() {
                "import" => 0,
                "type" | "struct" | "interface" | "enum" => 1,
                "trait" => 2,
                "constant" => 3,
                "function" => 4,
                "async_function" => 4,
                "class" => 5,
                "module" => 6,
                _ => 10,
            }
        });
        atoms
    }

    /// Create a separator comment
    fn create_separator(&self, node: &GraphNode, language: &str) -> String {
        let label = node.primary_alias.as_ref()
            .unwrap_or(&node.chunk_id);
        
        let comment_style = match language {
            "python" => "#",
            "rust" | "typescript" | "javascript" | "go" | "java" | "c" | "cpp" => "//",
            _ => "//",
        };
        
        format!("{} --- {} ---\n", comment_style, label)
    }

    /// Format content based on view format
    fn format_content(&self, content: &str, language: &str) -> String {
        match self.config.format {
            ViewFormat::Source => content.to_string(),
            ViewFormat::Minimal => self.minimize(content),
            ViewFormat::Documented => content.to_string(),
            ViewFormat::Signatures => self.extract_signatures(content, language),
            ViewFormat::Json => content.to_string(),
        }
    }

    /// Minimize content (remove comments, compact whitespace)
    fn minimize(&self, content: &str) -> String {
        let mut result = String::new();
        let mut in_block_comment = false;

        for line in content.lines() {
            let trimmed = line.trim();
            
            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }
            
            // Skip single-line comments
            if trimmed.starts_with("//") || trimmed.starts_with("#") {
                continue;
            }
            
            // Handle block comments (simplified)
            if trimmed.contains("/*") {
                in_block_comment = true;
            }
            if in_block_comment {
                if trimmed.contains("*/") {
                    in_block_comment = false;
                }
                continue;
            }
            
            result.push_str(line);
            result.push('\n');
        }

        result
    }

    /// Extract just the signatures (no bodies)
    fn extract_signatures(&self, content: &str, language: &str) -> String {
        match language {
            "rust" => self.extract_rust_signatures(content),
            "typescript" => self.extract_ts_signatures(content),
            "python" => self.extract_python_signatures(content),
            _ => content.to_string(),
        }
    }

    fn extract_rust_signatures(&self, content: &str) -> String {
        let mut result = String::new();
        
        // Match function signatures
        let fn_regex = regex::Regex::new(
            r"(?m)^(\s*)(?:pub(?:\([^)]*\))?\s+)?(async\s+)?fn\s+\w+[^{]+\{"
        ).unwrap();
        
        for cap in fn_regex.captures_iter(content) {
            let sig = cap.get(0).unwrap().as_str();
            let sig = sig.trim_end_matches('{').trim();
            result.push_str(sig);
            result.push_str(";\n");
        }
        
        // Match struct definitions
        let struct_regex = regex::Regex::new(
            r"(?m)^(?:pub(?:\([^)]*\))?\s+)?struct\s+\w+[^{]*\{[^}]+\}"
        ).unwrap();
        
        for cap in struct_regex.find_iter(content) {
            result.push_str(cap.as_str());
            result.push('\n');
        }
        
        result
    }

    fn extract_ts_signatures(&self, content: &str) -> String {
        let mut result = String::new();
        
        // Match function signatures
        let fn_regex = regex::Regex::new(
            r"(?m)^(?:export\s+)?(?:async\s+)?function\s+\w+\([^)]*\)[^{]*"
        ).unwrap();
        
        for cap in fn_regex.find_iter(content) {
            result.push_str(cap.as_str().trim());
            result.push_str(";\n");
        }
        
        // Match interface definitions
        let interface_regex = regex::Regex::new(
            r"(?m)^(?:export\s+)?interface\s+\w+[^{]*\{[^}]+\}"
        ).unwrap();
        
        for cap in interface_regex.find_iter(content) {
            result.push_str(cap.as_str());
            result.push('\n');
        }
        
        result
    }

    fn extract_python_signatures(&self, content: &str) -> String {
        let mut result = String::new();
        
        // Match function definitions
        let fn_regex = regex::Regex::new(
            r"(?m)^(\s*)(?:async\s+)?def\s+\w+\([^)]*\)(?:\s*->\s*[^:]+)?:"
        ).unwrap();
        
        for cap in fn_regex.find_iter(content) {
            result.push_str(cap.as_str().trim_end_matches(':'));
            result.push_str(": ...\n");
        }
        
        result
    }
}

/// Result of assembly operation
pub struct AssemblyResult {
    pub source: String,
    pub fragments: Vec<ViewFragment>,
    pub symbol_locations: std::collections::HashMap<String, usize>,
    pub total_tokens: usize,
    pub truncated: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimize() {
        let assembler = Assembler::new(ViewConfig::minimal());
        
        let content = r#"
// This is a comment
fn hello() {
    // Another comment
    println!("hello");
}
"#;
        
        let minimized = assembler.minimize(content);
        assert!(!minimized.contains("// This is a comment"));
        assert!(minimized.contains("fn hello()"));
    }

    #[test]
    fn test_rust_signatures() {
        let assembler = Assembler::new(ViewConfig::default());
        
        let content = r#"
pub fn hello(name: &str) -> String {
    format!("Hello, {}", name)
}

pub struct Person {
    name: String,
    age: u32,
}
"#;
        
        let signatures = assembler.extract_rust_signatures(content);
        assert!(signatures.contains("pub fn hello(name: &str) -> String;"));
        assert!(signatures.contains("pub struct Person"));
    }
}
