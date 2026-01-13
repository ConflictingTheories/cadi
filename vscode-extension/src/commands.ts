import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { spawn } from 'child_process';
import axios from 'axios';
import { CadiRegistryProvider } from './registryProvider';
import { CadiMcpClient } from './mcpClient';
import { CadiStatusBar } from './statusBar';

export class CadiCommands {
    constructor(
        private context: vscode.ExtensionContext,
        private registryProvider: CadiRegistryProvider,
        private mcpClient: CadiMcpClient,
        private statusBar: CadiStatusBar
    ) { }

    async searchChunks(): Promise<void> {
        const query = await vscode.window.showInputBox({
            prompt: 'Search CADI chunks',
            placeHolder: 'e.g., authentication middleware, database orm'
        });

        if (!query) {
            return;
        }

        this.statusBar.showProgress('Searching chunks...');

        try {
            const results = await this.searchRegistry(query);
            await this.showSearchResults(results);
        } catch (error) {
            vscode.window.showErrorMessage(`CADI search failed: ${error}`);
        } finally {
            this.statusBar.hideProgress();
        }
    }

    async buildProject(): Promise<void> {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            vscode.window.showErrorMessage('No workspace folder open');
            return;
        }

        this.statusBar.showProgress('Building project...');

        try {
            const result = await this.runCadiCommand(['build'], workspaceFolder.uri.fsPath);

            if (result.success) {
                vscode.window.showInformationMessage('CADI build completed successfully');

                // Show token savings if enabled
                if (vscode.workspace.getConfiguration('cadi').get('showTokenSavings') && result.output) {
                    const savings = this.extractTokenSavings(result.output);
                    if (savings) {
                        vscode.window.showInformationMessage(`🎉 ${savings} tokens saved with CADI!`);
                    }
                }
            } else {
                vscode.window.showErrorMessage(`CADI build failed: ${result.error}`);
            }
        } catch (error) {
            vscode.window.showErrorMessage(`CADI build failed: ${error}`);
        } finally {
            this.statusBar.hideProgress();
        }
    }

    async importCode(): Promise<void> {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            vscode.window.showErrorMessage('No workspace folder open');
            return;
        }

        const files = await vscode.window.showOpenDialog({
            canSelectFiles: true,
            canSelectFolders: true,
            canSelectMany: true,
            defaultUri: workspaceFolder.uri,
            filters: {
                'Source Files': ['rs', 'ts', 'js', 'py', 'java', 'go'],
                'All Files': ['*']
            }
        });

        if (!files || files.length === 0) {
            return;
        }

        this.statusBar.showProgress('Importing code...');

        try {
            const relativePaths = files.map(file =>
                vscode.workspace.asRelativePath(file)
            );

            const result = await this.runCadiCommand(
                ['import', ...relativePaths],
                workspaceFolder.uri.fsPath
            );

            if (result.success) {
                vscode.window.showInformationMessage(
                    `Successfully imported ${files.length} file(s) to CADI`
                );
            } else {
                vscode.window.showErrorMessage(`CADI import failed: ${result.error}`);
            }
        } catch (error) {
            vscode.window.showErrorMessage(`CADI import failed: ${error}`);
        } finally {
            this.statusBar.hideProgress();
        }
    }

    async viewRegistry(): Promise<void> {
        await vscode.commands.executeCommand('workbench.view.explorer');
        await vscode.commands.executeCommand('cadiRegistry.focus');
    }

    async createManifest(): Promise<void> {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            vscode.window.showErrorMessage('No workspace folder open');
            return;
        }

        const manifestPath = path.join(workspaceFolder.uri.fsPath, 'cadi.yaml');

        if (fs.existsSync(manifestPath)) {
            const overwrite = await vscode.window.showWarningMessage(
                'cadi.yaml already exists. Overwrite?',
                'Yes',
                'No'
            );

            if (overwrite !== 'Yes') {
                return;
            }
        }

        const template = `name: ${path.basename(workspaceFolder.uri.fsPath)}
version: "1.0.0"
description: "CADI project"

dependencies:
  # Add your CADI chunk dependencies here

build:
  target: "x86_64-linux"
  format: "binary"

# MCP configuration
mcp:
  enabled: true
  ghost_imports: true
`;

        try {
            fs.writeFileSync(manifestPath, template);
            const document = await vscode.workspace.openTextDocument(manifestPath);
            await vscode.window.showTextDocument(document);
            vscode.window.showInformationMessage('CADI manifest created successfully');
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to create manifest: ${error}`);
        }
    }

    async installExtension(): Promise<void> {
        const extensionName = await vscode.window.showInputBox({
            prompt: 'Enter CADI extension name',
            placeHolder: 'e.g., cadi-atomizer-java, cadi-backend-docker'
        });

        if (!extensionName) {
            return;
        }

        this.statusBar.showProgress(`Installing ${extensionName}...`);

        try {
            const result = await this.runCadiCommand(['extension', 'install', extensionName]);

            if (result.success) {
                vscode.window.showInformationMessage(
                    `Successfully installed CADI extension: ${extensionName}`
                );
                // Refresh registry view
                this.registryProvider.refresh();
            } else {
                vscode.window.showErrorMessage(`Failed to install extension: ${result.error}`);
            }
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to install extension: ${error}`);
        } finally {
            this.statusBar.hideProgress();
        }
    }

    async autoImportFile(uri: vscode.Uri): Promise<void> {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            return;
        }

        const relativePath = vscode.workspace.asRelativePath(uri);

        try {
            await this.runCadiCommand(['import', relativePath], workspaceFolder.uri.fsPath);
        } catch (error) {
            // Silent fail for auto-import to avoid spam
            console.warn(`Auto-import failed for ${relativePath}:`, error);
        }
    }

    async importChunk(chunkId: string): Promise<void> {
        this.statusBar.showProgress(`Importing chunk ${chunkId}...`);

        try {
            // Get chunk details from registry
            const chunk = await this.registryProvider.getChunkDetails(chunkId);
            if (!chunk) {
                vscode.window.showErrorMessage(`Chunk ${chunkId} not found`);
                return;
            }

            // For now, show information about the chunk
            // In a real implementation, this would download and integrate the chunk
            const result = await vscode.window.showInformationMessage(
                `Import chunk: ${chunk.name}?`,
                `Description: ${chunk.description}`,
                'Import',
                'Cancel'
            );

            if (result === 'Import') {
                // TODO: Implement actual chunk import logic
                vscode.window.showInformationMessage(
                    `Chunk "${chunk.name}" would be imported here. (Feature coming soon!)`
                );
            }
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to import chunk: ${error}`);
        } finally {
            this.statusBar.hideProgress();
        }
    }

    async adminCreateView(): Promise<void> {
        // Check if admin token is configured
        const config = vscode.workspace.getConfiguration('cadi');
        const adminToken = config.get('adminToken') as string;

        if (!adminToken) {
            vscode.window.showErrorMessage('Admin token not configured. Set cadi.adminToken in settings.');
            return;
        }

        const atomsInput = await vscode.window.showInputBox({
            prompt: 'Enter atom/chunk IDs (comma-separated)',
            placeHolder: 'chunk:sha256:abc123, chunk:sha256:def456'
        });

        if (!atomsInput) {
            return;
        }

        const atoms = atomsInput.split(',').map(id => id.trim()).filter(id => id.length > 0);

        if (atoms.length === 0) {
            vscode.window.showErrorMessage('No valid atom IDs provided');
            return;
        }

        const expansionDepth = await vscode.window.showInputBox({
            prompt: 'Expansion depth (optional)',
            placeHolder: '1'
        });

        const maxTokens = await vscode.window.showInputBox({
            prompt: 'Max tokens (optional)',
            placeHolder: '1024'
        });

        this.statusBar.showProgress('Creating virtual view...');

        try {
            const serverUrl = config.get('server.url', 'http://localhost:3000') as string;

            const requestBody = {
                atoms,
                expansion_depth: expansionDepth ? parseInt(expansionDepth) : undefined,
                max_tokens: maxTokens ? parseInt(maxTokens) : undefined
            };

            const response = await axios.post(`${serverUrl}/v1/views`, requestBody, {
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${adminToken}`
                }
            });

            const viewData = response.data;

            // Create webview to display the view
            this.showViewWebview(viewData);

        } catch (error) {
            vscode.window.showErrorMessage(`Failed to create view: ${error}`);
        } finally {
            this.statusBar.hideProgress();
        }
    }

    async adminDebugDb(): Promise<void> {
        // Check if admin token is configured
        const config = vscode.workspace.getConfiguration('cadi');
        const adminToken = config.get('adminToken') as string;

        if (!adminToken) {
            vscode.window.showErrorMessage('Admin token not configured. Set cadi.adminToken in settings.');
            return;
        }

        this.statusBar.showProgress('Loading database info...');

        try {
            const serverUrl = config.get('server.url', 'http://localhost:3000') as string;

            // Get stats
            const statsResponse = await axios.get(`${serverUrl}/v1/stats`, {
                headers: {
                    'Authorization': `Bearer ${adminToken}`
                }
            });

            const stats = statsResponse.data;

            // Create webview to display DB debug info
            this.showDbDebugWebview(stats, serverUrl, adminToken);

        } catch (error) {
            vscode.window.showErrorMessage(`Failed to load DB info: ${error}`);
        } finally {
            this.statusBar.hideProgress();
        }
    }

    async adminCheckStatus(): Promise<void> {
        const config = vscode.workspace.getConfiguration('cadi');
        const serverUrl = config.get('server.url', 'http://localhost:3000') as string;
        const adminToken = config.get('adminToken') as string;

        this.statusBar.showProgress('Checking server status...');

        try {
            // Try to connect to the server
            const response = await axios.get(`${serverUrl}/health`, {
                timeout: 5000,
                headers: adminToken ? { 'Authorization': `Bearer ${adminToken}` } : {}
            });

            if (response.status === 200) {
                const message = adminToken
                    ? `✅ CADI server is running at ${serverUrl} (authenticated)`
                    : `✅ CADI server is running at ${serverUrl} (unauthenticated)`;
                vscode.window.showInformationMessage(message);
            } else {
                vscode.window.showWarningMessage(`⚠️ CADI server responded with status ${response.status}`);
            }
        } catch (error) {
            const err = error as any;
            if (err.code === 'ECONNREFUSED') {
                vscode.window.showErrorMessage(`❌ Cannot connect to CADI server at ${serverUrl}. Is the server running?`);
            } else if (err.code === 'ENOTFOUND') {
                vscode.window.showErrorMessage(`❌ Cannot resolve host for ${serverUrl}. Check your server URL configuration.`);
            } else {
                vscode.window.showErrorMessage(`❌ Server check failed: ${err.message || error}`);
            }
        } finally {
            this.statusBar.hideProgress();
        }
    }

    private showViewWebview(viewData: any): void {
        const panel = vscode.window.createWebviewPanel(
            'cadiView',
            'CADI Virtual View',
            vscode.ViewColumn.One,
            {
                enableScripts: true,
                localResourceRoots: []
            }
        );

        panel.webview.html = this.getViewHtml(viewData);
    }

    private showDbDebugWebview(stats: any, serverUrl: string, adminToken: string): void {
        const panel = vscode.window.createWebviewPanel(
            'cadiDbDebug',
            'CADI Database Debug',
            vscode.ViewColumn.One,
            {
                enableScripts: true,
                localResourceRoots: []
            }
        );

        panel.webview.html = this.getDbDebugHtml(stats, serverUrl, adminToken);

        // Handle messages from webview
        panel.webview.onDidReceiveMessage(async (message) => {
            switch (message.type) {
                case 'getNodes':
                    try {
                        const response = await axios.get(`${serverUrl}/v1/admin/nodes`, {
                            headers: {
                                'Authorization': `Bearer ${adminToken}`
                            }
                        });
                        panel.webview.postMessage({ type: 'nodes', data: response.data });
                    } catch (error: any) {
                        panel.webview.postMessage({ type: 'error', message: error.message || error.toString() });
                    }
                    break;

                case 'getEdges':
                    try {
                        const response = await axios.get(`${serverUrl}/v1/admin/edges`, {
                            headers: {
                                'Authorization': `Bearer ${adminToken}`
                            }
                        });
                        panel.webview.postMessage({ type: 'edges', data: response.data });
                    } catch (error: any) {
                        panel.webview.postMessage({ type: 'error', message: error.message || error.toString() });
                    }
                    break;
            }
        });
    }

    private getViewHtml(viewData: any): string {
        return `
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>CADI Virtual View</title>
    <style>
        body { font-family: var(--vscode-font-family); margin: 20px; }
        .header { margin-bottom: 20px; }
        .metadata { background: var(--vscode-textBlockQuote-background); padding: 10px; margin: 10px 0; border-left: 4px solid var(--vscode-textBlockQuote-border); }
        .atoms { margin: 10px 0; }
        .atom { display: inline-block; background: var(--vscode-badge-background); color: var(--vscode-badge-foreground); padding: 2px 6px; margin: 2px; border-radius: 3px; }
        .ghost { opacity: 0.7; }
        .source { background: var(--vscode-textCodeBlock-background); border: 1px solid var(--vscode-textBlockQuote-border); padding: 10px; margin: 10px 0; font-family: monospace; white-space: pre-wrap; }
        .fragments { margin: 10px 0; }
        .fragment { border: 1px solid var(--vscode-list-inactiveSelectionBackground); padding: 5px; margin: 5px 0; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Virtual View</h1>
        <div class="metadata">
            <strong>Language:</strong> ${viewData.language}<br>
            <strong>Token Estimate:</strong> ${viewData.token_estimate}<br>
            <strong>Truncated:</strong> ${viewData.truncated}<br>
            <strong>Explanation:</strong> ${viewData.explanation}
        </div>
    </div>
    
    <div class="atoms">
        <h3>Atoms (${viewData.atoms.length})</h3>
        ${viewData.atoms.map((atom: string) => `<span class="atom">${atom}</span>`).join('')}
        
        <h3>Ghost Atoms (${viewData.ghost_atoms.length})</h3>
        ${viewData.ghost_atoms.map((atom: string) => `<span class="atom ghost">${atom}</span>`).join('')}
    </div>
    
    <div class="source">
        <h3>Source Code</h3>
        ${viewData.source}
    </div>
    
    <div class="fragments">
        <h3>Fragments</h3>
        ${viewData.fragments ? viewData.fragments.map((frag: any) => `
            <div class="fragment">
                <strong>${frag.chunk_id}</strong> (${frag.alias || 'no alias'})<br>
                Lines ${frag.start_line}-${frag.end_line}, ${frag.token_count} tokens<br>
                Reason: ${frag.inclusion_reason}<br>
                Defines: ${frag.defines.join(', ')}
            </div>
        `).join('') : 'No fragment data available'}
    </div>
</body>
</html>`;
    }

    private getDbDebugHtml(stats: any, serverUrl: string, adminToken: string): string {
        return `
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>CADI Database Debug</title>
    <style>
        body { font-family: var(--vscode-font-family); margin: 20px; }
        .stats { background: var(--vscode-textBlockQuote-background); padding: 10px; margin: 10px 0; border-left: 4px solid var(--vscode-textBlockQuote-border); }
        .section { margin: 20px 0; }
        button { background: var(--vscode-button-background); color: var(--vscode-button-foreground); border: none; padding: 8px 16px; cursor: pointer; }
        button:hover { background: var(--vscode-button-hoverBackground); }
        .data { background: var(--vscode-textCodeBlock-background); border: 1px solid var(--vscode-textBlockQuote-border); padding: 10px; margin: 10px 0; font-family: monospace; max-height: 400px; overflow-y: auto; }
        table { width: 100%; border-collapse: collapse; }
        th, td { border: 1px solid var(--vscode-list-inactiveSelectionBackground); padding: 5px; text-align: left; }
        th { background: var(--vscode-titleBar-activeBackground); }
    </style>
</head>
<body>
    <h1>Database Debug</h1>
    
    <div class="stats">
        <h3>Statistics</h3>
        <pre>${JSON.stringify(stats, null, 2)}</pre>
    </div>
    
    <div class="section">
        <h3>Nodes</h3>
        <button onclick="loadNodes()">Load Nodes</button>
        <div id="nodes" class="data">Click "Load Nodes" to fetch data...</div>
    </div>
    
    <div class="section">
        <h3>Edges</h3>
        <button onclick="loadEdges()">Load Edges</button>
        <div id="edges" class="data">Click "Load Edges" to fetch data...</div>
    </div>
    
    <script>
        const vscode = acquireVsCodeApi();
        
        function loadNodes() {
            document.getElementById('nodes').innerHTML = 'Loading...';
            vscode.postMessage({ type: 'getNodes' });
        }
        
        function loadEdges() {
            document.getElementById('edges').innerHTML = 'Loading...';
            vscode.postMessage({ type: 'getEdges' });
        }
        
        window.addEventListener('message', event => {
            const message = event.data;
            switch (message.type) {
                case 'nodes':
                    displayNodes(message.data);
                    break;
                case 'edges':
                    displayEdges(message.data);
                    break;
                case 'error':
                    alert('Error: ' + message.message);
                    break;
            }
        });
        
        function displayNodes(nodes) {
            const container = document.getElementById('nodes');
            if (!nodes || nodes.length === 0) {
                container.innerHTML = 'No nodes found.';
                return;
            }
            
            let html = '<table><thead><tr><th>Chunk ID</th><th>Language</th><th>Size</th><th>Defines</th><th>References</th></tr></thead><tbody>';
            nodes.forEach(node => {
                html += '<tr>';
                html += '<td>' + (node.chunk_id || '') + '</td>';
                html += '<td>' + (node.language || '') + '</td>';
                html += '<td>' + (node.size || 0) + '</td>';
                html += '<td>' + (node.defines ? node.defines.join(', ') : '') + '</td>';
                html += '<td>' + (node.references ? node.references.join(', ') : '') + '</td>';
                html += '</tr>';
            });
            html += '</tbody></table>';
            container.innerHTML = html;
        }
        
        function displayEdges(edges) {
            const container = document.getElementById('edges');
            if (!edges || edges.length === 0) {
                container.innerHTML = 'No edges found.';
                return;
            }
            
            let html = '<table><thead><tr><th>From</th><th>To</th><th>Type</th></tr></thead><tbody>';
            edges.forEach(edge => {
                html += '<tr>';
                html += '<td>' + (edge.from || '') + '</td>';
                html += '<td>' + (edge.to || '') + '</td>';
                html += '<td>' + (edge.edge_type || '') + '</td>';
                html += '</tr>';
            });
            html += '</tbody></table>';
            container.innerHTML = html;
        }
    </script>
</body>
</html>`;
    }

    private async searchRegistry(query: string): Promise<any[]> {
        // This would integrate with the CADI registry API
        // For now, return mock results
        return [
            {
                id: 'auth-middleware-v1.2.3',
                name: 'Authentication Middleware',
                description: 'JWT-based authentication for web APIs',
                language: 'typescript',
                downloads: 1250
            },
            {
                id: 'db-orm-v2.1.0',
                name: 'Database ORM',
                description: 'Type-safe database operations',
                language: 'rust',
                downloads: 890
            }
        ].filter(item =>
            item.name.toLowerCase().includes(query.toLowerCase()) ||
            item.description.toLowerCase().includes(query.toLowerCase())
        );
    }

    private async showSearchResults(results: any[]): Promise<void> {
        if (results.length === 0) {
            vscode.window.showInformationMessage('No chunks found matching your query');
            return;
        }

        const items = results.map(result => ({
            label: result.name,
            detail: result.description,
            description: `${result.language} • ${result.downloads} downloads`,
            result
        }));

        const selected = await vscode.window.showQuickPick(items, {
            matchOnDetail: true,
            matchOnDescription: true,
            placeHolder: 'Select a chunk to import'
        });

        if (selected) {
            // TODO: Implement chunk import
            vscode.window.showInformationMessage(
                `Selected: ${selected.result.name} (${selected.result.id})`
            );
        }
    }

    private async runCadiCommand(args: string[], cwd?: string): Promise<{ success: boolean, output?: string, error?: string }> {
        const config = vscode.workspace.getConfiguration('cadi');
        const cadiPath = config.get('cli.path', 'cadi');

        return new Promise((resolve) => {
            const workingDir = cwd || vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;

            vscode.window.showInformationMessage(`Running: ${cadiPath} ${args.join(' ')}`);

            const child = spawn(cadiPath, args, {
                cwd: workingDir,
                stdio: ['pipe', 'pipe', 'pipe']
            });

            let stdout = '';
            let stderr = '';

            child.stdout?.on('data', (data: Buffer) => {
                stdout += data.toString();
            });

            child.stderr?.on('data', (data: Buffer) => {
                stderr += data.toString();
            });

            child.on('close', (code: number) => {
                if (code === 0) {
                    resolve({
                        success: true,
                        output: stdout
                    });
                } else {
                    resolve({
                        success: false,
                        output: stdout,
                        error: stderr || `Command failed with exit code ${code}`
                    });
                }
            });

            child.on('error', (error: Error) => {
                resolve({
                    success: false,
                    error: error.message
                });
            });
        });
    }

    private extractTokenSavings(output: string): string | null {
        const match = output.match(/(\d+)% token savings/);
        return match ? `${match[1]}% token savings` : null;
    }
}