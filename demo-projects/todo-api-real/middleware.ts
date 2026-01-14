// Middleware layer - authentication, logging, error handling
import express, { Request, Response, NextFunction } from 'express';

export interface RequestWithUser extends Request {
    userId?: string;
    startTime?: number;
}

export class Middleware {
    // Logging middleware
    static logger() {
        return (req: RequestWithUser, res: Response, next: NextFunction) => {
            req.startTime = Date.now();
            const original = res.json;

            res.json = function (data: any) {
                const duration = Date.now() - (req.startTime || 0);
                console.log(`${req.method} ${req.path} - ${res.statusCode} (${duration}ms)`);
                return original.call(this, data);
            };

            next();
        };
    }

    // Simple API key authentication
    static authenticate(apiKey: string) {
        return (req: Request, res: Response, next: NextFunction) => {
            const key = req.headers['x-api-key'];

            if (!key || key !== apiKey) {
                res.status(401).json({ success: false, error: 'Unauthorized' });
                return;
            }

            next();
        };
    }

    // Error handler
    static errorHandler() {
        return (err: any, req: Request, res: Response, next: NextFunction) => {
            console.error('Error:', err);
            res.status(500).json({ success: false, error: 'Internal server error' });
        };
    }

    // Request validation
    static validateJson() {
        return express.json();
    }

    // CORS
    static cors() {
        return (req: Request, res: Response, next: NextFunction) => {
            res.header('Access-Control-Allow-Origin', '*');
            res.header('Access-Control-Allow-Methods', 'GET, POST, PATCH, DELETE, OPTIONS');
            res.header('Access-Control-Allow-Headers', 'Content-Type, X-API-Key');

            if (req.method === 'OPTIONS') {
                res.sendStatus(200);
                return;
            }

            next();
        };
    }

    // Rate limiting
    static rateLimit(requestsPerMinute: number = 60) {
        const requests: Map<string, number[]> = new Map();

        return (req: RequestWithUser, res: Response, next: NextFunction) => {
            const ip = (req.ip || 'unknown') as string;
            const now = Date.now();

            if (!requests.has(ip)) {
                requests.set(ip, []);
            }

            const times = requests.get(ip)!;
            const recentRequests = times.filter((t) => now - t < 60000);

            if (recentRequests.length >= requestsPerMinute) {
                res.status(429).json({ success: false, error: 'Rate limit exceeded' });
                return;
            }

            recentRequests.push(now);
            requests.set(ip, recentRequests);

            next();
        };
    }
}
