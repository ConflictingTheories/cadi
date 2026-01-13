# CADI VS Code Extension

A Visual Studio Code extension that integrates CADI (Content-Addressed Development Infrastructure) directly into your development workflow.

## Features

- **Registry Browser**: Browse and search CADI chunks from the sidebar
- **Code Import**: Import reusable code chunks with one click
- **Project Building**: Build projects using CADI's efficient chunk-based system
- **MCP Integration**: Connect to Model Context Protocol servers for enhanced AI assistance
- **Token Savings Tracking**: Monitor and display token savings achieved with CADI
- **Auto-Import**: Automatically import code chunks as you work
- **Admin Tools**: Debug database and visualize virtual views (admin only)

## Installation

1. Install the extension from the VS Code Marketplace or from the packaged `.vsix` file:
   ```bash
   code --install-extension cadi-2.0.0.vsix
   ```
2. Configure your CADI environment:
   - Set the CADI CLI path in settings
   - Configure MCP server connection (optional)
   - Enable auto-import features
   - For admin features: configure server URL and admin token

## Configuration

### Settings

- `cadi.enabled`: Enable/disable the CADI extension
- `cadi.cli.path`: Path to the CADI CLI executable
- `cadi.showTokenSavings`: Display token savings notifications
- `cadi.mcp.autoConnect`: Automatically connect to MCP server on startup
- `cadi.mcp.port`: MCP server port (default: 3000)
- `cadi.adminToken`: Admin token for accessing admin-only features
- `cadi.server.url`: URL of the CADI server (default: http://localhost:3000)

### Commands

- `CADI: Search Chunks`: Search for available code chunks
- `CADI: Build Project`: Build the current project with CADI
- `CADI: Import Code`: Import selected files as CADI chunks
- `CADI: View Registry`: Open the CADI registry browser
- `CADI: Create Manifest`: Create a new cadi.yaml manifest file
- `CADI: Install Extension`: Install a CADI extension
- `CADI Admin: Create Virtual View`: Create and visualize virtual views (admin only)
- `CADI Admin: Debug Database`: Debug and inspect database contents (admin only)
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

### Admin Features

The admin features require proper configuration and are intended for system administrators only.

#### Setup Admin Access

1. Set your admin token in VS Code settings: `cadi.adminToken`
2. Configure the CADI server URL: `cadi.server.url` (default: http://localhost:3000)
3. Ensure the CADI server is running with admin authentication enabled

#### Creating Virtual Views

1. Run "CADI Admin: Create Virtual View" command
2. Enter comma-separated chunk IDs (e.g., `chunk:sha256:abc123, chunk:sha256:def456`)
3. Optionally set expansion depth and max tokens
4. View the assembled source code, metadata, and fragment details in the webview

#### Debugging Database

1. Run "CADI Admin: Debug Database" command
2. View database statistics and click "Load Nodes" or "Load Edges"
3. Browse through nodes and edges in interactive tables
4. Inspect chunk metadata, dependencies, and relationships

## Requirements

- VS Code 1.74.0 or higher
- CADI CLI installed and configured
- Node.js 16+ (for MCP integration)
- For admin features: CADI server with admin authentication configured

## Contributing

Contributions are welcome! Please see the CADI project repository for contribution guidelines.

## License

This extension is part of the CADI project and follows the same license terms.