# CADI Benchmark Results

This document records quick micro-benchmark comparisons for `AtomExtractor` before and after enabling `ast-parsing` (Tree-sitter).

## Micro-bench (small, quick runs)

Command: `cargo run -p cadi-core --example bench_runner --release`

- Without `ast-parsing` (default disabled):
  - rust::extract — ~406 ms (avg of 3 iters)
  - jsx::extract — ~6,969 ms (avg of 3 iters)
  - tsx::extract — ~8,176 ms (avg of 3 iters)
  - html::extract — ~0.44 ms (avg of 3 iters)

- With `ast-parsing` (Tree-sitter enabled):
  - rust::extract — ~18 ms (avg of 3 iters)  ✅ major improvement
  - jsx::extract — ~7,101 ms (avg of 3 iters)  (no meaningful change)
  - tsx::extract — ~8,262 ms (avg of 3 iters)  (no meaningful change)
  - html::extract — ~0.44 ms (avg of 3 iters)

## Notes & Interpretation

- Enabling Tree-sitter (`ast-parsing`) produced a very large improvement for Rust extraction (likely due to the Tree-sitter Rust queries/path being significantly faster than the regex fallback).
- JSX/TSX extraction did not improve meaningfully in these quick measurements. Possible reasons:
  - The extractor path for JS/TSX may still be dominated by non-AST work, or Tree-sitter queries for JSX/TSX are not optimized yet.
  - Further work: add/optimize Tree-sitter queries for JSX/TSX in `atomizer/languages/typescript.rs` and `languages/jsx.rs`.

## Next Steps

1. Run full `criterion` benches for longer runs and record results to `target/criterion` (CI friendly).  
2. Improve Tree-sitter queries for JSX/TSX and re-bench.  
3. Given the Rust improvement, we enabled `ast-parsing` by default in `internal/cadi-core/Cargo.toml`.

---

(Generated automatically by the bench routine.)
