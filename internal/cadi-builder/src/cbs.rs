//! CADI Build Specification (CBS) Compiler
//!
//! This module handles the compilation of high-level Build Specifications (CBS)
//! into executable CADI Manifests. It resolves semantic queries, handles
//! component reuse, and sets up generation tasks.

use crate::build_spec::{BuildSpec, ComponentSpec, GenerateComponent, ReuseComponent, SearchComponent};
use cadi_core::{Manifest, GraphNode, Representation, BuildTarget, TargetNode, GraphEdge};
use cadi_core::{CadiResult, CadiError};
use cadi_registry::search::SearchEngine;
use std::sync::Arc;

/// Compiler for CADI Build Specifications (CBS)
pub struct CbsCompiler {
    search_engine: Arc<SearchEngine>,
}

impl CbsCompiler {
    /// Create a new CBS compiler with a search engine for resolving queries
    pub fn new(search_engine: Arc<SearchEngine>) -> Self {
        Self { search_engine }
    }

    /// Compile a CBS into a CADI Manifest
    pub async fn compile(&self, spec: BuildSpec) -> CadiResult<Manifest> {
        let mut manifest = Manifest::new(
            format!("{}-manifest", spec.project.name),
            spec.project.name.clone(),
        );

        manifest.application.description = spec.project.description.clone();
        manifest.application.version = spec.project.version.clone();

        // Process components
        for component in &spec.components {
            match component {
                ComponentSpec::Reuse(reuse) => {
                    self.add_reuse_node(&mut manifest, reuse)?;
                }
                ComponentSpec::Generate(gen) => {
                    self.add_generate_node(&mut manifest, gen)?;
                }
                ComponentSpec::Search(search) => {
                    self.resolve_and_add_search_node(&mut manifest, search).await?;
                }
            }
        }

        // Process targets
        for target in &spec.targets {
            let build_target = BuildTarget {
                name: target.name.clone(),
                platform: target.platform.clone().unwrap_or_else(|| "any".to_string()),
                nodes: target.components.iter().map(|id| TargetNode {
                    id: id.clone(),
                    require: None,
                    prefer: None,
                }).collect(),
                bundle: None,
                deploy: None,
                trust_requirements: None,
            };
            manifest.add_target(build_target);
        }

        Ok(manifest)
    }

    fn add_reuse_node(&self, manifest: &mut Manifest, reuse: &ReuseComponent) -> CadiResult<()> {
        let node = GraphNode {
            id: reuse.id.clone(),
            source_cadi: Some(reuse.source.clone()),
            ir_cadi: None,
            blob_cadi: None,
            container_cadi: None,
            representations: vec![
                Representation {
                    form: "source".to_string(),
                    language: None, // Could infer from project info or lookup
                    format: None,
                    architecture: None,
                    chunk: reuse.source.clone(),
                }
            ],
            selection_strategy: Some("prefer_source".to_string()),
            materialization: None,
        };
        manifest.add_node(node);
        Ok(())
    }

    fn add_generate_node(&self, manifest: &mut Manifest, gen: &GenerateComponent) -> CadiResult<()> {
        // For generation, we create a placeholder node that the build engine will need to fill
        // The build engine will see source_cadi is None and trigger generation logic
        let node = GraphNode {
            id: gen.id.clone(),
            source_cadi: None, // Indicates generation needed
            ir_cadi: None,
            blob_cadi: None,
            container_cadi: None,
            representations: vec![],
            selection_strategy: None,
            materialization: None,
        };
        manifest.add_node(node);

        // Add dependencies as edges
        if let Some(deps) = &gen.depends_on {
            for dep in deps {
                let edge = GraphEdge {
                    from: gen.id.clone(),
                    to: dep.clone(),
                    interface: None,
                    relation: "depends_on".to_string(),
                };
                manifest.add_edge(edge);
            }
        }
        
        Ok(())
    }

    async fn resolve_and_add_search_node(&self, manifest: &mut Manifest, search: &SearchComponent) -> CadiResult<()> {
        // Perform search using search_sync since we have an Arc<SearchEngine>
        // We use a limit of 1 to get the best match
        let results = self.search_engine.search_sync(&search.query, 1);
        
        if let Some(best_match) = results.first() {
             let node = GraphNode {
                id: search.id.clone(),
                source_cadi: Some(best_match.id.clone()),
                ir_cadi: None,
                blob_cadi: None,
                container_cadi: None,
                representations: vec![
                    Representation {
                        form: "source".to_string(),
                        language: search.language.clone(),
                        format: None,
                        architecture: None,
                        chunk: best_match.id.clone(),
                    }
                ],
                selection_strategy: Some("best_match".to_string()),
                materialization: None,
            };
            manifest.add_node(node);
        } else {
            return Err(CadiError::DependencyResolution(format!("No results found for query: {}", search.query)));
        }

        Ok(())
    }
}