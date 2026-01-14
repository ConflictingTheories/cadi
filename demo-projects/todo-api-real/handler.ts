// API handler layer - HTTP endpoint logic
import express, { Router, Request, Response } from 'express';
import { TodoDatabase } from './database';
import { CreateTodoInput, UpdateTodoInput } from './types';

export class TodoApiHandler {
    private db: TodoDatabase;
    private router: Router;

    constructor(db: TodoDatabase) {
        this.db = db;
        this.router = Router();
        this.setupRoutes();
    }

    private setupRoutes(): void {
        // GET /todos - List all todos
        this.router.get('/todos', (req, res) => this.listTodos(req, res));

        // GET /todos/:id - Get single todo
        this.router.get('/todos/:id', (req, res) => this.getTodo(req, res));

        // POST /todos - Create new todo
        this.router.post('/todos', (req, res) => this.createTodo(req, res));

        // PATCH /todos/:id - Update todo
        this.router.patch('/todos/:id', (req, res) => this.updateTodo(req, res));

        // DELETE /todos/:id - Delete todo
        this.router.delete('/todos/:id', (req, res) => this.deleteTodo(req, res));

        // GET /todos/priority/:priority - Filter by priority
        this.router.get('/todos/priority/:priority', (req, res) => this.getByPriority(req, res));

        // GET /todos/status/completed - Get completed todos
        this.router.get('/todos/status/completed', (req, res) => this.getCompleted(req, res));

        // GET /todos/status/pending - Get pending todos
        this.router.get('/todos/status/pending', (req, res) => this.getPending(req, res));

        // GET /search?q=... - Search todos
        this.router.get('/search', (req, res) => this.search(req, res));
    }

    private listTodos(req: Request, res: Response): void {
        try {
            const todos = this.db.getAllTodos();
            res.status(200).json({
                success: true,
                data: todos,
                count: todos.length,
            });
        } catch (error) {
            res.status(500).json({ success: false, error: 'Failed to list todos' });
        }
    }

    private getTodo(req: Request, res: Response): void {
        try {
            const { id } = req.params;
            const todo = this.db.getTodoById(id);

            if (!todo) {
                res.status(404).json({ success: false, error: 'Todo not found' });
                return;
            }

            res.status(200).json({ success: true, data: todo });
        } catch (error) {
            res.status(500).json({ success: false, error: 'Failed to get todo' });
        }
    }

    private createTodo(req: Request, res: Response): void {
        try {
            const { title, description, priority, dueDate, tags } = req.body;

            if (!title) {
                res.status(400).json({ success: false, error: 'Title is required' });
                return;
            }

            const input: CreateTodoInput = {
                title,
                description,
                priority,
                dueDate,
                tags,
            };

            const todo = this.db.createTodo(input);
            res.status(201).json({ success: true, data: todo });
        } catch (error) {
            res.status(500).json({ success: false, error: 'Failed to create todo' });
        }
    }

    private updateTodo(req: Request, res: Response): void {
        try {
            const { id } = req.params;
            const { title, completed, priority } = req.body;

            const input: UpdateTodoInput = {
                title,
                completed,
                priority,
            };

            const updated = this.db.updateTodo(id, input);

            if (!updated) {
                res.status(404).json({ success: false, error: 'Todo not found' });
                return;
            }

            res.status(200).json({ success: true, data: updated });
        } catch (error) {
            res.status(500).json({ success: false, error: 'Failed to update todo' });
        }
    }

    private deleteTodo(req: Request, res: Response): void {
        try {
            const { id } = req.params;
            const deleted = this.db.deleteTodo(id);

            if (!deleted) {
                res.status(404).json({ success: false, error: 'Todo not found' });
                return;
            }

            res.status(204).send();
        } catch (error) {
            res.status(500).json({ success: false, error: 'Failed to delete todo' });
        }
    }

    private getByPriority(req: Request, res: Response): void {
        try {
            const { priority } = req.params;
            const todos = this.db.getTodosByPriority(priority);

            res.status(200).json({
                success: true,
                data: todos,
                count: todos.length,
            });
        } catch (error) {
            res.status(500).json({ success: false, error: 'Failed to filter todos' });
        }
    }

    private getCompleted(req: Request, res: Response): void {
        try {
            const todos = this.db.getCompletedTodos();
            res.status(200).json({
                success: true,
                data: todos,
                count: todos.length,
            });
        } catch (error) {
            res.status(500).json({ success: false, error: 'Failed to get completed todos' });
        }
    }

    private getPending(req: Request, res: Response): void {
        try {
            const todos = this.db.getPendingTodos();
            res.status(200).json({
                success: true,
                data: todos,
                count: todos.length,
            });
        } catch (error) {
            res.status(500).json({ success: false, error: 'Failed to get pending todos' });
        }
    }

    private search(req: Request, res: Response): void {
        try {
            const { q } = req.query;

            if (!q || typeof q !== 'string') {
                res.status(400).json({ success: false, error: 'Search query required' });
                return;
            }

            const results = this.db.searchTodos(q);
            res.status(200).json({
                success: true,
                data: results,
                count: results.length,
            });
        } catch (error) {
            res.status(500).json({ success: false, error: 'Failed to search todos' });
        }
    }

    public getRouter(): Router {
        return this.router;
    }
}
