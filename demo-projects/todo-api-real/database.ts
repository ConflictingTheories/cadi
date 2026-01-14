// Database layer - handles all data persistence
import Database from 'better-sqlite3';
import { Todo, CreateTodoInput, UpdateTodoInput } from './types';

export class TodoDatabase {
    private db: Database.Database;

    constructor(filePath: string = ':memory:') {
        this.db = new Database(filePath);
        this.initSchema();
    }

    private initSchema(): void {
        this.db.exec(`
      CREATE TABLE IF NOT EXISTS todos (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        description TEXT,
        completed BOOLEAN DEFAULT 0,
        dueDate TEXT,
        priority TEXT DEFAULT 'medium',
        tags TEXT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
      )
    `);
    }

    // Get all todos
    getAllTodos(): Todo[] {
        const stmt = this.db.prepare('SELECT * FROM todos ORDER BY created_at DESC');
        return stmt.all() as Todo[];
    }

    // Get single todo by ID
    getTodoById(id: string): Todo | null {
        const stmt = this.db.prepare('SELECT * FROM todos WHERE id = ?');
        return stmt.get(id) as Todo | null;
    }

    // Create new todo
    createTodo(input: CreateTodoInput): Todo {
        const id = Math.random().toString(36).substr(2, 9);
        const stmt = this.db.prepare(`
      INSERT INTO todos (id, title, description, priority, dueDate, tags)
      VALUES (?, ?, ?, ?, ?, ?)
    `);

        stmt.run(
            id,
            input.title,
            input.description || null,
            input.priority || 'medium',
            input.dueDate || null,
            input.tags ? JSON.stringify(input.tags) : null
        );

        return this.getTodoById(id)!;
    }

    // Update existing todo
    updateTodo(id: string, input: UpdateTodoInput): Todo | null {
        const todo = this.getTodoById(id);
        if (!todo) return null;

        const updates: string[] = [];
        const values: any[] = [];

        if (input.title !== undefined) {
            updates.push('title = ?');
            values.push(input.title);
        }
        if (input.completed !== undefined) {
            updates.push('completed = ?');
            values.push(input.completed ? 1 : 0);
        }
        if (input.priority !== undefined) {
            updates.push('priority = ?');
            values.push(input.priority);
        }

        updates.push('updated_at = CURRENT_TIMESTAMP');
        values.push(id);

        const stmt = this.db.prepare(`
      UPDATE todos SET ${updates.join(', ')} WHERE id = ?
    `);
        stmt.run(...values);

        return this.getTodoById(id);
    }

    // Delete todo
    deleteTodo(id: string): boolean {
        const stmt = this.db.prepare('DELETE FROM todos WHERE id = ?');
        const result = stmt.run(id);
        return (result.changes ?? 0) > 0;
    }

    // Get todos by priority
    getTodosByPriority(priority: string): Todo[] {
        const stmt = this.db.prepare('SELECT * FROM todos WHERE priority = ? ORDER BY created_at DESC');
        return stmt.all(priority) as Todo[];
    }

    // Get completed todos
    getCompletedTodos(): Todo[] {
        const stmt = this.db.prepare('SELECT * FROM todos WHERE completed = 1 ORDER BY created_at DESC');
        return stmt.all() as Todo[];
    }

    // Get pending todos
    getPendingTodos(): Todo[] {
        const stmt = this.db.prepare('SELECT * FROM todos WHERE completed = 0 ORDER BY created_at DESC');
        return stmt.all() as Todo[];
    }

    // Search todos
    searchTodos(query: string): Todo[] {
        const search = `%${query}%`;
        const stmt = this.db.prepare(`
      SELECT * FROM todos 
      WHERE title LIKE ? OR description LIKE ? 
      ORDER BY created_at DESC
    `);
        return stmt.all(search, search) as Todo[];
    }

    // Close database
    close(): void {
        this.db.close();
    }
}
