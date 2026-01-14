# Stage 4: CADI Build Spec (CBS) for LLMs
## Detailed Implementation Specification

### Overview
A simple, declarative YAML format for LLMs to express project composition. LLMs write ~50-100 lines of CBS instead of 2000+ lines of code. The build engine then handles all resolution, scaffolding, and linking automatically.

---

## 4.1 CBS Schema

### File: `cadi-spec/build-spec.schema.json` (new)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://cadi.dev/schemas/build-spec.schema.json",
  "title": "CADI Build Spec",
  "description": "Declarative specification for composing CADI chunks into a project",
  "type": "object",
  "required": ["version", "project", "components"],
  "properties": {
    "version": {
      "type": "string",
      "default": "1.0",
      "description": "CBS format version"
    },
    
    "project": {
      "type": "object",
      "required": ["name"],
      "properties": {
        "name": {
          "type": "string",
          "description": "Project name"
        },
        "description": {
          "type": "string",
          "description": "Project description"
        },
        "language": {
          "type": "string",
          "enum": ["typescript", "python", "rust", "go", "java"],
          "description": "Primary language"
        },
        "version": {
          "type": "string",
          "description": "Project version"
        }
      }
    },
    
    "components": {
      "type": "array",
      "description": "Components to use or generate",
      "items": {
        "oneOf": [
          { "$ref": "#/definitions/reuse_component" },
          { "$ref": "#/definitions/generate_component" },
          { "$ref": "#/definitions/search_component" }
        ]
      }
    },
    
    "targets": {
      "type": "array",
      "description": "Build targets",
      "items": {
        "type": "object",
        "required": ["name", "components"],
        "properties": {
          "name": {
            "type": "string",
            "description": "Target name (api, worker, cli, etc.)"
          },
          "platform": {
            "type": "string",
            "description": "Target platform (nodejs, python3, wasm, etc.)"
          },
          "components": {
            "type": "array",
            "items": { "type": "string" },
            "description": "Component IDs to include in this target"
          },
          "entry_point": {
            "type": "string",
            "description": "Entry point file"
          },
          "environment": {
            "type": "object",
            "description": "Environment variables for this target"
          }
        }
      },
      "default": []
    },
    
    "constraints": {
      "type": "object",
      "description": "Composition constraints",
      "properties": {
        "min_reuse_percentage": {
          "type": "number",
          "description": "Minimum percentage of reused code (0-100)",
          "default": 0
        },
        "max_dependencies": {
          "type": "number",
          "description": "Maximum transitive dependencies",
          "default": 100
        },
        "language_mix": {
          "type": "array",
          "description": "Allowed language combinations",
          "items": { "type": "string" }
        }
      }
    }
  },
  
  "definitions": {
    "reuse_component": {
      "type": "object",
      "required": ["id", "source"],
      "properties": {
        "id": {
          "type": "string",
          "description": "Component ID (used in targets)"
        },
        "source": {
          "type": "string",
          "description": "CADI chunk ID to reuse"
        },
        "alias": {
          "type": "string",
          "description": "Local alias for this component"
        },
        "version": {
          "type": "string",
          "description": "Specific version/revision"
        }
      }
    },
    
    "generate_component": {
      "type": "object",
      "required": ["id", "generate"],
      "properties": {
        "id": {
          "type": "string",
          "description": "Component ID"
        },
        "generate": {
          "type": "boolean",
          "const": true,
          "description": "Mark as novel/generated"
        },
        "description": {
          "type": "string",
          "description": "What this component does"
        },
        "depends_on": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Component IDs this depends on"
        },
        "interface": {
          "type": "object",
          "description": "Expected interface",
          "properties": {
            "inputs": {
              "type": "array",
              "items": { "type": "string" }
            },
            "outputs": {
              "type": "array",
              "items": { "type": "string" }
            }
          }
        },
        "code_snippet": {
          "type": "string",
          "description": "Optional: Code snippet or scaffold"
        }
      }
    },
    
    "search_component": {
      "type": "object",
      "required": ["id", "query"],
      "properties": {
        "id": {
          "type": "string",
          "description": "Component ID"
        },
        "query": {
          "type": "string",
          "description": "Semantic search query to find component"
        },
        "language": {
          "type": "string",
          "description": "Filter by language"
        },
        "concepts": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Filter by concepts"
        },
        "expected_interface": {
          "type": "object",
          "description": "Expected interface for validation"
        }
      }
    }
  }
}
```

---

## 4.2 CBS Parser & Validator

### File: `internal/cadi-builder/src/build_spec.rs` (new)

**Goal**: Parse CBS YAML and validate it before build.

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
            .map_err(|e| format!("YAML parse error: {}", e))?;
        
        // Validate schema
        Self::validate(&spec)?;
        
        Ok(spec)
    }
    
    /// Validate CBS structure and references
    pub fn validate(spec: &BuildSpec) -> Result<()> {
        // 1. Validate project info
        if spec.project.name.is_empty() {
            return Err("Project name is required".into());
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
                return Err(format!("Duplicate component ID: {}", id));
            }
            component_ids.insert(id.clone());
        }
        
        // 3. Validate targets reference existing components
        for target in &spec.targets {
            for comp_id in &target.components {
                if !component_ids.contains(comp_id) {
                    return Err(format!(
                        "Target '{}' references undefined component '{}'",
                        target.name, comp_id
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
                            return Err(format!(
                                "Component '{}' depends on undefined '{}'",
                                gen.id, dep_id
                            ));
                        }
                    }
                }
            }
        }
        
        // 5. Validate constraints
        if let Some(constraints) = &spec.constraints {
            if let Some(min_reuse) = constraints.min_reuse_percentage {
                if min_reuse < 0.0 || min_reuse > 100.0 {
                    return Err("min_reuse_percentage must be 0-100".into());
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
                        return Err(format!("No chunks found for query: {}", search.query));
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
```

