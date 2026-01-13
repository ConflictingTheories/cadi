#!/usr/bin/env bash
set -euo pipefail

PORT=${PORT:-8081}
SERVER_LOG=$(mktemp -t cadi-server-log-XXXX)
STORAGE_DIR=$(mktemp -d -t cadi-storage-XXXX)
CACHE_DIR=$(mktemp -d -t cadi-cache-XXXX)
CONFIG_FILE=$(mktemp -t cadi-config-XXXX.yaml)
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

cleanup() {
  echo "Cleaning up..."
  if [ -n "${SERVER_PID:-}" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill $SERVER_PID || true
    wait $SERVER_PID 2>/dev/null || true
  fi
  rm -rf "$SERVER_LOG" "$STORAGE_DIR" "$CACHE_DIR" "$CONFIG_FILE"
}
trap cleanup EXIT

echo "Starting cadi server on 127.0.0.1:${PORT} with storage=${STORAGE_DIR}"
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

# Create test content and compute chunk id
printf "hello cadi smoke test\n" > "$SCRIPT_DIR/test.bin"
if ! command -v shasum >/dev/null 2>&1; then
  echo "shasum not available" >&2
  exit 1
fi
HASH=$(shasum -a 256 "$SCRIPT_DIR/test.bin" | awk '{print $1}')
CHUNK_ID="chunk:sha256:${HASH}"

echo "Uploading chunk ${CHUNK_ID} to registry"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X PUT --data-binary @"$SCRIPT_DIR/test.bin" "http://127.0.0.1:${PORT}/v1/chunks/${CHUNK_ID}")
if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "201" ]; then
  echo "Upload OK (HTTP $HTTP_CODE)"
else
  echo "Upload failed (HTTP $HTTP_CODE)" >&2
  tail -n 200 "$SERVER_LOG" >&2
  exit 1
fi

# Write a minimal cadi config pointing to local registry and test cache
cat > "$CONFIG_FILE" <<EOF
registry:
  url: "http://127.0.0.1:${PORT}"
cache:
  dir: "$CACHE_DIR"
security:
  verify_on_fetch: false
  trust_policy: "permissive"
EOF

# Use the cadi CLI to fetch the chunk
echo "Fetching chunk via CLI into cache: $CACHE_DIR"
cargo run -p cadi -- --config "$CONFIG_FILE" fetch "$CHUNK_ID"

# Verify the fetched file exists and matches
HASH_F=$(shasum -a 256 "$CACHE_DIR/chunks/${HASH}.bin" | awk '{print $1}')
if [ "$HASH_F" != "$HASH" ]; then
  echo "Fetched content hash mismatch: expected $HASH, got $HASH_F" >&2
  exit 1
fi

echo "Running cadi verify"
cargo run -p cadi -- --config "$CONFIG_FILE" verify "$CHUNK_ID"

echo "Smoke test completed successfully"

# Run atomizer + virtual view integration test (fast, in-process)
echo "Running atomizer -> virtual view integration test"
cargo test -p cadi-core -- --test-threads=1 integration_atomizer_virtual_view

# Run end-to-end view test (uses admin API to create nodes/edges)
echo "Running end-to-end view test (HTTP)"
bash ./test-env/integration/view_test.sh || { echo "View test failed"; exit 1; }

exit 0
