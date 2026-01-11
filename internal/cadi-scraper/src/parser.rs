use crate::error::{Error, Result};
use crate::types::ScraperConfig;
use pulldown_cmark::{Parser, html};
use std::path::Path;

/// Multi-format parser for various file types
#[allow(dead_code)]
pub struct ContentParser {
    config: ScraperConfig,
}

/// Parsed content with metadata
#[derive(Debug, Clone)]
pub struct ParsedContent {
    /// Detected language/format
    pub language: Option<String>,

    /// Parsed content as text
    pub text: String,

    /// Structured data if applicable (JSON, YAML)
    pub structured: Option<serde_json::Value>,

    /// HTML if applicable
    pub html: Option<String>,

    /// Detected encoding
    pub encoding: String,

    /// Content metadata
    pub metadata: ContentMetadata,
}

#[derive(Debug, Clone)]
pub struct ContentMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub line_count: usize,
    pub byte_size: usize,
}

impl ContentParser {
    pub fn new(config: ScraperConfig) -> Self {
        Self { config }
    }

    /// Parse content based on file extension or MIME type
    pub fn parse(&self, content: &[u8], file_path: Option<&Path>) -> Result<ParsedContent> {
        let encoding = detect_encoding(content);
        let text = String::from_utf8_lossy(content).to_string();

        let language = file_path.and_then(|p| detect_language(p));

        // Try to parse as JSON
        if language.as_deref() == Some("json") {
            if let Ok(structured) = serde_json::from_slice(content) {
                let metadata = extract_text_metadata(&text);
                return Ok(ParsedContent {
                    language: Some("json".to_string()),
                    text,
                    structured: Some(structured),
                    html: None,
                    encoding,
                    metadata,
                });
            }
        }

        // Try to parse as YAML
        if language.as_deref() == Some("yaml") || language.as_deref() == Some("yml") {
            if let Ok(structured) = serde_yaml::from_str(&text) {
                let metadata = extract_text_metadata(&text);
                return Ok(ParsedContent {
                    language: Some("yaml".to_string()),
                    text,
                    structured: Some(structured),
                    html: None,
                    encoding,
                    metadata,
                });
            }
        }

        // Parse Markdown to HTML
        if language.as_deref() == Some("md") || language.as_deref() == Some("markdown") {
            let parser = Parser::new(&text);
            let mut html = String::new();
            html::push_html(&mut html, parser);
            let metadata = extract_text_metadata(&text);

            return Ok(ParsedContent {
                language: Some("markdown".to_string()),
                text,
                structured: None,
                html: Some(html),
                encoding,
                metadata,
            });
        }

        // Default text parsing
        let metadata = extract_text_metadata(&text);
        Ok(ParsedContent {
            language,
            text,
            structured: None,
            html: None,
            encoding,
            metadata,
        })
    }

    /// Parse source code and extract AST information
    pub fn parse_code(&self, content: &str, language: &str) -> Result<CodeAst> {
        match language {
            "rust" => self.parse_rust_code(content),
            "typescript" | "ts" => self.parse_typescript_code(content),
            "javascript" | "js" => self.parse_javascript_code(content),
            "python" => self.parse_python_code(content),
            _ => Err(Error::UnsupportedFormat(format!(
                "Code parsing not supported for {}",
                language
            ))),
        }
    }

    fn parse_rust_code(&self, content: &str) -> Result<CodeAst> {
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut traits = Vec::new();
        let mut imports = Vec::new();

        // Simple regex-based parsing for MVP
        if let Ok(fn_regex) = regex::Regex::new(r"(?m)^(?:pub\s+)?(?:async\s+)?fn\s+(\w+)") {
            for cap in fn_regex.captures_iter(content) {
                if let Some(name) = cap.get(1) {
                    functions.push(name.as_str().to_string());
                }
            }
        }

        if let Ok(struct_regex) = regex::Regex::new(r"(?m)^(?:pub\s+)?struct\s+(\w+)") {
            for cap in struct_regex.captures_iter(content) {
                if let Some(name) = cap.get(1) {
                    structs.push(name.as_str().to_string());
                }
            }
        }

        if let Ok(trait_regex) = regex::Regex::new(r"(?m)^(?:pub\s+)?trait\s+(\w+)") {
            for cap in trait_regex.captures_iter(content) {
                if let Some(name) = cap.get(1) {
                    traits.push(name.as_str().to_string());
                }
            }
        }

        if let Ok(use_regex) = regex::Regex::new(r"(?m)^use\s+([\w:]+)") {
            for cap in use_regex.captures_iter(content) {
                if let Some(import) = cap.get(1) {
                    imports.push(import.as_str().to_string());
                }
            }
        }

        Ok(CodeAst {
            language: "rust".to_string(),
            functions,
            structs,
            traits,
            enums: Vec::new(),
            classes: Vec::new(),
            interfaces: Vec::new(),
            imports,
        })
    }


