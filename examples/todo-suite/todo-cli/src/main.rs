//! Todo CLI - Command-line interface for the todo list

use clap::{Parser, Subcommand};
use todo_core::TodoService;

#[derive(Parser)]
#[command(name = "todo")]
#[command(about = "A simple todo list manager")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new todo
    Add {
        /// Todo title
        title: String,
        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// List all todos
    List {
        /// Show only pending todos
        #[arg(short, long)]
        pending: bool,
        /// Show only completed todos
        #[arg(short, long)]
        completed: bool,
    },
    /// Mark a todo as complete
    Complete {
        /// Todo ID
        id: String,
    },
    /// Mark a todo as incomplete
    Uncomplete {
        /// Todo ID
        id: String,
    },
    /// Delete a todo
    Delete {
        /// Todo ID
        id: String,
    },
    /// Search todos
    Search {
        /// Search query
        query: String,
    },
    /// Show statistics
    Stats,
}

fn main() {
    let cli = Cli::parse();
    let mut service = TodoService::new();
    
    // In a real app, we'd load/save from a file
    // For demo purposes, we just show the operations
    
    match cli.command {
        Commands::Add { title, description } => {
            let todo = service.add(title, description);
            println!("✓ Added todo: {} ({})", todo.title, todo.id);
        }
        Commands::List { pending, completed } => {
            let todos = if pending {
                service.list_pending()
            } else if completed {
                service.list_completed()
            } else {
                service.list()
            };
            
            if todos.is_empty() {
                println!("No todos found.");
            } else {
                for todo in todos {
                    let status = if todo.completed { "✓" } else { " " };
                    println!("[{}] {} - {}", status, todo.id, todo.title);
                }
            }
        }
        Commands::Complete { id } => {
            if service.complete(&id).is_some() {
                println!("✓ Marked {} as complete", id);
            } else {
                println!("✗ Todo not found: {}", id);
            }
        }
        Commands::Uncomplete { id } => {
            if service.uncomplete(&id).is_some() {
                println!("✓ Marked {} as incomplete", id);
            } else {
                println!("✗ Todo not found: {}", id);
            }
        }
        Commands::Delete { id } => {
            if service.delete(&id) {
                println!("✓ Deleted {}", id);
            } else {
                println!("✗ Todo not found: {}", id);
            }
        }
        Commands::Search { query } => {
            let todos = service.search(&query);
            if todos.is_empty() {
                println!("No matching todos found.");
            } else {
                for todo in todos {
                    let status = if todo.completed { "✓" } else { " " };
                    println!("[{}] {} - {}", status, todo.id, todo.title);
                }
            }
        }
        Commands::Stats => {
            let stats = service.stats();
            println!("Todo Statistics:");
            println!("  Total:     {}", stats.total);
            println!("  Completed: {}", stats.completed);
            println!("  Pending:   {}", stats.pending);
        }
    }
}
