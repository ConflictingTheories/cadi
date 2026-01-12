# CADI Benchmarks

This folder contains guidelines and scripts to run benchmarks for CADI.

Quick start:

- Run all crate benches (from workspace root):

  cargo bench -p cadi-core

- The benches use `criterion` for consistent, repeatable results.

Notes:
- To measure Tree-sitter vs fallback extraction, toggle the `ast-parsing` feature on `internal/cadi-core` and rerun `cargo bench`.
- For CI: consider running benchmarks on an isolated machine to ensure minimal noise.
