use crate::error::Result;
use crate::types::{ChunkingStrategy, ScraperConfig};
use sha2::{Sha256, Digest};

/// Chunker that splits content into semantic or fixed-size chunks
pub struct Chunker {
    config: ScraperConfig,
}

/// Represents a single chunk with metadata
#[derive(Debug, Clone)]
pub struct Chunk {
    pub id: String,                      // Content hash
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub language: Option<String>,
    pub concepts: Vec<String>,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
    pub source_file: String,
}

impl Chunker {
    pub fn new(config: ScraperConfig) -> Self {
        Self { config }
    }

    /// Chunk content based on the configured strategy
    pub fn chunk(
        &self,
        content: &str,
        language: Option<&str>,
        source_file: &str,
    ) -> Result<Vec<Chunk>> {
        match self.config.chunking_strategy {
            ChunkingStrategy::ByFile => self.chunk_by_file(content, language, source_file),
            ChunkingStrategy::Semantic => self.chunk_semantic(content, language, source_file),
            ChunkingStrategy::FixedSize => self.chunk_fixed_size(content, language, source_file),
            ChunkingStrategy::Hierarchical => {
                self.chunk_hierarchical(content, language, source_file)
            }
            ChunkingStrategy::ByLineCount => self.chunk_by_lines(content, language, source_file),
        }
    }

    /// Return entire content as single chunk
    fn chunk_by_file(
        &self,
        content: &str,
        language: Option<&str>,
        source_file: &str,
    ) -> Result<Vec<Chunk>> {
        let id = self.compute_hash(content);
        let line_count = content.lines().count();

        Ok(vec![Chunk {
            id,
            content: content.to_string(),
            start_line: 0,
            end_line: line_count,
            language: language.map(String::from),
            concepts: extract_concepts(content, language),
            parent_id: None,
            children: Vec::new(),
            source_file: source_file.to_string(),
        }])
    }

    /// Chunk by semantic boundaries (functions, classes, etc)
    fn chunk_semantic(
        &self,
        content: &str,
        language: Option<&str>,
        source_file: &str,
    ) -> Result<Vec<Chunk>> {
        let boundaries = self.find_semantic_boundaries(content, language)?;

        if boundaries.is_empty() {
            return self.chunk_by_file(content, language, source_file);
        }

        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (i, boundary) in boundaries.iter().enumerate() {
            let start = if i == 0 { 0 } else { boundaries[i - 1] };
            let end = *boundary;

            if start >= end {
                continue;
            }

            let chunk_content = lines[start..end].join("\n");
            if chunk_content.trim().is_empty() {
                continue;
            }

            let id = self.compute_hash(&chunk_content);
            let mut chunk = Chunk {
                id,
                content: chunk_content,
                start_line: start,
                end_line: end,
                language: language.map(String::from),
                concepts: extract_concepts(&lines[start..end].join("\n"), language),
                parent_id: None,
                children: Vec::new(),
                source_file: source_file.to_string(),
            };

            // Add overlap if enabled
            if self.config.include_overlap && i > 0 {
                let overlap_start =
                    (start.saturating_sub(self.config.overlap_size / 100)).max(0);
                let overlap_lines = &lines[overlap_start..start];
                let overlap_content = overlap_lines.join("\n");
                chunk.content = format!("{}\n{}", overlap_content, chunk.content);
            }

            chunks.push(chunk);
        }

        Ok(chunks)
    }

    /// Chunk content into fixed sizes
    fn chunk_fixed_size(
        &self,
        content: &str,
        language: Option<&str>,
        source_file: &str,
    ) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let lines_per_chunk = (self.config.max_chunk_size / 80).max(10); // Rough estimate

        for (i, chunk_lines) in lines.chunks(lines_per_chunk).enumerate() {
            let chunk_content = chunk_lines.join("\n");
            let id = self.compute_hash(&chunk_content);

            chunks.push(Chunk {
                id,
                content: chunk_content,
                start_line: i * lines_per_chunk,
                end_line: (i + 1) * lines_per_chunk,
                language: language.map(String::from),
                concepts: extract_concepts(&lines.join("\n"), language),
                parent_id: None,
                children: Vec::new(),
                source_file: source_file.to_string(),
            });
        }

