# Stage 2: Semantic Hashing & Deduplication
## Detailed Implementation Specification

### Overview
Code with identical semantics produces identical hashes, enabling automatic discovery and reuse of equivalent solutions across projects. This is the foundation for forward acceleration: a blog app automatically discovers it can reuse the todo app's JWT auth because their semantic hashes match.

---

## 2.1 Enhanced Normalizer

### File: `internal/cadi-core/src/normalizer.rs` (complete rewrite)

**Current State**: TypeScript-only alpha-renaming with basic hashing.

**Goal**: Robust, language-agnostic semantic normalization using tree-sitter.

```rust
use sha2::{Sha256, Digest};
use tree_sitter::{Parser, Query, QueryCursor, Node};
use std::collections::HashMap;

/// The Semantic Normalizer
/// Transforms: Source Code → Canonical AST → Semantic Hash
/// Ensures identical semantics = identical hash across projects
pub struct SemanticNormalizer {
    language: String,
}

impl SemanticNormalizer {
    pub fn new(language: &str) -> Result<Self> {
        // Validate language support
        let supported = ["typescript", "python", "rust", "go", "java"];
        if !supported.contains(&language) {
            return Err(format!("Unsupported language: {}", language));
        }
        Ok(Self {
            language: language.to_string(),
        })
    }
    
    /// Main normalization pipeline
    pub fn normalize(&self, code: &str) -> Result<NormalizationResult> {
        // 1. Parse to AST
        let tree = self.parse(code)?;
        
        // 2. Apply alpha-renaming
        let alpha_renamed = self.alpha_rename(&tree, code)?;
        
        // 3. Normalize structure (comments, formatting, etc.)
        let canonical = self.canonicalize(&alpha_renamed)?;
        
        // 4. Compute semantic hash
        let hash = Self::compute_hash(&canonical);
        
        Ok(NormalizationResult {
            original: code.to_string(),
            alpha_renamed,
            canonical,
            hash,
        })
    }
    
    fn parse(&self, code: &str) -> Result<tree_sitter::Tree> {
        let mut parser = Parser::new();
        
        let language_fn = match self.language.as_str() {
            "typescript" => tree_sitter_typescript::language_typescript,
            "python" => tree_sitter_python::language_python,
            "rust" => tree_sitter_rust::language_rust,
            "go" => tree_sitter_go::language_go,
            "java" => tree_sitter_java::language_java,
            _ => return Err("Unsupported language".into()),
        };
        
        parser.set_language(&language_fn()).map_err(|e| format!("Failed to set language: {:?}", e))?;
        
        parser.parse(code, None)
            .ok_or_else(|| "Parse failed".to_string())
    }
    
    /// Alpha-rename all identifiers to var_0, var_1, etc.
    /// This ensures equivalent functions have identical names after normalization
    fn alpha_rename(&self, tree: &tree_sitter::Tree, code: &str) -> Result<String> {
        let source_bytes = code.as_bytes();
        
        // Query all identifiers (language-specific)
        let query_str = match self.language.as_str() {
            "typescript" => "(identifier) @id",
            "python" => "(identifier) @id",
            "rust" => "(identifier) @id",
            _ => "(identifier) @id",  // Fallback
        };
        
        let language = match self.language.as_str() {
            "typescript" => tree_sitter_typescript::language_typescript(),
            "python" => tree_sitter_python::language_python(),
            "rust" => tree_sitter_rust::language_rust(),
            _ => return Err("Language not supported for query".into()),
        };
        
        let query = Query::new(&language, query_str)
            .map_err(|e| format!("Query error: {}", e))?;
        
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source_bytes);
        
        let mut identifier_map: HashMap<String, String> = HashMap::new();
        let mut counter = 0;
        let mut replacements: Vec<(usize, usize, String)> = Vec::new();
        
        // Collect all identifier replacements
        for m in matches {
            for capture in m.captures {
                let range = capture.node.range();
                let start = range.start_byte;
                let end = range.end_byte;
                let original = &code[start..end];
                
                // Don't rename language keywords or built-ins
                if !Self::is_keyword(&original, &self.language) {
                    let new_name = identifier_map.entry(original.to_string())
                        .or_insert_with(|| {
                            let name = format!("_var{}", counter);
                            counter += 1;
                            name
                        })
                        .clone();
                    
                    replacements.push((start, end, new_name));
                }
            }
        }
        
        // Sort by position to avoid offset issues
        replacements.sort_by_key(|r| r.0);
        
        // Apply replacements
        let mut result = String::new();
        let mut last_pos = 0;
        
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
    
    /// Canonicalize structure: remove comments, normalize whitespace, etc.
    fn canonicalize(&self, code: &str) -> Result<String> {
        // Remove comments
        let no_comments = Self::strip_comments(code, &self.language);
        
        // Normalize whitespace
        let normalized = no_comments
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        
        // Remove non-essential punctuation variations
        let cleaned = Self::normalize_punctuation(&normalized, &self.language);
        
        Ok(cleaned)
    }
    
    fn strip_comments(code: &str, language: &str) -> String {
        match language {
            "typescript" | "rust" | "go" | "java" => {
                // Remove // comments and /* */ comments
                let mut result = String::new();
                let mut chars = code.chars().peekable();
                let mut in_string = false;
                let mut string_char = ' ';
                
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
                        // Skip until end of line
                        while let Some(c) = chars.next() {
                            if c == '\n' {
                                result.push('\n');
                                break;
                            }
                        }
                    } else if c == '/' && chars.peek() == Some(&'*') {
                        chars.next();
                        // Skip until */
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
                // Remove # comments and """ """ comments
                let mut result = String::new();
                let mut chars = code.chars().peekable();
                let mut in_string = false;
                let mut string_char = ' ';
                
                while let Some(c) = chars.next() {
                    if in_string {
                        result.push(c);
                        // Handle triple quotes
                        if string_char == '"' && c == '"' 
                            && chars.peek() == Some(&'"') {
                            result.push(chars.next().unwrap());
                            result.push(chars.next().unwrap());
                            in_string = false;
                        }
                    } else if c == '#' {
                        // Skip until end of line
                        while let Some(c) = chars.next() {
                            if c == '\n' {
                                result.push('\n');
                                break;
                            }
                        }
                    } else if c == '"' && chars.peek() == Some(&'"') && {
                        let mut peek = chars.clone();
                        peek.next();
                        peek.peek() == Some(&'"')
                    } {
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
        // Language-specific normalization
        let mut result = code.to_string();
        
        // Normalize spacing around operators
        result = regex::Regex::new(r"\s*([+\-*/%=<>!&|^]+)\s*")
            .unwrap()
            .replace_all(&result, " $1 ")
            .to_string();
        
        // Remove trailing semicolons (Python doesn't have them, JS normalizes)
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

#[derive(Debug, Clone)]
pub struct NormalizationResult {
    pub original: String,
    pub alpha_renamed: String,
    pub canonical: String,
    pub hash: String,
}
```

