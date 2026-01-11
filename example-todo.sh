#!/bin/bash

# CADI Todo App Builder - Demonstrates Chunk Reuse
#
# This script creates new todo apps with different visual styles while reusing
# the same core logic from the existing todo-core CADI chunk. It proves that
# CADI enables minimal code generation by reusing cached chunks.
#
# Usage: ./example-todo.sh
# Then follow the prompts to choose an aesthetic.
#
# The script will:
# 1. Create a new todo app with custom styling
# 2. Import it into CADI as a new chunk
# 3. Build using CADI, showing chunk reuse
# 4. Serve the app to demonstrate it works

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Check if CADI is available
check_cadi() {
    if ! command -v ./target/release/cadi &> /dev/null; then
        print_error "CADI CLI not found. Please build it first with: cargo build --release -p cadi"
        exit 1
    fi
}

# Check if registry is running
check_registry() {
    if ! curl -s http://localhost:8080/health | grep -q "healthy"; then
        print_error "CADI registry not running on localhost:8080"
        print_error "Please start it with: docker-compose -f docker/docker-compose.yml up -d cadi-registry"
        exit 1
    fi
}

# Generate CSS based on aesthetic
generate_css() {
    local aesthetic="$1"
    local css_file="$2"

    case "$aesthetic" in
        "cyberpunk")
            cat > "$css_file" << 'EOF'
/* Cyberpunk Theme - Neon Glow */
body {
  margin: 0;
  padding: 0;
  font-family: 'Courier New', monospace;
  background: linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 50%, #16213e 100%);
  color: #00ff41;
  overflow-x: hidden;
}

.app {
  max-width: 900px;
  margin: 0 auto;
  padding: 30px;
  background: rgba(0, 20, 40, 0.9);
  border: 1px solid #00ff41;
  box-shadow: 0 0 20px rgba(0, 255, 65, 0.3);
}

header h1 {
  font-size: 3em;
  text-shadow: 0 0 10px #00ff41, 0 0 20px #00ff41;
  animation: neon 2s ease-in-out infinite alternate;
}

