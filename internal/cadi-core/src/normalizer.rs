//! Semantic Normalizer
//!
//! Language-aware semantic normalization: parse → alpha-rename → canonicalize → semantic hash

use crate::error::CadiResult;
use sha2::{Sha256, Digest};
use regex::Regex;
use std::collections::HashMap;

#[cfg(feature = "ast-parsing")]
use tree_sitter::{Parser, Query, QueryCursor};

/// Result returned by normalization
#[derive(Debug, Clone)]
pub struct NormalizationResult {
    pub original: String,
    pub alpha_renamed: String,
    pub canonical: String,
    pub hash: String,
}

/// The Semantic Normalizer
#[derive(Debug, Clone)]
pub struct SemanticNormalizer {
    language: String,
}

impl SemanticNormalizer {
    /// Create a new normalizer for a supported language.
    pub fn new(language: &str) -> CadiResult<Self> {
        let supported = ["typescript", "python", "rust", "go"];
        if !supported.contains(&language) {
            return Err(crate::error::CadiError::UnsupportedPlatform(format!("Unsupported language: {}", language)));
        }
        Ok(Self {
            language: language.to_string(),
        })
    }

    /// Main normalization pipeline
    pub fn normalize(&self, code: &str) -> CadiResult<NormalizationResult> {
        // If AST parsing is enabled, use tree-sitter for alpha-renaming; otherwise fall back to simple canonicalization
        #[cfg(feature = "ast-parsing")]
        {
            let tree = self.parse(code)?;
            let alpha = self.alpha_rename(&tree, code)?;
            let canonical = self.canonicalize(&alpha)?;
            let hash = Self::compute_hash(&canonical);

            Ok(NormalizationResult {
                original: code.to_string(),
                alpha_renamed: alpha,
                canonical,
                hash,
            })
        }

        #[cfg(not(feature = "ast-parsing"))]
        {
            let alpha = self.simple_alpha_placeholder(code);
            let canonical = self.canonicalize(&alpha)?;
            let hash = Self::compute_hash(&canonical);

            Ok(NormalizationResult {
                original: code.to_string(),
                alpha_renamed: alpha,
                canonical,
                hash,
            })
        }
    }

    #[cfg(feature = "ast-parsing")]
    fn parse(&self, code: &str) -> CadiResult<tree_sitter::Tree> {
        let mut parser = Parser::new();

        let language_fn = match self.language.as_str() {
            "typescript" => tree_sitter_typescript::language_typescript,
            "python" => tree_sitter_python::language,
            "rust" => tree_sitter_rust::language,
            "go" => tree_sitter_go::language,
            _ => return Err(crate::error::CadiError::UnsupportedPlatform(format!("Unsupported language: {}", self.language))),
        };

        parser.set_language(&language_fn()).map_err(|e| crate::error::CadiError::AtomizerError(format!("Failed to set language: {:?}", e)))?;

        parser.parse(code, None).ok_or_else(|| crate::error::CadiError::AtomizerError("Parse failed".to_string()))
    }

    /// Alpha-rename all identifiers to _var0, _var1, etc.
    #[cfg(feature = "ast-parsing")]
    fn alpha_rename(&self, tree: &tree_sitter::Tree, code: &str) -> CadiResult<String> {
        let source_bytes = code.as_bytes();

        let query_str = match self.language.as_str() {
            "typescript" => "(identifier) @id",
            "python" => "(identifier) @id",
            "rust" => "(identifier) @id",
            "go" => "(identifier) @id",
            _ => "(identifier) @id",
        };

        let language = match self.language.as_str() {
            "typescript" => tree_sitter_typescript::language_typescript(),
            "python" => tree_sitter_python::language(),
            "rust" => tree_sitter_rust::language(),
            "go" => tree_sitter_go::language(),
            _ => return Err(crate::error::CadiError::UnsupportedPlatform(format!("Language not supported for query: {}", self.language))),
        };

        let query = Query::new(&language, query_str).map_err(|e| crate::error::CadiError::AtomizerError(format!("Query error: {}", e)))?;
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source_bytes);

        let mut identifier_map: HashMap<String, String> = HashMap::new();
        let mut counter: usize = 0;
        let mut replacements: Vec<(usize, usize, String)> = Vec::new();

        for m in matches {
            for capture in m.captures {
                let range = capture.node.range();
                let start = range.start_byte;
                let end = range.end_byte;
                let original = code.get(start..end).unwrap_or("").to_string();

                if !Self::is_keyword(&original, &self.language) {
                    let entry = identifier_map.entry(original.clone()).or_insert_with(|| {
                        let name = format!("_var{}", counter);
                        counter += 1;
                        name
                    }).clone();

                    replacements.push((start, end, entry));
                }
            }
        }

        // Sort by start position
        replacements.sort_by_key(|r| r.0);

        let mut result = String::new();
        let mut last_pos = 0usize;

        for (start, end, replacement) in replacements {
            if start >= last_pos {
                result.push_str(&code[last_pos..start]);
                result.push_str(&replacement);
                last_pos = end;
            }
        }
        result.push_str(&code[last_pos..]);

