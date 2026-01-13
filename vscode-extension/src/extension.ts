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

export async function activate(context: vscode.ExtensionContext) {
    console.log('CADI extension is now active!');

    // Initialize components
    registryProvider = new CadiRegistryProvider(context);
    mcpClient = new CadiMcpClient(context);
    statusBar = new CadiStatusBar(context);
    adminPanelProvider = new CadiAdminPanelProvider(context);
    cadiCommands = new CadiCommands(context, registryProvider, mcpClient, statusBar);

    // Initialize MCP
    await mcpClient.initialize(context);

    // Register tree data providers
    context.subscriptions.push(
        vscode.window.registerTreeDataProvider('cadiRegistry', registryProvider)
    );
    context.subscriptions.push(
        vscode.window.registerTreeDataProvider('cadiAdminPanel', adminPanelProvider)
    );

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('cadi.searchChunks', () => cadiCommands.searchChunks())
    );
    context.subscriptions.push(
        vscode.commands.registerCommand('cadi.buildProject', () => cadiCommands.buildProject())
    );
    context.subscriptions.push(
        vscode.commands.registerCommand('cadi.importCode', () => cadiCommands.importCode())
    );
    context.subscriptions.push(
        vscode.commands.registerCommand('cadi.viewRegistry', () => cadiCommands.viewRegistry())
    );
    context.subscriptions.push(
        vscode.commands.registerCommand('cadi.createManifest', () => cadiCommands.createManifest())
    );
    context.subscriptions.push(
        vscode.commands.registerCommand('cadi.configureRegistry', () => cadiCommands.configureRegistry())
    );
    context.subscriptions.push(
        vscode.commands.registerCommand('cadi.installExtension', () => cadiCommands.installExtension())
    );
    context.subscriptions.push(
        vscode.commands.registerCommand('cadi.admin.createView', () => cadiCommands.adminCreateView())
    );
    context.subscriptions.push(
        vscode.commands.registerCommand('cadi.admin.debugDb', () => cadiCommands.adminDebugDb())
    );
    context.subscriptions.push(
        vscode.commands.registerCommand('cadi.admin.checkStatus', () => cadiCommands.adminCheckStatus())
    );

    console.log('CADI commands registered successfully!');

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