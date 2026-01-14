# TODO: Add Visualization Tool to CADI

## Overview
Add a new `visualize` command to the CADI CLI that provides both a TUI (Terminal User Interface) for local exploration and a WEB GUI for remote network repo viewing. The tool should support paged browsing, searching, basic stats, storage details, and usage metrics.

## Steps

### 1. Update Dependencies
- [x] Add `ratatui` and `crossterm` for TUI functionality
- [x] Add `axum` for web server (if not already available)
- [x] Ensure `tokio` and `serde_json` are available (likely already in workspace)

### 2. Update CLI Structure
- [x] Add `mod visualize;` to `cmd/cadi/src/commands/mod.rs`
- [x] Add `Visualize(commands::visualize::VisualizeArgs)` to `Commands` enum in `cmd/cadi/src/main.rs`
- [x] Add case for `Commands::Visualize(args) => commands::visualize::execute(args, &config).await` in main.rs

### 3. Create Visualize Command Module
- [x] Create `cmd/cadi/src/commands/visualize.rs`
- [x] Define `VisualizeArgs` struct with `--mode` (tui/web), `--port` for web mode
- [x] Implement `execute` function that dispatches to TUI or web mode

### 4. Implement TUI Mode
- [x] Set up TUI with `ratatui` and `crossterm`
- [x] Create tabs/pages: Stats, Browse, Search, Storage
- [x] Implement data querying via `cadi-core` for graph-db, registry, metrics
- [x] Add interactive elements: paging, navigation, search input

### 5. Implement Web Mode
- [x] Set up local web server with `axum`
- [x] Create HTML/JS interface (extend `website/` or create new files)
- [x] Use D3.js for graphs and visualizations
- [x] Serve data via REST API endpoints

### 6. Data Access Layer
- [x] Implement functions to query stats, chunks, storage info from `cadi-core`
- [x] Handle pagination and search filtering

### 7. Testing and Refinement
- [x] Test TUI interface for responsiveness and data display
- [x] Test web mode for local serving and remote access
- [x] Ensure data accuracy and performance
- [x] Update documentation if needed

### 8. Final Integration
- [x] Verify command integration with existing CLI
- [x] Handle errors and edge cases
- [x] Optimize for large repositories
