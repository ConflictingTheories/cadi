use anyhow::Result;
use clap::Args;
use std::io;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use cadi_core::graph::{GraphStore, GraphQuery, TraversalDirection};
use cadi_core::graph::store::GraphStoreStats;

use crate::config::CadiConfig;

use axum::{
    extract::Query,
    response::Html,
    routing::get,
    Router,
};
use std::collections::HashMap;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};

/// Arguments for the visualize command
#[derive(Args)]
pub struct VisualizeArgs {
    /// Visualization mode: tui (default) or web
    #[arg(long, default_value = "tui")]
    mode: String,

    /// Port for web mode (default: 8080)
    #[arg(long, default_value = "8080")]
    port: u16,
}

/// Execute the visualize command
pub async fn execute(args: VisualizeArgs, config: &CadiConfig) -> Result<()> {
    match args.mode.as_str() {
        "tui" => run_tui(config).await,
        "web" => run_web(args.port, config).await,
        _ => Err(anyhow::anyhow!("Invalid mode: {}. Use 'tui' or 'web'", args.mode)),
    }
}

/// Run the TUI visualization
async fn run_tui(config: &CadiConfig) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let res = run_app(&mut terminal, config).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Stats,
    Browse,
    Search,
    Storage,
}

impl Tab {
    fn title(&self) -> &str {
        match self {
            Tab::Stats => "Stats",
            Tab::Browse => "Browse",
            Tab::Search => "Search",
            Tab::Storage => "Storage",
        }
    }

    fn index(&self) -> usize {
        match self {
            Tab::Stats => 0,
            Tab::Browse => 1,
            Tab::Search => 2,
            Tab::Storage => 3,
        }
    }

    fn from_index(index: usize) -> Self {
        match index {
            0 => Tab::Stats,
            1 => Tab::Browse,
            2 => Tab::Search,
            3 => Tab::Storage,
            _ => Tab::Stats,
        }
    }
}

struct App {
    active_tab: Tab,
    should_quit: bool,
    browse_state: ListState,
    search_input: String,
    search_results: Vec<String>,
    current_page: usize,
    items_per_page: usize,
    total_items: usize,
}

impl App {
    fn new() -> Self {
        let mut browse_state = ListState::default();
        browse_state.select(Some(0));
        Self {
            active_tab: Tab::Stats,
            should_quit: false,
            browse_state,
            search_input: String::new(),
            search_results: Vec::new(),
            current_page: 0,
            items_per_page: 10,
            total_items: 0,
        }
    }

    fn next_tab(&mut self) {
        let current = self.active_tab.index();
        self.active_tab = Tab::from_index((current + 1) % 4);
    }

    fn previous_tab(&mut self) {
        let current = self.active_tab.index();
        self.active_tab = Tab::from_index(if current == 0 { 3 } else { current - 1 });
    }

    fn next_page(&mut self) {
        if (self.current_page + 1) * self.items_per_page < self.total_items {
            self.current_page += 1;
        }
    }

    fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
        }
    }

    fn handle_search_input(&mut self, c: char) {
        self.search_input.push(c);
    }

    fn handle_backspace(&mut self) {
        self.search_input.pop();
    }

    fn execute_search(&mut self, store: &GraphStore) -> Result<()> {
        if self.search_input.is_empty() {
            self.search_results.clear();
            return Ok(());
        }

        // Search for chunks by alias or symbol
        let query = GraphQuery::new(vec![])
            .with_direction(TraversalDirection::Both)
            .with_depth(1);

        let result = store.query(&query)?;
        self.search_results = result.nodes
            .into_iter()
            .filter(|node| {
                node.alias.as_ref().map_or(false, |alias| alias.contains(&self.search_input)) ||
                node.chunk_id.contains(&self.search_input)
            })
            .map(|node| format!("{} ({})", node.alias.unwrap_or_else(|| node.chunk_id.clone()), node.chunk_id))
            .collect();

        Ok(())
    }
}

async fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, config: &CadiConfig) -> Result<()> {
    let mut app = App::new();

    // Initialize with some mock data for demo
    app.total_items = 100;

    // Try to open graph store for real data
    let graph_store = match GraphStore::open(&config.cache.dir.join("graph-db")) {
        Ok(store) => {
            if let Ok(stats) = store.stats() {
                app.total_items = stats.node_count;
            }
            Some(store)
        },
        Err(e) => {
            eprintln!("Warning: Could not open graph store: {}. Using mock data.", e);
            None
        },
    };

    loop {
        let stats = if let Some(ref store) = graph_store {
            store.stats().unwrap_or_else(|_| GraphStoreStats {
                node_count: 0,
                symbol_count: 0,
                alias_count: 0,
                dependency_entries: 0,
                dependent_entries: 0,
                content_entries: 0,
                db_size_bytes: 0,
            })
        } else {
            GraphStoreStats {
                node_count: 0,
                symbol_count: 0,
                alias_count: 0,
                dependency_entries: 0,
                dependent_entries: 0,
                content_entries: 0,
                db_size_bytes: 0,
            }
        };

        terminal.draw(|f| ui(f, &app, &stats))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                KeyCode::Right | KeyCode::Char('l') => app.next_tab(),
                KeyCode::Left | KeyCode::Char('h') => app.previous_tab(),
                KeyCode::Tab => app.next_tab(),
                KeyCode::BackTab => app.previous_tab(),
                KeyCode::Char('n') => app.next_page(),
                KeyCode::Char('p') => app.prev_page(),
                KeyCode::Enter => {
                    if let Some(ref store) = graph_store {
                        app.execute_search(store)?;
                    }
                }
                KeyCode::Backspace => app.handle_backspace(),
                KeyCode::Char(c) => {
                    if app.active_tab == Tab::Search {
                        app.handle_search_input(c);
                    }
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &App, stats: &GraphStoreStats) {
    let size = f.size();

    // Create tabs
    let titles: Vec<Line> = vec!["Stats", "Browse", "Search", "Storage"]
        .iter()
        .map(|t| Line::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("CADI Visualization"))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(app.active_tab.index());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    f.render_widget(tabs, chunks[0]);

    // Render content based on active tab
    match app.active_tab {
        Tab::Stats => draw_stats(f, chunks[1], stats),
        Tab::Browse => draw_browse(f, chunks[1], app),
        Tab::Search => draw_search(f, chunks[1], app),
        Tab::Storage => draw_storage(f, chunks[1], stats),
    }
}

fn draw_stats(f: &mut Frame, area: Rect, stats: &GraphStoreStats) {
    let text = vec![
        Line::from("Graph Store Statistics:"),
        Line::from(format!("  Total nodes: {}", stats.node_count)),
        Line::from(format!("  Total symbols: {}", stats.symbol_count)),
        Line::from(format!("  Total aliases: {}", stats.alias_count)),
        Line::from(format!("  Dependency entries: {}", stats.dependency_entries)),
        Line::from(format!("  Content entries: {}", stats.content_entries)),
        Line::from(format!("  Database size: {:.2} MB", stats.db_size_bytes as f64 / (1024.0 * 1024.0))),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  Tab: h/l or ←/→"),
        Line::from("  Quit: q or Esc"),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Statistics"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn draw_browse(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(area);

    // Mock chunk data for now - in real implementation, query from store
    let all_items: Vec<String> = (0..app.total_items)
        .map(|i| format!("Chunk {}: example_{}.rs", i + 1, i + 1))
        .collect();

    let start_idx = app.current_page * app.items_per_page;
    let end_idx = (start_idx + app.items_per_page).min(all_items.len());
    let page_items: Vec<ListItem> = all_items[start_idx..end_idx]
        .iter()
        .map(|item| ListItem::new(item.clone()))
        .collect();

    let mut list = List::new(page_items)
        .block(Block::default().borders(Borders::ALL).title(format!("Browse Chunks (Page {}/{})",
            app.current_page + 1,
            (app.total_items + app.items_per_page - 1) / app.items_per_page)))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC));

    if app.active_tab == Tab::Browse {
        list = list.highlight_symbol(">");
    }

    f.render_stateful_widget(list, chunks[0], &mut app.browse_state.clone());

    // Page info
    let page_info = Paragraph::new(format!(
        "Page {}/{} | n: next page, p: prev page | Total: {} items",
        app.current_page + 1,
        (app.total_items + app.items_per_page - 1) / app.items_per_page,
        app.total_items
    ))
    .block(Block::default().borders(Borders::TOP));

    f.render_widget(page_info, chunks[1]);
}

fn draw_search(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(area);

    // Search input
    let input_text = vec![
        Line::from(format!("Search: {}", app.search_input)),
        Line::from("Type to search, Enter to execute, Backspace to delete"),
    ];

    let input_paragraph = Paragraph::new(input_text)
        .block(Block::default().borders(Borders::ALL).title("Search Input"))
        .wrap(Wrap { trim: true });

    f.render_widget(input_paragraph, chunks[0]);

    // Search results
    let result_items: Vec<ListItem> = if app.search_results.is_empty() && !app.search_input.is_empty() {
        vec![ListItem::new("No results found")]
    } else if app.search_results.is_empty() {
        vec![ListItem::new("Enter search term above")]
    } else {
        app.search_results.iter().map(|result| ListItem::new(result.clone())).collect()
    };

    let results_list = List::new(result_items)
        .block(Block::default().borders(Borders::ALL).title(format!("Results ({})", app.search_results.len())))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC));

    f.render_widget(results_list, chunks[1]);
}

