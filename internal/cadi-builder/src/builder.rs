use cadi_core::error::CadiError;
use cadi_core::deduplication::DeduplicationEngine;
use cadi_core::normalizer::SemanticNormalizer;
use cadi_registry::graph::GraphDB;
use crate::dependency_resolver::DependencyResolver;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::fs;

pub struct BuildPlanner {
    dedup_engine: Arc<Mutex<DeduplicationEngine>>,
}

impl BuildPlanner {
    pub fn new(dedup_engine: Arc<Mutex<DeduplicationEngine>>) -> Self {
        Self { dedup_engine }
    }

    /// When building, check if generated code has semantic equivalents
    pub async fn check_for_equivalents(
        &self,
        generated_code: &str,
        language: &str,
        _purpose: &str,
    ) -> Result<Vec<EquivalentChunk>, CadiError> {
        let normalizer = SemanticNormalizer::new(language)?;
        let norm = normalizer.normalize(generated_code)?;
        let list = {
            let engine = self.dedup_engine.lock().unwrap();
            engine.find_equivalents(&norm.hash)
        };

        Ok(list
            .into_iter()
            .map(|id| EquivalentChunk {
                id,
                hash: norm.hash.clone(),
                message: "Semantically equivalent to existing chunk. Consider reusing instead of generating."
                    .to_string(),
            })
            .collect())
    }
}

pub struct EquivalentChunk {
    pub id: String,
    pub hash: String,
    pub message: String,
}

pub struct BuildScaffolder {
    graph_db: Arc<GraphDB>,
    dependency_resolver: Arc<DependencyResolver>,
}

impl BuildScaffolder {
    /// Generate import/linking code for a set of chunks
    pub async fn scaffold_imports(
        &self,
        chunks: &[String],
        language: &str,
        output_path: &Path,
    ) -> Result<(), CadiError> {
        // Resolve all dependencies
        let mut all_deps: HashSet<String> = HashSet::new();
        for chunk_id in chunks {
            let resolved = self
                .dependency_resolver
                .resolve_all_dependencies(chunk_id)
                .await?;
            all_deps.extend(resolved.all_dependencies);
        }

        // Get chunk info
        let mut imports = Vec::new();
        for chunk_id in all_deps.iter() {
            if let Some(node) = self.graph_db.get_chunk_node(chunk_id).await? {
                if let Some(interface) = &node.interface {
                    imports.push(ImportStatement {
                        chunk_id: chunk_id.to_string(),
                        name: interface.name.clone(),
                        language: node.language.clone(),
                    });
                }
            }
        }

        // Generate import code
        let import_code = self.generate_imports(&imports, language)?;

        // Write to file
        let import_file =
            output_path.join(format!("generated_imports.{}", language_to_extension(language)));
        fs::write(&import_file, import_code).await?;

        Ok(())
    }

    fn generate_imports(
        &self,
        imports: &[ImportStatement],
        language: &str,
    ) -> Result<String, CadiError> {
        match language {
            "typescript" => {
                let mut code = String::new();
                for import in imports {
                    code.push_str(&format!(
                        "import {{ {} }} from './chunks/{}';\n",
                        import.name, import.chunk_id
                    ));
                }
                Ok(code)
            }
            "python" => {
                let mut code = String::new();
                for import in imports {
                    code.push_str(&format!(
                        "from .chunks.{} import {}\n",
                        import.chunk_id, import.name
                    ));
                }
                Ok(code)
            }
            "rust" => {
                let mut code = String::new();
                for import in imports {
                    code.push_str(&format!(
                        "use chunks::{}::{};\n",
                        import.chunk_id, import.name
                    ));
                }
                Ok(code)
            }
            _ => Err(CadiError::RehydrationError(format!("Unsupported language: {}", language))),
        }
    }
}

pub struct ImportStatement {
    pub chunk_id: String,
    pub name: String,
    pub language: String,
}

fn language_to_extension(language: &str) -> &str {
    match language {
        "typescript" => "ts",
        "python" => "py",
        "rust" => "rs",
        _ => "txt",
    }
}