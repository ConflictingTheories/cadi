// Main server application
import express, { Express } from 'express';
import { TodoDatabase } from './database';
import { TodoApiHandler } from './handler';
import { Middleware } from './middleware';

export class TodoServer {
    private app: Express;
    private db: TodoDatabase;
    private handler: TodoApiHandler;
    private port: number;

    constructor(port: number = 3000, dbPath: string = ':memory:') {
        this.port = port;
        this.app = express();
        this.db = new TodoDatabase(dbPath);
        this.handler = new TodoApiHandler(this.db);

        this.configureMiddleware();
        this.setupRoutes();
    }

    private configureMiddleware(): void {
        // CORS first
        this.app.use(Middleware.cors());

        // Request parsing
        this.app.use(Middleware.validateJson());

        // Logging
        this.app.use(Middleware.logger());

        // Rate limiting
        this.app.use(Middleware.rateLimit(100));

        // API key authentication (optional - can be disabled)
        // this.app.use(Middleware.authenticate('secret-key'));
    }

    private setupRoutes(): void {
        // Health check
        this.app.get('/health', (req, res) => {
            res.status(200).json({
                status: 'healthy',
                timestamp: new Date().toISOString(),
            });
        });

        // API routes
        this.app.use('/api', this.handler.getRouter());

        // 404 handler
        this.app.use('*', (req, res) => {
            res.status(404).json({ success: false, error: 'Not found' });
        });

        // Error handler
        this.app.use(Middleware.errorHandler());
    }

    public start(): void {
        this.app.listen(this.port, () => {
            console.log(`✓ Todo API Server running on http://localhost:${this.port}`);
            console.log(`✓ Health check: GET http://localhost:${this.port}/health`);
            console.log(`✓ API Base: http://localhost:${this.port}/api`);
        });
    }

    public getApp(): Express {
        return this.app;
    }

    public getDatabase(): TodoDatabase {
        return this.db;
    }

    public close(): void {
        this.db.close();
    }
}

// Entry point
if (require.main === module) {
    const server = new TodoServer(3000, './todos.db');
    server.start();
}

export default TodoServer;
