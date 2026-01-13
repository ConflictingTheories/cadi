# Smoke Integration Test

This quickstart shows how to run the integration smoke test locally. The test exercises a local CADI registry server and validates the following flow:

1. Start `cadi-server` with a temporary storage directory
2. Upload a test chunk via HTTP PUT
3. Use the `cadi` CLI to `fetch` the chunk into a local cache
4. Run `cadi verify` against the fetched chunk

Quick commands:

- Run the smoke script directly:

  bash test-env/integration/smoke_test.sh

- Run the integration test (ignored by default):

  cargo test -- --ignored --nocapture

Notes:

- The script requires `shasum` (available on macOS and most Linux runners) and `curl`.
- The test builds the `cadi-server` and `cadi` binaries locally (so the first run may take a little time).
- A GitHub Actions workflow runs this script on PRs: `.github/workflows/integration_smoke.yml`.
Additional integration tests

- Atomizer + Virtual View test: `tests/integration_atomizer_view.rs` validates that the atomizer extracts function atoms, the graph store indexes symbols, and the `RehydrationEngine` can assemble virtual views with ghost imports.

Run the atomizer view test locally:

  cargo test -p cadi-core -- --test-threads=1 integration_atomizer_virtual_view