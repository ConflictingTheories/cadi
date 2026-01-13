"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const vscode = __importStar(require("vscode"));
const commands_1 = require("./commands");
const registryProvider_1 = require("./registryProvider");
const mcpClient_1 = require("./mcpClient");
const statusBar_1 = require("./statusBar");
const adminPanel_1 = require("./adminPanel");
let cadiCommands;
let registryProvider;
let mcpClient;
let statusBar;
let adminPanelProvider;
async function activate(context) {
    console.log('CADI extension is now active!');
    // Initialize components
    registryProvider = new registryProvider_1.CadiRegistryProvider(context);
    mcpClient = new mcpClient_1.CadiMcpClient(context);
    statusBar = new statusBar_1.CadiStatusBar(context);
    adminPanelProvider = new adminPanel_1.CadiAdminPanelProvider(context);
    cadiCommands = new commands_1.CadiCommands(context, registryProvider, mcpClient, statusBar);
    // Initialize MCP
    await mcpClient.initialize(context);
    // Register tree data providers
    context.subscriptions.push(vscode.window.registerTreeDataProvider('cadiRegistry', registryProvider));
    context.subscriptions.push(vscode.window.registerTreeDataProvider('cadiAdminPanel', adminPanelProvider));
    // Register commands
    context.subscriptions.push(vscode.commands.registerCommand('cadi.searchChunks', () => cadiCommands.searchChunks()));
    context.subscriptions.push(vscode.commands.registerCommand('cadi.buildProject', () => cadiCommands.buildProject()));
    context.subscriptions.push(vscode.commands.registerCommand('cadi.importCode', () => cadiCommands.importCode()));
    context.subscriptions.push(vscode.commands.registerCommand('cadi.viewRegistry', () => cadiCommands.viewRegistry()));
    context.subscriptions.push(vscode.commands.registerCommand('cadi.createManifest', () => cadiCommands.createManifest()));
    context.subscriptions.push(vscode.commands.registerCommand('cadi.configureRegistry', () => cadiCommands.configureRegistry()));
    context.subscriptions.push(vscode.commands.registerCommand('cadi.installExtension', () => cadiCommands.installExtension()));
    context.subscriptions.push(vscode.commands.registerCommand('cadi.admin.createView', () => cadiCommands.adminCreateView()));
    context.subscriptions.push(vscode.commands.registerCommand('cadi.admin.debugDb', () => cadiCommands.adminDebugDb()));
    context.subscriptions.push(vscode.commands.registerCommand('cadi.admin.checkStatus', () => cadiCommands.adminCheckStatus()));
    console.log('CADI commands registered successfully!');
    // Show welcome message
    showWelcomeMessage(context);
}
exports.activate = activate;
function deactivate() {
    if (mcpClient) {
        mcpClient.dispose();
    }
    if (statusBar) {
        statusBar.dispose();
    }
}
exports.deactivate = deactivate;
async function showWelcomeMessage(context) {
    const config = vscode.workspace.getConfiguration('cadi');
    const hasShownWelcome = context.globalState.get('cadi.welcomeShown', false);
    if (!hasShownWelcome) {
        const result = await vscode.window.showInformationMessage('Welcome to CADI! Would you like to configure your CADI environment?', 'Configure CADI', 'Learn More', 'Not Now');
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
//# sourceMappingURL=extension.js.map