        Ok(chunks)
    }

    /// Hierarchical chunking with parent-child relationships
    fn chunk_hierarchical(
        &self,
        content: &str,
        language: Option<&str>,
        source_file: &str,
    ) -> Result<Vec<Chunk>> {
        // First create file-level chunk as parent
        let parent_id = self.compute_hash(content);
        let mut chunks = vec![Chunk {
            id: parent_id.clone(),
            content: content.to_string(),
            start_line: 0,
            end_line: content.lines().count(),
            language: language.map(String::from),
            concepts: extract_concepts(content, language),
            parent_id: None,
            children: Vec::new(),
            source_file: source_file.to_string(),
        }];

        // Then create semantic sub-chunks as children
        let mut children_chunks = self.chunk_semantic(content, language, source_file)?;
        for child in &mut children_chunks {
            child.parent_id = Some(parent_id.clone());
        }

        chunks.extend(children_chunks);
        Ok(chunks)
    }

    /// Chunk by fixed line count
    fn chunk_by_lines(
        &self,
        content: &str,
        language: Option<&str>,
        source_file: &str,
    ) -> Result<Vec<Chunk>> {
        let lines_per_chunk = 100; // Default 100 lines per chunk
        self.chunk_by_custom_line_count(content, language, source_file, lines_per_chunk)
    }

    fn chunk_by_custom_line_count(
        &self,
        content: &str,
        language: Option<&str>,
        source_file: &str,
        lines_per_chunk: usize,
    ) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (i, chunk_lines) in lines.chunks(lines_per_chunk).enumerate() {
            let chunk_content = chunk_lines.join("\n");
            let id = self.compute_hash(&chunk_content);
            let concepts = extract_concepts(&chunk_content, language);

            chunks.push(Chunk {
                id,
                content: chunk_content,
                start_line: i * lines_per_chunk,
                end_line: std::cmp::min((i + 1) * lines_per_chunk, lines.len()),
                language: language.map(String::from),
                concepts,
                parent_id: None,
                children: Vec::new(),
                source_file: source_file.to_string(),
            });
        }

        Ok(chunks)
    }

    /// Find semantic boundaries in code
    fn find_semantic_boundaries(&self, content: &str, language: Option<&str>) -> Result<Vec<usize>> {
        let mut boundaries = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        match language {
            Some("rust") => {
                for (i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ")
                        || trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ")
                        || trimmed.starts_with("impl ") || trimmed.starts_with("pub impl ")
                        || trimmed.starts_with("trait ") || trimmed.starts_with("pub trait ")
                    {
                        boundaries.push(i);
                    }
                }
            }
            Some("typescript") | Some("javascript") => {
                for (i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("function ") || trimmed.starts_with("export function ")
                        || trimmed.starts_with("class ") || trimmed.starts_with("export class ")
                        || trimmed.starts_with("interface ") || trimmed.starts_with("export interface ")
                        || (trimmed.starts_with("const ") && trimmed.contains("=>"))
                    {
                        boundaries.push(i);
                    }
                }
            }
            Some("python") => {
                for (i, line) in lines.iter().enumerate() {
                    if !line.starts_with(' ') && (line.starts_with("def ") || line.starts_with("class ")) {
                        boundaries.push(i);
                    }
                }
            }
            _ => {}
        }

        Ok(boundaries)
    }

    /// Compute SHA256 hash of content
    pub fn compute_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        format!("chunk:{}", hex::encode(result))
    }
}

/// Extract key concepts from content
fn extract_concepts(content: &str, language: Option<&str>) -> Vec<String> {
    let mut concepts = Vec::new();

    match language {
        Some("rust") => {
            if content.contains("async") {
                concepts.push("async".to_string());
            }
            if content.contains("trait") {
                concepts.push("trait".to_string());
            }
            if content.contains("macro") {
                concepts.push("macro".to_string());
            }
            if content.contains("unsafe") {
                concepts.push("unsafe".to_string());
            }
        }
        Some("typescript") | Some("javascript") => {
            if content.contains("async") {
                concepts.push("async".to_string());
            }
            if content.contains("class") {
                concepts.push("oop".to_string());
            }
            if content.contains("react") || content.contains("React") {
                concepts.push("react".to_string());
            }
            if content.contains("@") {
                concepts.push("decorators".to_string());
            }
        }
        Some("python") => {
            if content.contains("async") {
                concepts.push("async".to_string());
            }
            if content.contains("@") {
                concepts.push("decorators".to_string());
            }
            if content.contains("class") {
                concepts.push("oop".to_string());
            }
        }
        _ => {}
    }

    concepts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_consistency() {
        let chunker = Chunker::new(ScraperConfig::default());
        let content = "fn hello() {}";
        let hash1 = chunker.compute_hash(content);
        let hash2 = chunker.compute_hash(content);
        assert_eq!(hash1, hash2);
        assert!(hash1.starts_with("chunk:"));
    }
}
