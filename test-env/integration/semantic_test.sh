#!/usr/bin/env bash
set -euo pipefail

PORT=${PORT:-8082}
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

# Upload a test blob
printf "semantics test content" > "$SCRIPT_DIR/test.bin"
HASH=$(shasum -a 256 "$SCRIPT_DIR/test.bin" | awk '{print $1}')
CHUNK_ID="chunk:sha256:${HASH}"

HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X PUT --data-binary @"$SCRIPT_DIR/test.bin" "http://127.0.0.1:${PORT}/v1/chunks/${CHUNK_ID}")
if [ "$HTTP_CODE" != "200" ] && [ "$HTTP_CODE" != "201" ]; then
  echo "Upload failed (HTTP $HTTP_CODE)" >&2
  tail -n 200 "$SERVER_LOG" >&2
  exit 1
fi

# Query semantic search
RESP=$(curl -s -X POST -H "Content-Type: application/json" -d '{"query":"semantics","limit":10}' "http://127.0.0.1:${PORT}/v1/semantic_search")

if echo "$RESP" | grep -q "${CHUNK_ID}"; then
  echo "Semantic search returned chunk"
  exit 0
else
  echo "Semantic search failed to return chunk. Response:" >&2
  echo "$RESP" >&2
  exit 1
fi
