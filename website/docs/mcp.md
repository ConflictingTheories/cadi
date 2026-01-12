# CADI MCP Integration Demo

This demonstrates how AI coding agents can use CADI's Model Context Protocol (MCP) integration for agentic development workflows.

## Example Agent Workflow

Here's how an AI assistant could use CADI tools to help with development:

### 1. Search for Existing Components

**Agent**: "I need to build a React todo app with local storage. Let me search for existing todo components."

```javascript
// Agent calls cadi_search tool
{
  "method": "tools/call",
  "params": {
    "name": "cadi_search",
    "arguments": {
      "query": "todo react component",
      "language": "typescript",
      "limit": 5
    }
  }
}
```

**CADI Response**: Returns chunk IDs for existing todo components.

### 2. Get Component Details

**Agent**: "Let me examine this todo component to understand its structure."

```javascript
// Agent calls cadi_get_chunk tool
{
  "method": "tools/call",
  "params": {
    "name": "cadi_get_chunk",
    "arguments": {
      "chunk_id": "chunk:sha256:b91ce4d7685ae2fb943924017f5f2631c1a3e6d16246424266dbb70951ea70e1",
      "include_source": true
    }
  }
}
```

**CADI Response**: Returns the complete source code and metadata for the chunk.

### 3. Build with Custom Styling

**Agent**: "Now I'll create a cyberpunk-themed version by reusing the core logic but adding custom styling."

```javascript
// Agent calls cadi_build tool
{
  "method": "tools/call",
  "params": {
    "name": "cadi_build",
    "arguments": {
      "manifest": "cyberpunk-todo/cadi.yaml",
      "target": "web",
      "prefer": "source"
    }
  }
}
```

### 4. Verify Build Integrity

**Agent**: "Let me verify that the build is trustworthy and hasn't been tampered with."

```javascript
// Agent calls cadi_verify tool
{
  "method": "tools/call",
  "params": {
    "name": "cadi_verify",
    "arguments": {
      "chunk_id": "chunk:sha256:newly_built_chunk_hash",
      "rebuild": false
    }
  }
}
```

## Benefits for Agentic Coding

1. **Component Discovery**: Agents can find and reuse existing, verified components
2. **Provenance Tracking**: Every component's origin and build process is verifiable
3. **Efficient Reuse**: Avoid rebuilding what's already available
4. **Trust & Security**: Cryptographic verification of component integrity
5. **Multi-Representation**: Components available in source, compiled, or containerized forms

## VS Code Integration

With the Copilot MCP extension installed and configured, AI assistants in VS Code can:

- Search CADI's chunk registry for relevant components
- Retrieve and analyze existing code chunks
- Build new applications using CADI manifests
- Verify the integrity of built components
- Get suggestions for components that might be useful for current tasks

This enables truly agentic development where AI assistants can discover, compose, and verify software components autonomously.