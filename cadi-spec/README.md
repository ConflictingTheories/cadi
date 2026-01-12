# CADI Specifications

This directory contains JSON Schema specifications for CADI artifacts. Each file defines the expected structure for a specific artifact type.

Specs included:

- `chunk.schema.json` — Base schema for all CADI chunks (meta, provides, licensing, lineage).
- `atomic-chunk.schema.json` — Schema for small, reusable "atomic" chunks (aliases, composition, metrics, sources).
- `source-cadi.schema.json` — Source code chunks (files, language, runtime deps).
- `ir-cadi.schema.json` — Intermediate representations (WASM, LLVM IR, module exports/imports).
- `blob-cadi.schema.json` — Binary blobs (architecture-specific binaries, build info).
- `container-cadi.schema.json` — OCI container image metadata and layers.
- `manifest.schema.json` — Application manifests describing build graphs and targets.
- `build-receipt.schema.json` — Build provenance (steps, tools, verification).
- `dependency-graph.schema.json` — Dependency graphs and resolution/lock information.
- `registry-federation.schema.json` — Registry federation and namespace/trust configuration.
- `abi-compatibility.schema.json` — ABI/platform compatibility rules and detection.
- `security-attestation.schema.json` — Signing and attestation schema (SLSA, signatures).
- `efficiency-metrics.schema.json` — Metrics for reuse, LLM efficiency, energy estimates.
- `llm-context.schema.json` — Optimized chunk representations for LLM consumption (summaries, API surface, embeddings).
- `versioning.schema.json` — Versioning semantics and history.
- `garbage-collection.schema.json` — Retention and GC policies for caches and registries.

Each schema file includes a `$schema` and `$id` entry and is intended to be used by tooling and validation pipelines.