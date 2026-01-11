//! Todo Core - Core todo list logic
//!
//! This crate provides the fundamental todo list operations
//! that can be used by CLI, web, and server applications.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A todo item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    /// Unique identifier
    pub id: String,
    /// Todo title
    pub title: String,
    /// Detailed description
    pub description: Option<String>,
    /// Whether the todo is completed
    pub completed: bool,
    /// Priority (1-5, 1 being highest)
    pub priority: u8,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Creation timestamp (RFC3339)
    pub created_at: String,
    /// Completion timestamp (RFC3339)
    pub completed_at: Option<String>,
}

/// Todo service for managing todos
pub struct TodoService {
    todos: HashMap<String, Todo>,
    next_id: u64,
}

impl TodoService {
    /// Create a new todo service
    pub fn new() -> Self {
        Self {
            todos: HashMap::new(),
            next_id: 1,
        }
    }

    /// Add a new todo
    pub fn add(&mut self, title: String, description: Option<String>) -> Todo {
        let id = format!("todo-{}", self.next_id);
        self.next_id += 1;
        
        let todo = Todo {
            id: id.clone(),
            title,
            description,
            completed: false,
            priority: 3,
            tags: Vec::new(),
            created_at: current_time(),
            completed_at: None,
        };
        
        self.todos.insert(id, todo.clone());
        todo
    }

    /// Get a todo by ID
    pub fn get(&self, id: &str) -> Option<&Todo> {
        self.todos.get(id)
    }

    /// List all todos
    pub fn list(&self) -> Vec<&Todo> {
        self.todos.values().collect()
    }

    /// List incomplete todos
    pub fn list_pending(&self) -> Vec<&Todo> {
        self.todos.values()
            .filter(|t| !t.completed)
            .collect()
    }

    /// List completed todos
    pub fn list_completed(&self) -> Vec<&Todo> {
        self.todos.values()
            .filter(|t| t.completed)
            .collect()
    }

    /// Mark a todo as completed
    pub fn complete(&mut self, id: &str) -> Option<&Todo> {
        if let Some(todo) = self.todos.get_mut(id) {
            todo.completed = true;
            todo.completed_at = Some(current_time());
            Some(todo)
        } else {
            None
        }
    }

    /// Mark a todo as incomplete
    pub fn uncomplete(&mut self, id: &str) -> Option<&Todo> {
        if let Some(todo) = self.todos.get_mut(id) {
            todo.completed = false;
            todo.completed_at = None;
            Some(todo)
        } else {
            None
        }
    }

    /// Update a todo's title
    pub fn update_title(&mut self, id: &str, title: String) -> Option<&Todo> {
        if let Some(todo) = self.todos.get_mut(id) {
            todo.title = title;
            Some(todo)
        } else {
            None
        }
    }

    /// Update a todo's priority
    pub fn set_priority(&mut self, id: &str, priority: u8) -> Option<&Todo> {
        if let Some(todo) = self.todos.get_mut(id) {
            todo.priority = priority.clamp(1, 5);
            Some(todo)
        } else {
            None
        }
    }

    /// Add a tag to a todo
    pub fn add_tag(&mut self, id: &str, tag: String) -> Option<&Todo> {
        if let Some(todo) = self.todos.get_mut(id) {
            if !todo.tags.contains(&tag) {
                todo.tags.push(tag);
            }
            Some(todo)
        } else {
            None
        }
    }

    /// Remove a tag from a todo
    pub fn remove_tag(&mut self, id: &str, tag: &str) -> Option<&Todo> {
        if let Some(todo) = self.todos.get_mut(id) {
            todo.tags.retain(|t| t != tag);
            Some(todo)
        } else {
            None
        }
    }

    /// Delete a todo
    pub fn delete(&mut self, id: &str) -> bool {
        self.todos.remove(id).is_some()
    }

    /// Search todos by title or tag
    pub fn search(&self, query: &str) -> Vec<&Todo> {
        let query_lower = query.to_lowercase();
        self.todos.values()
            .filter(|t| {
                t.title.to_lowercase().contains(&query_lower) ||
                t.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Get statistics
    pub fn stats(&self) -> TodoStats {
        let total = self.todos.len();
        let completed = self.todos.values().filter(|t| t.completed).count();
        let pending = total - completed;
        
        TodoStats {
            total,
            completed,
            pending,
        }
    }
}

impl Default for TodoService {
    fn default() -> Self {
        Self::new()
    }
}

/// Todo statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoStats {
    pub total: usize,
    pub completed: usize,
    pub pending: usize,
}

/// Get current time as RFC3339 string
fn current_time() -> String {
    // Simplified - would use chrono in real implementation
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}", now)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_todo() {
        let mut service = TodoService::new();
        let todo = service.add("Test todo".to_string(), None);
        assert_eq!(todo.title, "Test todo");
        assert!(!todo.completed);
    }

    #[test]
    fn test_complete_todo() {
        let mut service = TodoService::new();
        let todo = service.add("Test todo".to_string(), None);
        let id = todo.id.clone();
        
        service.complete(&id);
        
        let todo = service.get(&id).unwrap();
        assert!(todo.completed);
    }

    #[test]
    fn test_delete_todo() {
        let mut service = TodoService::new();
        let todo = service.add("Test todo".to_string(), None);
        let id = todo.id.clone();
        
        assert!(service.delete(&id));
        assert!(service.get(&id).is_none());
    }
}
