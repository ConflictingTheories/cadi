# Todo Suite Examples

This directory contains the "todo suite" - a collection of todo list implementations
that demonstrate CADI concepts.

## Applications

### todo-core
Core todo list logic, implemented in multiple languages:
- `rust/` - Rust implementation
- `python/` - Python implementation  
- `typescript/` - TypeScript implementation

### todo-cli
Command-line interface for the todo list:
- Uses the core logic from todo-core
- Demonstrates cross-language chunk dependencies

### todo-web
Web application frontend:
- Built with modern web technologies
- Can use WASM-compiled core for in-browser execution
- Or connect to todo-server via API

### todo-server
HTTP API server:
- RESTful API for todo operations
- Demonstrates container-cadi for deployment

## Manifests

Each component has a `cadi.yaml` manifest that defines:
- Source CADI chunks (the code)
- Build targets for different platforms
- Dependencies on other chunks

## Building

To build all examples for the web target:
```bash
cadi build --manifest examples/todo-suite/todo-web/cadi.yaml --target web
```

To run the demo:
```bash
cadi demo todo-suite --target all
```
