use crate::graph::GraphStore;
use super::policy::ExpansionPolicy;
use super::analyzer::DependencyAnalyzer;

/// Ghost Import Resolver
///
/// Automatically expands atom context to include necessary dependencies
/// so LLMs don't hallucinate missing types.
pub struct GhostResolver {
    graph: GraphStore,
}

#[derive(Debug)]
pub struct ExpansionResult {
    /// All atoms to include (original + ghosts)
    pub atoms: Vec<String>,
    /// Which atoms were "ghost" additions
    pub ghost_atoms: Vec<String>,
    /// Truncated due to limits?
    pub truncated: bool,
    /// Total token estimate
    pub total_tokens: usize,
    /// Explanation of what was included and why
    pub explanation: String,
}

impl GhostResolver {
    pub fn new(graph: GraphStore) -> Self {
        Self { graph }
    }

    pub fn analyzer(&self) -> DependencyAnalyzer<'_> {
        DependencyAnalyzer::new(&self.graph)
    }

    /// Resolve ghost imports for a set of atoms
    pub async fn resolve(&self, atom_ids: &[String]) -> Result<ExpansionResult, Box<dyn std::error::Error + Send + Sync>> {
        self.resolve_with_policy(atom_ids, &ExpansionPolicy::default()).await
    }

    /// Resolve with custom policy
    pub async fn resolve_with_policy(
        &self,
        atom_ids: &[String],
        policy: &ExpansionPolicy,
    ) -> Result<ExpansionResult, Box<dyn std::error::Error + Send + Sync>> {
        // Simulate the expansion first
        let simulation = self.analyzer().simulate_expansion(atom_ids, policy)?;

        // Separate original vs ghost atoms
        let mut ghost_atoms = Vec::new();
        let mut explanations = Vec::new();

        for atom_id in &simulation.included_atoms {
            if !atom_ids.contains(atom_id) {
                ghost_atoms.push(atom_id.clone());

                // Find why this atom was included
                if let Some(reason) = self.find_inclusion_reason(atom_id, atom_ids, policy).await? {
                    explanations.push(reason);
                }
            }
        }

        Ok(ExpansionResult {
            atoms: simulation.included_atoms,
            ghost_atoms,
            truncated: simulation.truncated,
            total_tokens: simulation.total_tokens,
            explanation: explanations.join("\n"),
        })
    }

    /// Get optimal policy for a set of atoms
    pub async fn suggest_policy(&self, atom_ids: &[String]) -> Result<ExpansionPolicy, Box<dyn std::error::Error + Send + Sync>> {
        let analysis = self.analyzer().analyze_dependencies(atom_ids)?;

        // Calculate average dependencies per atom
        let avg_deps = analysis.iter()
            .map(|info| info.dependencies.len())
            .sum::<usize>() as f64 / analysis.len() as f64;

        // Calculate total tokens
        let total_tokens = analysis.iter()
            .map(|info| info.token_estimate)
            .sum::<usize>();

        // Suggest policy based on complexity
        let mut policy = ExpansionPolicy::default();

        if avg_deps < 2.0 && total_tokens < 1000 {
            // Simple case - be conservative
            policy = ExpansionPolicy::conservative();
        } else if avg_deps > 5.0 || total_tokens > 5000 {
            // Complex case - be more aggressive but careful
            policy.max_depth = 3;
            policy.max_atoms = 30;
            policy.max_tokens = 6000;
        }

        Ok(policy)
    }

    async fn find_inclusion_reason(
        &self,
        atom_id: &str,
        original_atoms: &[String],
        policy: &ExpansionPolicy,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        // Find which original atom(s) led to this inclusion
        for original_id in original_atoms {
            let deps = self.graph.get_dependencies(original_id)?;
            for (edge_type, dep_id) in deps {
                if dep_id == atom_id && policy.follow_edges.contains(&edge_type) {
                    return Ok(Some(format!(
                        "Added '{}' because '{}' references it via {:?}",
                        atom_id, original_id, edge_type
                    )));
                }
            }
        }

        // Check transitive dependencies
        for original_id in original_atoms {
            if let Some(path) = self.find_dependency_path(original_id, atom_id, policy, 3)? {
                return Ok(Some(format!(
                    "Added '{}' through dependency chain: {}",
                    atom_id, path
                )));
            }
        }

        Ok(None)
    }

    fn find_dependency_path(
        &self,
        from: &str,
        to: &str,
        policy: &ExpansionPolicy,
        max_depth: usize,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        if max_depth == 0 {
            return Ok(None);
        }

        let deps = self.graph.get_dependencies(from)?;
        for (edge_type, dep_id) in deps {
            if dep_id == to && policy.follow_edges.contains(&edge_type) {
                return Ok(Some(format!("{} -> {}", from, to)));
            }

            if let Some(path) = self.find_dependency_path(&dep_id, to, policy, max_depth - 1)? {
                return Ok(Some(format!("{} -> {}", from, path)));
            }
        }

        Ok(None)
    }
}