---

## 4.3 CBS Examples

### File: `examples/blog-app.build-spec.yaml`

```yaml
version: "1.0"

project:
  name: "blog-app"
  description: "Blog platform with real-time comments and analytics"
  language: typescript
  version: "1.0.0"

components:
  # === REUSE FROM TODO APP ===
  
  - id: auth
    source: "chunk:sha256:todo-jwt-abc123"
    alias: jwt_auth
    version: "1.0"
  
  - id: database
    source: "chunk:sha256:todo-postgres-crud-def456"
    alias: db_ops
    version: "1.0"
  
  - id: logger
    source: "chunk:sha256:todo-logger-ghi789"
    alias: logger
  
  # === SEARCH & AUTO-RESOLVE ===
  
  - id: error_handler
    query: "error handling middleware for HTTP servers"
    language: typescript
    concepts: ["error", "http", "middleware"]
    expected_interface:
      inputs: ["error", "request", "response"]
      outputs: ["error_response"]
  
  # === NOVEL/GENERATED ===
  
  - id: content_service
    generate: true
    description: "Blog post creation, retrieval, and update operations"
    depends_on: [database, logger]
    interface:
      inputs: ["userId", "postData", "title", "body"]
      outputs: ["post"]
    code_snippet: |
      // Framework: Fetch existing CRUD patterns
      // Adapt to blog post schema
      export async function createPost(
        userId: string,
        title: string,
        body: string
      ) {
        // Use db_ops to insert
        const post = await db.create('posts', {
          user_id: userId,
          title,
          body,
          created_at: new Date()
        });
        logger.info(`Post created: ${post.id}`);
        return post;
      }
  
  - id: comment_threading
    generate: true
    description: "Threaded comments with conflict resolution for real-time sync"
    depends_on: [database, logger, auth]
    interface:
      inputs: ["postId", "userId", "comment"]
      outputs: ["comment_with_thread"]
  
  - id: analytics
    generate: true
    description: "Track views, engagement, trending posts"
    depends_on: [database, logger]
    interface:
      inputs: ["event_type", "data"]
      outputs: ["aggregated_stats"]

# === BUILD TARGETS ===

targets:
  - name: api
    platform: nodejs
    components: [auth, database, logger, content_service, comment_threading, error_handler]
    entry_point: src/api.ts
    environment:
      PORT: "3000"
      DATABASE_URL: "postgresql://localhost/blog"
  
  - name: worker
    platform: nodejs
    components: [database, analytics, logger]
    entry_point: src/worker.ts
    environment:
      DATABASE_URL: "postgresql://localhost/blog"
  
  - name: cli
    platform: nodejs
    components: [database, logger, auth]
    entry_point: src/cli.ts

# === CONSTRAINTS ===

constraints:
  min_reuse_percentage: 60  # At least 60% reused from todo app
  max_dependencies: 50
  language_mix: ["typescript"]
```

---

## 4.4 LLM Prompt Template for CBS Generation

### File: `docs/cadi-llm-prompt.md`

```markdown
# CADI-First Development Prompt for LLMs

You are building a new project using CADI (Content-Addressed Development Interface).
CADI's power is **semantic reuse**: find and compose existing code atoms instead of writing from scratch.

## Your Workflow

### Step 1: Plan with CADI
Before writing code, decompose the problem:
- "I need to build: [project]"
- "Breaking it down into: [components]"
- "Which of these likely exist in past projects? [reusable]"
- "Which are novel? [to generate]"

### Step 2: Write a Build Spec (not code!)
Create a `build.build-spec.yaml` that:
1. **References reusable chunks** - search by semantic intent
2. **Marks novel parts** with `generate: true`
3. **Declares dependencies** between components
4. **Defines targets** (api, worker, etc.)

Example:
\`\`\`yaml
components:
  - id: auth
    query: "JWT authentication middleware for Node.js"
    # Build engine searches CADI, finds todo's JWT chunk, resolves it
  
  - id: novel_feature
    generate: true
    depends_on: [auth]
    description: "New feature only in this project"
\`\`\`

### Step 3: Build & Iterate
1. Run: `cadi build blog.build-spec.yaml`
2. Engine resolves queries, generates scaffolds
3. If "needs_generation" → you provide details (summaries, not full code)
4. Engine links everything automatically

## CBS Format Quick Reference

### Reuse Pattern (50-200 tokens)
```yaml
- id: my_component
  source: chunk:sha256:abc123  # or use query below
  query: "what you need"  # Build engine finds it
  language: typescript
