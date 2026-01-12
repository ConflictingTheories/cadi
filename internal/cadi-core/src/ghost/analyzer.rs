use crate::graph::{GraphStore, EdgeType};
use std::collections::HashSet;

/// Analyzes atoms to determine what dependencies should be included
pub struct DependencyAnalyzer<'a> {
    graph: &'a GraphStore,
}

#[derive(Debug)]
pub struct DependencyInfo {
    pub atom_id: String,
    pub dependencies: Vec<DependencyEdge>,
    pub token_estimate: usize,
}

#[derive(Debug)]
pub struct DependencyEdge {
    pub target: String,
    pub edge_type: EdgeType,
    pub priority: DependencyPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DependencyPriority {
    Critical = 0,    // Type definitions, direct imports
    High = 1,        // Method signatures, interface implementations
    Medium = 2,      // Indirect dependencies
    Low = 3,         // Optional or weak references
}

impl<'a> DependencyAnalyzer<'a> {
    pub fn new(graph: &'a GraphStore) -> Self {
        Self { graph }
    }

    /// Analyze dependencies for a set of atoms
    pub fn analyze_dependencies(
        &self,
        atom_ids: &[String],
    ) -> Result<Vec<DependencyInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = Vec::new();

        for atom_id in atom_ids {
            let deps = self.graph.get_dependencies(atom_id)?;
            let token_estimate = self.graph.get_token_estimate(atom_id)?;

            let dependency_edges = deps
                .into_iter()
                .map(|(edge_type, target)| DependencyEdge {
                    target,
                    edge_type,
                    priority: self.calculate_priority(&edge_type),
                })
                .collect();

            results.push(DependencyInfo {
                atom_id: atom_id.clone(),
                dependencies: dependency_edges,
                token_estimate,
            });
        }

        Ok(results)
    }

    /// Find all atoms that would be included in an expansion
    pub fn simulate_expansion(
        &self,
        atom_ids: &[String],
        policy: &super::policy::ExpansionPolicy,
    ) -> Result<ExpansionSimulation, Box<dyn std::error::Error + Send + Sync>> {
        let mut included = HashSet::new();
        let mut total_tokens = 0;
        let mut depth_reached = 0;

        // Start with requested atoms
        let mut frontier: Vec<(String, usize)> = atom_ids.iter()
            .map(|id| (id.clone(), 0))
            .collect();

        for (atom_id, _depth) in &frontier {
            included.insert(atom_id.clone());
            total_tokens += self.graph.get_token_estimate(atom_id)?;
        }

        // BFS expansion
        while let Some((atom_id, depth)) = frontier.pop() {
            if depth >= policy.max_depth {
                depth_reached = depth_reached.max(depth);
                continue;
            }

            if included.len() >= policy.max_atoms || total_tokens >= policy.max_tokens {
                break;
            }

            let deps = self.graph.get_dependencies(&atom_id)?;
            for (edge_type, dep_id) in deps {
                if policy.follow_edges.contains(&edge_type) && !included.contains(&dep_id) {
                    let dep_tokens = self.graph.get_token_estimate(&dep_id)?;
                    if total_tokens + dep_tokens <= policy.max_tokens {
                        included.insert(dep_id.clone());
                        total_tokens += dep_tokens;
                        frontier.push((dep_id, depth + 1));
                    }
                }
            }
        }

        // Check if we were truncated
        let truncated = included.len() >= policy.max_atoms || total_tokens >= policy.max_tokens;

        Ok(ExpansionSimulation {
            included_atoms: included.into_iter().collect(),
            total_tokens,
            max_depth_reached: depth_reached,
            truncated,
        })
    }

    fn calculate_priority(&self, edge_type: &EdgeType) -> DependencyPriority {
        match edge_type {
            EdgeType::Imports => DependencyPriority::Critical,
            EdgeType::TypeRef => DependencyPriority::High,
            EdgeType::Calls => DependencyPriority::Medium,
            EdgeType::Implements => DependencyPriority::High,
            EdgeType::Extends => DependencyPriority::High,
            EdgeType::GenericRef => DependencyPriority::High,
            EdgeType::ComposedOf => DependencyPriority::Low,
            EdgeType::Exports => DependencyPriority::Low,
            EdgeType::MacroUse => DependencyPriority::Medium,
            EdgeType::Tests => DependencyPriority::Low,
            EdgeType::DocRef => DependencyPriority::Low,
        }
    }
}

#[derive(Debug)]
pub struct ExpansionSimulation {
    pub included_atoms: Vec<String>,
    pub total_tokens: usize,
    pub max_depth_reached: usize,
    pub truncated: bool,
}