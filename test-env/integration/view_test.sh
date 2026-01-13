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

# Prepare two snippets
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

# Start server (use admin token for tests)
TOKEN="devtoken"
CADI_BIND="127.0.0.1:${PORT}" CADI_STORAGE="$STORAGE_DIR" CADI_ADMIN_TOKEN="$TOKEN" RUST_LOG=info cargo run -p cadi-server --quiet >"$SERVER_LOG" 2>&1 &
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

# Insert nodes via admin API (use Authorization header)
BODY_A=$(python3 - <<PY
import json
print(json.dumps({"chunk_id": "$CHUNK_A", "content": open("$A_FILE").read(), "language":"rust", "defines":["helper"]}))
PY
)
HTTP_A=$(curl -s -o /dev/null -w "%{http_code}" -X POST -H "Content-Type: application/json" -H "Authorization: Bearer $TOKEN" --data "$BODY_A" "http://127.0.0.1:${PORT}/v1/admin/nodes")
if [ "$HTTP_A" -ne 201 ]; then
  echo "Failed to insert node A, status $HTTP_A" >&2
  exit 1
fi

BODY_B=$(python3 - <<PY
import json
print(json.dumps({"chunk_id": "$CHUNK_B", "content": open("$B_FILE").read(), "language":"rust", "references":["helper"]}))
PY
)
HTTP_B=$(curl -s -o /dev/null -w "%{http_code}" -X POST -H "Content-Type: application/json" -H "Authorization: Bearer $TOKEN" --data "$BODY_B" "http://127.0.0.1:${PORT}/v1/admin/nodes")
if [ "$HTTP_B" -ne 201 ]; then
  echo "Failed to insert node B, status $HTTP_B" >&2
  exit 1
fi

# Add edge B -> A
EDGE_BODY=$(python3 - <<PY
import json
print(json.dumps({"source": "$CHUNK_B", "target": "$CHUNK_A", "edge_type": "imports"}))
PY
)
EDGE_HTTP=$(curl -s -o /dev/null -w "%{http_code}" -X POST -H "Content-Type: application/json" -H "Authorization: Bearer $TOKEN" --data "$EDGE_BODY" "http://127.0.0.1:${PORT}/v1/admin/edges")
if [ "$EDGE_HTTP" -ne 201 ]; then
  echo "Failed to add edge, status $EDGE_HTTP" >&2
  exit 1
fi

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
