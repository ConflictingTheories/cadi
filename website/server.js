const express = require('express');
const helmet = require('helmet');
const rateLimit = require('express-rate-limit');
const { body, validationResult } = require('express-validator');
const sanitizeHtml = require('sanitize-html');
const cors = require('cors');
const compression = require('compression');
const morgan = require('morgan');
const path = require('path');

const app = express();
const PORT = process.env.PORT || 3000;

// Security Middleware
app.use(helmet({
    contentSecurityPolicy: {
        directives: {
            defaultSrc: ["'self'"],
            styleSrc: ["'self'", "'unsafe-inline'", "https://fonts.googleapis.com"],
            fontSrc: ["'self'", "https://fonts.gstatic.com"],
            scriptSrc: ["'self'"],
            imgSrc: ["'self'", "data:", "https:"],
            connectSrc: ["'self'"],
        },
    },
    hsts: {
        maxAge: 31536000,
        includeSubDomains: true,
        preload: true
    }
}));

// Rate Limiting - DDoS Protection
const limiter = rateLimit({
    windowMs: 15 * 60 * 1000, // 15 minutes
    max: 100, // limit each IP to 100 requests per windowMs
    message: 'Too many requests from this IP, please try again later.',
    standardHeaders: true,
    legacyHeaders: false,
});

const strictLimiter = rateLimit({
    windowMs: 15 * 60 * 1000,
    max: 10, // stricter limit for sensitive endpoints
    message: 'Too many requests, please try again later.',
});

app.use(limiter);

// Compression
app.use(compression());

// Logging
app.use(morgan('combined'));

// CORS Configuration
app.use(cors({
    origin: process.env.ALLOWED_ORIGINS ? process.env.ALLOWED_ORIGINS.split(',') : ['http://localhost:3000'],
    credentials: true,
    optionsSuccessStatus: 200
}));

// Body Parser with size limits
app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true, limit: '10mb' }));

// Input Sanitization Middleware
app.use((req, res, next) => {
    // Sanitize all string inputs
    const sanitizeObject = (obj) => {
        for (let key in obj) {
            if (typeof obj[key] === 'string') {
                obj[key] = sanitizeHtml(obj[key], {
                    allowedTags: [],
                    allowedAttributes: {}
                });
            } else if (typeof obj[key] === 'object' && obj[key] !== null) {
                sanitizeObject(obj[key]);
            }
        }
    };

    if (req.body) sanitizeObject(req.body);
    if (req.query) sanitizeObject(req.query);

    next();
});

// RPC Validation Middleware
const validateRpcRequest = (req, res, next) => {
    const { jsonrpc, method, params, id } = req.body;

    // Validate JSON-RPC 2.0 format
    if (jsonrpc !== '2.0') {
        return res.status(400).json({
            jsonrpc: '2.0',
            error: { code: -32600, message: 'Invalid Request' },
            id: null
        });
    }

    // Validate method
    const allowedMethods = ['getStats', 'getChunks', 'searchChunks', 'getStorageInfo'];
    if (!allowedMethods.includes(method)) {
        return res.status(400).json({
            jsonrpc: '2.0',
            error: { code: -32601, message: 'Method not found' },
            id
        });
    }

    // Validate params
    if (params && typeof params !== 'object') {
        return res.status(400).json({
            jsonrpc: '2.0',
            error: { code: -32602, message: 'Invalid params' },
            id
        });
    }

    next();
};

// Routes

// Serve static files
app.use(express.static(path.join(__dirname, 'assets')));
app.use('/css', express.static(path.join(__dirname, 'css')));
app.use('/js', express.static(path.join(__dirname, 'js')));
app.use('/docs', express.static(path.join(__dirname, 'docs')));

// Main page
app.get('/', (req, res) => {
    res.sendFile(path.join(__dirname, 'index.html'));
});

// RPC Endpoint with validation
app.post('/rpc', validateRpcRequest, [
    body('method').isString().isLength({ min: 1, max: 50 }),
    body('params').optional().isObject(),
    body('id').isInt({ min: 1 })
], (req, res) => {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
        return res.status(400).json({
            jsonrpc: '2.0',
            error: { code: -32602, message: 'Invalid params', data: errors.array() },
            id: req.body.id
        });
    }

    const { method, params, id } = req.body;

    // Handle RPC methods
    switch (method) {
        case 'getStats':
            res.json({
                jsonrpc: '2.0',
                result: {
                    cache: { hits: 42, misses: 8, hitRate: 0.84 },
                    reuse: { chunksReused: 156, bytesDeduplicated: 52428800 },
                    build: { totalBuilds: 23, avgDuration: 4.2 }
                },
                id
            });
            break;

        case 'getChunks':
            const page = Math.max(1, parseInt(params.page) || 1);
            const limit = Math.min(100, Math.max(1, parseInt(params.limit) || 20));
            const chunks = Array.from({ length: limit }, (_, i) => ({
                id: `chunk_${(page - 1) * limit + i + 1}`,
                name: `example_${(page - 1) * limit + i + 1}.rs`,
                size: 1024 + ((page - 1) * limit + i) * 100
            }));
            res.json({
                jsonrpc: '2.0',
                result: { chunks, page, limit, total: 1000 },
                id
            });
            break;

        case 'searchChunks':
            const query = params.query || '';
            if (query.length > 100) {
                return res.status(400).json({
                    jsonrpc: '2.0',
                    error: { code: -32602, message: 'Query too long' },
                    id
                });
            }
            res.json({
                jsonrpc: '2.0',
                result: { results: [], query },
                id
            });
            break;

        case 'getStorageInfo':
            res.json({
                jsonrpc: '2.0',
                result: {
                    totalChunks: 1234,
                    totalSize: 268435456,
                    registryConnected: true
                },
                id
            });
            break;

        default:
            res.status(404).json({
                jsonrpc: '2.0',
                error: { code: -32601, message: 'Method not found' },
                id
            });
    }
});

// Health check endpoint
app.get('/health', (req, res) => {
    res.json({ status: 'ok', timestamp: new Date().toISOString() });
});

// Error handling middleware
app.use((err, req, res, next) => {
    console.error(err.stack);
    res.status(500).json({
        error: 'Internal Server Error',
        message: process.env.NODE_ENV === 'development' ? err.message : 'Something went wrong'
    });
});

// 404 handler
app.use((req, res) => {
    res.status(404).json({ error: 'Not Found' });
});

// Start server
app.listen(PORT, () => {
    console.log(`CADI Website Server running on port ${PORT}`);
    console.log(`Security features enabled: Helmet, Rate Limiting, Input Sanitization, RPC Validation`);
});

module.exports = app;
