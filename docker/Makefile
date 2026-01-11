# =============================================================================
# CADI Docker Makefile
# =============================================================================
# Convenient targets for Docker operations

.PHONY: help build build-all up down logs shell clean

# Default target
help:
	@echo "CADI Docker Commands"
	@echo "===================="
	@echo ""
	@echo "Infrastructure:"
	@echo "  make up          - Start CADI server infrastructure"
	@echo "  make up-full     - Start with PostgreSQL, Redis, MinIO"
	@echo "  make down        - Stop all containers"
	@echo "  make logs        - View logs"
	@echo "  make status      - Show container status"
	@echo ""
	@echo "Development:"
	@echo "  make dev         - Start development environment"
	@echo "  make shell       - Open interactive dev shell"
	@echo "  make build-dev   - Build project in container"
	@echo "  make watch       - Start watch mode"
	@echo ""
	@echo "Images:"
	@echo "  make build-images   - Build all Docker images"
	@echo "  make build-server   - Build server image only"
	@echo "  make build-cli      - Build CLI image only"
	@echo "  make build-dev-img  - Build dev image only"
	@echo ""
	@echo "Maintenance:"
	@echo "  make clean       - Remove containers and volumes"
	@echo "  make prune       - Remove unused Docker resources"

# -----------------------------------------------------------------------------
# Infrastructure
# -----------------------------------------------------------------------------

up:
	docker compose up -d cadi-registry cadi-mcp
	@echo ""
	@echo "CADI Server running at http://localhost:8080"
	@echo "MCP Server running at http://localhost:9090"

up-full:
	docker compose --profile full up -d
	@echo ""
	@echo "Full CADI infrastructure running"

down:
	docker compose down
	docker compose -f docker-compose.dev.yml down 2>/dev/null || true

logs:
	docker compose logs -f

status:
	@echo "=== CADI Containers ==="
	@docker compose ps
	@echo ""
	@docker compose -f docker-compose.dev.yml ps 2>/dev/null || true

# -----------------------------------------------------------------------------
# Development
# -----------------------------------------------------------------------------

dev:
	docker compose -f docker-compose.dev.yml up -d cadi-dev cadi-registry
	@echo ""
	@echo "Development environment ready"
	@echo "Run 'make shell' to enter the dev container"

shell:
	docker compose -f docker-compose.dev.yml exec cadi-dev /bin/bash || \
	./scripts/cadi-build.sh --shell

build-dev:
	./scripts/cadi-build.sh

watch:
	docker compose -f docker-compose.dev.yml --profile watch up cadi-watch

# -----------------------------------------------------------------------------
# Images
# -----------------------------------------------------------------------------

build-images: build-server build-cli build-mcp build-dev-img

build-server:
	docker build -t cadi/registry:latest -f Dockerfile.server ..

build-cli:
	docker build -t cadi/cli:latest -f Dockerfile.cli ..

build-mcp:
	docker build -t cadi/mcp-server:latest -f Dockerfile.mcp ..

build-dev-img:
	docker build -t cadi/dev:latest -f Dockerfile.dev ..

# -----------------------------------------------------------------------------
# Maintenance
# -----------------------------------------------------------------------------

clean:
	docker compose down -v
	docker compose -f docker-compose.dev.yml down -v 2>/dev/null || true
	docker volume rm cadi-data cadi-cache cadi-dev-cache cadi-build-cache 2>/dev/null || true

prune:
	docker system prune -f
	docker volume prune -f