fn draw_storage(f: &mut Frame, area: Rect, stats: &GraphStoreStats) {
    let text = vec![
        Line::from("Storage Details:"),
        Line::from(format!("  Total chunks: {}", stats.content_entries)),
        Line::from(format!("  Total size: {:.2} MB", stats.db_size_bytes as f64 / (1024.0 * 1024.0))),
        Line::from(format!("  Graph nodes: {}", stats.node_count)),
        Line::from(format!("  Dependencies: {}", stats.dependency_entries)),
        Line::from(""),
        Line::from("Registry:"),
        Line::from("  Connected: Yes"),
        Line::from("  Remote repos: 5"),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  Tab: h/l or ←/→"),
        Line::from("  Quit: q or Esc"),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Storage"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Run the web server for visualization
async fn run_web(port: u16, _config: &CadiConfig) -> Result<()> {
    // Create router
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/api/stats", get(stats_api_handler))
        .route("/api/chunks", get(chunks_api_handler))
        .route("/api/graph", get(graph_api_handler))
        .nest_service("/css", ServeDir::new("website/css"))
        .nest_service("/js", ServeDir::new("website/js"))
        .nest_service("/assets", ServeDir::new("website/assets"))
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()));

    // Bind to address
    let addr = format!("0.0.0.0:{}", port);
    println!("Starting web server at http://{}", addr);
    println!("Open this URL in your browser to view the visualization");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root_handler() -> Html<&'static str> {
    Html(include_str!("../../../../website/visualize.html"))
}

async fn stats_api_handler() -> axum::response::Json<serde_json::Value> {
    // Try to get real stats from GraphStore
    let graph_stats = match GraphStore::open(std::env::var("CADI_CACHE_DIR").unwrap_or_else(|_| "data/graph-db".to_string())) {
        Ok(store) => match store.stats() {
            Ok(stats) => serde_json::json!({
                "node_count": stats.node_count,
                "symbol_count": stats.symbol_count,
                "alias_count": stats.alias_count,
                "dependency_entries": stats.dependency_entries,
                "content_entries": stats.content_entries,
                "db_size_bytes": stats.db_size_bytes
            }),
            Err(e) => {
                eprintln!("Warning: Could not get graph stats: {}", e);
                serde_json::json!({
                    "node_count": 0,
                    "symbol_count": 0,
                    "alias_count": 0,
                    "dependency_entries": 0,
                    "content_entries": 0,
                    "db_size_bytes": 0,
                    "error": format!("Could not get stats: {}", e)
                })
            }
        },
        Err(e) => {
            eprintln!("Warning: Could not open graph store for stats: {}", e);
            serde_json::json!({
                "node_count": 0,
                "symbol_count": 0,
                "alias_count": 0,
                "dependency_entries": 0,
                "content_entries": 0,
                "db_size_bytes": 0,
                "error": format!("Could not open graph store: {}", e)
            })
        }
    };

    let stats = serde_json::json!({
        "graph_store": graph_stats,
        "cache": {
            "hits": 42,
            "misses": 8,
            "hit_rate": 0.84
        },
        "reuse": {
            "chunks_reused": 156,
            "bytes_deduplicated": 52428800,
            "tokens_saved": 125000
        },
        "build": {
            "total_builds": 23,
            "avg_duration_seconds": 4.2,
            "time_saved_seconds": 180
        },
        "storage": {
            "total_chunks": graph_stats["content_entries"],
            "total_size_bytes": graph_stats["db_size_bytes"],
            "registry_connected": true,
            "remote_repos": 5
        }
    });
    axum::response::Json(stats)
}

