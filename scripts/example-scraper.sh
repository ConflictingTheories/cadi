#!/bin/bash
# Example CADI Scraper Workflow
# This script demonstrates how to use the CADI scraper to convert
# source code repositories into reusable CADI chunks and publish them.

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
CADI_CMD="${SCRIPT_DIR}/target/debug/cadi"
OUTPUT_DIR="${SCRIPT_DIR}/cadi-scraped-chunks"

echo "================================================"
echo "CADI Scraper Example Workflow"
echo "================================================"
echo ""

# Check if cadi is built
if [ ! -f "$CADI_CMD" ]; then
    echo "Building CADI CLI..."
    cargo build --bin cadi
    echo ""
fi

# Example 1: Scrape a local directory (the examples/todo-suite)
echo "Example 1: Scraping local directory (todo-suite)..."
echo "Command: $CADI_CMD scrape ./examples/todo-suite --strategy semantic --output $OUTPUT_DIR/todo-suite"
echo ""

if [ -d "./examples/todo-suite" ]; then
    "$CADI_CMD" scrape ./examples/todo-suite \
        --strategy semantic \
        --output "$OUTPUT_DIR/todo-suite" \
        --hierarchy true \
        --extract-api true \
        --detect-licenses true \
        --format table
    echo ""
    echo "✓ Successfully scraped todo-suite"
    echo ""
    
    # Show what was created
    echo "Scraped chunks:"
    ls -lh "$OUTPUT_DIR/todo-suite" | tail -5
    echo ""
else
    echo "⚠ examples/todo-suite not found, skipping this example"
    echo ""
fi

# Example 2: Scrape with hierarchical chunking
echo "Example 2: Scraping with hierarchical chunking..."
echo "Command: $CADI_CMD scrape ./internal/cadi-core --strategy hierarchical --output $OUTPUT_DIR/cadi-core-hierarchical"
echo ""

if [ -d "./internal/cadi-core" ]; then
    "$CADI_CMD" scrape ./internal/cadi-core \
        --strategy hierarchical \
        --output "$OUTPUT_DIR/cadi-core-hierarchical" \
        --hierarchy true \
        --extract-api true \
        --format table
    echo ""
    echo "✓ Successfully created hierarchical chunks for cadi-core"
    echo ""
else
    echo "⚠ internal/cadi-core not found, skipping this example"
    echo ""
fi

# Example 3: Dry run to see what would be scraped
echo "Example 3: Dry-run scrape (preview without saving)..."
echo "Command: $CADI_CMD scrape ./cmd/cadi/src --dry-run --strategy by-line-count"
echo ""

"$CADI_CMD" scrape ./cmd/cadi/src \
    --strategy by-line-count \
    --output "$OUTPUT_DIR/cadi-cli-preview" \
    --dry-run \
    --format table
echo ""

# Example 4: Demonstrate publishing workflow
echo "Example 4: Publishing workflow (setup required)..."
echo ""
echo "To publish chunks to a registry:"
echo ""
echo "  1. Scrape your repository:"
echo "     $ cadi scrape /path/to/repo --output ./my-chunks"
echo ""
echo "  2. Configure registry URL:"
echo "     $ export CADI_REGISTRY_URL='https://registry.example.com'"
echo "     $ export CADI_AUTH_TOKEN='your-auth-token'"
echo ""
echo "  3. Publish the chunks:"
echo "     $ cadi publish --registry \$CADI_REGISTRY_URL --auth-token \$CADI_AUTH_TOKEN --namespace myorg/myproject"
echo ""
echo "  4. Or directly with scraper:"
echo "     $ cadi scrape /path/to/repo --publish --namespace myorg/myproject"
echo ""

# Example 5: Show different chunking strategies
echo "Example 5: Comparing chunking strategies..."
echo ""
echo "Available strategies:"
echo "  - by-file: Chunk entire files (default, fastest)"
echo "  - semantic: Chunk by functions/classes/methods"
echo "  - fixed-size: Chunk by fixed byte size"
echo "  - hierarchical: Hierarchical chunks with parent-child relationships"
echo "  - by-line-count: Chunk by line count (100 lines default)"
echo ""

# Create a simple test file
TEST_FILE=$(mktemp /tmp/test-scrape-XXXXX.rs)
cat > "$TEST_FILE" << 'EOF'
/// Module documentation
pub mod example {
    /// A simple function
    pub fn hello() {
        println!("Hello, world!");
    }
    
    /// Another function
    pub fn goodbye() {
        println!("Goodbye!");
    }
}

/// A struct example
pub struct Person {
    name: String,
    age: u32,
}

impl Person {
    pub fn new(name: String, age: u32) -> Self {
        Self { name, age }
    }
}
EOF

echo "Testing with temporary file: $TEST_FILE"
echo ""

for strategy in "by-file" "semantic" "by-line-count"; do
    echo "Strategy: $strategy"
    "$CADI_CMD" scrape "$TEST_FILE" \
        --strategy "$strategy" \
        --output "$OUTPUT_DIR/test-$strategy" \
        --format table | head -10
    echo ""
done

rm "$TEST_FILE"

# Summary
echo "================================================"
echo "Example workflow completed!"
echo "================================================"
echo ""
echo "Created chunks in: $OUTPUT_DIR"
echo "Directory structure:"
find "$OUTPUT_DIR" -type d | head -10
echo ""
echo "Next steps:"
echo "  1. Review the scraped chunks in $OUTPUT_DIR"
echo "  2. Configure your registry URL and auth token"
echo "  3. Use 'cadi publish' to publish chunks to your registry"
echo "  4. Share your CADI repository with others!"
echo ""
