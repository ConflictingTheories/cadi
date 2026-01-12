# CADI VS Code Extension

A Visual Studio Code extension that integrates CADI (Content-Addressed Development Infrastructure) directly into your development workflow.

## Features

- **Registry Browser**: Browse and search CADI chunks from the sidebar
- **Code Import**: Import reusable code chunks with one click
- **Project Building**: Build projects using CADI's efficient chunk-based system
- **MCP Integration**: Connect to Model Context Protocol servers for enhanced AI assistance
- **Token Savings Tracking**: Monitor and display token savings achieved with CADI
- **Auto-Import**: Automatically import code chunks as you work

## Installation

1. Install the extension from the VS Code Marketplace
2. Configure your CADI environment:
   - Set the CADI CLI path in settings
   - Configure MCP server connection (optional)
   - Enable auto-import features

## Configuration

### Settings

- `cadi.enabled`: Enable/disable the CADI extension
- `cadi.cli.path`: Path to the CADI CLI executable
- `cadi.showTokenSavings`: Display token savings notifications
- `cadi.mcp.autoConnect`: Automatically connect to MCP server on startup
- `cadi.mcp.port`: MCP server port (default: 3000)

### Commands

- `CADI: Search Chunks`: Search for available code chunks
- `CADI: Build Project`: Build the current project with CADI
- `CADI: Import Code`: Import selected files as CADI chunks
- `CADI: View Registry`: Open the CADI registry browser
- `CADI: Create Manifest`: Create a new cadi.yaml manifest file
- `CADI: Install Extension`: Install a CADI extension

## Usage

### Browsing Chunks

1. Open the CADI Registry view from the sidebar
2. Browse categories: Atomizers, Build Backends, Registry Plugins, MCP Tools
3. Click on any chunk to see details and import options

### Importing Code

1. Select files in the explorer
2. Run "CADI: Import Code" command
3. Code is chunked and stored in the CADI registry

### Building Projects

1. Ensure you have a `cadi.yaml` manifest file
2. Run "CADI: Build Project" command
3. CADI will build your project using optimized chunks

## Requirements

- VS Code 1.74.0 or higher
- CADI CLI installed and configured
- Node.js 16+ (for MCP integration)

## Contributing

Contributions are welcome! Please see the CADI project repository for contribution guidelines.

## License

This extension is part of the CADI project and follows the same license terms.