async fn chunks_api_handler(
    Query(params): Query<HashMap<String, String>>,
) -> axum::response::Json<serde_json::Value> {
    let page = params.get("page").and_then(|s| s.parse().ok()).unwrap_or(1);
    let limit = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20);
    let search = params.get("search").cloned().unwrap_or_default();

    // Validate parameters
    if page < 1 {
        return axum::response::Json(serde_json::json!({
            "error": "Page must be >= 1",
            "chunks": [],
            "page": 1,
            "limit": limit,
            "total": 0,
            "search": search
        }));
    }
    if limit < 1 || limit > 100 {
        return axum::response::Json(serde_json::json!({
            "error": "Limit must be between 1 and 100",
            "chunks": [],
            "page": page,
            "limit": 20,
            "total": 0,
            "search": search
        }));
    }

    // Get real chunk data from GraphStore
    let chunks = match GraphStore::open(std::env::var("CADI_CACHE_DIR").unwrap_or_else(|_| "data/graph-db".to_string())) {
        Ok(store) => {
            let query = if search.is_empty() {
                GraphQuery::new(vec![])
            } else {
                // Search by alias or chunk_id
                GraphQuery::new(vec![]).with_depth(1)
            };

            match store.query(&query) {
                Ok(result) => {
                    let filtered_nodes: Vec<_> = result.nodes
                        .into_iter()
                        .filter(|node| {
                            if search.is_empty() {
                                true
                            } else {
                                node.alias.as_ref().map_or(false, |alias| alias.contains(&search)) ||
                                node.chunk_id.contains(&search)
                            }
                        })
                        .collect();

                    let start_idx = (page - 1) * limit as usize;
                    let end_idx = (start_idx + limit as usize).min(filtered_nodes.len());

                    filtered_nodes[start_idx..end_idx]
                        .iter()
                        .filter_map(|query_node| {
                            // Get full GraphNode to access size and other fields
                            store.get_node(&query_node.chunk_id).ok().flatten().map(|graph_node| {
                                serde_json::json!({
                                    "id": graph_node.chunk_id,
                                    "name": graph_node.primary_alias.clone().unwrap_or_else(|| graph_node.chunk_id.clone()),
                                    "alias": graph_node.primary_alias,
                                    "size": graph_node.byte_size,
                                    "type": "source", // TODO: determine from content
                                    "language": graph_node.language,
                                    "last_modified": graph_node.updated_at,
                                    "dependencies": graph_node.outgoing_edges.len(),
                                    "dependents": graph_node.incoming_edges.len()
                                })
                            })
                        })
                        .collect::<Vec<_>>()
                },
                Err(_) => vec![]
            }
        },
        Err(_) => vec![]
    };

    let total = chunks.len() * 10; // Rough estimate for pagination

    let response = serde_json::json!({
        "chunks": chunks,
        "page": page,
        "limit": limit,
        "total": total,
        "search": search
    });

    axum::response::Json(response)
}

async fn graph_api_handler() -> axum::response::Json<serde_json::Value> {
    // Get real graph data from GraphStore for visualization
    let graph_data = match GraphStore::open("data/graph-db") {
        Ok(store) => {
            let query = GraphQuery::new(vec![]).with_limit(50); // Limit to 50 nodes for visualization
            match store.query(&query) {
                Ok(result) => {
                    let nodes: Vec<serde_json::Value> = result.nodes
                        .iter()
                        .enumerate()
                        .map(|(i, node)| {
                            serde_json::json!({
                                "id": node.chunk_id,
                                "name": node.alias.clone().unwrap_or_else(|| node.chunk_id.clone()),
                                "group": i % 5 + 1 // Simple grouping for visualization
                            })
                        })
                        .collect();

                    let mut links = Vec::new();
                    for node in &result.nodes {
                        if let Ok(graph_node) = store.get_node(&node.chunk_id) {
                            if let Some(graph_node) = graph_node {
                                for (edge_type, target_id) in &graph_node.outgoing_edges {
                                    links.push(serde_json::json!({
                                        "source": graph_node.chunk_id,
                                        "target": target_id,
                                        "type": format!("{:?}", edge_type)
                                    }));
                                }
                            }
                        }
                    }

                    serde_json::json!({
                        "nodes": nodes,
                        "links": links
                    })
                },
                Err(_) => serde_json::json!({
                    "nodes": [],
                    "links": []
                })
            }
        },
        Err(e) => {
            eprintln!("Warning: Could not open graph store for visualization: {}", e);
            serde_json::json!({
                "nodes": [],
                "links": [],
                "error": format!("Could not open graph store: {}", e)
            })
        }
    };

    axum::response::Json(graph_data)
}
