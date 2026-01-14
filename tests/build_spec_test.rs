use cadi_builder::build_spec::{BuildSpec, BuildSpecValidator};
use cadi_registry::search::{ComponentMetadata, SearchEngine};
use serde_yaml;
use std::sync::Arc;

#[test]
fn test_build_spec_validation() {
    let yaml = r#"
version: "1.0"
project:
  name: "test-app"
  language: typescript

components:
  - id: auth
    source: "chunk:sha256:abc123"
  
  - id: novel
    generate: true
    depends_on: [auth]
    description: "Test component"

targets:
  - name: api
    components: [auth, novel]
"#;

    let spec: BuildSpec = serde_yaml::from_str(yaml).unwrap();
    assert!(BuildSpecValidator::validate(&spec).is_ok());
}

#[test]
fn test_build_spec_validation_duplicate_id() {
    let yaml = r#"
version: "1.0"
project:
  name: "test"
  language: typescript

components:
  - id: auth
    source: "chunk:sha256:abc"
  - id: auth
    source: "chunk:sha256:def"
"#;

    let spec_result = serde_yaml::from_str::<BuildSpec>(yaml);
    assert!(spec_result.is_ok(), "Should parse, validation is separate");
    let spec = spec_result.unwrap();
    assert!(BuildSpecValidator::validate(&spec).is_err());
}

#[test]
fn test_build_spec_validation_missing_dependency() {
    let yaml = r#"
version: "1.0"
project:
  name: "test"
  language: typescript

components:
  - id: novel
    generate: true
    depends_on: [nonexistent]
    description: "Test"
"#;

    let spec: BuildSpec = serde_yaml::from_str(yaml).unwrap();
    assert!(BuildSpecValidator::validate(&spec).is_err());
}

#[test]
fn test_validate_target_references_unknown_component() {
    let yaml = r#"
version: "1.0"
project:
  name: "test-app"
  language: typescript
components:
  - id: auth
    source: "chunk:sha256:abc123"
targets:
  - name: api
    components: [auth, database] # 'database' is not defined
"#;
    let spec: BuildSpec = serde_yaml::from_str(yaml).unwrap();
    let result = BuildSpecValidator::validate(&spec);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("references undefined component 'database'"));
}

#[test]
fn test_validate_invalid_constraint_value() {
    let yaml = r#"
version: "1.0"
project:
  name: "test-app"
  language: typescript
components: []
constraints:
  min_reuse_percentage: 101.0 # Invalid value
"#;
    let spec: BuildSpec = serde_yaml::from_str(yaml).unwrap();
    let result = BuildSpecValidator::validate(&spec);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("min_reuse_percentage must be 0-100"));
}

#[tokio::test]
async fn test_to_build_plan_with_search() {
    // 1. Setup a mock search engine
    let mut search_engine = SearchEngine::new();
    search_engine.register(
        "chunk:sha256:found-by-search",
        ComponentMetadata {
            name: "HTTP Error Handler".into(),
            description: "A middleware for handling HTTP errors gracefully.".into(),
            language: "typescript".into(),
            usage_count: 100,
            test_coverage: 0.9,
            quality_score: 0.9,
            concepts: vec!["http".into(), "error".into(), "middleware".into()],
        },
    );
    let search_db = Arc::new(search_engine);

    // 2. Create a BuildSpec with a 'search' component
    let yaml = r#"
version: "1.0"
project:
  name: "test-app"
  language: typescript
components:
  - id: error_handler
    query: "error handling middleware"
    language: typescript
"#;
    let spec: BuildSpec = serde_yaml::from_str(yaml).unwrap();

    // 3. Convert to BuildPlan
    let plan = BuildSpecValidator::to_build_plan(spec, search_db).await.unwrap();

    // 4. Validate the plan
    // The 'search' component should have been resolved into a 'reuse' component
    assert_eq!(plan.reuse_components.len(), 1);
    assert_eq!(plan.generate_components.len(), 0);

    let reuse_plan = &plan.reuse_components[0];
    assert_eq!(reuse_plan.id, "error_handler");
    assert_eq!(reuse_plan.chunk_id, "chunk:sha256:found-by-search");
}