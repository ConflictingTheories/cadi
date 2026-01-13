import * as vscode from 'vscode';
import axios from 'axios';

export class CadiAdminPanelProvider implements vscode.TreeDataProvider<AdminItem> {
    constructor(private context: vscode.ExtensionContext) { }

    getTreeItem(element: AdminItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: AdminItem): Thenable<AdminItem[]> {
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

class AdminItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly tooltip: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly command?: vscode.Command
    ) {
        super(label, collapsibleState);
        this.tooltip = tooltip;
        this.command = command;

        // Set appropriate icons
        if (label === 'Virtual Views') {
            this.iconPath = new vscode.ThemeIcon('file-code');
        } else if (label === 'Database Debug') {
            this.iconPath = new vscode.ThemeIcon('database');
        } else if (label === 'Server Status') {
            this.iconPath = new vscode.ThemeIcon('server');
        }
    }
}