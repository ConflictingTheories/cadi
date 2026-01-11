#!/bin/bash
# =============================================================================
# CADI Development Container Entrypoint
# =============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}"
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                    CADI Development Environment               ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# -----------------------------------------------------------------------------
# Initialize CADI
# -----------------------------------------------------------------------------
init_cadi() {
    echo -e "${YELLOW}→ Initializing CADI...${NC}"
    
    # Create config directory if it doesn't exist
    mkdir -p "${CADI_CONFIG_DIR:-/home/developer/.cadi}"
    mkdir -p "${CADI_CACHE_DIR:-/home/developer/.cadi/cache}"
    
    # Check if repos.cfg exists, create default if not
    if [ ! -f "${CADI_CONFIG_DIR}/repos.cfg" ]; then
        cat > "${CADI_CONFIG_DIR}/repos.cfg" << 'EOF'
# CADI Registry Configuration
[registries]
default = "local"

[registry.local]
url = "${CADI_REGISTRY_URL:-http://cadi-registry:8080}"
description = "Local development registry"

[registry.official]
url = "https://registry.cadi.dev"
description = "Official CADI registry"
EOF
        echo -e "${GREEN}  ✓ Created repos.cfg${NC}"
    fi
    
    # Test registry connection
    if [ -n "${CADI_REGISTRY_URL}" ]; then
        echo -e "${YELLOW}→ Checking registry connection...${NC}"
        if curl -sf "${CADI_REGISTRY_URL}/health" > /dev/null 2>&1; then
            echo -e "${GREEN}  ✓ Registry available at ${CADI_REGISTRY_URL}${NC}"
        else
            echo -e "${RED}  ⚠ Registry not available at ${CADI_REGISTRY_URL}${NC}"
            echo -e "${YELLOW}    Continuing anyway - registry may start later${NC}"
        fi
    fi
}

# -----------------------------------------------------------------------------
# Setup SSH Agent (if available)
# -----------------------------------------------------------------------------
setup_ssh() {
    if [ -S "/ssh-agent" ]; then
        export SSH_AUTH_SOCK=/ssh-agent
        echo -e "${GREEN}  ✓ SSH agent available${NC}"
    fi
}

# -----------------------------------------------------------------------------
# Setup Git
# -----------------------------------------------------------------------------
setup_git() {
    if [ -f "/home/developer/.gitconfig" ]; then
        echo -e "${GREEN}  ✓ Git configuration available${NC}"
    fi
}

# -----------------------------------------------------------------------------
# Docker-in-Docker Setup
# -----------------------------------------------------------------------------
setup_docker() {
    if [ -S "/var/run/docker.sock" ]; then
        # Ensure developer user can access docker socket
        if ! groups developer | grep -q docker; then
            echo -e "${YELLOW}→ Configuring Docker access...${NC}"
        fi
        
        if docker info > /dev/null 2>&1; then
            echo -e "${GREEN}  ✓ Docker available${NC}"
        else
            echo -e "${YELLOW}  ⚠ Docker socket exists but not accessible${NC}"
        fi
    fi
}

# -----------------------------------------------------------------------------
# Print Environment Info
# -----------------------------------------------------------------------------
print_info() {
    echo ""
    echo -e "${BLUE}Environment:${NC}"
    echo "  • CADI Version:    $(cadi --version 2>/dev/null || echo 'not installed')"
    echo "  • Rust:            $(rustc --version 2>/dev/null || echo 'not installed')"
    echo "  • Node.js:         $(node --version 2>/dev/null || echo 'not installed')"
    echo "  • Python:          $(python3 --version 2>/dev/null || echo 'not installed')"
    echo "  • Go:              $(go version 2>/dev/null | awk '{print $3}' || echo 'not installed')"
    echo ""
    echo -e "${BLUE}Directories:${NC}"
    echo "  • Workspace:       /workspace"
    echo "  • CADI Config:     ${CADI_CONFIG_DIR:-/home/developer/.cadi}"
    echo "  • CADI Cache:      ${CADI_CACHE_DIR:-/home/developer/.cadi/cache}"
    echo ""
    echo -e "${BLUE}Registry:${NC}"
    echo "  • URL:             ${CADI_REGISTRY_URL:-http://cadi-registry:8080}"
    echo ""
    echo -e "${GREEN}Ready! Type 'cadi --help' to get started.${NC}"
    echo ""
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    init_cadi
    setup_ssh
    setup_git
    setup_docker
    print_info
    
    # Execute the command passed to the container
    exec "$@"
}

main "$@"
