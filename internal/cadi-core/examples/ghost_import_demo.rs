//! Demo script to populate the graph store and test Ghost Import / Virtual Views
//!
//! Run with: cargo run --example populate_graph_demo

use cadi_core::graph::{GraphStore, GraphNode, EdgeType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create graph store in the cache directory
    let graph_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("dev.cadi.cadi")
        .join("graph");
    
    println!("╭─────────────────────────────────────────────╮");
    println!("│      CADI Graph Store Demo                  │");
    println!("╰─────────────────────────────────────────────╯");
    println!();
    println!("Creating graph store at: {:?}", graph_dir);
    
    let graph = GraphStore::open(&graph_dir)?;
    
    // ════════════════════════════════════════════════════════════════════════
    // Create test nodes representing a Todo + Timer application
    // ════════════════════════════════════════════════════════════════════════
    
    println!("\n📦 Adding chunks to graph...\n");
    
    // 1. Todo struct definition (leaf node - no dependencies)
    let todo_struct = GraphNode::new("chunk:todo_struct", "abc123")
        .with_alias("todo/types/Todo")
        .with_language("rust")
        .with_defines(vec!["Todo".to_string(), "TodoStatus".to_string()]);
    
    // 2. TodoService - depends on Todo struct
    let todo_service = GraphNode::new("chunk:todo_service", "def456")
        .with_alias("todo/service/TodoService")
        .with_language("rust")
        .with_defines(vec!["TodoService".to_string()])
        .with_references(vec!["Todo".to_string()]);
    
    // 3. Timer utility (leaf node - no dependencies)
    let timer = GraphNode::new("chunk:timer", "ghi789")
        .with_alias("utils/timer/Timer")
        .with_language("rust")
        .with_defines(vec!["Timer".to_string(), "start".to_string(), "stop".to_string()]);
    
    // 4. App main - depends on TodoService and Timer
    let app = GraphNode::new("chunk:app", "jkl012")
        .with_alias("app/main")
        .with_language("rust")
        .with_defines(vec!["main".to_string()])
        .with_references(vec!["TodoService".to_string(), "Timer".to_string()]);
    
    // Insert nodes
    graph.insert_node(&todo_struct)?;
    println!("  ✓ Added: todo/types/Todo");
    
    graph.insert_node(&todo_service)?;
    println!("  ✓ Added: todo/service/TodoService");
    
    graph.insert_node(&timer)?;
    println!("  ✓ Added: utils/timer/Timer");
    
    graph.insert_node(&app)?;
    println!("  ✓ Added: app/main");
    
    // ════════════════════════════════════════════════════════════════════════
    // Add edges (dependency relationships)
    // ════════════════════════════════════════════════════════════════════════
    
    println!("\n🔗 Adding dependency edges...\n");
    
    // TodoService imports and uses Todo type
    graph.add_dependency("chunk:todo_service", "chunk:todo_struct", EdgeType::Imports)?;
    graph.add_dependency("chunk:todo_service", "chunk:todo_struct", EdgeType::TypeRef)?;
    println!("  ✓ TodoService → Todo (Imports, TypeRef)");
    
    // App imports TodoService and Timer
    graph.add_dependency("chunk:app", "chunk:todo_service", EdgeType::Imports)?;
    graph.add_dependency("chunk:app", "chunk:timer", EdgeType::Imports)?;
    println!("  ✓ App → TodoService (Imports)");
    println!("  ✓ App → Timer (Imports)");
    
    // ════════════════════════════════════════════════════════════════════════
    // Store content for each chunk
    // ════════════════════════════════════════════════════════════════════════
    
    println!("\n📄 Storing chunk content...\n");
    
    graph.store_content("chunk:todo_struct", br#"
/// A todo item
#[derive(Debug, Clone)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

pub enum TodoStatus {
    Pending,
    InProgress,
    Done,
}
"#)?;
    
    graph.store_content("chunk:todo_service", br#"
use crate::Todo;

pub struct TodoService {
    todos: Vec<Todo>,
}

impl TodoService {
    pub fn new() -> Self {
        Self { todos: Vec::new() }
    }
    
    pub fn add(&mut self, title: String) -> Todo {
        let todo = Todo {
            id: format!("todo-{}", self.todos.len() + 1),
            title,
            completed: false,
        };
        self.todos.push(todo.clone());
        todo
    }
    
    pub fn list(&self) -> &[Todo] {
        &self.todos
    }
}
"#)?;
    
    graph.store_content("chunk:timer", br#"
pub struct Timer {
    start_time: Option<std::time::Instant>,
}

impl Timer {
    pub fn new() -> Self {
        Self { start_time: None }
    }
    
    pub fn start(&mut self) {
        self.start_time = Some(std::time::Instant::now());
    }
    
    pub fn stop(&mut self) -> std::time::Duration {
        self.start_time
            .take()
            .map(|t| t.elapsed())
            .unwrap_or_default()
    }
}
"#)?;
    
    graph.store_content("chunk:app", br#"
use todo::TodoService;
use utils::Timer;

fn main() {
    let mut timer = Timer::new();
    let mut service = TodoService::new();
    
    timer.start();
    service.add("Build with CADI".to_string());
    service.add("Test Ghost Imports".to_string());
    let elapsed = timer.stop();
    
    println!("Added {} todos in {:?}", service.list().len(), elapsed);
}
"#)?;
    
    println!("  ✓ Content stored for all 4 chunks");
    
    // ════════════════════════════════════════════════════════════════════════
    // Demonstrate Graph Queries
    // ════════════════════════════════════════════════════════════════════════
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 GRAPH QUERIES");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // What does App depend on?
    let app_deps = graph.get_dependencies("chunk:app")?;
    println!("What does 'app/main' depend on?");
    for (edge_type, dep_id) in &app_deps {
        if let Ok(Some(node)) = graph.get_node(dep_id) {
            println!("  {:?} → {}", edge_type, node.primary_alias.as_ref().unwrap_or(dep_id));
        }
    }
    
    // What uses Todo struct?
    let struct_dependents = graph.get_dependents("chunk:todo_struct")?;
    println!("\nWhat uses 'todo/types/Todo'?");
    for (edge_type, dep_id) in &struct_dependents {
        if let Ok(Some(node)) = graph.get_node(dep_id) {
            println!("  {:?} ← {}", edge_type, node.primary_alias.as_ref().unwrap_or(dep_id));
        }
    }
    
    // ════════════════════════════════════════════════════════════════════════
    // Demonstrate GHOST IMPORT
    // ════════════════════════════════════════════════════════════════════════
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("👻 GHOST IMPORT DEMO");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    println!("Starting from: chunk:app (app/main)");
    println!("Automatically expanding dependencies that should_auto_expand()...\n");
    
    let mut included = std::collections::HashSet::new();
    let mut frontier = vec!["chunk:app".to_string()];
    included.insert("chunk:app".to_string());
    
    let mut depth = 0;
    while !frontier.is_empty() && depth < 3 {
        depth += 1;
        let mut next_frontier = Vec::new();
        
        for chunk_id in &frontier {
            if let Ok(deps) = graph.get_dependencies(chunk_id) {
                for (edge_type, dep_id) in deps {
                    if edge_type.should_auto_expand() && !included.contains(&dep_id) {
                        if let Ok(Some(node)) = graph.get_node(&dep_id) {
                            println!("  [depth {}] 👻 Ghost importing: {} (via {:?})", 
                                depth, 
                                node.primary_alias.as_ref().unwrap_or(&dep_id), 
                                edge_type);
                        }
                        included.insert(dep_id.clone());
                        next_frontier.push(dep_id);
                    }
                }
            }
        }
        frontier = next_frontier;
    }
    
    println!("\n✓ Final context includes {} chunks:", included.len());
    for id in &included {
        if let Ok(Some(node)) = graph.get_node(id) {
            println!("  • {}", node.primary_alias.as_ref().unwrap_or(id));
        }
    }
    
    // ════════════════════════════════════════════════════════════════════════
    // Demonstrate VIRTUAL VIEW Assembly
    // ════════════════════════════════════════════════════════════════════════
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎯 VIRTUAL VIEW ASSEMBLY");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let mut assembled = String::new();
    let mut total_bytes = 0;
    
    // Sort by dependency order (leaves first)
    let ordered: Vec<_> = vec![
        "chunk:todo_struct",
        "chunk:timer", 
        "chunk:todo_service",
        "chunk:app",
    ];
    
    for id in ordered {
        if included.contains(id) {
            if let Ok(Some(content)) = graph.get_content_str(id) {
                if let Ok(Some(node)) = graph.get_node(id) {
                    assembled.push_str(&format!("// ═══ {} ═══\n", 
                        node.primary_alias.as_ref().unwrap_or(&id.to_string())));
                }
                assembled.push_str(&content);
                assembled.push_str("\n");
                total_bytes += content.len();
            }
        }
    }
    
    println!("{}", assembled);
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 STATS: {} bytes, ~{} tokens", total_bytes, total_bytes / 4);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    println!("\n✅ Graph store populated and demo complete!");
    println!("\nYou can now use the MCP tools:");
    println!("  • cadi_view_context - to get virtual views");
    println!("  • cadi_get_dependencies - to query dependencies");
    println!("  • cadi_get_dependents - to find what uses a chunk");
    
    Ok(())
}
