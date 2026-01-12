//! Java language atomizer for CADI

use async_trait::async_trait;
use cadi_core::{AtomicChunk, atomizer::ResolvedImport};
use cadi_extensions::{AtomizerExtension, Extension, ExtensionContext, ExtensionMetadata, ExtensionType, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono;
use semver;
use blake3;

/// Java atomizer extension
pub struct JavaAtomizer {
    metadata: ExtensionMetadata,
    config: JavaConfig,
    import_regex: Regex,
    class_regex: Regex,
    method_regex: Regex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JavaConfig {
    java_version: String,
}

impl JavaAtomizer {
    /// Create a new Java atomizer
    pub fn new() -> Self {
        Self {
            metadata: ExtensionMetadata {
                id: cadi_extensions::ExtensionId(Uuid::new_v4()),
                name: "cadi-atomizer-java".into(),
                version: "1.0.0".into(),
                description: "Atomizer for Java programming language".into(),
                author: "CADI Team".into(),
                homepage: Some("https://cadi.dev".into()),
                repository: Some("https://github.com/ConflictingTheories/cadi".into()),
                license: "MIT OR Apache-2.0".into(),
                extension_type: ExtensionType::Atomizer,
            },
            config: JavaConfig {
                java_version: "11".into(),
            },
            import_regex: Regex::new(r"^import\s+([a-zA-Z_][a-zA-Z0-9_.]*);").unwrap(),
            class_regex: Regex::new(r"(?s)class\s+(\w+).*?\{(.*)\}").unwrap(),
            method_regex: Regex::new(r"(?m)^\s*(?:public|private|protected)?\s*(?:static)?\s*(?:final)?\s*[\w\[\]<>]+(?:\s+\w+)?\s*\([^)]*\)\s*(?:throws\s+\w+(?:\s*,\s*\w+)*)?\s*\{([^}]*)\}").unwrap(),
        }
    }
}

#[async_trait]
impl Extension for JavaAtomizer {
    fn metadata(&self) -> ExtensionMetadata {
        self.metadata.clone()
    }

    async fn initialize(&mut self, context: &ExtensionContext) -> Result<()> {
        // Load configuration
        if let Some(config) = context.config.get("java_version") {
            if let Some(version) = config.as_str() {
                self.config.java_version = version.to_string();
            }
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl AtomizerExtension for JavaAtomizer {
    fn language(&self) -> &str {
        "java"
    }

    async fn extract_atoms(&self, source: &str) -> Result<Vec<AtomicChunk>> {
        let mut atoms = Vec::new();

        // Extract class-level atoms
        for capture in self.class_regex.captures_iter(source) {
            let class_name = capture.get(1).unwrap().as_str();
            let class_body = capture.get(2).unwrap().as_str();

            // Create class atom
            let class_atom = AtomicChunk {
                chunk_id: format!("java:class:{}", class_name),
                aliases: vec![],
                name: class_name.to_string(),
                description: Some(format!("Java class {}", class_name)),
                language: "java".into(),
                granularity: cadi_core::atomic::ChunkGranularity::Type,
                categories: vec![cadi_core::atomic::ChunkCategory::Logic],
                tags: vec!["class".into()],
                concepts: vec![],
                provides: vec![],
                requires: vec![],
                platform: Default::default(),
                composition: Default::default(),
                metrics: Default::default(),
                sources: vec![],
                content_hash: blake3::hash(class_body.as_bytes()).to_hex().to_string(),
                size: class_body.len(),
                license: "MIT".into(),
                created_at: Some(chrono::Utc::now().to_rfc3339()),
                version: Some("1.0.0".into()),
            };
            atoms.push(class_atom);

            // Extract method-level atoms from class
            for method_capture in self.method_regex.captures_iter(class_body) {
                let method_body = method_capture.get(1).unwrap().as_str();

                let method_atom = AtomicChunk {
                    chunk_id: format!("java:method:{}.{}", class_name, atoms.len()),
                    aliases: vec![],
                    name: format!("{}.method{}", class_name, atoms.len()),
                    description: Some(format!("Method in class {}", class_name)),
                    language: "java".into(),
                    granularity: cadi_core::atomic::ChunkGranularity::Function,
                    categories: vec![cadi_core::atomic::ChunkCategory::Logic],
                    tags: vec!["method".into()],
                    concepts: vec![],
                    provides: vec![],
                    requires: vec![],
                    platform: Default::default(),
                    composition: cadi_core::atomic::ChunkComposition {
                        composed_of: vec![cadi_core::atomic::ChunkReference {
                            chunk_id: format!("java:class:{}", class_name),
                            alias: None,
                            required: true,
                            imports: vec![],
                        }],
                        composed_by: vec![],
                        is_atomic: true,
                        composition_strategy: None,
                    },
                    metrics: Default::default(),
                    sources: vec![],
                    content_hash: blake3::hash(method_body.as_bytes()).to_hex().to_string(),
                    size: method_body.len(),
                    license: "MIT".into(),
                    created_at: Some(chrono::Utc::now().to_rfc3339()),
                    version: Some("1.0.0".into()),
                };
                atoms.push(method_atom);
            }
        }

        Ok(atoms)
    }

    async fn resolve_imports(&self, source: &str) -> Result<Vec<ResolvedImport>> {
        let mut imports = Vec::new();

        for line in source.lines() {
            if let Some(capture) = self.import_regex.captures(line) {
                let import_path = capture.get(1).unwrap().as_str();

                let resolved = ResolvedImport {
                    source_path: import_path.to_string(),
                    symbols: vec![cadi_core::atomizer::ImportedSymbol {
                        name: import_path.split('.').last().unwrap_or(import_path).to_string(),
                        alias: None,
                        chunk_id: format!("java:import:{}", import_path),
                        chunk_hash: blake3::hash(import_path.as_bytes()).to_hex().to_string(),
                        symbol_type: Some("class".into()),
                    }],
                    line: 0, // We don't track line numbers in this simple example
                };

                imports.push(resolved);
            }
        }

        Ok(imports)
    }
}

/// Export the extension constructor for dynamic loading
#[no_mangle]
pub extern "C" fn cadi_extension_create() -> *mut dyn Extension {
    Box::into_raw(Box::new(JavaAtomizer::new()))
}