---

## 2.2 Semantic Hash Equivalence Detection

### File: `internal/cadi-core/src/deduplication.rs` (new file)

**Goal**: Detect when two chunks are semantically equivalent and create graph edges.

```rust
use crate::normalizer::{SemanticNormalizer, NormalizationResult};
use std::collections::HashMap;

pub struct DeduplicationEngine {
    hash_index: HashMap<String, Vec<String>>,  // Hash → [chunk IDs]
}

impl DeduplicationEngine {
    pub fn new() -> Self {
        Self {
            hash_index: HashMap::new(),
        }
    }
    
    /// Register a chunk by semantic hash
    /// Returns: (is_new, equivalents)
    /// - is_new: true if this is the first chunk with this hash
    /// - equivalents: list of chunk IDs with identical hash
    pub fn register_chunk(
        &mut self,
        chunk_id: &str,
        semantic_hash: &str,
    ) -> (bool, Vec<String>) {
        let entry = self.hash_index.entry(semantic_hash.to_string())
            .or_insert_with(Vec::new);
        
        let is_new = entry.is_empty();
        let equivalents = entry.clone();
        
        entry.push(chunk_id.to_string());
        
        (is_new, equivalents)
    }
    
    /// Find all chunks equivalent to the given hash
    pub fn find_equivalents(&self, semantic_hash: &str) -> Vec<String> {
        self.hash_index.get(semantic_hash)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Check semantic similarity between two code strings
    /// Returns: (is_identical, similarity_score)
    pub async fn check_similarity(
        code_a: &str,
        code_b: &str,
        language: &str,
    ) -> Result<(bool, f32)> {
        let normalizer = SemanticNormalizer::new(language)?;
        
        let result_a = normalizer.normalize(code_a)?;
        let result_b = normalizer.normalize(code_b)?;
        
        // Identical canonical form = identical semantics
        if result_a.canonical == result_b.canonical {
            return Ok((true, 1.0));
        }
        
        // Compute Levenshtein distance for similarity
        let similarity = Self::levenshtein_similarity(
            &result_a.canonical,
            &result_b.canonical,
        );
        
        Ok((false, similarity))
    }
    
    fn levenshtein_similarity(a: &str, b: &str) -> f32 {
        let dist = Self::levenshtein_distance(a, b);
        let max_len = a.len().max(b.len());
        
        if max_len == 0 {
            return 1.0;
        }
        
        1.0 - (dist as f32 / max_len as f32)
    }
    
    fn levenshtein_distance(a: &str, b: &str) -> usize {
        let mut matrix = vec![vec![0; b.len() + 1]; a.len() + 1];
        
        for i in 0..=a.len() {
            matrix[i][0] = i;
        }
        for j in 0..=b.len() {
            matrix[0][j] = j;
        }
        
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        
        for (i, &a_char) in a_chars.iter().enumerate() {
            for (j, &b_char) in b_chars.iter().enumerate() {
                let cost = if a_char == b_char { 0 } else { 1 };
                matrix[i + 1][j + 1] = (
                    (matrix[i][j + 1] + 1).min(matrix[i + 1][j] + 1)
                ).min(matrix[i][j] + cost);
            }
        }
        
        matrix[a.len()][b.len()]
    }
}
```