        Ok(result)
    }

    /// Fallback simple alpha renaming when tree-sitter isn't enabled (best-effort)
    #[cfg(not(feature = "ast-parsing"))]
    fn simple_alpha_placeholder(&self, code: &str) -> String {
        // Naive: replace contiguous word characters that are not keywords with numbered vars
        let word_re = Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\b").unwrap();
        let mut id_map: HashMap<String, String> = HashMap::new();
        let mut counter = 0usize;

        word_re.replace_all(code, |caps: &regex::Captures| {
            let s = &caps[1];
            if Self::is_keyword(s, &self.language) {
                s.to_string()
            } else {
                id_map.entry(s.to_string()).or_insert_with(|| {
                    let name = format!("_var{}", counter);
                    counter += 1;
                    name
                }).clone()
            }
        }).to_string()
    }

    /// Canonicalize structure: remove comments, normalize whitespace, punctuation
    fn canonicalize(&self, code: &str) -> CadiResult<String> {
        let no_comments = Self::strip_comments(code, &self.language);

        let normalized = no_comments
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        let cleaned = Self::normalize_punctuation(&normalized, &self.language);

        Ok(cleaned)
    }

    fn strip_comments(code: &str, language: &str) -> String {
        match language {
            "typescript" | "rust" | "go" | "java" => {
                let mut result = String::new();
                let mut chars = code.chars().peekable();
                let mut in_string = false;
                let mut string_char = '\0';

                while let Some(c) = chars.next() {
                    if in_string {
                        result.push(c);
                        if c == string_char && result.chars().last() != Some('\\') {
                            in_string = false;
                        }
                    } else if c == '"' || c == '\'' || c == '`' {
                        result.push(c);
                        in_string = true;
                        string_char = c;
                    } else if c == '/' && chars.peek() == Some(&'/') {
                        chars.next();
                        while let Some(c) = chars.next() {
                            if c == '\n' {
                                result.push('\n');
                                break;
                            }
                        }
                    } else if c == '/' && chars.peek() == Some(&'*') {
                        chars.next();
                        while let Some(c) = chars.next() {
                            if c == '*' && chars.peek() == Some(&'/') {
                                chars.next();
                                break;
                            }
                        }
                    } else {
                        result.push(c);
                    }
                }
                result
            }
            "python" => {
                let mut result = String::new();
                let mut chars = code.chars().peekable();
                let mut in_string = false;
                let mut string_char = '\0';

                while let Some(c) = chars.next() {
                    if in_string {
                        result.push(c);
                        if string_char == '"' && c == '"' && chars.peek() == Some(&'"') {
                            result.push(chars.next().unwrap());
                            result.push(chars.next().unwrap());
                            in_string = false;
                        }
                    } else if c == '#' {
                        while let Some(c) = chars.next() {
                            if c == '\n' {
                                result.push('\n');
                                break;
                            }
                        }
                    } else if c == '"' && chars.peek() == Some(&'"') {
                        // triple quote start
                        result.push(c);
                        result.push(chars.next().unwrap());
                        result.push(chars.next().unwrap());
                        in_string = true;
                        string_char = '"';
                    } else {
                        result.push(c);
                    }
                }
                result
            }
            _ => code.to_string(),
        }
    }

    fn normalize_punctuation(code: &str, language: &str) -> String {
        let mut result = code.to_string();

        // Normalize spacing around operators
        result = Regex::new(r"\s*([+\-*/%=<>!&|^]+)\s*")
            .unwrap()
            .replace_all(&result, " $1 ")
            .to_string();

        // Collapse multiple whitespace to a single space and trim
        result = Regex::new(r"\s+")
            .unwrap()
            .replace_all(&result, " ")
            .to_string()
            .trim()
            .to_string();

        // Normalize parentheses and commas: remove space before '(', ensure single space after comma, no extra spaces before ')'
        result = Regex::new(r"\s*\(\s*")
            .unwrap()
            .replace_all(&result, "(")
            .to_string();
        result = Regex::new(r"\s*,\s*")
            .unwrap()
            .replace_all(&result, ", ")
            .to_string();
        result = Regex::new(r"\s*\)\s*")
            .unwrap()
            .replace_all(&result, ")")
            .to_string();

        // Remove trailing semicolons for python normalization
        if language == "python" {
            result = result.replace(';', "");
        }

        result
    }

    fn is_keyword(word: &str, language: &str) -> bool {
        let keywords = match language {
            "typescript" => vec![
                "function", "const", "let", "var", "return", "if", "else", "for", "while",
                "class", "interface", "async", "await", "export", "import", "type",
            ],
            "python" => vec![
                "def", "return", "if", "else", "for", "while", "class", "import", "from",
                "async", "await", "with", "try", "except", "lambda",
            ],
            "rust" => vec![
                "fn", "let", "return", "if", "else", "for", "while", "impl", "trait",
                "struct", "enum", "pub", "async", "await", "use",
            ],
            _ => vec![],
        };
        keywords.contains(&word)
    }

    fn compute_hash(canonical: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        format!("semantic:{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_normalization_smoke() {
        let normalizer = SemanticNormalizer::new("typescript").unwrap();
        let a = r#"function add(x, y) { return x + y; }"#;
        let b = r#"function  add  ( a , b ) { return a + b; }"#;
        let r1 = normalizer.normalize(a).unwrap();
        let r2 = normalizer.normalize(b).unwrap();
        if r1.hash != r2.hash {
            // Print canonical forms to diagnose differences
            eprintln!("canonical A:\n{}\n---\ncanonical B:\n{}\n---", r1.canonical, r2.canonical);
        }
        assert_eq!(r1.hash, r2.hash);
    }
}
