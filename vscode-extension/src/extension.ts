import * as vscode from 'vscode';
import { CadiCommands } from './commands';
import { CadiRegistryProvider } from './registryProvider';
import { CadiMcpClient } from './mcpClient';
import { CadiStatusBar } from './statusBar';
import { CadiAdminPanelProvider } from './adminPanel';

let cadiCommands: CadiCommands;
let registryProvider: CadiRegistryProvider;
let mcpClient: CadiMcpClient;
let statusBar: CadiStatusBar;
let adminPanelProvider: CadiAdminPanelProvider;

export function activate(context: vscode.ExtensionContext) {
    console.log('CADI extension is now active!');

    // Initialize components
    statusBar = new CadiStatusBar(context);
    registryProvider = new CadiRegistryProvider(context);
    mcpClient = new CadiMcpClient(context);
    cadiCommands = new CadiCommands(context, registryProvider, mcpClient, statusBar);
    adminPanelProvider = new CadiAdminPanelProvider(context);

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('cadi.searchChunks', cadiCommands.searchChunks.bind(cadiCommands)),
        vscode.commands.registerCommand('cadi.buildProject', cadiCommands.buildProject.bind(cadiCommands)),
        vscode.commands.registerCommand('cadi.importCode', cadiCommands.importCode.bind(cadiCommands)),
        vscode.commands.registerCommand('cadi.viewRegistry', cadiCommands.viewRegistry.bind(cadiCommands)),
        vscode.commands.registerCommand('cadi.createManifest', cadiCommands.createManifest.bind(cadiCommands)),
        vscode.commands.registerCommand('cadi.installExtension', cadiCommands.installExtension.bind(cadiCommands)),
        vscode.commands.registerCommand('cadi.importChunk', cadiCommands.importChunk.bind(cadiCommands)),
        vscode.commands.registerCommand('cadi.admin.createView', cadiCommands.adminCreateView.bind(cadiCommands)),
        vscode.commands.registerCommand('cadi.admin.debugDb', cadiCommands.adminDebugDb.bind(cadiCommands)),
        vscode.commands.registerCommand('cadi.admin.checkStatus', cadiCommands.adminCheckStatus.bind(cadiCommands))
    );

    // Register tree data provider for registry view
    vscode.window.registerTreeDataProvider('cadiRegistry', registryProvider);

    // Register tree data provider for admin panel
    vscode.window.registerTreeDataProvider('cadiAdminPanel', adminPanelProvider);

    // Register file system watcher for auto-import
    if (vscode.workspace.workspaceFolders) {
        const watcher = vscode.workspace.createFileSystemWatcher(
            '**/*.{rs,ts,js,py,java,go}',
            false, // ignoreCreateEvents
            false, // ignoreChangeEvents
            false  // ignoreDeleteEvents
        );

        watcher.onDidChange(async (uri) => {
            if (vscode.workspace.getConfiguration('cadi').get('autoImport')) {
                await cadiCommands.autoImportFile(uri);
            }
        });

        context.subscriptions.push(watcher);
    }

    // Initialize MCP client if enabled
    if (vscode.workspace.getConfiguration('cadi').get('enableMcp')) {
        mcpClient.initialize(context);
    }

    // Show welcome message
    showWelcomeMessage(context);
}

export function deactivate() {
    if (mcpClient) {
        mcpClient.dispose();
    }
    if (statusBar) {
        statusBar.dispose();
    }
}

async function showWelcomeMessage(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('cadi');
    const hasShownWelcome = context.globalState.get('cadi.welcomeShown', false);

    if (!hasShownWelcome) {
        const result = await vscode.window.showInformationMessage(
            'Welcome to CADI! Would you like to configure your CADI environment?',
            'Configure CADI',
            'Learn More',
            'Not Now'
        );

        switch (result) {
            case 'Configure CADI':
                vscode.commands.executeCommand('cadi.createManifest');
                break;
            case 'Learn More':
                vscode.env.openExternal(vscode.Uri.parse('https://cadi.dev'));
                break;
        }

        await context.globalState.update('cadi.welcomeShown', true);
    }
}