---

## 2.3 Import Pipeline Integration

### File: `internal/cadi-builder/src/importer.rs` (enhance)

**Goal**: Integrate semantic hashing into import pipeline for automatic deduplication.

```rust
pub struct ProjectImporter {
    dedup_engine: Arc<DeduplicationEngine>,
    search_db: Arc<VectorSearchDB>,
    graph_db: Arc<GraphDB>,  // NEW: for storing equivalence edges
}

impl ProjectImporter {
    pub async fn import_and_deduplicate(
        &self,
        project_path: &Path,
        namespace: &str,
    ) -> Result<ImportReport> {
        let chunks = self.analyze_project(project_path).await?;
        
        let mut import_report = ImportReport {
            new_chunks: Vec::new(),
            deduplicated: Vec::new(),
            equivalents_found: Vec::new(),
        };
        
        for chunk in chunks {
            // 1. Normalize and hash
            let normalizer = SemanticNormalizer::new(&chunk.language)?;
            let norm_result = normalizer.normalize(&chunk.code)?;
            
            // 2. Check for equivalents
            let (is_new, equivalents) = self.dedup_engine.register_chunk(
                &chunk.id,
                &norm_result.hash,
            );
            
            if is_new {
                // NEW chunk: Create and index
                let metadata = MetadataExtractor::extract(
                    &chunk.code,
                    &chunk.language,
                    &chunk.module_path,
                )?;
                
                let embedding = self.embedding_provider
                    .generate(&format!("{} {}", metadata.name, metadata.description))
                    .await?;
                
                // Store chunk
                let chunk_id = format!("{}:{}", namespace, chunk.id);
                self.search_db.store_chunk(
                    &chunk_id,
                    &norm_result.hash,
                    &metadata,
                    &embedding,
                ).await?;
                
                // Store in graph DB
                self.graph_db.create_chunk_node(
                    &chunk_id,
                    &metadata,
                    &norm_result.hash,
                ).await?;
                
                import_report.new_chunks.push(ChunkInfo {
                    id: chunk_id,
                    hash: norm_result.hash.clone(),
                });
            } else if !equivalents.is_empty() {
                // EQUIVALENT chunk found: Create EQUIVALENT_TO edge
                for equivalent_id in &equivalents {
                    self.graph_db.create_edge(
                        &chunk.id,
                        equivalent_id,
                        GraphEdgeType::EQUIVALENT_TO,
                        Some(json!({ "semantic_hash": norm_result.hash })),
                    ).await?;
                }
                
                import_report.equivalents_found.push((chunk.id.clone(), equivalents));
            }
        }
        
        Ok(import_report)
    }
}

pub struct ImportReport {
    pub new_chunks: Vec<ChunkInfo>,
    pub deduplicated: Vec<DeduplicatedInfo>,
    pub equivalents_found: Vec<(String, Vec<String>)>,
}

pub struct ChunkInfo {
    pub id: String,
    pub hash: String,
}
```

