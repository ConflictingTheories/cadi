# CADI MCP Server

Model Context Protocol (MCP) server for LLM integration with CADI.

## Overview

The CADI MCP server enables AI agents and LLMs to interact with CADI's content-addressed development system. It provides tools for searching, retrieving, and managing code chunks.

## Installation

```bash
cargo install cadi-mcp-server
```

## Usage

### Stdio Mode (for Claude Desktop, etc.)

```bash
cadi-mcp-server
# or explicitly:
cadi-mcp-server --transport stdio
```

### HTTP Mode (for Docker/containers)

```bash
cadi-mcp-server --transport http --bind 0.0.0.0:9090
```

Or via environment variables:
```bash
CADI_MCP_TRANSPORT=http CADI_MCP_BIND_ADDRESS=0.0.0.0:9090 cadi-mcp-server
```

## MCP Tools

- `cadi_search` - Search for code chunks by query
- `cadi_get_chunk` - Retrieve chunk content by ID
- `cadi_resolve_alias` - Look up chunks by human-readable alias
- `cadi_import` - Import a project into CADI
- `cadi_publish` - Publish chunks to a registry
- `cadi_build` - Build from CADI manifests
- `cadi_explain` - Get AI-friendly explanations of chunks

## MCP Resources

- `cadi://guide` - CADI usage guide for agents
- `cadi://aliases` - List of cached chunk aliases

## Docker

```bash
docker run -p 9090:9090 cadi/mcp-server:latest
```

## License

MIT
