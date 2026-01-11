# CADI Local Test Environment - Results

## Test Date: January 11, 2026

## Environment Setup

### Registry Server
- **Status**: ✅ Running
- **URL**: http://localhost:8080
- **Storage**: `./test-env/registry-data` (in-memory)

### Configuration
- **repos.cfg**: `.cadi/repos.cfg`
  - Default registry: `local` (http://localhost:8080)
  - Official registry: https://registry.cadi.dev (placeholder)

## CLI Commands Tested

| Command | Status | Notes |
|---------|--------|-------|
| `cadi init` | ✅ Pass | Creates project structure with cadi.yaml and .cadi/ |
| `cadi import` | ✅ Pass | Imports source code, detects language, creates chunks |
| `cadi build` | ✅ Pass | Shows build plan and executes builds |
| `cadi publish` | ✅ Pass | Uploads chunks to registry via HTTP PUT |
| `cadi fetch` | ✅ Pass | Downloads chunks from registry |
| `cadi verify` | ✅ Pass | Verifies chunk hash and signatures |
| `cadi plan` | ✅ Pass | Shows build plan without executing |
| `cadi stats` | ✅ Pass | Displays cache and efficiency statistics |
| `cadi trust` | ✅ Pass | Lists trusted signers |
| `cadi gc` | ✅ Pass | Garbage collection dry run works |
| `cadi demo` | ✅ Pass | Demo command with todo-suite target |

## Registry API Tested

| Endpoint | Method | Status |
|----------|--------|--------|
| `/health` | GET | ✅ Returns `{"status":"healthy","version":"1.0.0-dev"}` |
| `/v1/chunks` | GET | ✅ Lists all chunks |
| `/v1/chunks/:id` | GET | ✅ Returns chunk data |
| `/v1/chunks/:id` | PUT | ✅ Stores chunk |
| `/v1/chunks/:id` | HEAD | ✅ Checks existence |
| `/v1/stats` | GET | ✅ Returns store statistics |

## Test Projects

### 1. test-project (initialized)
Location: `test-env/test-project/`
- Created via `cadi init`
- Contains `cadi.yaml`, `.cadi/repos.cfg`, `src/lib.rs`

### 2. todo-suite (example)
Location: `examples/todo-suite/`
- Multi-component example with todo-core and todo-cli
- Successfully imported todo-core
- Chunk published to local registry

## Sample Workflow Verified

```bash
# 1. Initialize project
cadi init --name my-project

# 2. Import source code
cadi import ./src --name my-module --no-publish

# 3. View build plan
cadi plan cadi.yaml --target dev

# 4. Build project
cadi build cadi.yaml

# 5. Publish to registry
cadi publish "chunk:sha256:..." --registry http://localhost:8080

# 6. Fetch from registry
cadi fetch "chunk:sha256:..." --registry http://localhost:8080

# 7. Verify chunk
cadi verify "chunk:sha256:..."
```

## Chunk Registry Contents

After testing, the local registry contains:
- `chunk:sha256:ff735870447ea42904de7d82394bbfddbd25bc898c771390a65d3ccc131d4a21`
  - Size: 1140 bytes
  - Type: Source CADI (todo-core Rust code)

## Local Cache

Location: `~/Library/Caches/dev.cadi.cadi/`
- Contains `.json` (metadata) and `.chunk` (data) files
- Chunks are stored by hash

## Known Limitations (Phase 0)

1. **Registry Storage**: Currently in-memory only, chunks lost on restart
2. **Signing**: Signing is simulated, not cryptographically implemented
3. **Build Execution**: Build steps are simulated, don't actually compile
4. **Transform Pipeline**: IR/Blob transformations are stubs

## Next Steps for Phase 1

1. Implement persistent registry storage
2. Add real cryptographic signing
3. Implement actual build execution
4. Add WebSocket support for real-time updates
5. Implement MCP server protocol handlers
