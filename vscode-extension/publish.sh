#!/bin/bash

# CADI Extension Publishing Script
# This script helps automate the publishing process

set -e

echo "🚀 CADI VS Code Extension Publisher"
echo "=================================="

# Check if vsce is installed
if ! command -v vsce &> /dev/null; then
    echo "❌ vsce not found. Installing globally..."
    npm install -g @vscode/vsce
fi

# Check if we're in the right directory
if [ ! -f "package.json" ] || [ ! -d "out" ]; then
    echo "❌ Not in vscode-extension directory or extension not built"
    echo "Run this script from the vscode-extension directory"
    exit 1
fi

echo "📦 Checking extension package..."
npm run compile

echo "🔐 Checking VSCE login status..."
if ! vsce verify-pat ConflictingTheories &> /dev/null; then
    echo "⚠️  Not logged in to VS Code marketplace"
    echo "Run: vsce login ConflictingTheories"
    echo "Then enter your Personal Access Token"
    exit 1
fi

echo "✅ Login verified"

# Get version from package.json
VERSION=$(node -p "require('./package.json').version")
echo "📋 Current version: $VERSION"

# Ask for version bump type
echo "Select version bump type:"
echo "1) Patch ($VERSION -> $(node -p "const v='$VERSION'.split('.'); v[2]=parseInt(v[2])+1; v.join('.')"))"
echo "2) Minor ($VERSION -> $(node -p "const v='$VERSION'.split('.'); v[1]=parseInt(v[1])+1; v[2]=0; v.join('.')"))"
echo "3) Major ($VERSION -> $(node -p "const v='$VERSION'.split('.'); v[0]=parseInt(v[0])+1; v[1]=0; v[2]=0; v.join('.')"))"
echo "4) No version bump (publish current version)"
read -p "Enter choice (1-4): " choice

case $choice in
    1)
        echo "📦 Publishing patch version..."
        vsce publish patch
        ;;
    2)
        echo "📦 Publishing minor version..."
        vsce publish minor
        ;;
    3)
        echo "📦 Publishing major version..."
        vsce publish major
        ;;
    4)
        echo "📦 Publishing current version..."
        vsce publish
        ;;
    *)
        echo "❌ Invalid choice"
        exit 1
        ;;
esac

echo "✅ Extension published successfully!"
echo "🔗 Check it at: https://marketplace.visualstudio.com/items?itemName=ConflictingTheories.cadi"
echo "📊 Monitor stats at: https://marketplace.visualstudio.com/manage/publishers/ConflictingTheories"