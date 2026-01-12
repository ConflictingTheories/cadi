import * as vscode from 'vscode';

export class CadiStatusBar {
    private statusBarItem: vscode.StatusBarItem;
    private progressItem?: vscode.StatusBarItem;

    constructor(private context: vscode.ExtensionContext) {
        this.statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
        this.statusBarItem.command = 'cadi.viewRegistry';
        this.context.subscriptions.push(this.statusBarItem);

        this.updateStatus();
    }

    private updateStatus(): void {
        const config = vscode.workspace.getConfiguration('cadi');
        const enabled = config.get('enabled', true);

        if (enabled) {
            this.statusBarItem.text = '$(package) CADI';
            this.statusBarItem.tooltip = 'CADI Registry - Click to browse chunks';
            this.statusBarItem.color = undefined;
        } else {
            this.statusBarItem.text = '$(package) CADI (Disabled)';
            this.statusBarItem.tooltip = 'CADI is disabled - Click to enable';
            this.statusBarItem.color = new vscode.ThemeColor('statusBarItem.warningForeground');
        }

        this.statusBarItem.show();
    }

    showProgress(message: string): void {
        if (this.progressItem) {
            this.progressItem.dispose();
        }

        this.progressItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 99);
        this.progressItem.text = `$(sync~spin) ${message}`;
        this.progressItem.tooltip = 'CADI operation in progress...';
        this.progressItem.show();

        this.context.subscriptions.push(this.progressItem);
    }

    hideProgress(): void {
        if (this.progressItem) {
            this.progressItem.dispose();
            this.progressItem = undefined;
        }
    }

    updateTokenSavings(savings: number): void {
        if (savings > 0) {
            this.statusBarItem.text = `$(package) CADI (${savings}% saved)`;
            this.statusBarItem.tooltip = `CADI Registry - ${savings}% token savings achieved`;
            this.statusBarItem.color = new vscode.ThemeColor('statusBarItem.successForeground');

            // Reset after 5 seconds
            setTimeout(() => {
                this.updateStatus();
            }, 5000);
        }
    }

    showError(message: string): void {
        this.statusBarItem.text = '$(error) CADI Error';
        this.statusBarItem.tooltip = message;
        this.statusBarItem.color = new vscode.ThemeColor('statusBarItem.errorForeground');

        // Reset after 10 seconds
        setTimeout(() => {
            this.updateStatus();
        }, 10000);
    }

    showWarning(message: string): void {
        this.statusBarItem.text = '$(warning) CADI Warning';
        this.statusBarItem.tooltip = message;
        this.statusBarItem.color = new vscode.ThemeColor('statusBarItem.warningForeground');

        // Reset after 5 seconds
        setTimeout(() => {
            this.updateStatus();
        }, 5000);
    }

    showSuccess(message: string): void {
        this.statusBarItem.text = '$(check) CADI Success';
        this.statusBarItem.tooltip = message;
        this.statusBarItem.color = new vscode.ThemeColor('statusBarItem.successForeground');

        // Reset after 3 seconds
        setTimeout(() => {
            this.updateStatus();
        }, 3000);
    }

    dispose(): void {
        this.statusBarItem.dispose();
        if (this.progressItem) {
            this.progressItem.dispose();
        }
    }
}