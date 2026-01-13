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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.CadiChunkItem = exports.CadiRegistryProvider = void 0;
const vscode = __importStar(require("vscode"));
const axios_1 = __importDefault(require("axios"));
class CadiRegistryProvider {
    constructor(context) {
        this.context = context;
        this._onDidChangeTreeData = new vscode.EventEmitter();
        this.onDidChangeTreeData = this._onDidChangeTreeData.event;
        this.chunks = [];
        this.loadChunks();
    }
    refresh() {
        this.loadChunks();
        this._onDidChangeTreeData.fire();
    }
    getTreeItem(element) {
        return element;
    }
    getChildren(element) {
        if (!element) {
            // Root level - return categories
            return Promise.resolve([
                new CadiChunkItem('Atomizers', vscode.TreeItemCollapsibleState.Expanded, 'category', undefined, this.context),
                new CadiChunkItem('Build Backends', vscode.TreeItemCollapsibleState.Expanded, 'category', undefined, this.context),
                new CadiChunkItem('Registry Plugins', vscode.TreeItemCollapsibleState.Expanded, 'category', undefined, this.context),
                new CadiChunkItem('MCP Tools', vscode.TreeItemCollapsibleState.Expanded, 'category', undefined, this.context)
            ]);
        }
        // Return chunks for the category
        const categoryChunks = this.chunks.filter(chunk => {
            switch (element.label) {
                case 'Atomizers':
                    return chunk.tags.includes('atomizer');
                case 'Build Backends':
                    return chunk.tags.includes('build-backend');
                case 'Registry Plugins':
                    return chunk.tags.includes('registry-plugin');
                case 'MCP Tools':
                    return chunk.tags.includes('mcp-tool');
                default:
                    return false;
            }
        });
        return Promise.resolve(categoryChunks.map(chunk => new CadiChunkItem(chunk.name, vscode.TreeItemCollapsibleState.None, 'chunk', chunk, this.context)));
    }
    async loadChunks() {
        try {
            const config = vscode.workspace.getConfiguration('cadi');
            const registryUrl = config.get('registry.url', 'https://registry.cadi.dev');
            const response = await axios_1.default.get(`${registryUrl}/v1/chunks`, {
                timeout: 5000
            });
            // Map ChunkMetadata to CadiChunk format
            const chunks = response.data || [];
            this.chunks = chunks.map((chunk) => ({
                id: chunk.chunk_id,
                name: chunk.chunk_id.split('/').pop() || chunk.chunk_id,
                description: `Chunk ${chunk.chunk_id} (${chunk.size} bytes, ${chunk.content_type})`,
                language: this.guessLanguage(chunk.chunk_id),
                version: '1.0.0',
                downloads: 0,
                author: 'CADI',
                tags: [chunk.content_type],
                dependencies: []
            }));
        }
        catch (error) {
            console.warn('Failed to load chunks from registry, using mock data:', error);
            // Fallback to mock data
            this.chunks = [
                {
                    id: 'atomizer-typescript-v1.0.0',
                    name: 'TypeScript Atomizer',
                    description: 'Converts TypeScript code into reusable CADI chunks',
                    language: 'typescript',
                    version: '1.0.0',
                    downloads: 1250,
                    author: 'CADI Team',
                    tags: ['atomizer', 'typescript'],
                    dependencies: []
                },
                {
                    id: 'backend-rust-v2.1.0',
                    name: 'Rust Backend Builder',
                    description: 'Build backend services with Rust and CADI',
                    language: 'rust',
                    version: '2.1.0',
                    downloads: 890,
                    author: 'CADI Team',
                    tags: ['build-backend', 'rust', 'backend'],
                    dependencies: ['atomizer-rust']
                },
                {
                    id: 'registry-github-v1.2.0',
                    name: 'GitHub Registry Plugin',
                    description: 'Store and retrieve chunks from GitHub repositories',
                    language: 'javascript',
                    version: '1.2.0',
                    downloads: 567,
                    author: 'CADI Community',
                    tags: ['registry-plugin', 'github'],
                    dependencies: []
                },
                {
                    id: 'mcp-code-analyzer-v0.8.0',
                    name: 'Code Analyzer MCP Tool',
                    description: 'Analyze codebases using Model Context Protocol',
                    language: 'python',
                    version: '0.8.0',
                    downloads: 432,
                    author: 'CADI Team',
                    tags: ['mcp-tool', 'analyzer'],
                    dependencies: ['mcp-core']
                }
            ];
        }
    }
    guessLanguage(chunkId) {
        const ext = chunkId.split('.').pop()?.toLowerCase();
        switch (ext) {
            case 'ts': return 'typescript';
            case 'js': return 'javascript';
            case 'rs': return 'rust';
            case 'py': return 'python';
            case 'java': return 'java';
            case 'go': return 'go';
            default: return 'unknown';
        }
    }
    async searchChunks(query) {
        return this.chunks.filter(chunk => chunk.name.toLowerCase().includes(query.toLowerCase()) ||
            chunk.description.toLowerCase().includes(query.toLowerCase()) ||
            chunk.tags.some(tag => tag.toLowerCase().includes(query.toLowerCase())));
    }
    async getChunkDetails(chunkId) {
        return this.chunks.find(chunk => chunk.id === chunkId);
    }
}
exports.CadiRegistryProvider = CadiRegistryProvider;
class CadiChunkItem extends vscode.TreeItem {
    constructor(label, collapsibleState, type, chunk, context) {
        super(label, collapsibleState);
        this.label = label;
        this.collapsibleState = collapsibleState;
        this.type = type;
        this.chunk = chunk;
        this.context = context;
        if (type === 'category') {
            this.iconPath = {
                light: vscode.Uri.joinPath(context?.extensionUri || vscode.Uri.file(''), 'resources', 'icons', 'category.svg'),
                dark: vscode.Uri.joinPath(context?.extensionUri || vscode.Uri.file(''), 'resources', 'icons', 'category.svg')
            };
        }
        else if (chunk) {
            this.tooltip = `${chunk.name}\n${chunk.description}\nLanguage: ${chunk.language}\nDownloads: ${chunk.downloads}`;
            this.description = `${chunk.language} • v${chunk.version}`;
            // Set icon based on language
            const iconName = this.getLanguageIcon(chunk.language);
            this.iconPath = {
                light: vscode.Uri.joinPath(context?.extensionUri || vscode.Uri.file(''), 'resources', 'icons', `${iconName}.svg`),
                dark: vscode.Uri.joinPath(context?.extensionUri || vscode.Uri.file(''), 'resources', 'icons', `${iconName}.svg`)
            };
            // Add command to import chunk
            this.command = {
                command: 'cadi.importChunk',
                title: 'Import Chunk',
                arguments: [chunk.id]
            };
        }
    }
    getLanguageIcon(language) {
        const iconMap = {
            'typescript': 'typescript',
            'javascript': 'javascript',
            'rust': 'rust',
            'python': 'python',
            'java': 'java',
            'go': 'go',
            'cpp': 'cpp'
        };
        return iconMap[language.toLowerCase()] || 'code';
    }
}
exports.CadiChunkItem = CadiChunkItem;
//# sourceMappingURL=registryProvider.js.map