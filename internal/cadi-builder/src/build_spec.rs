use anyhow::Result;
use cadi_registry::search::SearchEngine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSpec {
    pub version: String,
    pub project: ProjectInfo,
    pub components: Vec<ComponentSpec>,
    pub targets: Vec<TargetSpec>,
    pub constraints: Option<ConstraintsSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub description: Option<String>,
    pub language: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ComponentSpec {
    Reuse(ReuseComponent),
    Generate(GenerateComponent),
    Search(SearchComponent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReuseComponent {
    pub id: String,
    pub source: String,
    pub alias: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateComponent {
    pub id: String,
    pub generate: bool,
    pub description: String,
    pub depends_on: Option<Vec<String>>,
    pub interface: Option<InterfaceSpec>,
    pub code_snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchComponent {
    pub id: String,
    pub query: String,
    pub language: Option<String>,
    pub concepts: Option<Vec<String>>,
    pub expected_interface: Option<InterfaceSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceSpec {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetSpec {
    pub name: String,
    pub platform: Option<String>,
    pub components: Vec<String>,
    pub entry_point: Option<String>,
    pub environment: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintsSpec {
    pub min_reuse_percentage: Option<f32>,
    pub max_dependencies: Option<usize>,
    pub language_mix: Option<Vec<String>>,
}

pub struct BuildSpecValidator;

impl BuildSpecValidator {
    /// Load and parse CBS from YAML file
    pub fn from_yaml(path: &Path) -> Result<BuildSpec> {
        let content = std::fs::read_to_string(path)?;
        let spec: BuildSpec = serde_yaml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("YAML parse error: {}", e))?;

        // Validate schema
        Self::validate(&spec)?;

        Ok(spec)
    }

    /// Validate CBS structure and references
    pub fn validate(spec: &BuildSpec) -> Result<()> {
        // 1. Validate project info
        if spec.project.name.is_empty() {
            return Err(anyhow::anyhow!("Project name is required"));
        }

        // 2. Validate components exist and are properly defined
        let mut component_ids = std::collections::HashSet::new();
        for component in &spec.components {
            let id = match component {
                ComponentSpec::Reuse(c) => &c.id,
                ComponentSpec::Generate(c) => &c.id,
                ComponentSpec::Search(c) => &c.id,
            };

            if component_ids.contains(id) {
                return Err(anyhow::anyhow!("Duplicate component ID: {}", id));
            }
            component_ids.insert(id.clone());
        }

        // 3. Validate targets reference existing components
        for target in &spec.targets {
            for comp_id in &target.components {
                if !component_ids.contains(comp_id) {
                    return Err(anyhow::anyhow!(
                        "Target '{}' references undefined component '{}'",
                        target.name,
                        comp_id
                    ));
                }
            }
        }

        // 4. Validate dependencies
        for component in &spec.components {
            if let ComponentSpec::Generate(gen) = component {
                if let Some(deps) = &gen.depends_on {
                    for dep_id in deps {
                        if !component_ids.contains(dep_id) {
                            return Err(anyhow::anyhow!(
                                "Component '{}' depends on undefined '{}'",
                                gen.id,
                                dep_id
                            ));
                        }
                    }
                }
            }
        }

        // 5. Validate constraints
        if let Some(constraints) = &spec.constraints {
            if let Some(min_reuse) = constraints.min_reuse_percentage {
                if !(0.0..=100.0).contains(&min_reuse) {
                    return Err(anyhow::anyhow!("min_reuse_percentage must be 0-100"));
                }
            }
        }

        Ok(())
    }

    /// Convert CBS to internal build plan
    pub async fn to_build_plan(
        spec: BuildSpec,
        search_db: Arc<SearchEngine>,
    ) -> Result<BuildPlan> {
        let mut plan = BuildPlan {
            project: spec.project,
            reuse_components: Vec::new(),
            generate_components: Vec::new(),
            targets: spec.targets,
        };

        for component in spec.components {
            match component {
                ComponentSpec::Reuse(reuse) => {
                    plan.reuse_components.push(ReusePlan {
                        id: reuse.id,
                        chunk_id: reuse.source,
                    });
                }

                ComponentSpec::Generate(gen) => {
                    plan.generate_components.push(GeneratePlan {
                        id: gen.id,
                        description: gen.description,
                        depends_on: gen.depends_on.unwrap_or_default(),
                        interface: gen.interface,
                        code_snippet: gen.code_snippet,
                    });
                }

                ComponentSpec::Search(search) => {
                    // Resolve search query to chunk ID
                    let results = search_db.search(&search.query, 5).await?;

                    if results.is_empty() {
                        return Err(anyhow::anyhow!("No chunks found for query: {}", search.query));
                    }

                    // Use top result
                    plan.reuse_components.push(ReusePlan {
                        id: search.id,
                        chunk_id: results[0].id.clone(),
                    });
                }
            }
        }

        Ok(plan)
    }
}

pub struct BuildPlan {
    pub project: ProjectInfo,
    pub reuse_components: Vec<ReusePlan>,
    pub generate_components: Vec<GeneratePlan>,
    pub targets: Vec<TargetSpec>,
}

pub struct ReusePlan {
    pub id: String,
    pub chunk_id: String,
}

pub struct GeneratePlan {
    pub id: String,
    pub description: String,
    pub depends_on: Vec<String>,
    pub interface: Option<InterfaceSpec>,
    pub code_snippet: Option<String>,
}