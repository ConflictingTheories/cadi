// CADI Visualization Dashboard JavaScript

class CadiDashboard {
    constructor() {
        this.currentTab = 'browse';
        this.currentPage = 1;
        this.pageSize = 20;
        this.searchQuery = '';
        this.stats = {};
        this.chunks = [];
        this.filteredChunks = [];

        this.init();
    }

    async init() {
        await this.loadStats();
        this.setupEventListeners();
        this.renderStats();
        await this.loadChunks();
        this.renderChunks();
    }

    async loadStats() {
        try {
            const response = await fetch('/api/stats');
            this.stats = await response.json();
        } catch (error) {
            console.error('Failed to load stats:', error);
            this.stats = {
                graph_store: {
                    node_count: 0,
                    symbol_count: 0,
                    alias_count: 0,
                    dependency_entries: 0,
                    content_entries: 0,
                    db_size_bytes: 0
                }
            };
            // Show error message to user
            document.getElementById('stats-grid').innerHTML = '<div class="error-message">Failed to load statistics from CADI database</div>';
        }
    }

    async loadChunks(page = 1, search = '') {
        try {
            const params = new URLSearchParams({
                page: page.toString(),
                limit: this.pageSize.toString(),
                search: search
            });
            const response = await fetch(`/api/chunks?${params}`);
            const data = await response.json();
            this.chunks = data.chunks;
            this.totalPages = Math.ceil(data.total / this.pageSize);
            this.currentPage = page;
        } catch (error) {
            console.error('Failed to load chunks:', error);
            this.chunks = [];
            this.totalPages = 1;
            // Show error message in the chunk list
            document.getElementById('chunk-list').innerHTML = '<div class="error-message">Failed to load chunks from CADI database</div>';
        }
    }

