use cadi_builder::build_spec::{BuildSpec, BuildSpecValidator};
use serde_yaml;

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