//! Manifest types for CADI application build graphs

use serde::{Deserialize, Serialize};

/// Application manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub manifest_id: String,
    pub manifest_version: String,
    pub application: ApplicationInfo,
    pub build_graph: BuildGraph,
    #[serde(default)]
    pub build_targets: Vec<BuildTarget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<DependencyConfig>,
}

/// Application information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
}

/// Build graph structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildGraph {
    pub nodes: Vec<GraphNode>,
    #[serde(default)]
    pub edges: Vec<GraphEdge>,
}

/// A node in the build graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_cadi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ir_cadi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_cadi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_cadi: Option<String>,
    #[serde(default)]
    pub representations: Vec<Representation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection_strategy: Option<String>,
}

/// A representation of a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Representation {
    pub form: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    pub chunk: String,
}

/// An edge in the build graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
    #[serde(default = "default_relation")]
    pub relation: String,
}

fn default_relation() -> String {
    "depends_on".to_string()
}

/// Build target definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildTarget {
    pub name: String,
    pub platform: String,
    #[serde(default)]
    pub nodes: Vec<TargetNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle: Option<BundleConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deploy: Option<DeployConfig>,
}

/// Node configuration within a target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetNode {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer: Option<Vec<String>>,
}

/// Bundle configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BundleConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(default)]
    pub minify: bool,
}

/// Deployment configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeployConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replicas: Option<u32>,
    #[serde(default)]
    pub environment: std::collections::HashMap<String, String>,
}

/// Dependency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_file: Option<String>,
    #[serde(default = "default_resolution_strategy")]
    pub resolution_strategy: String,
}

fn default_resolution_strategy() -> String {
    "newest".to_string()
}

impl Manifest {
    /// Create a new manifest with the given ID and application name
    pub fn new(manifest_id: String, app_name: String) -> Self {
        Self {
            manifest_id,
            manifest_version: "1.0".to_string(),
            application: ApplicationInfo {
                name: app_name,
                description: None,
                version: None,
                authors: Vec::new(),
                license: None,
                repository: None,
            },
            build_graph: BuildGraph {
                nodes: Vec::new(),
                edges: Vec::new(),
            },
            build_targets: Vec::new(),
            dependencies: None,
        }
    }

    /// Add a node to the build graph
    pub fn add_node(&mut self, node: GraphNode) {
        self.build_graph.nodes.push(node);
    }

    /// Add an edge to the build graph
    pub fn add_edge(&mut self, edge: GraphEdge) {
        self.build_graph.edges.push(edge);
    }

    /// Add a build target
    pub fn add_target(&mut self, target: BuildTarget) {
        self.build_targets.push(target);
    }

    /// Find a node by ID
    pub fn find_node(&self, id: &str) -> Option<&GraphNode> {
        self.build_graph.nodes.iter().find(|n| n.id == id)
    }

    /// Find a target by name
    pub fn find_target(&self, name: &str) -> Option<&BuildTarget> {
        self.build_targets.iter().find(|t| t.name == name)
    }
}
