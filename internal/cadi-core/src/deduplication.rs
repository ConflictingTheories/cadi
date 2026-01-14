use crate::normalizer::SemanticNormalizer;
use crate::error::CadiResult;
use std::collections::HashMap;

/// In-memory deduplication engine for semantic hashes
#[derive(Debug, Default)]
pub struct DeduplicationEngine {
    pub hash_index: HashMap<String, Vec<String>>, // hash -> [chunk ids]
}

impl DeduplicationEngine {
    pub fn new() -> Self {
        Self {
            hash_index: HashMap::new(),
        }
    }

    /// Register a chunk by semantic hash
    /// Returns: (is_new, equivalents)
    pub fn register_chunk(&mut self, chunk_id: &str, semantic_hash: &str) -> (bool, Vec<String>) {
        let entry = self.hash_index.entry(semantic_hash.to_string()).or_insert_with(Vec::new);
        let is_new = entry.is_empty();
        let equivalents = entry.clone();
        entry.push(chunk_id.to_string());
        (is_new, equivalents)
    }

    pub fn find_equivalents(&self, semantic_hash: &str) -> Vec<String> {
        self.hash_index.get(semantic_hash).cloned().unwrap_or_default()
    }

    /// Check semantic similarity between two code strings
    /// Returns: (is_identical, similarity_score)
    pub async fn check_similarity(code_a: &str, code_b: &str, language: &str) -> CadiResult<(bool, f32)> {
        let normalizer = SemanticNormalizer::new(language)?;
        let result_a = normalizer.normalize(code_a)?;
        let result_b = normalizer.normalize(code_b)?;

        if result_a.canonical == result_b.canonical {
            return Ok((true, 1.0));
        }

        let similarity = Self::levenshtein_similarity(&result_a.canonical, &result_b.canonical);
        Ok((false, similarity))
    }

    fn levenshtein_similarity(a: &str, b: &str) -> f32 {
        let dist = Self::levenshtein_distance(a, b);
        let max_len = a.len().max(b.len());
        if max_len == 0 { return 1.0; }
        1.0 - (dist as f32 / max_len as f32)
    }

    fn levenshtein_distance(a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let mut matrix = vec![vec![0usize; b_chars.len() + 1]; a_chars.len() + 1];

        for i in 0..=a_chars.len() {
            matrix[i][0] = i;
        }
        for j in 0..=b_chars.len() {
            matrix[0][j] = j;
        }

        for (i, &a_char) in a_chars.iter().enumerate() {
            for (j, &b_char) in b_chars.iter().enumerate() {
                let cost = if a_char == b_char { 0 } else { 1 };
                matrix[i+1][j+1] = ((matrix[i][j+1] + 1).min(matrix[i+1][j] + 1)).min(matrix[i][j] + cost);
            }
        }

        matrix[a_chars.len()][b_chars.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedup_register_and_find() {
        let mut engine = DeduplicationEngine::new();
        let hash = "semantic:abc123";
        let (is_new_1, eq1) = engine.register_chunk("chunk1", hash);
        assert!(is_new_1);
        assert!(eq1.is_empty());

        let (is_new_2, eq2) = engine.register_chunk("chunk2", hash);
        assert!(!is_new_2);
        assert_eq!(eq2, vec!["chunk1".to_string()]);

        let found = engine.find_equivalents(hash);
        assert_eq!(found, vec!["chunk1".to_string(), "chunk2".to_string()]);
    }
}
