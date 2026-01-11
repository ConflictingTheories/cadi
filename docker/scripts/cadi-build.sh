#!/bin/bash
# =============================================================================
# CADI Docker Build Script
# =============================================================================
# Run CADI builds inside Docker containers
# Usage: ./cadi-build.sh [options] [cadi-args...]

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCKER_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$DOCKER_DIR")"

# Default values
IMAGE_NAME="cadi/dev:latest"
REGISTRY_URL="http://cadi-registry:8080"
NETWORK="cadi-network"
CACHE_VOLUME="cadi-build-cache"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# -----------------------------------------------------------------------------
# Usage
# -----------------------------------------------------------------------------
usage() {
    cat << EOF
CADI Docker Build System

Usage: $(basename "$0") [options] [-- cadi-args...]

Options:
    -p, --project PATH     Project directory to build (default: current dir)
    -t, --target TARGET    Build target (default: dev)
    -i, --image IMAGE      Docker image to use (default: cadi/dev:latest)
    -r, --registry URL     Registry URL (default: http://cadi-registry:8080)
    -n, --network NAME     Docker network (default: cadi-network)
    --no-cache             Don't use cached chunks
    --publish              Publish built artifacts to registry
    --shell                Start interactive shell instead of building
    -h, --help             Show this help

Examples:
    # Build current project
    $(basename "$0")
    
    # Build specific project with target
    $(basename "$0") -p ./my-project -t release
    
    # Build and publish
    $(basename "$0") --publish
    
    # Interactive development shell
    $(basename "$0") --shell
    
    # Pass additional args to cadi
    $(basename "$0") -- build --verbose --target prod

EOF
}

# -----------------------------------------------------------------------------
# Parse Arguments
# -----------------------------------------------------------------------------
PROJECT_PATH="$(pwd)"
BUILD_TARGET="dev"
NO_CACHE=""
PUBLISH=""
INTERACTIVE=""
CADI_ARGS=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--project)
            PROJECT_PATH="$2"
            shift 2
            ;;
        -t|--target)
            BUILD_TARGET="$2"
            shift 2
            ;;
        -i|--image)
            IMAGE_NAME="$2"
            shift 2
            ;;
        -r|--registry)
            REGISTRY_URL="$2"
            shift 2
            ;;
        -n|--network)
            NETWORK="$2"
            shift 2
            ;;
        --no-cache)
            NO_CACHE="--no-cache"
            shift
            ;;
        --publish)
            PUBLISH="1"
            shift
            ;;
        --shell)
            INTERACTIVE="1"
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        --)
            shift
            CADI_ARGS="$*"
            break
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            exit 1
            ;;
    esac
done

# -----------------------------------------------------------------------------
# Ensure Network Exists
# -----------------------------------------------------------------------------
ensure_network() {
    if ! docker network inspect "$NETWORK" >/dev/null 2>&1; then
        echo -e "${YELLOW}Creating Docker network: $NETWORK${NC}"
        docker network create "$NETWORK"
    fi
}

# -----------------------------------------------------------------------------
# Ensure Registry is Running
# -----------------------------------------------------------------------------
ensure_registry() {
    if ! docker ps --format '{{.Names}}' | grep -q "cadi-registry"; then
        echo -e "${YELLOW}Starting CADI registry...${NC}"
        docker compose -f "$DOCKER_DIR/docker-compose.yml" up -d cadi-registry
        
        # Wait for health
        echo -e "${YELLOW}Waiting for registry to be healthy...${NC}"
        for i in {1..30}; do
            if curl -sf "http://localhost:8080/health" >/dev/null 2>&1; then
                echo -e "${GREEN}Registry is ready${NC}"
                break
            fi
            sleep 1
        done
    fi
}

# -----------------------------------------------------------------------------
# Run Build
# -----------------------------------------------------------------------------
run_build() {
    local project_abs
    project_abs="$(cd "$PROJECT_PATH" && pwd)"
    
    echo -e "${BLUE}"
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║                    CADI Docker Build                          ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    echo "  Project:  $project_abs"
    echo "  Target:   $BUILD_TARGET"
    echo "  Image:    $IMAGE_NAME"
    echo "  Registry: $REGISTRY_URL"
    echo ""
    
    # Build command
    local cmd
    if [ -n "$CADI_ARGS" ]; then
        cmd="cadi $CADI_ARGS"
    else
        cmd="cadi build --target $BUILD_TARGET $NO_CACHE"
        if [ -n "$PUBLISH" ]; then
            cmd="$cmd && cadi publish"
        fi
    fi
    
    # Run container
    docker run --rm \
        --network "$NETWORK" \
        -v "$project_abs:/workspace:cached" \
        -v "$CACHE_VOLUME:/home/developer/.cadi/cache" \
        -e "CADI_REGISTRY_URL=$REGISTRY_URL" \
        -e "RUST_LOG=${RUST_LOG:-info}" \
        -w /workspace \
        "$IMAGE_NAME" \
        bash -c "$cmd"
}

# -----------------------------------------------------------------------------
# Run Interactive Shell
# -----------------------------------------------------------------------------
run_shell() {
    local project_abs
    project_abs="$(cd "$PROJECT_PATH" && pwd)"
    
    echo -e "${BLUE}Starting interactive CADI development shell...${NC}"
    
    docker run --rm -it \
        --network "$NETWORK" \
        -v "$project_abs:/workspace:cached" \
        -v "$CACHE_VOLUME:/home/developer/.cadi/cache" \
        -v /var/run/docker.sock:/var/run/docker.sock \
        -e "CADI_REGISTRY_URL=$REGISTRY_URL" \
        -e "RUST_LOG=${RUST_LOG:-info}" \
        -w /workspace \
        "$IMAGE_NAME" \
        /bin/bash
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    ensure_network
    ensure_registry
    
    if [ -n "$INTERACTIVE" ]; then
        run_shell
    else
        run_build
    fi
}

main
