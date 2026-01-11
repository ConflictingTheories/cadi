use crate::error::Result;
use crate::parser::CodeAst;
use regex::Regex;
use std::path::Path;

/// Metadata extractor for chunks
pub struct MetadataExtractor;

#[derive(Debug, Clone)]
pub struct ExtractedMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub concepts: Vec<String>,
    pub license: Option<String>,
    pub authors: Vec<String>,
    pub tags: Vec<String>,
    pub detected_frameworks: Vec<String>,
}

impl MetadataExtractor {
    /// Extract metadata from content and file path
    pub fn extract(content: &str, file_path: Option<&Path>) -> Result<ExtractedMetadata> {
        let mut metadata = ExtractedMetadata {
            title: Self::extract_title(content, file_path),
            description: Self::extract_description(content),
            keywords: Self::extract_keywords(content),
            concepts: Self::extract_concepts(content),
            license: Self::detect_license(content),
            authors: Self::extract_authors(content),
            tags: Self::extract_tags(content),
            detected_frameworks: Self::detect_frameworks(content),
        };

        // Enhance title from filename if not extracted
        if metadata.title.is_none() {
            if let Some(path) = file_path {
                if let Some(name) = path.file_stem() {
                    metadata.title = Some(name.to_string_lossy().to_string());
                }
            }
        }

        Ok(metadata)
    }

    fn extract_title(content: &str, file_path: Option<&Path>) -> Option<String> {
        // Try to find # Heading in markdown
        if let Some(line) = content.lines().find(|l| l.starts_with("# ")) {
            return Some(line.trim_start_matches("# ").trim().to_string());
        }

        // Try package.json name
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(name) = value.get("name").and_then(|v| v.as_str()) {
                return Some(name.to_string());
            }
        }

        // Try Cargo.toml name
        if let Ok(table) = toml::from_str::<toml::Table>(content) {
            if let Some(name) = table.get("package")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
            {
                return Some(name.to_string());
            }
        }

        // Fall back to filename
        file_path.and_then(|p| p.file_stem()?.to_str().map(|s| s.to_string()))
    }

    fn extract_description(content: &str) -> Option<String> {
        // Try to find markdown description after heading
        let mut lines = content.lines();
        while let Some(line) = lines.next() {
            if line.starts_with("# ") {
                // Skip the heading, get next non-empty line
                while let Some(desc) = lines.next() {
                    if !desc.trim().is_empty() && !desc.starts_with("#") {
                        return Some(desc.trim().to_string());
                    }
                }
            }
        }

        // Try package.json description
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(desc) = value.get("description").and_then(|v| v.as_str()) {
                return Some(desc.to_string());
            }
        }

        None
    }

    fn extract_keywords(content: &str) -> Vec<String> {
        let mut keywords = Vec::new();

        // From JSON keywords
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(kw) = value.get("keywords").and_then(|v| v.as_array()) {
                for item in kw {
                    if let Some(s) = item.as_str() {
                        keywords.push(s.to_string());
                    }
                }
            }
        }

        keywords
    }

    fn extract_concepts(content: &str) -> Vec<String> {
        let mut concepts = Vec::new();

        let concept_patterns = [
            ("database", "db|postgres|mysql|mongodb|redis"),
            ("api", "api|rest|graphql|rpc"),
            ("ui", "ui|component|react|vue|angular"),
            ("testing", "test|spec|jest|mocha|unittest"),
            ("async", "async|await|promise|future"),
            ("concurrency", "thread|concurrent|parallel|mutex"),
            ("cli", "cli|command|argv|argument"),
            ("storage", "storage|cache|file|s3"),
        ];

        for (concept, pattern) in &concept_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(content) {
                    concepts.push(concept.to_string());
                }
            }
        }

        concepts
    }

    fn detect_license(content: &str) -> Option<String> {
        // Common license patterns
        let licenses = [
            ("MIT", "MIT"),
            ("Apache", "Apache-2.0"),
            ("GPL", "GPL-3.0"),
            ("BSD", "BSD-2-Clause"),
            ("ISC", "ISC"),
        ];

        for (pattern, license) in &licenses {
            if content.contains(pattern) {
                return Some(license.to_string());
            }
        }

        // Try package.json license field
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(license) = value.get("license").and_then(|v| v.as_str()) {
                return Some(license.to_string());
            }
        }

        None
    }

    fn extract_authors(content: &str) -> Vec<String> {
        let mut authors = Vec::new();

        // Try package.json author field
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(author) = value.get("author").and_then(|v| v.as_str()) {
                authors.push(author.to_string());
            }
            if let Some(contributors) = value.get("contributors").and_then(|v| v.as_array()) {
                for item in contributors {
                    if let Some(s) = item.as_str() {
                        authors.push(s.to_string());
                    }
                }
            }
        }

        // Try Cargo.toml authors field
        if let Ok(table) = toml::from_str::<toml::Table>(content) {
            if let Some(authors_arr) = table.get("package")
                .and_then(|p| p.get("authors"))
                .and_then(|a| a.as_array())
            {
                for item in authors_arr {
                    if let Some(s) = item.as_str() {
                        authors.push(s.to_string());
                    }
                }
            }
        }

        authors
    }

    fn extract_tags(content: &str) -> Vec<String> {
        let mut tags = Vec::new();

        // Extract tags from comments
        let tag_pattern = Regex::new(r"@tags?\s*:\s*([^\n]+)").ok();
        if let Some(re) = tag_pattern {
            for cap in re.captures_iter(content) {
                if let Some(tag_str) = cap.get(1) {
                    let parts: Vec<&str> = tag_str.as_str().split(',').collect();
                    for part in parts {
                        tags.push(part.trim().to_string());
                    }
                }
            }
        }

        tags
    }

    fn detect_frameworks(content: &str) -> Vec<String> {
        let mut frameworks = Vec::new();

        let framework_patterns = [
            ("react", r"react|React"),
            ("vue", r"vue|Vue"),
            ("angular", r"angular|Angular"),
            ("svelte", r"svelte|Svelte"),
            ("next.js", r"next|Next"),
            ("express", r"express|Express"),
            ("fastapi", r"fastapi|FastAPI"),
            ("django", r"django|Django"),
            ("rails", r"rails|Rails"),
            ("spring", r"spring|Spring"),
            ("actix", r"actix|Actix"),
            ("axum", r"axum|Axum"),
        ];

        for (framework, pattern) in &framework_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(content) {
                    frameworks.push(framework.to_string());
                }
            }
        }

        frameworks
    }
}

/// Extract API surface from code AST
pub fn extract_api_surface(ast: &CodeAst) -> ApiSurface {
    ApiSurface {
        functions: ast.functions.clone(),
        structs: ast.structs.clone(),
        traits: ast.traits.clone(),
        classes: ast.classes.clone(),
        interfaces: ast.interfaces.clone(),
        exports: extract_public_api(ast),
    }
}

#[derive(Debug, Clone)]
pub struct ApiSurface {
    pub functions: Vec<String>,
    pub structs: Vec<String>,
    pub traits: Vec<String>,
    pub classes: Vec<String>,
    pub interfaces: Vec<String>,
    pub exports: Vec<String>,
}

fn extract_public_api(ast: &CodeAst) -> Vec<String> {
    // Combine all public API elements
    let mut api = Vec::new();
    api.extend(ast.functions.clone());
    api.extend(ast.structs.clone());
    api.extend(ast.classes.clone());
    api.extend(ast.interfaces.clone());
    api.extend(ast.traits.clone());
    api
}