    setupEventListeners() {
        // Tab switching
        document.querySelectorAll('.tab').forEach(tab => {
            tab.addEventListener('click', (e) => {
                this.switchTab(e.target.dataset.tab);
            });
        });

        // Browse search
        document.getElementById('browse-search').addEventListener('input', (e) => {
            this.filterChunks(e.target.value);
        });

        // Search functionality
        document.getElementById('search-input').addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                this.performSearch();
            }
        });

        document.getElementById('search-btn').addEventListener('click', () => {
            this.performSearch();
        });

        // Pagination
        document.getElementById('prev-page').addEventListener('click', () => {
            if (this.currentPage > 1) {
                this.loadChunks(this.currentPage - 1, this.searchQuery);
            }
        });

        document.getElementById('next-page').addEventListener('click', () => {
            if (this.currentPage < this.totalPages) {
                this.loadChunks(this.currentPage + 1, this.searchQuery);
            }
        });
    }

    switchTab(tabName) {
        // Update tab buttons
        document.querySelectorAll('.tab').forEach(tab => {
            tab.classList.remove('active');
        });
        document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');

        // Update tab content
        document.querySelectorAll('.tab-pane').forEach(pane => {
            pane.classList.remove('active');
        });
        document.getElementById(`${tabName}-tab`).classList.add('active');

        this.currentTab = tabName;

        if (tabName === 'graph') {
            this.renderGraph();
        }
    }

    renderStats() {
        const statsGrid = document.getElementById('stats-grid');
        const graphStats = this.stats.graph_store || {};

        const statCards = [
            { value: graphStats.node_count || 0, label: 'Total Nodes' },
            { value: graphStats.symbol_count || 0, label: 'Symbols' },
            { value: graphStats.alias_count || 0, label: 'Aliases' },
            { value: graphStats.dependency_entries || 0, label: 'Dependencies' },
            { value: graphStats.content_entries || 0, label: 'Content Entries' },
            { value: `${(graphStats.db_size_bytes / (1024 * 1024) || 0).toFixed(2)} MB`, label: 'Database Size' }
        ];

        statsGrid.innerHTML = statCards.map(card => `
            <div class="stat-card">
                <div class="stat-value">${card.value}</div>
                <div class="stat-label">${card.label}</div>
            </div>
        `).join('');
    }

    async renderChunks() {
        const chunkList = document.getElementById('chunk-list');
        const pageInfo = document.getElementById('page-info');

        chunkList.innerHTML = this.chunks.map(chunk => `
            <div class="chunk-item">
                <div>
                    <strong>${chunk.alias || chunk.name}</strong>
                    <br>
                    <small>${chunk.id} • ${chunk.language} • ${(chunk.size / 1024).toFixed(1)} KB</small>
                </div>
                <div>
                    <span title="Dependencies">${chunk.dependencies} deps</span> |
                    <span title="Dependents">${chunk.dependents} deps on</span>
                </div>
            </div>
        `).join('');

        pageInfo.textContent = `Page ${this.currentPage} of ${this.totalPages}`;

        // Update pagination buttons
        document.getElementById('prev-page').disabled = this.currentPage <= 1;
        document.getElementById('next-page').disabled = this.currentPage >= this.totalPages;
    }

    filterChunks(query) {
        if (!query) {
            this.filteredChunks = this.chunks;
        } else {
            this.filteredChunks = this.chunks.filter(chunk =>
                chunk.name.toLowerCase().includes(query.toLowerCase()) ||
                chunk.alias.toLowerCase().includes(query.toLowerCase()) ||
                chunk.id.toLowerCase().includes(query.toLowerCase())
            );
        }
        this.renderFilteredChunks();
    }

    renderFilteredChunks() {
        const chunkList = document.getElementById('chunk-list');
        chunkList.innerHTML = this.filteredChunks.map(chunk => `
            <div class="chunk-item">
                <div>
                    <strong>${chunk.alias || chunk.name}</strong>
                    <br>
                    <small>${chunk.id} • ${chunk.language} • ${(chunk.size / 1024).toFixed(1)} KB</small>
                </div>
                <div>
                    <span title="Dependencies">${chunk.dependencies} deps</span> |
                    <span title="Dependents">${chunk.dependents} deps on</span>
                </div>
            </div>
        `).join('');
    }

    async performSearch() {
        const query = document.getElementById('search-input').value.trim();
        this.searchQuery = query;

        const resultsDiv = document.getElementById('search-results');

        if (!query) {
            resultsDiv.innerHTML = '<p>Enter a search term above</p>';
            return;
        }

        resultsDiv.innerHTML = '<p>Searching...</p>';

        // For now, just filter locally - in real implementation, this would call a search API
        await this.loadChunks(1, query);
        this.filterChunks(query);

        if (this.filteredChunks.length === 0) {
            resultsDiv.innerHTML = '<p>No results found</p>';
        } else {
            this.renderFilteredChunks();
        }
    }

    async renderGraph() {
        const container = d3.select('#graph-container');
        container.html(''); // Clear previous content

        // Create a simple force-directed graph
        const width = container.node().getBoundingClientRect().width;
        const height = 400;

        const svg = container.append('svg')
            .attr('width', width)
            .attr('height', height);

        // Get real graph data from API
        let graphData;
        try {
            const response = await fetch('/api/graph');
            graphData = await response.json();
        } catch (error) {
            console.error('Failed to load graph data:', error);
            // Fallback to mock data
            graphData = {
                nodes: [
                    { id: 'A', name: 'Node A', group: 1 },
                    { id: 'B', name: 'Node B', group: 1 },
                    { id: 'C', name: 'Node C', group: 2 },
                    { id: 'D', name: 'Node D', group: 2 },
                    { id: 'E', name: 'Node E', group: 3 }
                ],
                links: [
                    { source: 'A', target: 'B', type: 'Imports' },
                    { source: 'B', target: 'C', type: 'Imports' },
                    { source: 'C', target: 'D', type: 'TypeRef' },
                    { source: 'D', target: 'E', type: 'Imports' }
                ]
            };
        }

        const nodes = graphData.nodes;
        const links = graphData.links;

        const simulation = d3.forceSimulation(nodes)
            .force('link', d3.forceLink(links).id(d => d.id))
            .force('charge', d3.forceManyBody())
            .force('center', d3.forceCenter(width / 2, height / 2));

        const link = svg.append('g')
            .selectAll('line')
            .data(links)
            .enter().append('line')
            .attr('stroke', '#999')
            .attr('stroke-opacity', 0.6)
            .attr('stroke-width', 2);

        const node = svg.append('g')
            .selectAll('circle')
            .data(nodes)
            .enter().append('circle')
            .attr('r', 8)
            .attr('fill', d => d3.schemeCategory10[d.group % 10])
            .call(d3.drag()
                .on('start', (event, d) => {
                    if (!event.active) simulation.alphaTarget(0.3).restart();
                    d.fx = d.x;
                    d.fy = d.y;
                })
                .on('drag', (event, d) => {
                    d.fx = event.x;
                    d.fy = event.y;
                })
                .on('end', (event, d) => {
                    if (!event.active) simulation.alphaTarget(0);
                    d.fx = null;
                    d.fy = null;
                }));

        const label = svg.append('g')
            .selectAll('text')
            .data(nodes)
            .enter().append('text')
            .text(d => d.id)
            .attr('font-size', 12)
            .attr('dx', 12)
            .attr('dy', 4);

        simulation.on('tick', () => {
            link
                .attr('x1', d => d.source.x)
                .attr('y1', d => d.source.y)
                .attr('x2', d => d.target.x)
                .attr('y2', d => d.target.y);

            node
                .attr('cx', d => d.x)
                .attr('cy', d => d.y);

            label
                .attr('x', d => d.x)
                .attr('y', d => d.y);
        });
    }
}

// Initialize dashboard when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    new CadiDashboard();
});
