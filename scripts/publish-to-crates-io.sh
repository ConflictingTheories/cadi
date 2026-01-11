#!/bin/bash

# CADI Crates.io Publishing Script
# Publishes all CADI library crates to crates.io in dependency order
# NOTE: Use this for PUBLISHING ONLY. Library crates publish independently.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "╔═══════════════════════════════════════════════════════════════════════════════╗"
echo "║                   CADI Crates.io Publishing                                  ║"
echo "║                      Library Crates Only                                      ║"
echo "╚═══════════════════════════════════════════════════════════════════════════════╝"
echo ""

# Library crates only (publish in dependency order)
# The CLI will be published separately after libraries are available on crates.io
LIBRARY_CRATES=(
  "internal/cadi-core:cadi-core"
  "internal/cadi-builder:cadi-builder"
  "internal/cadi-registry:cadi-registry"
  "internal/cadi-scraper:cadi-scraper"
)

# Function to publish a crate
publish_crate() {
  local path=$1
  local name=$2
  local root=$3
  
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "📦 Publishing: $name"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  
  cd "$root/$path"
  
  echo "  📋 Checking Cargo.toml..."
  if grep -q "path =" Cargo.toml; then
    echo "  ⚠️  WARNING: Crate has path dependencies (this is OK for workspace development)"
    echo "     When publishing, these must be available on crates.io"
    echo ""
  fi
  
  echo "  🔍 Running cargo package --allow-dirty..."
  cargo package --allow-dirty 2>&1 | grep -E "Packaged|error|warning" | head -20
  
  if [ $? -eq 0 ]; then
    echo "  ✓ Crate packaged successfully"
  else
    echo "  ✗ Crate packaging failed"
    cd "$root"
    return 1
  fi
  
  echo ""
  echo "  🧪 Running cargo publish --dry-run..."
  cargo publish --dry-run --allow-dirty 2>&1 | grep -E "Uploading|Verifying|Compiling|Finished|error" | head -30
  
  if [ $? -eq 0 ]; then
    echo "  ✓ Dry-run successful"
  else
    echo "  ✗ Dry-run failed"
    cd "$root"
    return 1
  fi
  
  echo ""
  read -p "  ▶ Publish '$name' to crates.io NOW? (y/n) " -n 1 -r
  echo
  if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "  📤 Publishing to crates.io..."
    if cargo publish --allow-dirty 2>&1 | grep -E "Uploaded|error"; then
      echo "  ✓ $name published successfully!"
      echo ""
      echo "  ⏳ Waiting for crates.io index to update (60s)..."
      echo "     This ensures the next crate can find this one as a dependency"
      for i in {1..6}; do
        echo -ne "  [$i/6] 10s..."
        sleep 10
      done
      echo ""
    else
      echo "  ✗ Publication failed"
      cd "$root"
      return 1
    fi
  else
    echo "  ⊘ Skipped publication"
    cd "$root"
    return 1
  fi
  
  cd "$root"
  return 0
}

# Check authentication
echo "🔐 Checking crates.io authentication..."
echo ""
if [ -f ~/.cargo/credentials.toml ]; then
  echo "✓ You are logged into crates.io"
else
  echo "❌ You are NOT logged into crates.io"
  echo ""
  echo "To login:"
  echo "  1. Get your token from: https://crates.io/me"
  echo "  2. Run: cargo login <YOUR_TOKEN>"
  echo ""
  exit 1
fi
echo ""

# Verify git is clean
echo "📁 Checking git status..."
cd "$PROJECT_ROOT"

if [ -d .git ]; then
  if git diff-index --quiet HEAD -- 2>/dev/null; then
    echo "✓ Git repository is clean"
  else
    echo "⚠ Git repository has uncommitted changes"
    echo "  WARNING: Uncommitted changes will be included in the published crate"
    echo ""
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
      exit 1
    fi
  fi
fi
echo ""

# Publish crates in order
echo "📦 Publishing $(echo ${#LIBRARY_CRATES[@]}) library crates..."
echo "(CLI will be published separately after libraries are available on crates.io)"
echo ""

PUBLISHED=()
FAILED=()

for crate_info in "${LIBRARY_CRATES[@]}"; do
  IFS=':' read -r path name <<< "$crate_info"
  
  if publish_crate "$path" "$name" "$PROJECT_ROOT"; then
    PUBLISHED+=("$name")
  else
    FAILED+=("$name")
  fi
  echo ""
done

# Summary
echo "╔═══════════════════════════════════════════════════════════════════════════════╗"
echo "║                           PUBLICATION SUMMARY                                ║"
echo "╚═══════════════════════════════════════════════════════════════════════════════╝"
echo ""

if [ ${#PUBLISHED[@]} -gt 0 ]; then
  echo "✓ Successfully published:"
  for crate in "${PUBLISHED[@]}"; do
    echo "  • https://crates.io/crates/$crate"
  done
  echo ""
fi

if [ ${#FAILED[@]} -gt 0 ]; then
  echo "✗ Failed or skipped:"
  for crate in "${FAILED[@]}"; do
    echo "  • $crate"
  done
  echo ""
fi

if [ ${#FAILED[@]} -eq 0 ]; then
  echo "🎉 All library crates published successfully!"
  echo ""
  echo "📝 Next: Publish the CLI"
  echo "   cd cmd/cadi && cargo publish"
  echo ""
else
  echo "⚠ Some crates were not published"
  echo ""
  echo "To retry, fix any issues and run this script again"
  exit 1
fi
