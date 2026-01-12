import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
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

        const terminal = vscode.window.createTerminal('CADI');
        const command = `${cadiPath} ${args.join(' ')}`;

        return new Promise((resolve) => {
            // This is a simplified implementation
            // In a real extension, you'd use child_process or VS Code tasks
            vscode.window.showInformationMessage(`Running: ${command}`);

            // Mock success for now
            setTimeout(() => {
                resolve({
                    success: true,
                    output: 'Build completed successfully. 40% token savings achieved.'
                });
            }, 2000);
        });
    }

    private extractTokenSavings(output: string): string | null {
        const match = output.match(/(\d+)% token savings/);
        return match ? `${match[1]}% token savings` : null;
    }
}