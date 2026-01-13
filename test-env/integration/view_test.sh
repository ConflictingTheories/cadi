#!/usr/bin/env bash
set -euo pipefail

PORT=${PORT:-8083}
SERVER_LOG=$(mktemp -t cadi-server-log-XXXX)
STORAGE_DIR=$(mktemp -d -t cadi-storage-XXXX)
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

cleanup() {
  echo "Cleaning up..."
  if [ -n "${SERVER_PID:-}" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill $SERVER_PID || true
    wait $SERVER_PID 2>/dev/null || true
  fi
  rm -rf "$SERVER_LOG" "$STORAGE_DIR"
}
trap cleanup EXIT

# Prepare two snippets and insert them into the graph DB (before starting server)
A_FILE="$SCRIPT_DIR/a.rs"
B_FILE="$SCRIPT_DIR/b.rs"
cat > "$A_FILE" <<'EOF'
pub fn helper() -> i32 { 42 }
EOF
cat > "$B_FILE" <<'EOF'
pub fn use_helper() -> i32 { helper() }
EOF

# Compute chunk IDs
HASH_A=$(shasum -a 256 "$A_FILE" | awk '{print $1}')
HASH_B=$(shasum -a 256 "$B_FILE" | awk '{print $1}')
CHUNK_A="chunk:sha256:${HASH_A}"
CHUNK_B="chunk:sha256:${HASH_B}"

# Use a small Rust example to insert nodes into the graph DB before server starts
cargo run -p cadi-core --example insert_nodes --quiet -- "$STORAGE_DIR" "$A_FILE" "$B_FILE"

# Start server now that graph DB has nodes
CADI_BIND="127.0.0.1:${PORT}" CADI_STORAGE="$STORAGE_DIR" RUST_LOG=info cargo run -p cadi-server --quiet >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!

# Wait for health
for i in {1..60}; do
  if curl -sSf "http://127.0.0.1:${PORT}/health" >/dev/null 2>&1; then
    echo "Server healthy"
    break
  fi
  sleep 0.5
  if [ $i -eq 60 ]; then
    echo "Server failed to become healthy. Log output:" >&2
    tail -n +1 "$SERVER_LOG" >&2
    exit 1
  fi
done

# Ask server for a view
RESP=$(curl -s -X POST -H "Content-Type: application/json" -d "{\"atoms\":[\"$CHUNK_B\"],\"expansion_depth\":1}" "http://127.0.0.1:${PORT}/v1/views")

if echo "$RESP" | grep -q "$CHUNK_A"; then
  echo "View endpoint returned ghost atom $CHUNK_A"
  exit 0
else
  echo "View endpoint failed. Response:" >&2
  echo "$RESP" >&2
  exit 1
fi
