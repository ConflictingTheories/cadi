//! Build planning for CADI

use cadi_core::{CadiError, CadiResult, Manifest};
use std::collections::{HashMap, HashSet};

/// A build plan
#[derive(Debug)]
pub struct BuildPlan {
    /// Steps to execute in order
    pub steps: Vec<BuildStep>,
    /// Estimated total time in ms
    pub estimated_time_ms: u64,
}

/// A single build step
#[derive(Debug)]
pub struct BuildStep {
    /// Step name
    pub name: String,
    /// Chunk ID (if known)
    pub chunk_id: Option<String>,
    /// Transformation to apply
    pub transform: super::TransformType,
    /// Input chunk IDs
    pub inputs: Vec<super::TransformInput>,
    /// Dependencies (step names that must complete first)
    pub depends_on: Vec<String>,
}

impl BuildPlan {
    /// Create a build plan from a manifest
    pub fn from_manifest(manifest: &Manifest, target: &str) -> CadiResult<Self> {
        let target_config = manifest.find_target(target)
            .ok_or_else(|| CadiError::BuildFailed(format!("Target '{}' not found", target)))?;
        
        let mut steps = Vec::new();
        let mut visited = HashSet::new();
        
        // Build dependency graph
        let deps = build_dependency_graph(&manifest.build_graph);
        
        // Determine nodes needed for this target
        let nodes_needed: HashSet<_> = if target_config.nodes.is_empty() {
            manifest.build_graph.nodes.iter().map(|n| n.id.as_str()).collect()
        } else {
            target_config.nodes.iter().map(|n| n.id.as_str()).collect()
        };
        
        // Topological sort of nodes
        for node_id in &nodes_needed {
            collect_steps(
                &manifest.build_graph,
                node_id,
                &target_config.platform,
                &deps,
                &mut visited,
                &mut steps,
            )?;
        }
        
        // Estimate time (placeholder)
        let estimated_time_ms = steps.len() as u64 * 1000;
        
        Ok(Self {
            steps,
            estimated_time_ms,
        })
    }

    /// Check if plan is empty
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Get step count
    pub fn len(&self) -> usize {
        self.steps.len()
    }
}

/// Build a dependency graph from the build graph edges
fn build_dependency_graph(build_graph: &cadi_core::BuildGraph) -> HashMap<String, Vec<String>> {
    let mut deps: HashMap<String, Vec<String>> = HashMap::new();
    
    for node in &build_graph.nodes {
        deps.entry(node.id.clone()).or_default();
    }
    
    for edge in &build_graph.edges {
        deps.entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
    }
    
    deps
}

/// Recursively collect build steps for a node
fn collect_steps(
    build_graph: &cadi_core::BuildGraph,
    node_id: &str,
    platform: &str,
    deps: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    steps: &mut Vec<BuildStep>,
) -> CadiResult<()> {
    if visited.contains(node_id) {
        return Ok(());
    }
    
    // Process dependencies first
    if let Some(node_deps) = deps.get(node_id) {
        for dep_id in node_deps {
            collect_steps(build_graph, dep_id, platform, deps, visited, steps)?;
        }
    }
    
    visited.insert(node_id.to_string());
    
    // Find the node
    let node = build_graph.nodes.iter()
        .find(|n| n.id == node_id)
        .ok_or_else(|| CadiError::BuildFailed(format!("Node '{}' not found", node_id)))?;
    
    // Determine the best representation for this platform
    let repr = select_representation(node, platform);
    
    // Create build step
    let step = BuildStep {
        name: node_id.to_string(),
        chunk_id: repr.map(|r| r.chunk.clone()),
        transform: determine_transform(node, platform),
        inputs: build_inputs(node, deps),
        depends_on: deps.get(node_id)
            .map(|d| d.clone())
            .unwrap_or_default(),
    };
    
    steps.push(step);
    
    Ok(())
}

/// Select the best representation for a platform
fn select_representation<'a>(
    node: &'a cadi_core::GraphNode,
    platform: &str,
) -> Option<&'a cadi_core::Representation> {
    // Try to find exact platform match
    if let Some(r) = node.representations.iter().find(|r| {
        r.architecture.as_ref().map(|a| a == platform).unwrap_or(false)
    }) {
        return Some(r);
    }
    
    // Fall back to any representation
    node.representations.first()
}

/// Determine the transformation needed for a node
fn determine_transform(node: &cadi_core::GraphNode, platform: &str) -> super::TransformType {
    // If we have source, need to compile
    if node.source_cadi.is_some() {
        return super::TransformType::Compile {
            target: platform.to_string(),
        };
    }
    
    // If we have IR, need to compile to blob
    if node.ir_cadi.is_some() {
        return super::TransformType::Compile {
            target: platform.to_string(),
        };
    }
    
    // If we have container, just fetch
    if node.container_cadi.is_some() {
        return super::TransformType::Custom {
            name: "fetch".to_string(),
            args: Default::default(),
        };
    }
    
    // Default to bundle
    super::TransformType::Bundle {
        format: "default".to_string(),
    }
}

/// Build input list for a node
fn build_inputs(
    node: &cadi_core::GraphNode,
    deps: &HashMap<String, Vec<String>>,
) -> Vec<super::TransformInput> {
    let mut inputs = Vec::new();
    
    // Add main input
    if let Some(ref source_id) = node.source_cadi {
        inputs.push(super::TransformInput {
            chunk_id: source_id.clone(),
            data: None,
            role: "main".to_string(),
            path: None,
        });
    } else if let Some(ref ir_id) = node.ir_cadi {
        inputs.push(super::TransformInput {
            chunk_id: ir_id.clone(),
            data: None,
            role: "main".to_string(),
            path: None,
        });
    } else if let Some(ref blob_id) = node.blob_cadi {
        inputs.push(super::TransformInput {
            chunk_id: blob_id.clone(),
            data: None,
            role: "main".to_string(),
            path: None,
        });
    }
    
    // Add dependency inputs
    if let Some(node_deps) = deps.get(&node.id) {
        for dep_id in node_deps {
            inputs.push(super::TransformInput {
                chunk_id: format!("pending:{}", dep_id),
                data: None,
                role: "dependency".to_string(),
                path: None,
            });
        }
    }
    
    inputs
}
