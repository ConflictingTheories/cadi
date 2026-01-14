# GitHub Copilot Instructions for CADI Workspace

## 🛡️ THE NORTH STAR (NON-NEGOTIABLE)
- **CADI is not a tool. CADI is the way code should work with LLMs.**
- **Goal**: Move from "pasting codebases" to "**referencing semantic atoms**."
- **Invariant**: LLM + CADI > LLM alone (Cost, Tokens, Speed, Quality).
- **Simplicity**: No more complex than working with an LLM directly. 

## ⚡ CADI-FIRST WORKFLOW (keeps costs low)
- **Search first** with CADI tools before writing code: `cadi_search`, `cadi_resolve_alias`, `cadi_get_chunk`.
- **Reference, don't generate**: If a semantic atom exists, use its ID.
- **Only write new code if no reusable atom exists** and then `cadi_import` it into the registry.

## 🔍 Big-picture architecture (Simplified)
- **Universal Storage**: **SurrealDB** handles everything—Graph, Relational, and Document data.
- **Core Truth**: `internal/cadi-core/src/normalizer.rs` defines the semantic identity of code (Alpha-Renaming).
- CLI: `cmd/cadi` is the user's entry point.
- MCP bridge: `cmd/cadi-mcp-server` exposes CADI tools to LLMs.
- Core logic and features live in `internal/*` (e.g., `cadi-core`, `cadi-builder`, `cadi-registry`, `llm`).
- Formal contract and types: `cadi-spec/` contains CADL schemas and JSON schemas used for validation.

Files to consult for design intent:
- `README.md` (project overview & MCP examples)
- `docs/architecture.md` (dataflows, chunk lifecycle)
- `cadi-spec/` (contracts & schema rules)

## ✅ Key developer workflows (concrete commands)
Build
- Local Rust build: `cargo build --workspace` or `cargo build --release` (CI uses `cargo build --release`).
- Cross-language or reproducible builds: use `./scripts/cadi-build.sh` (or `make dev` inside `docker/`).

Test & CI
- Unit/integration: `cargo test --all-features` (CI runs tests with this flag).
- Smoke integration: `bash test-env/integration/smoke_test.sh` (or `cargo test -- --ignored --nocapture`).
- MCP end-to-end smoke test: `./scripts/test-mcp-integration.sh` (runs `target/release/cadi-mcp-server`).
- Formatting & linting used by CI:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`

Development environment
- Dev workstation (container): `docker/make dev` or `make dev` in `docker/`.
- Interactive dev shell: `make shell` (inside `docker/`) or `./scripts/cadi-build.sh --shell`.

Run servers locally
- Start the registry server: `target/release/cadi-server` (or via `docker compose up` in `docker/`).
- MCP server: `target/release/cadi-mcp-server` (used by `./scripts/test-mcp-integration.sh`).

Debugging & logs
- Docker logs: `docker compose logs cadi-registry` (from `docker/`).
- Health check: `curl http://localhost:8080/health`.

## 🧭 Project-specific conventions & patterns
- CADL-first: interfaces capture behavior, effects and contracts (see `cadi-spec/` for examples).
- Chunk identity: chunks are referenced as `chunk:sha256:<digest>` and carry metadata + representations.
- Don't duplicate functionality that exists as a chunk—prefer composition and imports.
- Example manifests: `examples/todo-suite/*.cadi.yaml` show real manifest usage and targets.
- CI enforces strict formatting and linting; keep `rustfmt`-style formatting and resolve Clippy warnings early.

## 🔗 Integration points & external systems
- Optional infra: PostgreSQL, Redis, MinIO (configured via `docker/` and `.env.example`).
- WASM support (Wasmtime) and OCI container publishing (`oci-distribution`).
- MCP protocol: `cmd/cadi-mcp-server` exposes tools like `cadi_search`, `cadi_get_chunk`, and resources `cadi://config`.
- Vector/embedding work: see `internal/llm` and use `pgvector` for search indices.

## ✍️ How an AI agent should behave here (practical rules)
1. **Search for chunks first** (use `cadi_search` → `cadi_get_chunk`) before proposing or writing code.
2. **If proposing new code**, include a short plan and tests; prefer adding code under a focused path and add a `cadi_import` step in the workflow.
3. **Use CI mirrors**: follow `cargo test --all-features`, `cargo fmt --check`, `cargo clippy` locally before opening PRs.
4. **Reference concrete files** in suggestions (e.g., `cmd/cadi`, `internal/cadi-core`, `cadi-spec/`).

## ⛑️ Useful examples & quick refs
- MCP test script: `./scripts/test-mcp-integration.sh` (checks `cadi_search`, `cadi://config`).
- Smoke test: `test-env/integration/smoke_test.sh` (end-to-end registry/CLI validation).
- CI workflow: `.github/workflows/ci.yml` (shows exact commands used in CI).
- Docker dev: `docker/docker-compose.dev.yml` and `docker/Makefile` for launch & debugging.

---

If anything here is unclear or you'd like more examples (e.g., a short agent-playbook for a typical change), tell me which area to expand and I will iterate. 
