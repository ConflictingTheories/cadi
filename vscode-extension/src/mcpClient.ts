import * as vscode from 'vscode';
import * as net from 'net';

export interface McpMessage {
    jsonrpc: '2.0';
    id?: number;
    method?: string;
    params?: any;
    result?: any;
    error?: any;
}

export class CadiMcpClient {
    private socket?: net.Socket;
    private messageId = 1;
    private pendingRequests = new Map<number, { resolve: Function; reject: Function }>();
    private onMessageHandlers: ((message: McpMessage) => void)[] = [];

    constructor(private context: vscode.ExtensionContext) { }

    async initialize(context: vscode.ExtensionContext): Promise<void> {
        // Initialize MCP connection if configured
        const config = vscode.workspace.getConfiguration('cadi');
        const autoConnect = config.get('mcp.autoConnect', false);

        if (autoConnect) {
            const port = config.get('mcp.port', 3000);
            try {
                await this.connect(port);
            } catch (error) {
                console.warn('Failed to auto-connect to MCP server:', error);
            }
        }
    }

    dispose(): void {
        this.disconnect();
    }

    async connect(port: number = 3000): Promise<void> {
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

    disconnect(): void {
        if (this.socket) {
            this.socket.end();
            this.socket = undefined;
        }
    }

    async sendRequest(method: string, params?: any): Promise<any> {
        if (!this.socket) {
            throw new Error('MCP client not connected');
        }

        const id = this.messageId++;
        const message: McpMessage = {
            jsonrpc: '2.0',
            id,
            method,
            params
        };

        return new Promise((resolve, reject) => {
            this.pendingRequests.set(id, { resolve, reject });

            const data = JSON.stringify(message) + '\n';
            this.socket!.write(data);
        });
    }

    async sendNotification(method: string, params?: any): Promise<void> {
        if (!this.socket) {
            throw new Error('MCP client not connected');
        }

        const message: McpMessage = {
            jsonrpc: '2.0',
            method,
            params
        };

        const data = JSON.stringify(message) + '\n';
        this.socket!.write(data);
    }

    onMessage(handler: (message: McpMessage) => void): void {
        this.onMessageHandlers.push(handler);
    }

    private handleData(data: Buffer): void {
        const messages = data.toString().split('\n').filter(line => line.trim());

        for (const messageStr of messages) {
            try {
                const message: McpMessage = JSON.parse(messageStr);

                // Handle responses to requests
                if (message.id && this.pendingRequests.has(message.id)) {
                    const { resolve, reject } = this.pendingRequests.get(message.id)!;
                    this.pendingRequests.delete(message.id);

                    if (message.error) {
                        reject(new Error(message.error.message));
                    } else {
                        resolve(message.result);
                    }
                }

                // Notify message handlers
                for (const handler of this.onMessageHandlers) {
                    handler(message);
                }
            } catch (error) {
                console.error('Failed to parse MCP message:', error);
            }
        }
    }

    // CADI-specific MCP methods
    async searchChunks(query: string): Promise<any[]> {
        return this.sendRequest('cadi/search', { query });
    }

    async getChunk(id: string): Promise<any> {
        return this.sendRequest('cadi/getChunk', { id });
    }

    async importChunk(id: string, targetPath: string): Promise<void> {
        return this.sendRequest('cadi/import', { id, targetPath });
    }

    async buildProject(manifestPath: string): Promise<any> {
        return this.sendRequest('cadi/build', { manifestPath });
    }

    async getRegistryStats(): Promise<any> {
        return this.sendRequest('cadi/registryStats');
    }

    async validateManifest(manifestPath: string): Promise<any> {
        return this.sendRequest('cadi/validate', { manifestPath });
    }

    // Auto-completion and IntelliSense support
    async getCompletions(context: string, position: { line: number; character: number }): Promise<any[]> {
        return this.sendRequest('cadi/completions', { context, position });
    }

    async getHoverInfo(symbol: string): Promise<any> {
        return this.sendRequest('cadi/hover', { symbol });
    }

    async getDefinition(symbol: string): Promise<any> {
        return this.sendRequest('cadi/definition', { symbol });
    }

    // Token usage tracking
    async getTokenUsage(): Promise<any> {
        return this.sendRequest('cadi/tokenUsage');
    }

    async resetTokenCounter(): Promise<void> {
        return this.sendRequest('cadi/resetTokens');
    }
}