```

### Generate Pattern (100-300 tokens for description)
```yaml
- id: novel_feature
  generate: true
  depends_on: [auth, db]
  description: "Brief description of what it does"
  interface:
    inputs: ["userId", "data"]
    outputs: ["result"]
```

## Token Savings

**Without CADI** (your baseline):
- Read existing code: 500 tokens
- Understand patterns: 300 tokens
- Write new code: 1000 tokens
- **Total: ~1800 tokens**

**With CADI** (your target):
- Search: 50 tokens
- Review 5 results: 100 tokens
- Build spec: 200 tokens
- (Novel code only): 500 tokens
- **Total: ~850 tokens (53% savings!)**

## Constraints to Remember

- **Prefer reuse**: If something similar exists, search CADI first
- **Query semantically**: "How would a human describe this?" not keywords
- **Mark novel clearly**: `generate: true` means "I'm writing new logic"
- **Declare dependencies**: Build engine links them automatically
- **No manual imports**: Engine generates imports/scaffolding for you

## Success = Reuse + Minimal Novel Code

Target composition: **60-80% reused, 20-40% novel**
If you're writing >50% from scratch, search CADI harder!

---

## Example: Building a Blog App (CADI-First)

### Decomposition
- Need: JWT auth (✓ from todo app)
- Need: DB operations (✓ from todo app)
- Need: Error handling (? search CADI)
- Need: Blog content service (✗ novel)
- Need: Comments (✗ novel)
- Need: Analytics (✗ novel)

### Build Spec (80 lines total!)
\`\`\`yaml
version: "1.0"
project:
  name: blog-app
  language: typescript

components:
  # REUSE
  - id: auth
    query: "JWT authentication for Node.js"
  - id: database
    query: "PostgreSQL CRUD operations with TypeScript"
  - id: logger
    query: "logging middleware for HTTP servers"
  
  # NOVEL (only ~40 lines each for descriptions)
  - id: content_service
    generate: true
    depends_on: [database, logger]
    description: "Create, read, update blog posts"
  
  - id: comments
    generate: true
    depends_on: [database, auth, logger]
    description: "Threaded comments with real-time sync"
  
  - id: analytics
    generate: true
    depends_on: [database]
    description: "Track views and engagement"

targets:
  - name: api
    components: [auth, database, logger, content_service, comments, analytics]
\`\`\`

### Build Output
```
cadi build blog.build-spec.yaml
→ Resolves auth, database, logger via semantic search
→ Scaffolds content_service, comments, analytics
→ Generates imports automatically
→ Output: Ready-to-run project
```

**Reuse rate: 70% | Novel: 30% | Tokens: ~900**

---

## When to Adjust

### "No results for my search"
- Try broader query: "HTTP server" instead of "Express.js HTTP server"
- Fallback: Mark as `generate: true`, write it yourself

### "Found results but not quite right"
- Adjust in build spec: `version: "2.0"` to pin specific variant
- Or compose multiple chunks: `depends_on: [chunk1, chunk2]`

### "Too many dependencies"
- Review composition: Are you pulling in unused transitive deps?
- Refactor: Break into smaller targets (api, worker)

---

## Integration with Your Workflow

1. **Plan** → Decompose, search, draft build spec
2. **Write Spec** → CBS in 5-10 minutes
3. **Build** → Engine resolves + scaffolds (1-2 minutes)
4. **Develop** → Fill in novel logic with full context

Compared to:
1. Read codebase → 30 minutes (500+ tokens)
2. Copy-paste patterns → 20 minutes
3. Write glue code → 40 minutes
4. Debug wiring → 30 minutes

**CADI workflow: 15-20 minutes, 50-75% token savings**
```

---

## 4.5 Testing

### File: `tests/build_spec_test.rs`

```rust
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
    
    let spec: BuildSpec = serde_yaml::from_str(yaml).unwrap();
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
```

---

## Success Criteria

✅ CBS format is simple enough for LLMs to generate  
✅ Validation catches common errors early  
✅ Supports reuse, search, and generate patterns  
✅ Conversion to build plan is deterministic  
✅ ~50-100 line specs encode typical projects  
✅ Parser handles YAML + schema validation  
✅ LLM prompt guidance reduces manual wiring  

---

## Next: Stage 5 (Build Engine)
Once CBS parsing works, build the engine that executes it: resolving queries, scaffolding imports, and outputting runnable projects.
