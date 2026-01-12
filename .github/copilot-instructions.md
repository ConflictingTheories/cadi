# GitHub Copilot Instructions for CADI Workspace

## ⚡ CADI-FIRST WORKFLOW: SAVE TOKENS!

Before writing ANY code, use CADI to find existing solutions:

### Step 1: Search First (~50 tokens)
```
cadi_search(query: "what you need")
```
This searches all registries for existing code chunks.

### Step 2: Check Aliases (~30 tokens)
```
cadi_resolve_alias(alias: "namespace/component")
```
Look up cached code by human-readable alias.

### Step 3: Retrieve Code (~100 tokens)
```
cadi_get_chunk(id: "chunk:sha256:...")
```
Get the actual code to use.

### Step 4: Only Write New Code If Needed
If CADI has nothing, then write new code. But **import it after** so it's reusable:
```
cadi_import(path: "./new-code", namespace: "myproject", publish: true)
```

## Token Cost Comparison

| Action | Cost | Notes |
|--------|------|-------|
| `cadi_search` | ~50 tokens | Check first! |
| `cadi_resolve_alias` | ~30 tokens | Fast lookup |
| `cadi_get_chunk` | ~100 tokens | Get code |
| Writing new code | 500-5000 tokens | Expensive! |

## Available Resources

Read these for context:
- `cadi://guide` - Full CADI usage guide
- `cadi://aliases` - All cached chunk aliases

## Anti-Patterns to Avoid

❌ Writing code without searching CADI first
❌ Reading entire files when you need one function  
❌ Re-implementing common utilities
❌ Not importing projects you're working with

## CADI Tools Quick Reference

| Tool | When to Use |
|------|-------------|
| `cadi_search` | Finding existing solutions |
| `cadi_resolve_alias` | Looking up known chunks |
| `cadi_get_chunk` | Retrieving chunk content |
| `cadi_import` | Importing new projects |
| `cadi_publish` | Sharing to registry |
| `cadi_suggest` | Getting AI suggestions |
| `cadi_build` | Building from manifests |
| `cadi_explain` | Understanding chunks |

## Project Context

This is the CADI project itself - a content-addressed development system that:
- Chunks code into reusable pieces
- Creates human-readable aliases
- Enables code reuse across projects
- Saves tokens for AI agents

Key directories:
- `cmd/cadi` - CLI tool
- `cmd/cadi-mcp-server` - MCP server for AI integration
- `internal/cadi-core` - Core library
- `internal/cadi-registry` - Registry client
- `seed-src/` - Example projects to import
