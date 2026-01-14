#!/bin/bash

# CADI MCP Test Script
# Tests the MCP server integration with CADI

echo "🧪 Testing CADI MCP Server Integration"
echo "======================================"

# Test 1: Initialize
echo ""
echo "1. Testing MCP Initialize..."
INIT_RESPONSE=$(echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test-client", "version": "1.0.0"}}}' | target/debug/cadi-mcp-server 2>/dev/null)

if echo "$INIT_RESPONSE" | grep -q '"result"'; then
    echo "✅ Initialize successful"
else
    echo "❌ Initialize failed"
    exit 1
fi

# Test 2: List Tools
echo ""
echo "2. Testing Tools List..."
TOOLS_RESPONSE=$(echo '{"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}' | target/debug/cadi-mcp-server 2>/dev/null)

if echo "$TOOLS_RESPONSE" | grep -q "cadi_search"; then
    echo "✅ Tools list successful - found cadi_search tool"
else
    echo "❌ Tools list failed"
    exit 1
fi

# Test 3: List Resources
echo ""
echo "3. Testing Resources List..."
RESOURCES_RESPONSE=$(echo '{"jsonrpc": "2.0", "id": 3, "method": "resources/list", "params": {}}' | target/debug/cadi-mcp-server 2>/dev/null)

if echo "$RESOURCES_RESPONSE" | grep -q "cadi://config"; then
    echo "✅ Resources list successful - found cadi://config resource"
else
    echo "❌ Resources list failed"
    exit 1
fi

# Test 4: Call a tool
echo ""
echo "4. Testing Tool Call (cadi_search)..."
SEARCH_RESPONSE=$(echo '{"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "cadi_search", "arguments": {"query": "test"}}}' | target/debug/cadi-mcp-server 2>/dev/null)

if echo "$SEARCH_RESPONSE" | grep -q '"result"'; then
    echo "✅ Tool call successful"
else
    echo "❌ Tool call failed"
    exit 1
fi

echo ""
echo "🎉 All MCP tests passed!"
echo ""
echo "📋 Available CADI Tools via MCP:"
echo "$TOOLS_RESPONSE" | grep -o '"name":"[^"]*"' | sed 's/"name":"//g' | sed 's/"//g' | while read tool; do
    echo "   • $tool"
done

echo ""
echo "📚 Available CADI Resources via MCP:"
echo "$RESOURCES_RESPONSE" | grep -o '"uri":"[^"]*"' | sed 's/"uri":"//g' | sed 's/"//g' | while read resource; do
    echo "   • $resource"
done

echo ""
echo "🔗 VS Code Integration:"
echo "   • MCP server configured in .vscode/settings.json"
echo "   • Copilot MCP extension installed"
echo "   • Ready for agentic coding workflows!"