---

## 2.4 Semantic Equivalence Detection at Build Time

### File: `internal/cadi-builder/src/builder.rs` (enhance)

**Goal**: During build, discover when generated code can use existing semantic equivalents.

```rust
pub struct BuildPlanner {
    dedup_engine: Arc<DeduplicationEngine>,
    graph_db: Arc<GraphDB>,
}

impl BuildPlanner {
    /// When building, check if novel generated code has semantic equivalents
    pub async fn check_for_equivalents(
        &self,
        generated_code: &str,
        language: &str,
        purpose: &str,  // e.g., "comment threading"
    ) -> Result<Vec<EquivalentChunk>> {
        let normalizer = SemanticNormalizer::new(language)?;
        let norm_result = normalizer.normalize(generated_code)?;
        
        // Find all chunks with this semantic hash
        let equivalents = self.dedup_engine.find_equivalents(&norm_result.hash);
        
        if !equivalents.is_empty() {
            return Ok(equivalents
                .iter()
                .map(|id| EquivalentChunk {
                    id: id.clone(),
                    hash: norm_result.hash.clone(),
                    message: format!(
                        "Semantically equivalent to existing chunk. Consider reusing instead of generating."
                    ),
                })
                .collect());
        }
        
        Ok(vec![])
    }
}

pub struct EquivalentChunk {
    pub id: String,
    pub hash: String,
    pub message: String,
}
```

---

## 2.5 Testing & Validation

### File: `tests/semantic_deduplication_test.rs`

```rust
#[test]
fn test_identical_semantics_same_hash() {
    let code_a = r#"
        function addNumbers(x, y) {
            return x + y;
        }
    "#;
    
    let code_b = r#"
        function   add  (  a  ,  b  )   {
            return    a    +    b  ;
        }
    "#;
    
    let normalizer = SemanticNormalizer::new("typescript").unwrap();
    let result_a = normalizer.normalize(code_a).unwrap();
    let result_b = normalizer.normalize(code_b).unwrap();
    
    assert_eq!(result_a.hash, result_b.hash);
}

#[test]
fn test_deduplication_engine() {
    let mut engine = DeduplicationEngine::new();
    
    let hash = "semantic:abc123";
    
    let (is_new_1, equiv_1) = engine.register_chunk("chunk1", hash);
    assert!(is_new_1);
    assert!(equiv_1.is_empty());
    
    let (is_new_2, equiv_2) = engine.register_chunk("chunk2", hash);
    assert!(!is_new_2);
    assert_eq!(equiv_2, vec!["chunk1"]);
    
    let found = engine.find_equivalents(hash);
    assert_eq!(found, vec!["chunk1", "chunk2"]);
}

#[tokio::test]
async fn test_semantic_similarity_threshold() {
    let code_a = "function add(x, y) { return x + y; }";
    let code_b = "function add(x, y) { return x + y + 1; }";
    
    let (is_identical, similarity) = DeduplicationEngine::check_similarity(
        code_a,
        code_b,
        "typescript"
    ).await.unwrap();
    
    assert!(!is_identical);
    assert!(similarity > 0.8);  // Very similar but not identical
}
```

---

## Success Criteria

✅ Identical code across projects produces identical semantic hash  
✅ Deduplication engine correctly identifies equivalents  
✅ Import pipeline automatically creates EQUIVALENT_TO edges  
✅ <50ms hash computation for typical function (~500 lines)  
✅ Supports TypeScript, Python, Rust, Go, Java  
✅ Comments, whitespace, formatting don't affect hash  
✅ Variable name changes don't affect hash (alpha-renaming works)  
✅ Demo: Blog app finds todo app's JWT auth via semantic hash  

---

## Next: Stage 3 (Graph-Based Linking)
Once Stage 2 is complete, build the graph infrastructure to automatically link chunks via semantic equivalence and dependency edges, enabling full forward acceleration.
