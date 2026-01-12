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
exports.CadiStatusBar = void 0;
const vscode = __importStar(require("vscode"));
class CadiStatusBar {
    constructor(context) {
        this.context = context;
        this.statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
        this.statusBarItem.command = 'cadi.viewRegistry';
        this.context.subscriptions.push(this.statusBarItem);
        this.updateStatus();
    }
    updateStatus() {
        const config = vscode.workspace.getConfiguration('cadi');
        const enabled = config.get('enabled', true);
        if (enabled) {
            this.statusBarItem.text = '$(package) CADI';
            this.statusBarItem.tooltip = 'CADI Registry - Click to browse chunks';
            this.statusBarItem.color = undefined;
        }
        else {
            this.statusBarItem.text = '$(package) CADI (Disabled)';
            this.statusBarItem.tooltip = 'CADI is disabled - Click to enable';
            this.statusBarItem.color = new vscode.ThemeColor('statusBarItem.warningForeground');
        }
        this.statusBarItem.show();
    }
    showProgress(message) {
        if (this.progressItem) {
            this.progressItem.dispose();
        }
        this.progressItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 99);
        this.progressItem.text = `$(sync~spin) ${message}`;
        this.progressItem.tooltip = 'CADI operation in progress...';
        this.progressItem.show();
        this.context.subscriptions.push(this.progressItem);
    }
    hideProgress() {
        if (this.progressItem) {
            this.progressItem.dispose();
            this.progressItem = undefined;
        }
    }
    updateTokenSavings(savings) {
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
    showError(message) {
        this.statusBarItem.text = '$(error) CADI Error';
        this.statusBarItem.tooltip = message;
        this.statusBarItem.color = new vscode.ThemeColor('statusBarItem.errorForeground');
        // Reset after 10 seconds
        setTimeout(() => {
            this.updateStatus();
        }, 10000);
    }
    showWarning(message) {
        this.statusBarItem.text = '$(warning) CADI Warning';
        this.statusBarItem.tooltip = message;
        this.statusBarItem.color = new vscode.ThemeColor('statusBarItem.warningForeground');
        // Reset after 5 seconds
        setTimeout(() => {
            this.updateStatus();
        }, 5000);
    }
    showSuccess(message) {
        this.statusBarItem.text = '$(check) CADI Success';
        this.statusBarItem.tooltip = message;
        this.statusBarItem.color = new vscode.ThemeColor('statusBarItem.successForeground');
        // Reset after 3 seconds
        setTimeout(() => {
            this.updateStatus();
        }, 3000);
    }
    dispose() {
        this.statusBarItem.dispose();
        if (this.progressItem) {
            this.progressItem.dispose();
        }
    }
}
exports.CadiStatusBar = CadiStatusBar;
//# sourceMappingURL=statusBar.js.map