//! MCP Prompts for CADI
//!
//! Pre-built prompt templates that guide agents to use CADI efficiently.
//! These prompts encode the "CADI-first" workflow to save tokens.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Prompt definition for MCP
#[derive(Debug, Clone, Serialize)]
pub struct PromptDefinition {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// Get all available prompts
pub fn get_prompts() -> Vec<PromptDefinition> {
    vec![
        PromptDefinition {
            name: "cadi_workflow".to_string(),
            description: "⚡ CADI-first workflow - ALWAYS use before writing code to save tokens".to_string(),
            arguments: Some(vec![
                PromptArgument {
                    name: "task".to_string(),
                    description: "What you need to accomplish".to_string(),
                    required: true,
                },
            ]),
        },
        PromptDefinition {
            name: "import_project".to_string(),
            description: "Import a codebase into CADI for reuse - do this first with any new project".to_string(),
            arguments: Some(vec![
                PromptArgument {
                    name: "path".to_string(),
                    description: "Path to the project".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "namespace".to_string(),
                    description: "Namespace for aliases (e.g., 'myorg')".to_string(),
                    required: false,
                },
            ]),
        },
        PromptDefinition {
            name: "find_reusable_code".to_string(),
            description: "Search for existing code before writing new - saves 500+ tokens".to_string(),
            arguments: Some(vec![
                PromptArgument {
                    name: "what".to_string(),
                    description: "What functionality you need".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "language".to_string(),
                    description: "Programming language".to_string(),
                    required: false,
                },
            ]),
        },
        PromptDefinition {
            name: "use_chunk".to_string(),
            description: "Retrieve and use a CADI chunk by alias or ID".to_string(),
            arguments: Some(vec![
                PromptArgument {
                    name: "alias_or_id".to_string(),
                    description: "Chunk alias (e.g., 'myproject/utils') or chunk ID".to_string(),
                    required: true,
                },
            ]),
        },
    ]
}

/// Get a prompt by name with arguments filled in
pub fn get_prompt(name: &str, arguments: &Value) -> Result<Vec<Value>, String> {
    match name {
        "cadi_workflow" => get_cadi_workflow_prompt(arguments),
        "import_project" => get_import_project_prompt(arguments),
        "find_reusable_code" => get_find_reusable_prompt(arguments),
        "use_chunk" => get_use_chunk_prompt(arguments),
        _ => Err(format!("Unknown prompt: {}", name)),
    }
}

fn get_cadi_workflow_prompt(args: &Value) -> Result<Vec<Value>, String> {
    let task = args.get("task").and_then(|t| t.as_str()).unwrap_or("complete the task");
    
    Ok(vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": format!(r#"# Task: {}

## ☯️ Core Philosophy: CADI is the way code works with LLMs

CADI is not a tool; it is the fundamental way code works with LLMs. We move from "pasting codebases" to "**referencing semantic atoms**." 

### The CADI Invariant:
$$LLM + CADI > LLM alone$$

Our goal: To be simpler and faster than working with an LLM directly. Referencing a semantic atom is more powerful than generating one from scratch.

## ⚡ CADI-First Workflow (Follow These Steps to Save Tokens)

### Step 1: Search for existing semantic atoms
Use `cadi_search` to find if this has been solved before.
- Look for atoms by intent, not just keywords.
- Referencing an atom is always superior to re-generating it.

### Step 2: Check cached aliases  
Use `cadi_resolve_alias` if you know a likely alias:
- Format: namespace/component (e.g., "myproject/utils/logger")

### Step 3: Retrieve the atom reference
Use `cadi_get_chunk` with the ID from results.
This provides the "Truth" of the implementation without the noise of generation.

### Step 4: Only create new atoms if needed
If no atom exists:
1. Write minimal new code
2. Canonicalize and Import it with `cadi_import` to create a new permanent atom.

**Start with Step 1 now. Optimize for the Invariant.**"#, task)
        }
    })])
}

fn get_import_project_prompt(args: &Value) -> Result<Vec<Value>, String> {
    let path = args.get("path").and_then(|p| p.as_str()).unwrap_or(".");
    let namespace = args.get("namespace").and_then(|n| n.as_str()).unwrap_or("project");
    
    Ok(vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": format!(r#"# Import Project into CADI

Import the project at `{}` with namespace `{}`.

## Action
Call `cadi_import` with:
- path: "{}"
- namespace: "{}"
- publish: true (to share to registry)

## What This Does
1. Analyzes the codebase structure
2. Intelligently chunks code into reusable pieces
3. Creates human-readable aliases like `{}/utils/helper`
4. Saves to local cache for instant reuse
5. Optionally publishes to registry for team sharing

## After Import
You can reference any chunk by alias:
- `cadi_resolve_alias("{}/lib")` → gets chunk ID
- `cadi_get_chunk(chunk_id)` → gets the code

This is a ONE-TIME cost that saves tokens forever."#, 
                path, namespace, path, namespace, namespace, namespace)
        }
    })])
}

fn get_find_reusable_prompt(args: &Value) -> Result<Vec<Value>, String> {
    let what = args.get("what").and_then(|w| w.as_str()).unwrap_or("the functionality");
    let language = args.get("language").and_then(|l| l.as_str());
    
    let lang_hint = language.map(|l| format!(" in {}", l)).unwrap_or_default();
    
    Ok(vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": format!(r#"# Find Reusable Code{lang_hint}

Looking for: {}

## Search Strategy
1. First, try `cadi_search` with:
   - query: "{}"
   {}

2. If no results, try alternative terms:
   - Synonyms for the functionality
   - More general/specific terms

3. Check local aliases with `cadi://aliases` resource

## If Found
- Use `cadi_get_chunk` to retrieve
- The code is already tested and working
- Saves 500+ tokens vs writing new

## If Not Found
- Write minimal implementation
- Import with `cadi_import(publish: true)` for future reuse"#, 
                what, what,
                language.map(|l| format!("- language: \"{}\"", l)).unwrap_or_default())
        }
    })])
}

fn get_use_chunk_prompt(args: &Value) -> Result<Vec<Value>, String> {
    let alias_or_id = args.get("alias_or_id").and_then(|a| a.as_str()).unwrap_or("");
    
    let is_chunk_id = alias_or_id.starts_with("chunk:");
    
    Ok(vec![json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": if is_chunk_id {
                format!(r#"# Use CADI Chunk

Chunk ID: `{}`

## Action
Call `cadi_get_chunk` with chunk_id: "{}"

This retrieves the full chunk content which you can use directly."#, 
                    alias_or_id, alias_or_id)
            } else {
                format!(r#"# Use CADI Chunk by Alias

Alias: `{}`

## Steps
1. Resolve alias: `cadi_resolve_alias(alias: "{}")`
2. Get chunk: `cadi_get_chunk(chunk_id: <result from step 1>)`

## Why Use Aliases
- Human-readable: `myproject/utils/logger` vs `chunk:sha256:abc123...`
- Memorable and discoverable
- Faster than searching"#, 
                    alias_or_id, alias_or_id)
            }
        }
    })])
}
