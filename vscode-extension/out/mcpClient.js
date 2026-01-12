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
exports.CadiMcpClient = void 0;
const vscode = __importStar(require("vscode"));
const net = __importStar(require("net"));
class CadiMcpClient {
    constructor(context) {
        this.context = context;
        this.messageId = 1;
        this.pendingRequests = new Map();
        this.onMessageHandlers = [];
    }
    async initialize(context) {
        // Initialize MCP connection if configured
        const config = vscode.workspace.getConfiguration('cadi');
        const autoConnect = config.get('mcp.autoConnect', false);
        if (autoConnect) {
            const port = config.get('mcp.port', 3000);
            try {
                await this.connect(port);
            }
            catch (error) {
                console.warn('Failed to auto-connect to MCP server:', error);
            }
        }
    }
    dispose() {
        this.disconnect();
    }
    async connect(port = 3000) {
        return new Promise((resolve, reject) => {
            this.socket = net.createConnection(port, 'localhost', () => {
                console.log('Connected to CADI MCP server');
                resolve();
            });
            this.socket.on('error', (error) => {
                console.error('MCP connection error:', error);
                reject(error);
            });
            this.socket.on('data', (data) => {
                this.handleData(data);
            });
            this.socket.on('close', () => {
                console.log('MCP connection closed');
                this.socket = undefined;
            });
        });
    }
    disconnect() {
        if (this.socket) {
            this.socket.end();
            this.socket = undefined;
        }
    }
    async sendRequest(method, params) {
        if (!this.socket) {
            throw new Error('MCP client not connected');
        }
        const id = this.messageId++;
        const message = {
            jsonrpc: '2.0',
            id,
            method,
            params
        };
        return new Promise((resolve, reject) => {
            this.pendingRequests.set(id, { resolve, reject });
            const data = JSON.stringify(message) + '\n';
            this.socket.write(data);
        });
    }
    async sendNotification(method, params) {
        if (!this.socket) {
            throw new Error('MCP client not connected');
        }
        const message = {
            jsonrpc: '2.0',
            method,
            params
        };
        const data = JSON.stringify(message) + '\n';
        this.socket.write(data);
    }
    onMessage(handler) {
        this.onMessageHandlers.push(handler);
    }
    handleData(data) {
        const messages = data.toString().split('\n').filter(line => line.trim());
        for (const messageStr of messages) {
            try {
                const message = JSON.parse(messageStr);
                // Handle responses to requests
                if (message.id && this.pendingRequests.has(message.id)) {
                    const { resolve, reject } = this.pendingRequests.get(message.id);
                    this.pendingRequests.delete(message.id);
                    if (message.error) {
                        reject(new Error(message.error.message));
                    }
                    else {
                        resolve(message.result);
                    }
                }
                // Notify message handlers
                for (const handler of this.onMessageHandlers) {
                    handler(message);
                }
            }
            catch (error) {
                console.error('Failed to parse MCP message:', error);
            }
        }
    }
    // CADI-specific MCP methods
    async searchChunks(query) {
        return this.sendRequest('cadi/search', { query });
    }
    async getChunk(id) {
        return this.sendRequest('cadi/getChunk', { id });
    }
    async importChunk(id, targetPath) {
        return this.sendRequest('cadi/import', { id, targetPath });
    }
    async buildProject(manifestPath) {
        return this.sendRequest('cadi/build', { manifestPath });
    }
    async getRegistryStats() {
        return this.sendRequest('cadi/registryStats');
    }
    async validateManifest(manifestPath) {
        return this.sendRequest('cadi/validate', { manifestPath });
    }
    // Auto-completion and IntelliSense support
    async getCompletions(context, position) {
        return this.sendRequest('cadi/completions', { context, position });
    }
    async getHoverInfo(symbol) {
        return this.sendRequest('cadi/hover', { symbol });
    }
    async getDefinition(symbol) {
        return this.sendRequest('cadi/definition', { symbol });
    }
    // Token usage tracking
    async getTokenUsage() {
        return this.sendRequest('cadi/tokenUsage');
    }
    async resetTokenCounter() {
        return this.sendRequest('cadi/resetTokens');
    }
}
exports.CadiMcpClient = CadiMcpClient;
//# sourceMappingURL=mcpClient.js.map