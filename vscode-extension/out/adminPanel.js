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
exports.CadiAdminPanelProvider = void 0;
const vscode = __importStar(require("vscode"));
class CadiAdminPanelProvider {
    constructor(context) {
        this.context = context;
    }
    getTreeItem(element) {
        return element;
    }
    getChildren(element) {
        if (!element) {
            // Root level items
            return Promise.resolve([
                new AdminItem('Virtual Views', 'Create and manage virtual views', vscode.TreeItemCollapsibleState.None, {
                    command: 'cadi.admin.createView',
                    title: 'Create Virtual View'
                }),
                new AdminItem('Database Debug', 'Inspect database contents', vscode.TreeItemCollapsibleState.None, {
                    command: 'cadi.admin.debugDb',
                    title: 'Debug Database'
                }),
                new AdminItem('Server Status', 'Check server connectivity', vscode.TreeItemCollapsibleState.None, {
                    command: 'cadi.admin.checkStatus',
                    title: 'Check Server Status'
                })
            ]);
        }
        return Promise.resolve([]);
    }
}
exports.CadiAdminPanelProvider = CadiAdminPanelProvider;
class AdminItem extends vscode.TreeItem {
    constructor(label, tooltip, collapsibleState, command) {
        super(label, collapsibleState);
        this.label = label;
        this.tooltip = tooltip;
        this.collapsibleState = collapsibleState;
        this.command = command;
        this.tooltip = tooltip;
        this.command = command;
        // Set appropriate icons
        if (label === 'Virtual Views') {
            this.iconPath = new vscode.ThemeIcon('file-code');
        }
        else if (label === 'Database Debug') {
            this.iconPath = new vscode.ThemeIcon('database');
        }
        else if (label === 'Server Status') {
            this.iconPath = new vscode.ThemeIcon('server');
        }
    }
}
//# sourceMappingURL=adminPanel.js.map