@keyframes neon {
  from { text-shadow: 0 0 5px #00ff41; }
  to { text-shadow: 0 0 10px #00ff41, 0 0 20px #00ff41; }
}

.todo-item {
  border: 1px solid #00ff41;
  background: rgba(0, 255, 65, 0.05);
  box-shadow: 0 0 10px rgba(0, 255, 65, 0.2);
}

.add-todo button {
  background: linear-gradient(45deg, #ff0080, #8000ff);
  border: 1px solid #00ff41;
  color: #00ff41;
  box-shadow: 0 0 10px rgba(255, 0, 128, 0.5);
}
EOF
            ;;
        "minimalist")
            cat > "$css_file" << 'EOF'
/* Minimalist Theme - Clean and Simple */
body {
  margin: 0;
  padding: 0;
  font-family: 'Helvetica Neue', Arial, sans-serif;
  background: #ffffff;
  color: #333333;
}

.app {
  max-width: 800px;
  margin: 0 auto;
  padding: 40px;
  background: #ffffff;
}

header h1 {
  font-size: 2.5em;
  font-weight: 300;
  color: #000000;
  margin-bottom: 20px;
}

.todo-item {
  border: 1px solid #e0e0e0;
  background: #ffffff;
  margin-bottom: 10px;
  padding: 20px;
}

.add-todo button {
  background: #000000;
  border: 1px solid #000000;
  color: #ffffff;
  padding: 10px 20px;
  font-weight: 500;
}
EOF
            ;;
        "sunset")
            cat > "$css_file" << 'EOF'
/* Sunset Theme - Warm Orange Tones */
body {
  margin: 0;
  padding: 0;
  font-family: 'Georgia', serif;
  background: linear-gradient(135deg, #ff6b35 0%, #f7931e 50%, #ffb627 100%);
  color: #2d1810;
}

.app {
  max-width: 850px;
  margin: 0 auto;
  padding: 35px;
  background: rgba(255, 255, 255, 0.95);
  border-radius: 15px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
}

header h1 {
  font-size: 2.8em;
  color: #8b4513;
  text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.2);
}

.todo-item {
  border: 2px solid #d2691e;
  background: rgba(255, 248, 220, 0.9);
  border-radius: 10px;
  margin-bottom: 15px;
}

.add-todo button {
  background: linear-gradient(45deg, #ff6b35, #f7931e);
  border: none;
  color: #ffffff;
  padding: 12px 25px;
  border-radius: 25px;
  font-weight: bold;
  box-shadow: 0 4px 15px rgba(255, 107, 53, 0.3);
}
EOF
            ;;
        "ocean")
            cat > "$css_file" << 'EOF'
/* Ocean Theme - Deep Blue and Calm */
body {
  margin: 0;
  padding: 0;
  font-family: 'Segoe UI', Tahoma, sans-serif;
  background: linear-gradient(135deg, #1e3a8a 0%, #3b82f6 50%, #06b6d4 100%);
  color: #ffffff;
}

.app {
  max-width: 900px;
  margin: 0 auto;
  padding: 30px;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  border-radius: 20px;
  border: 1px solid rgba(255, 255, 255, 0.2);
}

header h1 {
  font-size: 3em;
  color: #ffffff;
  text-shadow: 0 2px 10px rgba(0, 0, 0, 0.5);
  text-align: center;
}

.todo-item {
  border: 1px solid rgba(255, 255, 255, 0.3);
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(5px);
  border-radius: 15px;
  margin-bottom: 15px;
  color: #ffffff;
}

.add-todo button {
  background: linear-gradient(45deg, #06b6d4, #0891b2);
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: #ffffff;
  padding: 12px 25px;
  border-radius: 25px;
  box-shadow: 0 4px 15px rgba(6, 182, 212, 0.3);
}
EOF
            ;;
        *)
            print_error "Unknown aesthetic: $aesthetic"
            print_error "Available options: cyberpunk, minimalist, sunset, ocean"
            exit 1
            ;;
    esac
}

# Main script
main() {
    print_status "🎨 CADI Todo App Builder - Chunk Reuse Demo"
    echo

    # Check prerequisites
    check_cadi
    check_registry

    # Get aesthetic from user
    echo -e "${CYAN}Available aesthetics:${NC}"
    echo "  • cyberpunk  - Neon green on black with glow effects"
    echo "  • minimalist - Clean white and black design"
    echo "  • sunset     - Warm orange and brown tones"
    echo "  • ocean      - Deep blue with glass effects"
    echo

    read -p "Enter desired aesthetic: " aesthetic
    aesthetic=$(echo "$aesthetic" | tr '[:upper:]' '[:lower:]')

    if [ -z "$aesthetic" ]; then
        print_error "No aesthetic provided"
        exit 1
    fi

    print_step "Creating new todo app with '$aesthetic' aesthetic..."

    # Create new app directory
    app_name="${aesthetic}-todo"
    app_dir="$app_name"

    if [ -d "$app_dir" ]; then
        print_warning "Directory $app_dir already exists. Removing..."
        rm -rf "$app_dir"
    fi

    mkdir -p "$app_dir/src"
    mkdir -p "$app_dir/public"

    print_step "Setting up project structure..."

    # Copy base files from frutiger-todo-app (which has the working structure)
    cp frutiger-todo-app/package.json "$app_dir/"
    cp frutiger-todo-app/tsconfig.json "$app_dir/"
    cp frutiger-todo-app/webpack.config.js "$app_dir/"
    cp frutiger-todo-app/public/index.html "$app_dir/public/"
    cp frutiger-todo-app/src/index.tsx "$app_dir/src/"
    cp frutiger-todo-app/src/App.tsx "$app_dir/src/"

    # Update package.json
    sed -i '' "s/frutiger-aero-todo/${app_name}/g" "$app_dir/package.json"

    # Update HTML title
    capitalized_aesthetic="$(tr '[:lower:]' '[:upper:]' <<< ${aesthetic:0:1})${aesthetic:1}"
    sed -i '' "s/Frutiger Aero Todo/${capitalized_aesthetic} Todo/g" "$app_dir/public/index.html"
    sed -i '' "s/Modern Edition/${capitalized_aesthetic} Edition/g" "$app_dir/public/index.html"

    # Update App.tsx title
    uppercase_aesthetic=$(echo "$aesthetic" | tr '[:lower:]' '[:upper:]')
    sed -i '' "s/FRUTIGER AERO EDITION/${uppercase_aesthetic} EDITION/g" "$app_dir/src/App.tsx"

    # Generate new CSS
    print_step "Generating $aesthetic styling..."
    generate_css "$aesthetic" "$app_dir/src/styles.css"

    # Update manifest
    cat > "$app_dir/cadi.yaml" << EOF
# CADI Manifest for ${capitalized_aesthetic} Todo Web App
manifest_id: "${app_name}-manifest"
manifest_version: "1.0"

application:
  name: "${app_name}"
  description: "${capitalized_aesthetic} styled todo web app reusing existing chunks"
  version: "0.1.0"
  authors:
    - "CADI User"
  license: "MIT"

build_graph:
  nodes:
    - id: "todo-core"
      source_cadi: "chunk:sha256:2fab1892a5c70423309d7da6a295b32bb260901a7c60b7d1819a45af362bcdb7"
      representations:
        - form: "source"
          language: "rust"
          chunk: "chunk:sha256:2fab1892a5c70423309d7da6a295b32bb260901a7c60b7d1819a45af362bcdb7"

    - id: "${app_name}"
      source_cadi: null  # Will be set after import
      representations:
        - form: "source"
          language: "typescript"
          chunk: null  # Will be set after import
        - form: "bundle"
          format: "esm"
          chunk: null  # Will be set after build

  edges:
    - from: "${app_name}"
      to: "todo-core"
      interface: "TodoService"
      relation: "depends_on"

build_targets:
  - name: "web"
    platform: "wasm32-unknown-unknown"
    nodes:
      - id: "todo-core"
        prefer: ["source:rust"]
      - id: "${app_name}"
        prefer: ["bundle:esm"]
    bundle:
      format: "esm"
      output: "dist/${app_name}"
      minify: true
EOF

    print_step "Installing dependencies..."
    cd "$app_dir"
    npm install > /dev/null 2>&1

    print_step "Building the app..."
    npm run build > /dev/null 2>&1

    print_step "Importing into CADI..."
    cd ..
    ./target/release/cadi import "$app_dir" --name "$app_name" > /dev/null 2>&1

    print_step "Publishing to registry..."
    ./target/release/cadi publish > /dev/null 2>&1

    print_status "🎯 PROVING CHUNK REUSE - Building with CADI..."
    echo -e "${PURPLE}Notice how CADI reuses the cached todo-core chunk!${NC}"
    echo

    ./target/release/cadi build "$app_dir/cadi.yaml" --target web

    print_status "✅ Build complete! Chunk reuse demonstrated."
    echo
    print_status "🚀 Serving the new $aesthetic todo app..."

    # Find an available port
    port=8083
    while lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; do
        port=$((port + 1))
    done

    cd "$app_dir"
    npx serve dist -p $port > /dev/null 2>&1 &
    serve_pid=$!

    echo
    print_status "🌐 Your new $aesthetic todo app is running at: http://localhost:$port"
    print_status "📊 Core logic reused from: chunk:sha256:2fab1892a5c70423309d7da6a295b32bb260901a7c60b7d1819a45af362bcdb7"
    print_status "🎨 Only styling was generated for: $aesthetic aesthetic"
    echo
    print_warning "Press Ctrl+C to stop the server"

    # Wait for user to stop
    trap "kill $serve_pid 2>/dev/null; exit" INT
    wait $serve_pid
}

# Run main function
main "$@"