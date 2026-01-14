use cadi_core::error::CadiError;
use cadi_registry::graph::GraphDB;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

pub struct DependencyResolver {
    graph_db: Arc<GraphDB>,
}

impl DependencyResolver {
    pub fn new(graph_db: Arc<GraphDB>) -> Self {
        Self { graph_db }
    }

    /// Resolve all dependencies for a chunk (BFS traversal)
    pub async fn resolve_all_dependencies(
        &self,
        chunk_id: &str,
    ) -> Result<ResolutionResult, CadiError> {
        let mut resolved = HashSet::new();
        let mut queue = VecDeque::new();
        let mut dependency_graph = HashMap::new();

        queue.push_back(chunk_id.to_string());
        resolved.insert(chunk_id.to_string());

        while let Some(current) = queue.pop_front() {
            // Get direct dependencies
            let deps = self.graph_db.get_dependencies(&current).await?;

            for dep in deps {
                if !resolved.contains(&dep.dependency_id) {
                    resolved.insert(dep.dependency_id.clone());
                    queue.push_back(dep.dependency_id.clone());
                }

                // Record edge
                dependency_graph
                    .entry(current.clone())
                    .or_insert_with(Vec::new)
                    .push((dep.dependency_id.clone(), dep.edge_type));
            }
        }

        Ok(ResolutionResult {
            root: chunk_id.to_string(),
            all_dependencies: resolved,
            dependency_graph,
        })
    }

    /// Check if composition is valid (no cycles, compatible versions)
    pub async fn validate_composition(
        &self,
        chunks: &[String],
    ) -> Result<CompositionValidation, CadiError> {
        let mut issues = Vec::new();

        // Check for cycles
        for chunk in chunks {
            for other in chunks {
                if chunk != other {
                    if let Some(_path) = self.graph_db.find_path(chunk, other).await? {
                        if let Some(_reverse) = self.graph_db.find_path(other, chunk).await? {
                            issues.push(CompositionIssue {
                                severity: "error".to_string(),
                                message: format!("Circular dependency: {} -> {}", chunk, other),
                            });
                        }
                    }
                }
            }
        }

        Ok(CompositionValidation {
            is_valid: issues.is_empty(),
            issues,
        })
    }

    /// Get interface compatibility between two chunks
    pub async fn check_interface_compatibility(
        &self,
        provider_id: &str,
        consumer_id: &str,
    ) -> Result<InterfaceCompatibility, CadiError> {
        // Query interfaces
        let provider = self.graph_db.get_chunk_node(provider_id).await?;
        let consumer = self.graph_db.get_chunk_node(consumer_id).await?;

        let provider_interface = provider.unwrap().interface.ok_or_else(|| CadiError::RehydrationError("Provider has no interface".to_string()))?;
        let consumer_interface = consumer.unwrap().interface.ok_or_else(|| CadiError::RehydrationError("Consumer has no interface".to_string()))?;

        // Check type compatibility
        let mut incompatibilities = Vec::new();

        // Compare output of provider with input of consumer
        if provider_interface.output.name != consumer_interface.inputs[0].type_sig.name {
            incompatibilities.push(format!(
                "Type mismatch: {} output vs {} input",
                provider_interface.output.name, consumer_interface.inputs[0].type_sig.name
            ));
        }

        Ok(InterfaceCompatibility {
            is_compatible: incompatibilities.is_empty(),
            incompatibilities,
        })
    }
}

#[derive(Debug)]
pub struct ResolutionResult {
    pub root: String,
    pub all_dependencies: HashSet<String>,
    pub dependency_graph: HashMap<String, Vec<(String, String)>>,
}

#[derive(Debug)]
pub struct CompositionValidation {
    pub is_valid: bool,
    pub issues: Vec<CompositionIssue>,
}

#[derive(Debug)]
pub struct CompositionIssue {
    pub severity: String, // "error", "warning"
    pub message: String,
}

#[derive(Debug)]
pub struct InterfaceCompatibility {
    pub is_compatible: bool,
    pub incompatibilities: Vec<String>,
}