import React, { useState, useEffect } from 'react';

// Todo interface matching the Rust TodoService
interface Todo {
  id: string;
  title: string;
  description?: string;
  completed: boolean;
  priority: number;
  tags: string[];
  created_at: string;
  completed_at?: string;
}

// Mock TodoService - in real implementation, this would be the WASM-compiled Rust code
class TodoService {
  private todos: Map<string, Todo> = new Map();
  private nextId: number = 1;

  constructor() {
    // Initialize with some sample data
    this.add("Hack the planet", "Take over the world through code");
    this.add("Learn Rust", "Master the memory-safe programming language");
    const todo = this.todos.get("todo-1")!;
    this.complete(todo.id);
  }

  add(title: string, description?: string): Todo {
    const id = `todo-${this.nextId++}`;
    const todo: Todo = {
      id,
      title,
      description,
      completed: false,
      priority: 3,
      tags: [],
      created_at: new Date().toISOString(),
    };
    this.todos.set(id, todo);
    return todo;
  }

  get(id: string): Todo | undefined {
    return this.todos.get(id);
  }

  list(): Todo[] {
    return Array.from(this.todos.values());
  }

  listPending(): Todo[] {
    return this.list().filter(t => !t.completed);
  }

  listCompleted(): Todo[] {
    return this.list().filter(t => t.completed);
  }

  complete(id: string): Todo | undefined {
    const todo = this.todos.get(id);
    if (todo) {
      todo.completed = true;
      todo.completed_at = new Date().toISOString();
      return todo;
    }
    return undefined;
  }

  uncomplete(id: string): Todo | undefined {
    const todo = this.todos.get(id);
    if (todo) {
      todo.completed = false;
      todo.completed_at = undefined;
      return todo;
    }
    return undefined;
  }

  delete(id: string): boolean {
    return this.todos.delete(id);
  }

  stats() {
    const total = this.todos.size;
    const completed = this.listCompleted().length;
    return {
      total,
      completed,
      pending: total - completed,
    };
  }
}

const App: React.FC = () => {
  const [service] = useState(() => new TodoService());
  const [todos, setTodos] = useState<Todo[]>([]);
  const [newTodoTitle, setNewTodoTitle] = useState('');
  const [newTodoDesc, setNewTodoDesc] = useState('');
  const [filter, setFilter] = useState<'all' | 'pending' | 'completed'>('all');

  const refreshTodos = () => {
    setTodos(service.list());
  };

  useEffect(() => {
    refreshTodos();
  }, [service]);

  const addTodo = () => {
    if (!newTodoTitle.trim()) return;

    service.add(newTodoTitle, newTodoDesc || undefined);
    setNewTodoTitle('');
    setNewTodoDesc('');
    refreshTodos();
  };

  const toggleTodo = (id: string) => {
    const todo = service.get(id);
    if (todo) {
      if (todo.completed) {
        service.uncomplete(id);
      } else {
        service.complete(id);
      }
      refreshTodos();
    }
  };

  const deleteTodo = (id: string) => {
    service.delete(id);
    refreshTodos();
  };

  const filteredTodos = todos.filter(todo => {
    switch (filter) {
      case 'pending': return !todo.completed;
      case 'completed': return todo.completed;
      default: return true;
    }
  });

  const stats = service.stats();

  const getPriorityColor = (priority: number) => {
    switch (priority) {
      case 1: return '#ff4444'; // Red for high priority
      case 2: return '#ffaa44'; // Orange
      case 3: return '#ffff44'; // Yellow
      case 4: return '#44ff44'; // Green
      case 5: return '#4444ff'; // Blue for low priority
      default: return '#ffffff';
    }
  };

  return (
    <div className="app">
      <header>
        <h1>TODO:// OCEAN EDITION</h1>
        <div className="stats">
          Total: {stats.total} | Pending: {stats.pending} | Completed: {stats.completed}
        </div>
      </header>

      <div className="controls">
        <div className="add-todo">
          <input
            type="text"
            placeholder="Enter todo title..."
            value={newTodoTitle}
            onChange={(e) => setNewTodoTitle(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && addTodo()}
          />
          <input
            type="text"
            placeholder="Description (optional)..."
            value={newTodoDesc}
            onChange={(e) => setNewTodoDesc(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && addTodo()}
          />
          <button onClick={addTodo}>[ADD]</button>
        </div>

        <div className="filters">
          <button
            className={filter === 'all' ? 'active' : ''}
            onClick={() => setFilter('all')}
          >
            ALL
          </button>
          <button
            className={filter === 'pending' ? 'active' : ''}
            onClick={() => setFilter('pending')}
          >
            PENDING
          </button>
          <button
            className={filter === 'completed' ? 'active' : ''}
            onClick={() => setFilter('completed')}
          >
            COMPLETED
          </button>
        </div>
      </div>

      <div className="todo-list">
        {filteredTodos.map(todo => (
          <div key={todo.id} className={`todo-item ${todo.completed ? 'completed' : ''}`}>
            <div className="todo-header">
              <span
                className="priority-indicator"
                style={{ color: getPriorityColor(todo.priority) }}
              >
                [{todo.priority}]
              </span>
              <span className="todo-title">{todo.title}</span>
              <div className="todo-actions">
                <button onClick={() => toggleTodo(todo.id)}>
                  {todo.completed ? '[UNDO]' : '[DONE]'}
                </button>
                <button onClick={() => deleteTodo(todo.id)}>[DEL]</button>
              </div>
            </div>
            {todo.description && (
              <div className="todo-description">{todo.description}</div>
            )}
            <div className="todo-meta">
              <span>ID: {todo.id}</span>
              <span>Created: {new Date(todo.created_at).toLocaleString()}</span>
              {todo.completed_at && (
                <span>Completed: {new Date(todo.completed_at).toLocaleString()}</span>
              )}
            </div>
            {todo.tags.length > 0 && (
              <div className="todo-tags">
                {todo.tags.map(tag => (
                  <span key={tag} className="tag">#{tag}</span>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

export default App;