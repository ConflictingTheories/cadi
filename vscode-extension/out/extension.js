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
let cadiCommands;
let registryProvider;
let mcpClient;
let statusBar;
function activate(context) {
    console.log('CADI extension is now active!');
    // Initialize components
    statusBar = new statusBar_1.CadiStatusBar(context);
    registryProvider = new registryProvider_1.CadiRegistryProvider(context);
    mcpClient = new mcpClient_1.CadiMcpClient(context);
    cadiCommands = new commands_1.CadiCommands(context, registryProvider, mcpClient, statusBar);
    // Register commands
    context.subscriptions.push(vscode.commands.registerCommand('cadi.searchChunks', cadiCommands.searchChunks.bind(cadiCommands)), vscode.commands.registerCommand('cadi.buildProject', cadiCommands.buildProject.bind(cadiCommands)), vscode.commands.registerCommand('cadi.importCode', cadiCommands.importCode.bind(cadiCommands)), vscode.commands.registerCommand('cadi.viewRegistry', cadiCommands.viewRegistry.bind(cadiCommands)), vscode.commands.registerCommand('cadi.createManifest', cadiCommands.createManifest.bind(cadiCommands)), vscode.commands.registerCommand('cadi.installExtension', cadiCommands.installExtension.bind(cadiCommands)), vscode.commands.registerCommand('cadi.importChunk', cadiCommands.importChunk.bind(cadiCommands)));
    // Register tree data provider for registry view
    vscode.window.registerTreeDataProvider('cadiRegistry', registryProvider);
    // Register file system watcher for auto-import
    if (vscode.workspace.workspaceFolders) {
        const watcher = vscode.workspace.createFileSystemWatcher('**/*.{rs,ts,js,py,java,go}', false, // ignoreCreateEvents
        false, // ignoreChangeEvents
        false // ignoreDeleteEvents
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