    fn parse_typescript_code(&self, content: &str) -> Result<CodeAst> {
        let mut functions = Vec::new();
        let mut classes = Vec::new();
        let mut interfaces = Vec::new();
        let mut imports = Vec::new();

        let fn_regex = regex::Regex::new(r"(?m)(?:export\s+)?(?:async\s+)?function\s+(\w+)|(?:export\s+)?const\s+(\w+)\s*=")?;
        for cap in fn_regex.captures_iter(content) {
            if let Some(name) = cap.get(1).or_else(|| cap.get(2)) {
                functions.push(name.as_str().to_string());
            }
        }

        let class_regex = regex::Regex::new(r"(?m)(?:export\s+)?class\s+(\w+)")?;
        for cap in class_regex.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                classes.push(name.as_str().to_string());
            }
        }

        let interface_regex = regex::Regex::new(r"(?m)(?:export\s+)?interface\s+(\w+)")?;
        for cap in interface_regex.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                interfaces.push(name.as_str().to_string());
            }
        }

        let import_regex = regex::Regex::new(r#"(?m)^import\s+(?:\{[^}]*\}|[\w*]+)\s+from\s+['"]([^'"]+)['"]"#)?;
        for cap in import_regex.captures_iter(content) {
            if let Some(module) = cap.get(1) {
                imports.push(module.as_str().to_string());
            }
        }

        Ok(CodeAst {
            language: "typescript".to_string(),
            functions,
            structs: Vec::new(),
            traits: Vec::new(),
            enums: Vec::new(),
            classes,
            interfaces,
            imports,
        })
    }

    fn parse_javascript_code(&self, _content: &str) -> Result<CodeAst> {
        // Similar to TypeScript but without type info
        Ok(CodeAst {
            language: "javascript".to_string(),
            functions: Vec::new(),
            structs: Vec::new(),
            traits: Vec::new(),
            enums: Vec::new(),
            classes: Vec::new(),
            interfaces: Vec::new(),
            imports: Vec::new(),
        })
    }

    fn parse_python_code(&self, content: &str) -> Result<CodeAst> {
        let mut functions = Vec::new();
        let mut classes = Vec::new();
        let mut imports = Vec::new();

        let fn_regex = regex::Regex::new(r"(?m)^def\s+(\w+)")?;
        for cap in fn_regex.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                functions.push(name.as_str().to_string());
            }
        }

        let class_regex = regex::Regex::new(r"(?m)^class\s+(\w+)")?;
        for cap in class_regex.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                classes.push(name.as_str().to_string());
            }
        }

        let import_regex = regex::Regex::new(r"(?m)^(?:from\s+[\w.]+\s+)?import\s+([\w., ]+)")?;
        for cap in import_regex.captures_iter(content) {
            if let Some(module) = cap.get(1) {
                imports.push(module.as_str().to_string());
            }
        }

        Ok(CodeAst {
            language: "python".to_string(),
            functions,
            structs: Vec::new(),
            traits: Vec::new(),
            enums: Vec::new(),
            classes,
            interfaces: Vec::new(),
            imports,
        })
    }
}

/// Abstract Syntax Tree representation
#[derive(Debug, Clone)]
pub struct CodeAst {
    pub language: String,
    pub functions: Vec<String>,
    pub structs: Vec<String>,
    pub traits: Vec<String>,
    pub enums: Vec<String>,
    pub classes: Vec<String>,
    pub interfaces: Vec<String>,
    pub imports: Vec<String>,
}

fn detect_language(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "rs" => Some("rust".to_string()),
        "ts" => Some("typescript".to_string()),
        "tsx" => Some("typescript".to_string()),
        "js" => Some("javascript".to_string()),
        "jsx" => Some("javascript".to_string()),
        "py" => Some("python".to_string()),
        "go" => Some("go".to_string()),
        "c" => Some("c".to_string()),
        "h" => Some("c".to_string()),
        "cpp" | "cc" | "cxx" => Some("cpp".to_string()),
        "java" => Some("java".to_string()),
        "md" => Some("markdown".to_string()),
        "json" => Some("json".to_string()),
        "yaml" | "yml" => Some("yaml".to_string()),
        "toml" => Some("toml".to_string()),
        "xml" => Some("xml".to_string()),
        "html" | "htm" => Some("html".to_string()),
        "css" => Some("css".to_string()),
        _ => None,
    }
}

fn detect_encoding(content: &[u8]) -> String {
    // Simple UTF-8 detection for MVP
    if content.is_empty() || String::from_utf8(content.to_vec()).is_ok() {
        "utf-8".to_string()
    } else {
        "unknown".to_string()
    }
}

fn extract_text_metadata(text: &str) -> ContentMetadata {
    let line_count = text.lines().count();
    let byte_size = text.len();
    let title = text.lines().next().map(|l| l.trim().to_string());

    ContentMetadata {
        title,
        description: None,
        keywords: Vec::new(),
        line_count,
        byte_size,
    }
}
