# implementation.md

## CADI v1 Implementation Plan

This document describes the complete v1 implementation plan for CADI (Content-Addressed Development Interface), including concepts, architecture, roadmap, and concrete todos for a team to execute. The scope includes:

- CADI repository (specs, server, CLI, demo suite)
- CADI build toolchain (builder, transformations, cache)
- CADI server & registry
- CADI MCP server (Model Context Protocol interface)[1][2][3][4][5]
- A full demo project suite (web apps, Node.js, C servers, shared SQL DB)[6][7][8]

***

## 1. Vision, Concepts, and Success Criteria

### 1.1 Vision

CADI is a universal build and distribution system for software artifacts, especially AI-generated code, treating all artifacts as content-addressed chunks with multiple interchangeable representations (source, IR/WASM, binaries, containers). It acts as a “global linker” and provenance-aware registry so humans, tools, and LLMs can discover, assemble, and verify software components.

### 1.2 Core Concepts

- **Chunk**
  - Immutable content-addressed unit identified by `chunk:sha256:<digest>`.
  - Contains metadata, representations, lineage, licensing, and “provides” information.

- **Representation**
  - A specific form of a chunk:
    - `source.*` (e.g., `source.typescript`, `source.c`)
    - `intermediate.*` (e.g., `intermediate.wasm`)
    - `binary.<arch>` (e.g., `binary.x86_64-linux`)
    - `container.oci` (OCI image reference)[7][8][6]

- **CADI Spec Tiers**
  - **Source CADI**: Transparency and modifiability; describes source code and links to derived IR/blob specs.
  - **IR CADI**: Portable representations (e.g., WASM) with execution metadata and links upstream (source) and downstream (blobs).
  - **Blob CADI**: Optimized architecture-specific binaries and containers, with ABI and provenance details.

- **Manifest / Build Graph**
  - Describes an application as a graph of chunk nodes with edges (interfaces/relations).
  - Contains build targets mapping nodes to desired representations and platform-specific strategies.

- **CADI Registry**
  - Stores blobs (content-addressed) and CADI specs (chunks, manifests, receipts).
  - Exposes search, retrieval, and planning APIs.
  - Inspired by Nix/Bazel content-addressed stores and OCI registries.[8][9][10][11][1][6][7]

- **CADI Builder (CLI)**
  - Resolves manifests, selects representations, orchestrates transformations, and publishes artifacts.
  - Uses deterministic action keys for caching, similar to Bazel and Nix.[9][10][11][1]

- **CADI Server**
  - HTTP API on top of the registry for chunk/manifest management, search, planning, and provenance inspection.

- **CADI MCP Server**
  - Implements the Model Context Protocol so LLMs can:
    - Discover chunks and manifests as resources.
    - Call tools to plan builds, trigger builds, and fetch/open source.
  - Uses JSON-RPC per MCP spec.[2][3][4][5]

### 1.3 v1 Scope and Non-Goals

**In-scope**

- Languages/platforms:
  - Web frontend: React + TypeScript, with optional WASM modules.
  - Node.js: REST and WebSocket servers.
  - C: REST HTTP server compiled to native Linux binary and optionally WASM.

- Target platforms (Linux + macOS focus):
  - `x86_64-unknown-linux-gnu` (Primary Linux)
  - `x86_64-unknown-linux-musl` (Static Linux)
  - `arm64-unknown-linux-gnu` (ARM Linux / Raspberry Pi / ARM servers)
  - `x86_64-apple-darwin` (Intel Mac)
  - `arm64-apple-darwin` (Apple Silicon Mac)
  - `wasm32-unknown-unknown` (Universal WASM fallback)
  - `wasm32-wasi` (WASI-compliant WASM)

- Artifacts:
  - `source` (TS, JS, C).
  - `intermediate.wasm`.
  - `binary.x86_64-linux`, `binary.arm64-linux`, `binary.x86_64-darwin`, `binary.arm64-darwin`.
  - `container.oci` for Node and C servers using OCI registries/OCI Artifacts.[6][7][8]

- System components:
  - CADI registry and server (storage + API).
  - CADI CLI (builder/importer/runner).
  - CADI MCP server.
  - LLM optimization layer (summaries, semantic search, context optimization).
  - Security and attestation subsystem.
  - Dependency resolution engine.

**Out-of-scope (v1)**

- Windows native binaries (future v1.1+).
- Rich, programmable policy engine and governance/certification program.
- Deep multi-CI integration (only reference workflows).
- GUI tools (CLI-first approach).

### 1.4 v1 Success Criteria

**Technical**

- Developers can:
  - Import TS+Node+React + C demo into CADI chunks.
  - Build all demo targets with a single `cadi build` run.
  - On a second machine, fetch and run chosen targets by downloading only required artifacts (WASM/blobs/containers), with no rebuild.
  - Cross-compile from macOS to Linux targets using container-based builders.
  - Resolve and lock dependencies reproducibly across environments.

- Registry content:
  - ≥20–30 chunks across tiers (source, IR, blobs, containers).
  - Every blob/IR created via CADI has a BuildReceipt enabling reproducible verification.
  - All chunks have signed attestations from build system.

**Developer Experience**

- New user can:
  - Install CLI.
  - Configure default registry.
  - Run `cadi demo todo-suite --target web-dev` and get a working todo web app using CADI-built backend.
  - From an MCP-enabled client, search for “TodoApi C server” and open the corresponding CADI chunk source.[3][4][5][2]

**Performance**

- Cold build of full demo suite on a laptop: < 5 minutes.
- Rebuild after no code changes: < 30 seconds due to caching.
- Blob-only fetch + run of one component: < 3 seconds after download.
- Semantic search across registry: < 500ms for top-10 results.
- Chunk summary retrieval: < 100ms.

**Token Efficiency (LLM Focus)**

- Every chunk has auto-generated summaries at 3 tiers (50, 200, 500 tokens).
- API surface extraction reduces context needed by ≥70% vs full source.
- Semantic search enables finding relevant chunks without full codebase context.
- Measured metrics:
  - Context tokens saved vs regenerating code: tracked per session.
  - Chunk reuse rate: ≥50% of builds use cached/existing chunks.
  - Deduplication ratio: ≥30% storage savings from content-addressing.

**Energy Efficiency (Sustainability Focus)**

- Build transformations report CPU time and estimated energy usage.
- Cache hit rates tracked and reported (target: ≥80% on incremental builds).
- Registry tracks aggregate efficiency metrics:
  - Total bytes deduplicated.
  - Estimated tokens saved via reuse.
  - Estimated energy saved vs rebuild-from-scratch.

**Security and Trust**

- All blobs and containers are signed with build attestations.
- SLSA Level 2 provenance for all CI-built artifacts.
- Signature verification on fetch (configurable policy).
- Untrusted WASM/binaries execute in sandboxed environment.

***

## 2. Data Model and Specifications

All specs live in `cadi-spec/` and are versioned (e.g., `cadi-spec-v1.0`).

### 2.1 Common Chunk Schema

`cadi-spec/chunk.schema.json`:

```yaml
chunk_id: "chunk:sha256:<digest>"      # global identifier
cadi_type: "source" | "intermediate" | "blob" | "container"

meta
  name: string
  description: string
  version: string | null
  tags: [string]

provides:
  concepts: [string]                   # e.g., ["todo_storage", "http_api"]
  interfaces: [string]                 # e.g., ["TodoApi", "TaskStorage"]
  abi: string | null                   # for blobs (e.g., "system-v-x86-64")

licensing:
  license: string                      # SPDX ID: "MIT", "Apache-2.0", "proprietary"
  restrictions: [string]

lineage:
  parents: [string]                    # parent chunk_ids
  build_receipt: string | null         # sha256 of BuildReceipt
```

Type-specific sections (only one per chunk depending on `cadi_type`):

### 2.2 Source CADI Schema

`cadi-spec/source-cadi.schema.json`:

```yaml
cadi_type: "source"
source:
  language: "typescript" | "javascript" | "c" | "sql" | "rust" | string
  version: string                      # toolchain version where relevant
  dialect: string | null               # "standard", "jsx", "c11", etc.

  files:                               # file map
    - path: string
      hash: "sha256:<digest>"

  entrypoints:                         # optional
    - symbol: string                   # e.g., "main", "index.tsx"
      path: string

  runtime_dependencies:                # language-specific dependencies
    - id: string                       # e.g., "npm:express@^4"
      optional: boolean

compiled_forms:
  - ir_cadi: string                    # chunk_id of IR CADI derived from this source
    derived_at: string                 # ISO timestamp
    compiler: string
    deterministic: boolean
  - blob_cadi: string                  # chunk_id of Blob CADI
    architectures: [string]
    derived_at: string
```

### 2.3 IR CADI Schema (WASM-focused for v1)

`cadi-spec/ir-cadi.schema.json`:

```yaml
cadi_type: "intermediate"
intermediate:
  format: "wasm"
  version: string                      # e.g., "mvp", "simd"
  module:
    hash: "sha256:<digest>"
    exports:
      - name: string
        kind: "function" | "memory" | "global"
        signature: string | null
  imports:
    - module: string
      name: string
      kind: "function" | "memory" | "global"
      signature: string | null

  execution:
    sandbox_runtime: "wasmtime" | "wasmer" | "wasm_edge" | string
    default_memory_limit_mb: integer
    default_timeout_ms: integer

source_link:
  source_cadi: string                  # upstream chunk_id
  verification: "sha256:<build_receipt_digest>" | null

compiled_forms:
  - blob_cadi: string
    architectures: [string]
```

### 2.4 Blob CADI Schema

`cadi-spec/blob-cadi.schema.json`:

```yaml
cadi_type: "blob"
blobs:
  - architecture: "x86_64-linux-gnu" | "arm64-darwin" | string
    format: "ELF" | "Mach-O" | "PE" | string
    hash: "sha256:<digest>"
    size: integer
    linking:
      type: "static" | "dynamic"
      dependencies: [string]          # e.g., ["libc.so.6", "libpq.so.5"]
    security:
      pie: boolean
      nx: boolean
      stack_canary: boolean
      relro: "none" | "partial" | "full"
    build_info:
      compiler: string                # e.g., "clang-17"
      flags: [string]
      source_hash: "sha256:<digest>" | null
      reproducible: boolean

lineage:
  source_cadi: string | null
  ir_cadi: string | null
  build_provenance:
    build_receipt: "sha256:<digest>"
```

### 2.5 Container CADI Schema

`cadi-spec/container-cadi.schema.json`:

```yaml
cadi_type: "container"
container:
  format: "oci"
  image_ref: string                    # e.g., "ghcr.io/org/todo-node@sha256:..."
  layers:
    - hash: "sha256:<digest>"
      size: integer
      description: string
      cadi_chunk: string | null       # link to underlying blob/source, if applicable

  entrypoint: [string]
  cmd: [string]
  environment:
    - name: string
      value: string
  exposed_ports:
    - "8080/tcp" | string

deployment_target:
  - "docker"
  - "kubernetes"
  - "serverless"
```

### 2.6 Manifest Schema

`cadi-spec/manifest.schema.json`:

```yaml
manifest_id: "app:uuid:<uuid>"
manifest_version: string
application:
  name: string
  description: string

build_graph:
  nodes:
    - id: string                      # e.g., "web_frontend_basic"
      source_cadi: string | null
      ir_cadi: string | null
      blob_cadi: string | null
      container_cadi: string | null

      representations:
        - form: "source" | "intermediate" | "binary" | "container"
          language: string | null
          format: string | null
          architecture: string | null
          chunk: string               # chunk_id

      selection_strategy: string      # e.g., "prefer_blob_for_prod"
  edges:
    - from: string
      to: string
      interface: string               # reference to provided interface

build_targets:
  - name: string                      # e.g., "web-dev"
    platform: string                  # e.g., "browser", "x86_64-linux", "linux-container"
    nodes:
      - id: string
        require: [string] | null      # required forms
        prefer: [string] | null       # ordered preferences
    bundle:
      format: string | null
      output: string | null
    deploy:
      target: string | null
      replicas: integer | null
```

### 2.7 BuildReceipt Schema

`cadi-spec/build-receipt.schema.json`:

```yaml
build_receipt_id: "sha256:<digest>"
source_cadi: string | null
ir_cadi: string | null
blob_cadi: string | null
container_cadi: string | null

builder: string                        # e.g., "cadi-builder-v1.0.0"
timestamp: string                      # ISO8601
host_id: string                        # e.g., "github-actions-runner-42"

tools:
  - name: string
    version: string

environment_digest: "sha256:<digest>"

steps:
  - step: string                       # e.g., "compile_to_ir"
    input: string                      # input chunk_id
    output: string                     # output chunk_id
    tool: string
    flags: [string]
    duration_ms: integer

verification:
  deterministic: boolean
  source_hash_matches: boolean | null
  ir_hash_matches: boolean | null
  blob_hash_matches: boolean | null

signatures:
  - signer: string
    signature: string
    timestamp: string
```

### 2.8 Dependency Graph Schema

`cadi-spec/dependency-graph.schema.json`:

```yaml
# Defines how chunks declare and resolve dependencies
dependency_graph_id: "depgraph:sha256:<digest>"

# The chunk this dependency graph belongs to
chunk_id: string

# Declared dependencies with version constraints
dependencies:
  - id: string                         # dependency identifier
    source: string                     # "cadi" | "npm" | "crates" | "apt" | "brew" | string
    constraint: string                 # version constraint expression
    constraint_type: "semver" | "exact" | "range" | "latest" | "pinned"
    optional: boolean
    features: [string] | null          # optional feature flags
    platform_filter: string | null     # e.g., "linux", "darwin", "wasm32"

# Resolved dependency tree (lock file equivalent)
resolution:
  resolved_at: string                  # ISO8601 timestamp
  resolver_version: string             # CADI resolver version used
  
  nodes:
    - id: string                       # unique node id in resolution
      dependency_id: string            # matches dependencies[].id
      resolved_version: string         # exact resolved version
      resolved_chunk: string | null    # chunk_id if available in CADI
      integrity: "sha256:<digest>"     # hash of resolved artifact
      
  edges:
    - from: string                     # node id
      to: string                       # node id
      relation: "runtime" | "build" | "dev" | "peer" | "optional"

# Conflict resolution record
conflicts:
  - dependency_id: string
    requested_by: [string]             # chunk_ids that requested this
    versions_requested: [string]       # different versions requested
    resolution_strategy: "newest" | "oldest" | "manual" | "error"
    resolved_to: string                # final chosen version
    rationale: string | null           # human/AI explanation

# Lock file metadata
lock:
  format_version: "1.0"
  content_hash: "sha256:<digest>"      # hash of entire resolution
  platforms_resolved: [string]         # platforms this resolution covers
```

**Resolution Algorithm:**

1. **Parse Constraints**: Convert all version constraints to normalized form
2. **Build Requirement Graph**: Create DAG of all transitive requirements  
3. **Detect Conflicts**: Identify diamond dependencies and version mismatches
4. **Apply Resolution Strategy**:
   - `semver`: Use newest compatible version within constraint
   - `exact`: Require precise match or fail
   - `pinned`: Use locked version from existing resolution
5. **Verify Integrity**: Confirm all resolved artifacts match recorded hashes
6. **Generate Lock**: Produce deterministic lock file for reproducibility

### 2.9 Registry Federation Schema

`cadi-spec/registry-federation.schema.json`:

```yaml
# Defines how registries interact, namespace, and trust
federation_id: "federation:sha256:<digest>"

# Registry identity
registry:
  id: string                           # unique registry identifier
  name: string                         # human-readable name
  url: string                          # base URL
  api_version: "v1"                    # CADI API version

# Namespace configuration
namespace:
  scheme: "hierarchical"               # "flat" | "hierarchical" | "scoped"
  separator: "/"                       # namespace separator character
  root: string                         # e.g., "github.com/org" or "cadi.io"
  
  # Namespace allocation rules
  allocation:
    mode: "open" | "verified" | "invite"
    verification_required: [string]    # e.g., ["domain", "github"]
    
  # Collision prevention
  collision_policy:
    check_upstream: boolean            # check federated registries
    allow_shadows: boolean             # allow local chunks that shadow upstream
    conflict_resolution: "local_priority" | "upstream_priority" | "error"

# Federation relationships
upstreams:
  - registry_id: string                # upstream registry identifier
    url: string
    priority: integer                  # lower = higher priority
    
    sync:
      mode: "mirror" | "proxy" | "cache" | "none"
      refresh_interval_seconds: integer
      sync_filter:                     # what to sync
        namespaces: [string] | null    # specific namespaces
        concepts: [string] | null      # specific concepts
        languages: [string] | null     # specific languages
    
    trust_policy:
      mode: "verify_all" | "trust_signatures" | "trust_upstream" | "local_only"
      required_signers: [string] | null    # required signer identities
      signature_threshold: integer | null  # min signatures required
      allowed_attestation_types: [string]  # e.g., ["slsa-provenance-v1"]

# Downstream registries (who mirrors us)
downstreams:
  - registry_id: string
    url: string
    last_sync: string | null           # ISO8601

# Offline/air-gapped support
offline:
  enabled: boolean
  snapshot_schedule: string | null     # cron expression
  snapshot_retention_days: integer
  
# Cross-registry reference format
reference_format:
  # Full qualified chunk reference:
  # cadi://registry.example.com/namespace/chunk:sha256:<digest>
  scheme: "cadi"
  include_registry: boolean            # include registry in chunk_id
  canonical_registry: string | null    # default registry for unqualified refs
```

### 2.10 ABI Compatibility Schema

`cadi-spec/abi-compatibility.schema.json`:

```yaml
# Defines platform capabilities and ABI compatibility rules
abi_compatibility_id: "abi:sha256:<digest>"

# Platform definition
platform:
  os: "linux" | "darwin" | "windows" | "wasi" | "browser" | string
  arch: "x86_64" | "arm64" | "wasm32" | "wasm64" | string
  variant: string | null               # e.g., "musl", "gnu", "android"
  
  # Canonical platform triple
  triple: string                       # e.g., "x86_64-unknown-linux-gnu"

# ABI specification
abi:
  name: string                         # e.g., "system-v-x86-64", "arm64-apple"
  version: string | null
  
  calling_convention: string           # e.g., "cdecl", "stdcall", "aapcs"
  data_model: string                   # e.g., "LP64", "LLP64", "ILP32"
  endianness: "little" | "big"
  
  # Type sizes and alignments
  type_info:
    pointer_size: integer              # bytes
    int_size: integer
    long_size: integer
    size_t_size: integer
  
  # Required system libraries
  system_libs:
    - name: string                     # e.g., "libc.so.6"
      version_min: string | null
      version_max: string | null

# Compatibility matrix
compatibility:
  # What this platform can run
  can_execute:
    - platform_triple: string
      compatibility: "native" | "emulated" | "translated"
      overhead_estimate: "none" | "low" | "medium" | "high"
      
  # Fallback chain for representation selection
  fallback_chain:
    - representation: string           # e.g., "binary.arm64-darwin"
      priority: integer                # lower = try first
      
    - representation: "intermediate.wasm"
      priority: 100                    # WASM as universal fallback
      runtime_required: string         # e.g., "wasmtime"
      
  # What representations this platform can produce
  can_build:
    - target_triple: string
      cross_compile: boolean
      toolchain: string                # required toolchain

# Runtime detection
detection:
  # Commands to detect platform at runtime
  os_detection: string                 # e.g., "uname -s"
  arch_detection: string               # e.g., "uname -m"
  
  # Environment variables to check
  env_hints:
    - name: string
      indicates: string                # what it indicates

# Compatibility checking algorithm
checking:
  strict_mode: boolean                 # fail on any mismatch
  warn_on:
    - "minor_version_mismatch"
    - "optional_feature_missing"
  fail_on:
    - "abi_incompatible"
    - "missing_system_lib"
    - "arch_mismatch"
```

**Platform Resolution Algorithm:**

1. **Detect Current Platform**: Query OS, arch, available runtimes
2. **Build Candidate List**: Find all representations for target chunk
3. **Filter Compatible**: Remove incompatible representations
4. **Sort by Preference**: 
   - Native binary (highest priority)
   - Cross-compiled binary
   - WASM with available runtime
   - Source (requires build)
5. **Select Best Match**: Return highest-priority compatible representation
6. **Fallback**: If no binary available, return WASM or source with build instructions

### 2.11 Security and Attestation Schema

`cadi-spec/security-attestation.schema.json`:

```yaml
# Comprehensive security model for chunk signing and attestation
attestation_id: "attestation:sha256:<digest>"

# What is being attested
subject:
  chunk_id: string
  digest: "sha256:<digest>"            # content hash being signed

# Signer identity
signer:
  id: string                           # unique signer identifier
  type: "individual" | "organization" | "service" | "ci_system"
  
  # Identity verification
  identity:
    method: "keyless" | "key_based" | "oidc" | "x509"
    
    # For OIDC (Sigstore-style keyless)
    oidc:
      issuer: string | null            # e.g., "https://accounts.google.com"
      subject: string | null           # e.g., "user@example.com"
      
    # For key-based
    key:
      algorithm: "ed25519" | "ecdsa-p256" | "rsa-4096" | string
      public_key: string               # base64 encoded
      key_id: "sha256:<digest>"        # fingerprint
      
    # For X.509 certificates
    x509:
      certificate_chain: [string]      # PEM encoded chain
      
  # Verification endpoints
  verification:
    transparency_log: string | null    # Rekor-style log URL
    timestamp_authority: string | null # RFC 3161 TSA

# Signature
signature:
  algorithm: string                    # e.g., "ed25519", "ecdsa-p256-sha256"
  value: string                        # base64 encoded signature
  timestamp: string                    # ISO8601
  
  # Optional countersignature from timestamp authority
  timestamp_signature:
    authority: string
    timestamp: string
    signature: string

# Attestation type and claims
attestation:
  type: string                         # predicate type URI
  # Supported types:
  # - "https://slsa.dev/provenance/v1" (SLSA provenance)
  # - "https://cadi.dev/attestation/build-receipt/v1"
  # - "https://cadi.dev/attestation/code-review/v1"
  # - "https://cadi.dev/attestation/security-scan/v1"
  
  # SLSA Provenance (if type is slsa)
  slsa_provenance:
    build_type: string                 # e.g., "https://cadi.dev/build/v1"
    builder:
      id: string                       # builder identity
      version: string
    invocation:
      config_source:
        uri: string
        digest: "sha256:<digest>"
      parameters: object
      environment: object
    materials:
      - uri: string                    # input chunk or external resource
        digest: "sha256:<digest>"
    metadata:
      build_started_on: string
      build_finished_on: string
      reproducible: boolean
      
  # Code review attestation
  code_review:
    reviewers: [string]                # reviewer identities
    approval_timestamp: string
    review_type: "full" | "partial" | "automated"
    
  # Security scan attestation  
  security_scan:
    scanner: string                    # scanner tool name
    version: string
    scan_timestamp: string
    findings:
      critical: integer
      high: integer
      medium: integer
      low: integer
    policy_passed: boolean

# Trust policy for consumers
trust_requirements:
  # Minimum requirements to trust this chunk
  minimum_signatures: integer          # e.g., 1
  required_attestation_types: [string] # e.g., ["slsa-provenance-v1"]
  required_signers: [string] | null    # specific signers required
  max_age_days: integer | null         # signature freshness requirement
  
  # Policy for transitive dependencies
  transitive_policy:
    inherit: boolean                   # apply same policy to deps
    allow_weaker: boolean              # allow deps with weaker attestations
```

### 2.12 Efficiency Metrics Schema

`cadi-spec/efficiency-metrics.schema.json`:

```yaml
# Track token efficiency, energy usage, and sustainability metrics
metrics_id: "metrics:sha256:<digest>"

# What these metrics describe
subject:
  chunk_id: string | null
  manifest_id: string | null
  operation: "build" | "fetch" | "search" | "reuse" | string

# Timestamp and context
recorded_at: string                    # ISO8601
context:
  user_id: string | null               # anonymized
  session_id: string | null
  platform: string

# Reuse and deduplication metrics
reuse:
  chunk_reuse_count: integer           # times this chunk was reused
  bytes_deduplicated: integer          # bytes saved via content-addressing
  
  # Comparison to regeneration
  vs_regeneration:
    estimated_tokens_if_regenerated: integer
    actual_tokens_used: integer
    tokens_saved: integer
    savings_percentage: float
    
  # Comparison to traditional package managers
  vs_traditional:
    estimated_download_size_traditional: integer
    actual_download_size_cadi: integer
    bandwidth_saved: integer

# LLM interaction efficiency
llm_efficiency:
  # Context window usage
  context_tokens_used: integer
  context_tokens_available: integer
  context_utilization: float
  
  # Summary effectiveness
  summary_tokens: integer              # tokens in chunk summary
  full_source_tokens: integer          # tokens if full source used
  compression_ratio: float
  
  # Search efficiency
  search_queries: integer
  chunks_examined: integer
  chunks_selected: integer
  precision: float                     # selected / examined

# Build and transformation costs
build:
  cpu_seconds: float
  memory_peak_mb: integer
  wall_clock_seconds: float
  
  # Cache effectiveness
  cache_hits: integer
  cache_misses: integer
  cache_hit_rate: float
  
  # Incremental build savings
  full_build_time_seconds: float | null
  incremental_build_time_seconds: float
  time_saved_seconds: float

# Energy estimation (approximate)
energy:
  estimated_kwh: float
  carbon_intensity_region: string | null  # e.g., "us-west-2"
  estimated_co2_grams: float | null
  
  methodology: string                  # how estimates were calculated
  confidence: "low" | "medium" | "high"

# Aggregated statistics (for registry-level reporting)
aggregated:
  period: "hourly" | "daily" | "weekly" | "monthly"
  period_start: string
  period_end: string
  
  total_chunks_served: integer
  total_bytes_transferred: integer
  total_bytes_deduplicated: integer
  unique_users: integer
  
  top_reused_chunks:
    - chunk_id: string
      reuse_count: integer
      
  estimated_total_tokens_saved: integer
  estimated_total_energy_saved_kwh: float
```

### 2.13 LLM Context Schema

`cadi-spec/llm-context.schema.json`:

```yaml
# Optimized chunk representation for LLM consumption
llm_context_id: "llm:sha256:<digest>"

# Source chunk
chunk_id: string
generated_at: string                   # ISO8601
generator_version: string              # CADI version that generated this

# Token-efficient summary
summary:
  # Ultra-short description (< 50 tokens)
  one_liner: string
  
  # Short description (< 200 tokens)
  brief: string
  
  # Medium description (< 500 tokens) 
  standard: string
  
  # Token counts for each
  token_counts:
    one_liner: integer
    brief: integer
    standard: integer

# Extracted API surface (for interface discovery)
api_surface:
  # Public interfaces/exports
  exports:
    - name: string
      kind: "function" | "class" | "type" | "constant" | "module"
      signature: string | null         # language-appropriate signature
      description: string | null       # extracted from docstring/comment
      
  # Required imports/dependencies
  imports:
    - name: string
      source: string                   # where it comes from
      
  # Concepts this implements
  implements_concepts: [string]
  
  # Concepts this requires
  requires_concepts: [string]

# Semantic embedding for vector search
embedding:
  model: string                        # e.g., "text-embedding-3-large"
  dimensions: integer
  vector: [float]                      # the actual embedding
  
  # What was embedded
  embedded_content: "summary" | "full_source" | "api_surface"

# Usage examples (for few-shot prompting)
examples:
  - description: string
    code: string
    language: string
    token_count: integer

# Related chunks (for context expansion)
related:
  - chunk_id: string
    relation: "implements" | "extends" | "uses" | "similar" | "alternative"
    relevance_score: float             # 0.0 to 1.0

# Diff-aware context for modifications
diff_context:
  # Minimal context needed for modifications
  modification_zones:
    - name: string                     # e.g., "add_endpoint"
      start_marker: string             # unique string to locate
      end_marker: string
      context_before_lines: integer
      context_after_lines: integer
      
  # Common modification patterns
  patterns:
    - name: string                     # e.g., "add_new_function"
      template: string                 # template with placeholders
      insertion_point: string          # where to insert
```

### 2.14 Versioning and Evolution Schema

`cadi-spec/versioning.schema.json`:

```yaml
# Defines immutability semantics and version evolution
versioning_id: "version:sha256:<digest>"

# Core principle: chunks are IMMUTABLE
# New versions create NEW chunk_ids
# This schema tracks relationships between versions

# Version metadata
version:
  chunk_id: string                     # current chunk
  semantic_version: string | null      # optional semver (e.g., "1.2.3")
  
  # Version lineage
  lineage:
    supersedes: string | null          # chunk_id this replaces
    superseded_by: string | null       # chunk_id that replaced this
    
    # Full version history (optional, for discoverability)
    history:
      - chunk_id: string
        version: string | null
        timestamp: string
        change_type: "major" | "minor" | "patch" | "initial"

# Change description
changes:
  from_chunk: string | null            # previous version chunk_id
  change_type: "breaking" | "feature" | "fix" | "refactor" | "docs"
  
  description: string
  
  # Structured change log
  changelog:
    added: [string]
    changed: [string]
    deprecated: [string]
    removed: [string]
    fixed: [string]
    security: [string]

# Compatibility declarations
compatibility:
  # API compatibility
  api:
    backward_compatible: boolean       # can replace previous version
    forward_compatible: boolean        # previous can handle this format
    
  # ABI compatibility (for blobs)
  abi:
    compatible_with: [string]          # list of compatible chunk_ids
    
  # Migration path
  migration:
    automatic: boolean                 # can auto-migrate
    migration_chunk: string | null     # chunk containing migration logic
    migration_instructions: string | null

# Deprecation
deprecation:
  deprecated: boolean
  deprecated_at: string | null         # ISO8601
  reason: string | null
  replacement: string | null           # recommended replacement chunk_id
  removal_target: string | null        # version/date when will be removed

# Schema evolution (for CADI spec itself)
schema_evolution:
  spec_version: string                 # e.g., "cadi-spec-v1.0"
  
  # Compatibility with other spec versions
  compatible_spec_versions: [string]
  
  # Migration between spec versions
  spec_migrations:
    - from_version: string
      to_version: string
      migration_type: "automatic" | "manual" | "breaking"
```

### 2.15 Garbage Collection Schema

`cadi-spec/garbage-collection.schema.json`:

```yaml
# Defines retention policies and garbage collection rules
gc_policy_id: "gc:sha256:<digest>"

# Scope of this policy
scope:
  type: "local_cache" | "registry" | "namespace"
  path: string | null                  # for local cache
  registry_id: string | null           # for registry
  namespace: string | null             # for namespace-specific

# Local cache policies
local_cache:
  enabled: boolean
  
  # Size-based limits
  size_limits:
    max_total_bytes: integer | null    # e.g., 10GB
    max_chunk_count: integer | null
    
  # Time-based limits  
  time_limits:
    max_age_days: integer | null       # evict if not accessed
    min_age_days: integer              # never evict if younger
    
  # Eviction strategy
  eviction:
    strategy: "lru" | "lfu" | "fifo" | "size_weighted_lru"
    
    # Priority adjustments
    keep_priorities:
      - match: "cadi_type:source"      # keep source longer
        priority_boost: 2
      - match: "tag:pinned"
        priority_boost: 100            # effectively never evict
        
  # What to preserve
  preserve:
    - "referenced_by_manifest"         # chunks referenced by local manifests
    - "recently_built"                 # chunks from recent builds
    - "pinned"                         # explicitly pinned chunks

# Registry retention policies
registry:
  enabled: boolean
  
  # Namespace-specific policies
  namespace_policies:
    - namespace: string
      policy: "preserve_all" | "standard" | "aggressive"
      
  # Global policies
  retention:
    # Keep at least N versions
    min_versions: integer              # e.g., 3
    
    # Keep versions younger than
    min_age_days: integer              # e.g., 90
    
    # Maximum storage per namespace
    max_namespace_bytes: integer | null
    
  # Reference counting
  reference_counting:
    enabled: boolean
    orphan_grace_period_days: integer  # days before deleting unreferenced

# Orphan detection
orphan_detection:
  # What counts as an orphan
  orphan_criteria:
    - no_manifest_references: true
    - no_lineage_references: true
    - age_days_minimum: integer        # must be older than this
    
  # Actions for orphans
  actions:
    - age_days: 30
      action: "mark_for_review"
    - age_days: 90
      action: "archive"
    - age_days: 180
      action: "delete"

# CLI commands
commands:
  # cadi gc [--dry-run] [--aggressive]
  gc:
    dry_run_default: true              # show what would be deleted
    require_confirmation: true
    
  # cadi gc --status
  status:
    show_reclaimable_bytes: true
    show_orphan_count: true
    show_cache_stats: true
```

***

## 3. System Architecture

### 3.1 CADI Registry and Server

**Responsibilities**

- Store blobs and CADI specs.
- Expose API for:
  - Chunk CRUD.
  - Manifest CRUD.
  - Blob upload/download.
  - Search.
  - Build planning.
  - Prove/provenance retrieval.

**Components**

- **API Layer** (Go or Rust)
  - `GET /chunks/{chunk_id}`
  - `PUT /chunks/{chunk_id}`
  - `GET /blobs/{sha256}`
  - `PUT /blobs/{sha256}`
  - `GET /manifests/{manifest_id}`
  - `PUT /manifests/{manifest_id}`
  - `GET /search`
    - Query params: `q`, `concept`, `interface`, `language`, `artifact_type`.
  - `POST /plan`
    - Body: manifest_id, target, optional environment.
    - Returns: list of operations (fetch/build) and estimated size/time.

- **Storage Layer**
  - Blob store:
    - S3-compatible (MinIO in dev, S3/GCS prod) to align with OCI/ORAS practices.[7][8][6]
    - Keys: `blobs/sha256/<hash>`.
  - Metadata DB:
    - Postgres with tables for chunks, manifests, build_receipts, users/tokens.
    - Search indexes on `concepts`, `interfaces`, `language`, `cadi_type`.

- **Auth**
  - Token-based auth via HTTP header.
  - Public read for demo namespaces.

### 3.2 CADI Builder CLI

**Responsibilities**

- Manage local cache.
- Build artifacts from manifests and transformations.
- Publish/fetch from registry.
- Plan and verify builds.

**Commands**

- `cadi init`
  - Create `~/.cadi/config.yaml`:
    - `registry_url`
    - `auth_token`
    - `cache_dir`
- `cadi import <path>`
  - Detect project type:
    - TS/React, Node.js, C.
  - Generate Source CADI chunks and a base manifest.
- `cadi build <manifest> [--target <name>] [--prefer source|ir|blob]`
  - Resolve build graph.
  - For each node: select best representation or build transformations.
  - Emit IR/Blob/Container CADIs and build receipts.
- `cadi publish [options]`
  - Push local chunks and blobs to registry.
- `cadi fetch <app-or-chunk> [--tier source|ir|blob|all]`
  - Retrieve necessary chunks and blobs into local cache.
- `cadi run <manifest-or-chunk> [--target <name>]`
  - Run chosen target using appropriate runtime:
    - Node server (via `node`).
    - Native binary (direct exec).
    - Container (Docker/Podman).
    - Browser dev server/static build for web.
    - WASM demo via embedded runtime.
- `cadi plan <manifest> [--target <name>]`
  - Show operations: fetch vs build, sizes, estimated times.
- `cadi verify <manifest-or-chunk>`
  - Fetch build receipts and optionally rebuild to verify reproducibility.

**Local Cache**

- Path: `~/.cadi/store`
- Layout:
  - `~/.cadi/store/chunks/<chunk_id>` → JSON spec.
  - `~/.cadi/store/blobs/sha256/<hash>` → binary/WASM/source bundles.
- Cache key:
  - Derived from:
    - Input source hashes.
    - Tool names/versions.
    - Flags.
    - Environment digest.
  - Inspired by Nix/Bazel derivations/action keys.[10][11][1][9]

### 3.3 Transformation Engine

**General Model**

- Transformations are declarative objects describing:
  - From form.
  - To form.
  - Toolchain invocation.
  - Verification requirements.

Example (TS→JS):

```yaml
transformation:
  from: "source.typescript"
  to: "source.javascript"

  process:
    - tool: "tsc"
      version: "5.x"
      args: ["--project", "tsconfig.json"]

  verification:
    - type: "unit_tests"
      command: ["npm", "test"]
  deterministic: true
  cache_result: true
```

Example (C→WASM):

```yaml
transformation:
  from: "source.c"
  to: "intermediate.wasm"

  process:
    - tool: "clang"
      version: "17"
      args: ["--target=wasm32-unknown-unknown", "-O3", "-nostdlib", "source.c", "-o", "out.wasm"]

  verification:
    - type: "run_tests"
      runtime: "wasmtime"
      args: ["tests.wasm"]
  cache_result: true
```

Example (WASM→binary):

```yaml
transformation:
  from: "intermediate.wasm"
  to: "binary.x86_64-linux"

  process:
    - tool: "wasm-aot"
      version: "x.y.z"
      args: ["--target=x86_64-unknown-linux-gnu", "--opt-level=3", "in.wasm", "-o", "out.bin"]
  cache_result: true
```

The builder consults transformation definitions to construct a pipeline from existing representation to desired representation, caching results.

### 3.4 MCP Server

**Protocol**

- MCP is JSON-RPC based and defines:
  - Hosts (LLM apps).
  - Clients (connectors).
  - Servers (services like CADI).[4][5][2][3]

**CADI MCP Server Features**

- Runs as standalone process (`cadi-mcp-server`) connecting to CADI server and local CLI.

- **Resources**
  - `cadi.chunks`
    - List/search chunks.
    - Fields: id, name, concepts, interfaces, cadi_type, languages.
  - `cadi.manifests`
    - List/search manifests.
  - `cadi.build_receipts`
    - Retrieve receipts for a given chunk/app.

- **Tools**
  - `cadi.get_chunk(chunk_id)`
    - Returns full CADI spec.
  - `cadi.search_chunks(query, filters)`
    - Query by concept/interface/language/artifact_type.
  - `cadi.plan_build(manifest_id, target)`
    - Returns plan of fetch/build operations.
  - `cadi.trigger_build(manifest_id, target)`
    - Calls `cadi build` under the hood (requires explicit user approval).
  - `cadi.fetch_and_open_source(chunk_id)`
    - Fetches source CADI and returns file listing/paths so clients can open files.

- **Security**
  - Read-only tools enabled by default.
  - Mutating operations (build/publish) require explicit user consent, following MCP guidance on consent and controls.[5][2][3]

### 3.5 Cross-Compilation and Platform Matrix

**Supported Platforms (v1)**

| Platform Triple | OS | Arch | Format | Status |
|----------------|-----|------|--------|--------|
| `x86_64-unknown-linux-gnu` | Linux | x86_64 | ELF | Primary |
| `x86_64-unknown-linux-musl` | Linux | x86_64 | ELF (static) | Primary |
| `arm64-unknown-linux-gnu` | Linux | ARM64 | ELF | Primary |
| `x86_64-apple-darwin` | macOS | x86_64 | Mach-O | Primary |
| `arm64-apple-darwin` | macOS | ARM64 | Mach-O | Primary |
| `wasm32-unknown-unknown` | WASI | WASM32 | WASM | Universal Fallback |
| `wasm32-wasi` | WASI | WASM32 | WASM | Universal Fallback |

**Cross-Compilation Strategy**

```yaml
cross_compile:
  # Host → Target mappings
  strategies:
    - host: "arm64-apple-darwin"
      can_target:
        - triple: "x86_64-unknown-linux-gnu"
          method: "cross-toolchain"
          toolchain: "x86_64-linux-gnu-gcc"
          
        - triple: "x86_64-unknown-linux-musl"
          method: "cross-toolchain"
          toolchain: "musl-cross"
          
        - triple: "wasm32-unknown-unknown"
          method: "native"
          toolchain: "clang --target=wasm32"
          
    - host: "x86_64-unknown-linux-gnu"
      can_target:
        - triple: "arm64-unknown-linux-gnu"
          method: "cross-toolchain"
          toolchain: "aarch64-linux-gnu-gcc"
          
        - triple: "wasm32-unknown-unknown"
          method: "native"
          toolchain: "clang --target=wasm32"

  # Container-based cross-compilation
  container_builds:
    enabled: true
    images:
      - target: "x86_64-unknown-linux-gnu"
        image: "ghcr.io/cadi/builder-linux-x86_64:latest"
        
      - target: "arm64-unknown-linux-gnu"
        image: "ghcr.io/cadi/builder-linux-arm64:latest"
```

**WASM as Universal Fallback**

WASM serves as the universal intermediate representation when native binaries are unavailable:

1. **Detection**: If requested platform has no native blob, check for WASM
2. **Runtime Selection**: Choose available runtime (wasmtime, wasmer, node --experimental-wasm)
3. **AOT Compilation**: Optionally compile WASM to native for performance
4. **Capability Mapping**: Map platform capabilities to WASM imports

```yaml
wasm_fallback:
  enabled: true
  
  runtime_preference:
    - runtime: "wasmtime"
      version_min: "14.0"
      features: ["simd", "threads"]
      
    - runtime: "wasmer"
      version_min: "4.0"
      
    - runtime: "node"
      version_min: "20.0"
      flags: ["--experimental-wasm-modules"]
      
  # Performance tier expectations
  performance:
    native: 1.0                        # baseline
    wasm_aot: 0.8                      # 80% of native
    wasm_interpreted: 0.3              # 30% of native
```

### 3.6 Dependency Resolution Engine

**Resolution Process**

```
┌─────────────────────────────────────────────────────────────┐
│                    Dependency Resolution                     │
├─────────────────────────────────────────────────────────────┤
│  1. Parse manifest dependencies                              │
│  2. Expand transitive dependencies (BFS)                     │
│  3. Detect version conflicts                                 │
│  4. Apply resolution strategy                                │
│  5. Generate lock file                                       │
│  6. Verify integrity                                         │
└─────────────────────────────────────────────────────────────┘
```

**Version Constraint Syntax**

```yaml
# Supported constraint formats
constraints:
  # Exact version
  - "=1.2.3"
  
  # Semver ranges
  - "^1.2.3"      # >=1.2.3 <2.0.0
  - "~1.2.3"      # >=1.2.3 <1.3.0
  - ">=1.2.3"     # >=1.2.3
  - "1.2.x"       # >=1.2.0 <1.3.0
  
  # Pinned to chunk (content-addressed)
  - "chunk:sha256:abc123..."
  
  # Latest from namespace
  - "latest"
  - "latest:stable"
  - "latest:beta"
```

**Conflict Resolution Strategies**

| Strategy | Description | Use Case |
|----------|-------------|----------|
| `newest` | Select newest version satisfying all constraints | Default |
| `oldest` | Select oldest version (conservative) | Stability-focused |
| `pinned` | Use versions from existing lock file | CI/reproducibility |
| `manual` | Fail and require explicit resolution | Security-critical |

**Diamond Dependency Handling**

```
        A
       / \
      B   C
       \ /
        D (v1.0 vs v2.0?)
        
Resolution:
1. If B requires D@^1.0 and C requires D@^2.0 → CONFLICT
2. If B requires D@^1.0 and C requires D@>=1.5 → Resolve to D@1.5+
3. Record resolution rationale in lock file
```

### 3.7 LLM Optimization Layer

**Purpose**: Maximize token efficiency and enable intelligent code discovery for LLM-assisted development.

**Components**

```
┌─────────────────────────────────────────────────────────────┐
│                    LLM Optimization Layer                    │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Summary   │  │   Vector    │  │  Context Window     │  │
│  │  Generator  │  │   Search    │  │    Optimizer        │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │    API      │  │   Diff      │  │    Usage            │  │
│  │  Extractor  │  │   Context   │  │   Examples          │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

**Summary Generation**

For each chunk, automatically generate:

1. **One-liner** (< 50 tokens): For search results and quick scanning
2. **Brief** (< 200 tokens): For context window stuffing
3. **Standard** (< 500 tokens): For detailed understanding
4. **Full API Surface**: Just signatures and types, no implementation

**Semantic Search Index**

```yaml
search_index:
  # Embedding configuration
  embedding:
    model: "text-embedding-3-large"
    dimensions: 3072
    
  # What gets embedded
  content_sources:
    - type: "summary"
      weight: 1.0
    - type: "api_surface"
      weight: 0.8
    - type: "concepts"
      weight: 0.6
    - type: "tags"
      weight: 0.4
      
  # Search behavior
  search:
    default_limit: 10
    similarity_threshold: 0.7
    boost_recent: true
    boost_popular: true
```

**Context Window Optimization**

```yaml
context_optimization:
  # Strategies for fitting chunks into context
  strategies:
    - name: "summary_only"
      max_tokens: 200
      content: ["one_liner", "api_surface"]
      
    - name: "interface_focused"
      max_tokens: 500
      content: ["brief", "api_surface", "examples"]
      
    - name: "full_understanding"
      max_tokens: 2000
      content: ["standard", "api_surface", "examples", "related"]
      
  # Automatic selection based on available context
  auto_select:
    remaining_context_tokens: integer
    select_strategy: "largest_that_fits"
```

**MCP Tools for LLM Optimization**

Additional MCP tools specifically for LLM efficiency:

```yaml
mcp_tools:
  # Get token-optimized chunk summary
  - name: "cadi.get_chunk_summary"
    params:
      chunk_id: string
      max_tokens: integer              # default: 500
      include_api: boolean             # default: true
      include_examples: boolean        # default: false
    returns:
      summary: string
      token_count: integer
      api_surface: object | null
      
  # Get only the API surface (minimal tokens)
  - name: "cadi.get_interface_only"
    params:
      chunk_id: string
    returns:
      exports: [object]
      imports: [object]
      token_count: integer
      
  # Semantic search across chunks
  - name: "cadi.semantic_search"
    params:
      query: string
      limit: integer                   # default: 10
      filters:
        concepts: [string] | null
        languages: [string] | null
        cadi_type: string | null
    returns:
      results:
        - chunk_id: string
          score: float
          one_liner: string
          concepts: [string]
          
  # Get related/similar chunks
  - name: "cadi.find_similar"
    params:
      chunk_id: string
      relation_types: [string]         # "similar", "implements", "extends"
      limit: integer
    returns:
      related: [object]
      
  # Suggest chunks for a task description
  - name: "cadi.suggest_for_task"
    params:
      task_description: string
      context_chunks: [string] | null  # already in context
      max_suggestions: integer
    returns:
      suggestions:
        - chunk_id: string
          relevance: float
          rationale: string
```

### 3.8 Security and Trust Architecture

**Trust Model**

```
┌─────────────────────────────────────────────────────────────┐
│                      Trust Hierarchy                         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                   Root of Trust                      │   │
│   │     (CADI Project Keys / Well-Known Authorities)     │   │
│   └─────────────────────────────────────────────────────┘   │
│                            │                                 │
│              ┌─────────────┼─────────────┐                  │
│              ▼             ▼             ▼                  │
│   ┌──────────────┐ ┌──────────────┐ ┌──────────────┐       │
│   │ Organization │ │ Organization │ │  CI Systems  │       │
│   │    Keys      │ │    Keys      │ │    Keys      │       │
│   └──────────────┘ └──────────────┘ └──────────────┘       │
│          │                 │                │               │
│          ▼                 ▼                ▼               │
│   ┌──────────────┐ ┌──────────────┐ ┌──────────────┐       │
│   │  Individual  │ │  Individual  │ │ Build Agents │       │
│   │    Keys      │ │    Keys      │ │              │       │
│   └──────────────┘ └──────────────┘ └──────────────┘       │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**Signature Requirements**

```yaml
signature_policy:
  # Default policy for consuming chunks
  default:
    require_signature: true
    minimum_signatures: 1
    
  # Policy by chunk type
  by_type:
    source:
      require_signature: false         # source is human-reviewable
      
    intermediate:
      require_signature: true
      require_build_attestation: true
      
    blob:
      require_signature: true
      require_build_attestation: true
      require_slsa_level: 2            # SLSA Build L2 minimum
      
    container:
      require_signature: true
      require_build_attestation: true
      require_sbom: true               # Software Bill of Materials
      
  # Trusted signers
  trusted_signers:
    - pattern: "github.com/cadi-project/*"
      trust_level: "full"
      
    - pattern: "github.com/verified-orgs/*"
      trust_level: "standard"
      required_attestations: ["slsa-provenance-v1"]
```

**Sandbox Execution**

```yaml
sandbox:
  # For running untrusted WASM
  wasm:
    enabled: true
    memory_limit_mb: 256
    timeout_ms: 30000
    allowed_imports: ["wasi_snapshot_preview1"]
    denied_capabilities: ["network", "filesystem_write"]
    
  # For running untrusted native binaries
  native:
    enabled: true
    use_container: true                # run in isolated container
    network_mode: "none"
    read_only_root: true
    drop_capabilities: ["all"]
    seccomp_profile: "strict"
```

### 3.9 Garbage Collection System

**Local Cache GC**

```
┌─────────────────────────────────────────────────────────────┐
│                    cadi gc --local                           │
├─────────────────────────────────────────────────────────────┤
│  1. Scan ~/.cadi/store for all chunks and blobs             │
│  2. Build reference graph from local manifests              │
│  3. Mark referenced chunks as "keep"                        │
│  4. Apply retention policy to unreferenced chunks           │
│  5. Evict based on LRU/size until under limits             │
│  6. Report space reclaimed                                  │
└─────────────────────────────────────────────────────────────┘
```

**CLI Commands**

```bash
# Show GC status
cadi gc --status

# Dry run - show what would be deleted
cadi gc --dry-run

# Run GC with default policy
cadi gc

# Aggressive GC - remove everything not pinned
cadi gc --aggressive

# Pin a chunk (never GC)
cadi gc --pin <chunk_id>

# Unpin a chunk
cadi gc --unpin <chunk_id>

# Set cache size limit
cadi config set cache.max_size_gb 10
```

**Registry GC**

Registries run periodic GC with configurable policies:

```yaml
registry_gc:
  schedule: "0 2 * * *"                # 2 AM daily
  
  policies:
    - name: "orphan_cleanup"
      description: "Remove unreferenced chunks"
      conditions:
        - no_manifest_references: true
        - age_days_min: 30
      action: "archive"
      
    - name: "old_version_cleanup"
      description: "Keep only N most recent versions"
      conditions:
        - version_rank: "> 5"          # beyond 5th most recent
        - age_days_min: 90
      action: "archive"
      
    - name: "archived_cleanup"
      description: "Delete old archives"
      conditions:
        - status: "archived"
        - archived_days_min: 180
      action: "delete"
```

***

## 4. Demo Suite: Todo Suite

The demo suite shows CADI working across multiple languages and artifact types.

### 4.1 Shared Components

**Shared SQL DB (Postgres)**

- Schema:
  - Table: `todos`
    - `id uuid primary key`
    - `title text`
    - `completed boolean`
    - `created_at timestamptz`
    - `updated_at timestamptz`

- Files:
  - `examples/todo-suite/db/schema.sql`
  - `examples/todo-suite/db/docker-compose.yml` (Postgres + adminer/pgAdmin dev).

- CADI chunk:
  - `chunk_id: chunk:sha256:<db_schema>`
  - `cadi_type: source`
  - `source.language: "sql"`
  - `provides.concepts: ["todo_storage"]`

**Shared API Contract (OpenAPI)**

- OpenAPI spec describing REST endpoints:

  - `GET /todos`
  - `POST /todos`
  - `PUT /todos/{id}`
  - `DELETE /todos/{id}`

- And WebSocket events, e.g.:

  - `todos/updates` channel.

- CADI chunk:
  - `source.language: "openapi"`
  - `provides.interfaces: ["TodoApi"]`

### 4.2 Components

1. **Basic Web App (React/TS)**

   - Location:
     - `examples/todo-suite/web/basic/`
   - Features:
     - Render todo list.
     - Create, toggle, delete todos.
     - REST backend configurable via env (Node or C server).
   - Tech:
     - React + TypeScript.
     - Bundler (Vite or Webpack).
   - CADI:
     - Source CADI for TS.
     - Build:
       - TS→JS bundling (dev, prod).
     - Optional WASM module for a small logic function to showcase IR use.

2. **Styled Web App**

   - Location:
     - `examples/todo-suite/web/styled/`
   - Features:
     - Same as basic, with CSS framework (Tailwind, etc.).
   - CADI:
     - Separate chunk for shared UI components (buttons, layout).
     - Provides concept `todo_ui`.

3. **WebSocket Todo App (Node.js)**

   - Location:
     - `examples/todo-suite/backend/node-ws/`
   - Features:
     - WebSocket endpoint broadcasting todo changes to all connected clients.
     - Option to connect multiple WS servers for simple p2p-like behavior (e.g., via a shared Redis pub/sub).
   - CADI:
     - Source CADI for Node code.
     - Container CADI wrapping image built via Docker/Podman, pushing to OCI registry (e.g., GHCR) using OCI Artifact best practices.[8][6][7]

4. **Node.js REST Server**

   - Location:
     - `examples/todo-suite/backend/node-rest/`
   - Features:
     - Implements TodoApi from OpenAPI spec.
     - Connects to Postgres DB.
   - CADI:
     - Source CADI (JS/TS).
     - Container CADI for Node server image.

5. **C REST Server**

   - Location:
     - `examples/todo-suite/backend/c-rest/`
   - Features:
     - Minimal HTTP server in C.
     - JSON marshaling/unmarshaling.
     - Connects to Postgres DB.
   - Build:
     - `Makefile` with targets:
       - `native` → `binary.x86_64-linux`.
       - `wasm` → `intermediate.wasm` (optional but preferred for demo).
   - CADI:
     - Source CADI (C).
     - Blob CADI (binary).
     - Optional IR CADI (WASM).
     - Container CADI for C server image.

### 4.3 Todo Suite Manifest

`examples/todo-suite/todo-suite.cadi.yaml`:

- `manifest_id: app:uuid:todo-suite`
- Nodes:
  - `db_schema`
  - `todo_api_spec`
  - `web_frontend_basic`
  - `web_frontend_styled`
  - `node_rest_server`
  - `node_ws_server`
  - `c_rest_server`
- Edges:
  - `db_schema → node_rest_server` (interface: `todo_storage`).
  - `db_schema → c_rest_server` (interface: `todo_storage`).
  - `todo_api_spec → node_rest_server` (interface: `TodoApi`).
  - `todo_api_spec → c_rest_server` (interface: `TodoApi`).
  - `todo_api_spec → web_frontend_*` (client contracts).

- Build targets:
  - `web-dev`
    - Platform: `browser + node-dev`
    - Nodes:
      - `web_frontend_basic`: prefer `source.typescript`.
      - `node_rest_server`: prefer `source.javascript`.
  - `web-prod`
    - Platform: `browser + linux-container`
    - Nodes:
      - `web_frontend_styled`: prefer bundled JS.
      - `node_rest_server`: require `container.oci`.
  - `c-server-prod`
    - Platform: `linux-container`
    - Nodes:
      - `c_rest_server`: prefer `binary.x86_64-linux` inside `container.oci`.
  - `wasm-demo`
    - Platform: `browser` or `linux-server`
    - Nodes:
      - `todo_model`: `intermediate.wasm` (for selected logic).

***

## 5. Roadmap and Todos

> **Total Estimated Timeline**: 32–44 weeks (8–11 months)
> 
> **Key Milestone**: v0.1 Proof of Concept at Week 10 (end of Phase 1)

### Phase 0 (2–3 weeks): Foundations & Design

**Goals**

- Solidify CADI v1 spec.
- Lock demo suite architecture.
- Define security and federation models.

**Todos**

- [ ] Create `cadi-spec/` directory with:
  - [ ] `chunk.schema.json`
  - [ ] `source-cadi.schema.json`
  - [ ] `ir-cadi.schema.json`
  - [ ] `blob-cadi.schema.json`
  - [ ] `container-cadi.schema.json`
  - [ ] `manifest.schema.json`
  - [ ] `build-receipt.schema.json`
  - [ ] `dependency-graph.schema.json`
  - [ ] `registry-federation.schema.json`
  - [ ] `abi-compatibility.schema.json`
  - [ ] `security-attestation.schema.json`
  - [ ] `efficiency-metrics.schema.json`
  - [ ] `llm-context.schema.json`
  - [ ] `versioning.schema.json`
  - [ ] `garbage-collection.schema.json`
- [ ] Author `cadi-spec-v1.0.md` summarizing the schemas and versioning strategy.
- [ ] Decide implementation languages:
  - [ ] Server + CLI: Go or Rust (recommend Rust for safety + WASM story).
  - [ ] WASM toolchain: clang/emscripten/wasm-ld for C; optional Rust toolchain for internal tools.[6][7]
- [ ] Design directory structure for repo:
  - `/cmd/cadi`
  - `/cmd/cadi-server`
  - `/cmd/cadi-mcp-server`
  - `/internal/...`
  - `/cadi-spec/...`
  - `/examples/todo-suite/...`
  - `/docs/...`

**Exit criteria**

- All spec schemas drafted and reviewed.
- Tech stack chosen and basic repository scaffold created.
- Security model documented.

### Phase 0.5 (1–2 weeks): Federation & Security Design

**Goals**

- Finalize registry federation model.
- Define trust hierarchy and signing requirements.
- Document namespace allocation strategy.

**Todos**

- [ ] Define registry namespace model:
  - [ ] Hierarchical namespace scheme (e.g., `github.com/org/chunk`).
  - [ ] Namespace allocation and verification process.
  - [ ] Collision prevention rules.
- [ ] Design trust hierarchy:
  - [ ] Root of trust (CADI project keys).
  - [ ] Organization key delegation.
  - [ ] CI system key management.
- [ ] Define signing requirements by artifact type:
  - [ ] Source: optional (human-reviewable).
  - [ ] IR/Blob/Container: required with build attestation.
  - [ ] SLSA level requirements.
- [ ] Design federation protocols:
  - [ ] Upstream sync modes (mirror, proxy, cache).
  - [ ] Trust policy inheritance.
  - [ ] Cross-registry reference format.
- [ ] Define offline/air-gapped support:
  - [ ] Snapshot format.
  - [ ] Sync reconciliation.
- [ ] Document sandbox requirements:
  - [ ] WASM sandbox configuration.
  - [ ] Native binary container isolation.

**Exit criteria**

- Federation design document approved.
- Security model document approved.
- Namespace allocation policy defined.

### Phase 1 (4–6 weeks): Registry & Minimal CLI

**Goals**

- Basic registry + CLI import/publish/fetch.
- **v0.1 Proof of Concept milestone**.

**Server Todos**

- [ ] Implement `cadi-server` HTTP API:
  - [ ] `GET /health` (health check).
  - [ ] `GET /chunks/{id}`
  - [ ] `PUT /chunks/{id}`
  - [ ] `GET /blobs/{hash}`
  - [ ] `PUT /blobs/{hash}`
  - [ ] `GET /manifests/{id}`
  - [ ] `PUT /manifests/{id}`
  - [ ] `GET /search?q=&concept=&interface=&language=&type=`
  - [ ] `GET /namespaces` (federation support).
  - [ ] `POST /attestations/{chunk_id}` (upload attestation).
  - [ ] `GET /attestations/{chunk_id}` (fetch attestations).
- [ ] Implement auth middleware (API tokens).
- [ ] Implement blob storage:
  - [ ] Abstract interface.
  - [ ] S3-compatible backend (MinIO dev, S3/GCS prod).[7][8][6]
- [ ] Implement Postgres schema:
  - [ ] `chunks` table.
  - [ ] `manifests` table.
  - [ ] `build_receipts` table.
  - [ ] `attestations` table.
  - [ ] `users` / `tokens` tables.
  - [ ] `namespaces` table.
  - [ ] Indexes for search fields.

**CLI Todos (minimal)**

- [ ] `cadi init`:
  - [ ] Create config.
  - [ ] Generate local signing key (optional).
- [ ] `cadi import <path>`:
  - [ ] TS/React:
    - [ ] Detect TS (presence of `tsconfig.json`, `package.json`).
    - [ ] Create Source CADI and base manifest node.
  - [ ] Node.js:
    - [ ] Detect `package.json`, JS entrypoint.
  - [ ] C:
    - [ ] Detect `Makefile` or `CMakeLists.txt` or fallback `compile.sh`.
    - [ ] Create Source CADI.
- [ ] `cadi publish`:
  - [ ] Package blobs and specs, send to server.
  - [ ] Sign chunks on publish (if key configured).
- [ ] `cadi fetch <chunk_or_manifest>`:
  - [ ] Download specs and blobs into local cache.
  - [ ] Verify signatures (if policy requires).

**Exit criteria (v0.1 PoC)**

- Two simple test projects (TS + C) imported, published, and fetched successfully between two environments.
- Basic signature generation and verification working.
- Demo video showing end-to-end flow.

### Phase 2 (6–8 weeks): Builder, Transformations, and Build Graphs

**Goals**

- Implement builder, local cache, transformations, and manifest-based builds.
- Implement dependency resolution engine.

**Builder Core**

- [ ] Implement local cache:
  - [ ] Layout under `~/.cadi/store`.
  - [ ] Index referencing chunk_ids and blob hashes.
- [ ] Implement transformation execution:
  - [ ] Runner abstraction for steps (exec external tools, capture logs).
  - [ ] Derive action keys (hash of inputs, tool versions, flags, environment).
- [ ] Implement `cadi build`:
  - [ ] Parse manifest.
  - [ ] Resolve nodes and target definitions.
  - [ ] For each node:
    - [ ] Determine required representation(s).
    - [ ] Check local cache.
    - [ ] If not found:
      - [ ] Check registry.
      - [ ] If still not found, find and execute transformation chain.
- [ ] Implement `cadi plan`:
  - [ ] Build dry-run plan listing:
    - [ ] Cached items.
    - [ ] Downloads and sizes.
    - [ ] Builds needed and estimated times.
- [ ] Implement `cadi run` for:
  - [ ] Node servers: spawn Node process.
  - [ ] C servers: exec native binary.
  - [ ] Containers: call Docker/Podman.
  - [ ] Web apps: dev server (for dev) or static server (for prod).
  - [ ] WASM demos: use chosen runtime.

**Dependency Resolution**

- [ ] Implement constraint parser:
  - [ ] Semver ranges (^, ~, >=, etc.).
  - [ ] Exact version pinning.
  - [ ] Chunk ID pinning.
- [ ] Implement resolution algorithm:
  - [ ] Build requirement graph (BFS).
  - [ ] Detect version conflicts.
  - [ ] Apply resolution strategy (newest, oldest, pinned, manual).
- [ ] Implement lock file:
  - [ ] Generate `cadi.lock.yaml`.
  - [ ] Respect lock file on subsequent builds.
  - [ ] `cadi update` to refresh lock file.
- [ ] Handle diamond dependencies:
  - [ ] Detect conflicting version requirements.
  - [ ] Report clear error messages.
  - [ ] Support manual override.

**Cross-Compilation**

- [ ] Implement platform detection:
  - [ ] Detect host platform triple.
  - [ ] Query available toolchains.
- [ ] Implement cross-compilation support:
  - [ ] Container-based builders for Linux targets.
  - [ ] Native cross-toolchain support where available.
- [ ] Implement WASM fallback:
  - [ ] Detect when native binary unavailable.
  - [ ] Select and configure WASM runtime.
  - [ ] Report performance expectations.

**Transformations**

- [ ] TS→JS:
  - [ ] Use `tsc` or SWC.
  - [ ] Support dev vs prod builds.
- [ ] C→binary (all platforms):
  - [ ] `x86_64-unknown-linux-gnu`
  - [ ] `x86_64-unknown-linux-musl`
  - [ ] `arm64-unknown-linux-gnu`
  - [ ] `x86_64-apple-darwin`
  - [ ] `arm64-apple-darwin`
- [ ] C→intermediate.wasm:
  - [ ] Using clang/emscripten pipeline.[6][7]
- [ ] WASM→binary (optional AOT):
  - [ ] Choose one AOT tool/runtime.

**Exit criteria**

- A small sample manifest builds using `cadi build`, with caching working (repeat builds are fast).
- Dependency resolution works with lock files.
- Cross-compilation from macOS to Linux working via containers.

### Phase 2.5 (2–3 weeks): LLM Optimization Layer

**Goals**

- Implement token-efficient chunk summaries.
- Implement semantic search.
- Add efficiency metrics collection.

**Summary Generation**

- [ ] Implement summary generator:
  - [ ] Parse source code (TS, JS, C).
  - [ ] Extract docstrings/comments.
  - [ ] Generate one-liner (< 50 tokens).
  - [ ] Generate brief (< 200 tokens).
  - [ ] Generate standard (< 500 tokens).
- [ ] Implement API surface extraction:
  - [ ] Extract function/method signatures.
  - [ ] Extract type definitions.
  - [ ] Extract exported constants.
  - [ ] Generate minimal API representation.
- [ ] Store LLM context with chunks:
  - [ ] Add `llm_context` field to chunk storage.
  - [ ] Generate on import/publish.
  - [ ] Regenerate on request.

**Semantic Search**

- [ ] Implement embedding generation:
  - [ ] Integrate with embedding model (OpenAI, local model, or hybrid).
  - [ ] Embed summaries and API surfaces.
  - [ ] Store embeddings in vector index.
- [ ] Implement vector search:
  - [ ] Add pgvector extension to Postgres (or separate vector DB).
  - [ ] Implement similarity search API.
  - [ ] Add `GET /search/semantic?q=...` endpoint.
- [ ] Implement search ranking:
  - [ ] Combine semantic similarity with keyword matching.
  - [ ] Boost recent and popular chunks.
  - [ ] Filter by concepts, languages, types.

**Efficiency Metrics**

- [ ] Implement metrics collection:
  - [ ] Track cache hits/misses.
  - [ ] Track bytes transferred vs deduplicated.
  - [ ] Track build CPU time.
  - [ ] Estimate tokens saved.
- [ ] Implement metrics storage:
  - [ ] Add `metrics` table to Postgres.
  - [ ] Aggregate hourly/daily/weekly.
- [ ] Implement metrics API:
  - [ ] `GET /metrics/efficiency` endpoint.
  - [ ] `cadi stats` CLI command.

**Exit criteria**

- Every imported chunk has auto-generated summaries.
- Semantic search returns relevant results for natural language queries.
- Efficiency metrics are collected and reportable.

### Phase 3 (6–8 weeks): Todo Suite Implementation

**Goals**

- Implement all demo components and describe them in CADI.

**Shared DB**

- [ ] Implement `schema.sql` and `docker-compose.yml`.
- [ ] Create and publish Source CADI for `db_schema`.

**OpenAPI Spec**

- [ ] Define `openapi.yaml` for TodoApi.
- [ ] Create and publish Source CADI for `todo_api_spec`.

**Web Frontend: Basic**

- [ ] Implement `examples/todo-suite/web/basic/` React TS app.
- [ ] Integrate with REST backend via env config.
- [ ] Write transformation for bundling (TS→JS + bundler).
- [ ] Create Source CADI and corresponding manifest nodes.
- [ ] Generate LLM context (summaries, API surface).

**Web Frontend: Styled**

- [ ] Implement `web/styled/` with UI components & CSS framework.
- [ ] Factor UI components into separate chunk (Source CADI).
- [ ] Update manifest with `web_frontend_styled` node.

**Node REST Server**

- [ ] Implement `backend/node-rest/` using Express/Fastify.
- [ ] Implement endpoints from OpenAPI, connect to Postgres.
- [ ] Create Source CADI.
- [ ] Create container Dockerfile and build + push to OCI registry.
- [ ] Create Container CADI referencing OCI image digest.[8][7][6]
- [ ] Sign all artifacts with build attestations.

**Node WebSocket Server**

- [ ] Implement `backend/node-ws/` for WebSockets.
- [ ] Broadcast todo events to connected clients.
- [ ] Optional: simple pub/sub for scaling.
- [ ] Source CADI and optional container CADI.
  - [ ] Index referencing chunk_ids and blob hashes.
- [ ] Implement transformation execution:
  - [ ] Runner abstraction for steps (exec external tools, capture logs).
  - [ ] Derive action keys (hash of inputs, tool versions, flags, environment).
- [ ] Implement `cadi build`:
  - [ ] Parse manifest.
  - [ ] Resolve nodes and target definitions.
  - [ ] For each node:
    - [ ] Determine required representation(s).
    - [ ] Check local cache.
    - [ ] If not found:
      - [ ] Check registry.
      - [ ] If still not found, find and execute transformation chain.
- [ ] Implement `cadi plan`:
  - [ ] Build dry-run plan listing:
    - [ ] Cached items.
    - [ ] Downloads and sizes.
    - [ ] Builds needed and estimated times.
- [ ] Implement `cadi run` for:
  - [ ] Node servers: spawn Node process.
  - [ ] C servers: exec native binary.
  - [ ] Containers: call Docker/Podman.
  - [ ] Web apps: dev server (for dev) or static server (for prod).
  - [ ] WASM demos: use chosen runtime.

**Transformations**

- [ ] TS→JS:
  - [ ] Use `tsc` or SWC.
  - [ ] Support dev vs prod builds.
- [ ] C→binary.x86_64-linux:
  - [ ] Using clang/gcc.
- [ ] C→intermediate.wasm:
  - [ ] Using clang/emscripten pipeline.[6][7]
- [ ] WASM→binary.x86_64-linux:
  - [ ] Choose one AOT tool/runtime.

**Exit criteria**

- A small sample manifest builds using `cadi build`, with caching working (repeat builds are fast).

### Phase 3 (6–8 weeks): Todo Suite Implementation

**Goals**

- Implement all demo components and describe them in CADI.

**Shared DB**

- [ ] Implement `schema.sql` and `docker-compose.yml`.
- [ ] Create and publish Source CADI for `db_schema`.

**OpenAPI Spec**

- [ ] Define `openapi.yaml` for TodoApi.
- [ ] Create and publish Source CADI for `todo_api_spec`.

**Web Frontend: Basic**

- [ ] Implement `examples/todo-suite/web/basic/` React TS app.
- [ ] Integrate with REST backend via env config.
- [ ] Write transformation for bundling (TS→JS + bundler).
- [ ] Create Source CADI and corresponding manifest nodes.

**Web Frontend: Styled**

- [ ] Implement `web/styled/` with UI components & CSS framework.
- [ ] Factor UI components into separate chunk (Source CADI).
- [ ] Update manifest with `web_frontend_styled` node.

**Node REST Server**

- [ ] Implement `backend/node-rest/` using Express/Fastify.
- [ ] Implement endpoints from OpenAPI, connect to Postgres.
- [ ] Create Source CADI.
- [ ] Create container Dockerfile and build + push to OCI registry.
- [ ] Create Container CADI referencing OCI image digest.[8][7][6]

**Node WebSocket Server**

- [ ] Implement `backend/node-ws/` for WebSockets.
- [ ] Broadcast todo events to connected clients.
- [ ] Optional: simple pub/sub for scaling.
- [ ] Source CADI and optional container CADI.

**C REST Server**

- [ ] Implement minimal HTTP server + JSON handling.
- [ ] Connect to Postgres DB.
- [ ] Provide same TodoApi semantics.
- [ ] Implement Makefile:
  - [ ] `make native` → binary (all supported platforms).
  - [ ] `make wasm` → WASM.
  - [ ] `make container` → OCI container.
- [ ] Create Source CADI, Blob CADI, IR CADI (WASM).
- [ ] Container Dockerfile for the C binary, plus Container CADI.
- [ ] Sign all artifacts with build attestations.
- [ ] Generate LLM context (summaries, API surface).

**Manifest**

- [ ] Author `todo-suite.cadi.yaml` with:
  - [ ] Nodes and edges as defined above.
  - [ ] Build targets: `web-dev`, `web-prod`, `c-server-prod`, `wasm-demo`.
  - [ ] Dependency declarations with version constraints.
- [ ] Test:
  - [ ] `cadi build todo-suite.cadi.yaml --target web-dev`.
  - [ ] `cadi build ... --target c-server-prod`.
  - [ ] `cadi run ...` for each target.
  - [ ] Cross-compilation from macOS to Linux.
  - [ ] WASM fallback when native unavailable.

**Exit criteria**

- All demo targets build and run successfully through `cadi build`/`cadi run`.
- CADI registry holds all required chunks (source, IR, blobs, containers) for the demo suite.
- All artifacts are signed with valid attestations.
- LLM context generated for all chunks.
- Semantic search can find demo components by description.

### Phase 4 (4–6 weeks): Provenance, Verification, and Security

**Goals**

- Implement build receipts and verification tooling.
- Implement full attestation and signing pipeline.
- Implement sandbox execution for untrusted code.

**Builder Enhancements**

- [ ] After each successful transformation, create BuildReceipt:
  - [ ] Record input/output chunk_ids, tools, flags, environment digest.
  - [ ] Compute `build_receipt_id`.
  - [ ] Include SLSA provenance metadata.
- [ ] Save BuildReceipt in local store.
- [ ] Publish BuildReceipt to registry.
- [ ] Generate attestations:
  - [ ] Build attestation (what was built, by whom).
  - [ ] SLSA provenance attestation.
  - [ ] Sign with builder key.

**Server**

- [ ] Add CRUD for build receipts:
  - [ ] `GET /build_receipts/{id}`
  - [ ] `GET /build_receipts?chunk_id=...`
- [ ] Link receipts to chunks via `lineage.build_receipt`.
- [ ] Implement attestation verification:
  - [ ] Signature validation.
  - [ ] Trust policy enforcement.
  - [ ] SLSA level checking.

**CLI**

- [ ] Implement `cadi verify <manifest_or_chunk>`:
  - [ ] Fetch related CADI specs and receipts.
  - [ ] Verify all signatures in chain.
  - [ ] Check attestation requirements.
  - [ ] Optionally rebuild from source:
    - [ ] Compare resulting hashes with recorded ones.
  - [ ] Output human-readable verification status.
- [ ] Implement `cadi trust`:
  - [ ] `cadi trust add <signer>` - add trusted signer.
  - [ ] `cadi trust list` - list trusted signers.
  - [ ] `cadi trust policy` - configure trust policy.

**Sandbox Execution**

- [ ] Implement WASM sandbox:
  - [ ] Configure memory limits.
  - [ ] Configure timeout.
  - [ ] Restrict capabilities (no network, no filesystem write).
- [ ] Implement container sandbox for native binaries:
  - [ ] Read-only root filesystem.
  - [ ] No network by default.
  - [ ] Drop all capabilities.
  - [ ] Seccomp profile.
- [ ] Implement `cadi run --sandbox`:
  - [ ] Auto-detect untrusted artifacts.
  - [ ] Apply appropriate sandbox.

**Garbage Collection**

- [ ] Implement `cadi gc`:
  - [ ] `--status` - show cache status.
  - [ ] `--dry-run` - show what would be deleted.
  - [ ] `--aggressive` - delete everything not pinned.
- [ ] Implement `cadi gc --pin/--unpin`:
  - [ ] Pin chunks to prevent eviction.
- [ ] Implement eviction policies:
  - [ ] LRU eviction.
  - [ ] Size limits.
  - [ ] Age limits.

**Exit criteria**

- Verification passes for all demo components (source→IR→blob chain).
- Tampering with a binary is detected by `cadi verify`.
- Untrusted code runs in sandbox.
- GC keeps cache under size limits.

### Phase 5 (4–6 weeks): MCP Integration

**Goals**

- Let LLM clients query and orchestrate CADI via MCP.
- Integrate LLM optimization layer with MCP.

**MCP Server**

- [ ] Implement `cadi-mcp-server`:
  - [ ] Use MCP base protocol (JSON-RPC, resources, tools).[2][3][4][5]
  - [ ] Connect to CADI server API and local builder where necessary.

**Resources**

- [ ] `cadi.chunks` resource:
  - [ ] List/search with filters (concepts, interfaces, language).
- [ ] `cadi.manifests` resource:
  - [ ] List/search manifests and targets.
- [ ] `cadi.build_receipts` resource:
  - [ ] Fetch provenance information.

**Core Tools**

- [ ] `cadi.get_chunk(chunk_id)`
- [ ] `cadi.search_chunks(query, filters)`
- [ ] `cadi.plan_build(manifest_id, target)`
- [ ] `cadi.trigger_build(manifest_id, target)` (explicit user approval required).
- [ ] `cadi.fetch_and_open_source(chunk_id)` (return file structure and content handles).

**LLM Optimization Tools**

- [ ] `cadi.get_chunk_summary(chunk_id, max_tokens)`:
  - [ ] Return token-optimized summary.
  - [ ] Configurable detail level.
- [ ] `cadi.get_interface_only(chunk_id)`:
  - [ ] Return only API surface (signatures, types).
  - [ ] Minimal token usage.
- [ ] `cadi.semantic_search(query, filters, limit)`:
  - [ ] Natural language search.
  - [ ] Return ranked results with one-liners.
- [ ] `cadi.find_similar(chunk_id, relation_types)`:
  - [ ] Find related/alternative chunks.
- [ ] `cadi.suggest_for_task(task_description)`:
  - [ ] Suggest relevant chunks for a task.
  - [ ] Include rationale.
- [ ] `cadi.get_efficiency_metrics()`:
  - [ ] Report token savings.
  - [ ] Report reuse statistics.

**Security**

- [ ] Enforce read-only defaults.
- [ ] Gate build and publish actions behind interactive confirmations and host-side user consent, aligned with MCP’s security recommendations.[3][5][2]

**Exit criteria**

- Example MCP client configuration provided (for at least one MCP-enabled host).
- From an LLM client, it is possible to:
  - Search for `TodoApi` components via semantic search.
  - Get token-efficient summaries of chunks.
  - Inspect a C server implementation (API surface only).
  - Ask CADI to plan or trigger a build (with user approval).
- LLM optimization tools reduce context usage by ≥70%.

### Phase 6 (2–4 weeks): Hardening, Docs, Release

**Goals**

- Stabilize, document, and release CADI v1.

**Docs**

- [ ] `docs/getting-started.md`:
  - [ ] Install CLI.
  - [ ] Configure registry.
  - [ ] Run the todo-suite demo.
- [ ] `docs/spec-overview.md`:
  - [ ] Summary of CADI specs and versioning.
- [ ] `docs/mcp-integration.md`:
  - [ ] How to integrate CADI MCP server into an MCP host.
  - [ ] LLM optimization tools guide.
- [ ] `docs/demo-walkthrough.md`:
  - [ ] Step-by-step todo suite journey.
- [ ] `docs/security-model.md`:
  - [ ] Trust hierarchy.
  - [ ] Signing requirements.
  - [ ] Verification process.
- [ ] `docs/federation-guide.md`:
  - [ ] Setting up federated registries.
  - [ ] Namespace management.
  - [ ] Trust policies.
- [ ] `docs/llm-optimization.md`:
  - [ ] How CADI optimizes for LLM usage.
  - [ ] Token efficiency best practices.
  - [ ] Semantic search usage.

**Testing**

- [ ] Unit tests:
  - [ ] Builder core.
  - [ ] Dependency resolver.
  - [ ] Registry API.
  - [ ] MCP server.
  - [ ] LLM optimization layer.
  - [ ] Security/attestation.
- [ ] Integration tests:
  - [ ] Import + publish + fetch for simple projects.
  - [ ] End-to-end build + run for each demo target.
  - [ ] Verification flows.
  - [ ] Cross-compilation flows.
  - [ ] Semantic search accuracy.
- [ ] Security tests:
  - [ ] Signature verification.
  - [ ] Sandbox escape attempts.
  - [ ] Trust policy enforcement.

**Performance & Security**

- [ ] Measure build times and cache effectiveness.
- [ ] Measure semantic search latency.
- [ ] Tune parallelism in transformations where safe.
- [ ] Harden server endpoints (rate limiting, auth checks).
- [ ] Ensure MCP server has appropriate limitation of capabilities.
- [ ] Security audit of sandbox implementation.

**Release**

- [ ] Tag v1.0.0 for:
  - [ ] `cadi` CLI
  - [ ] `cadi-server`
  - [ ] `cadi-mcp-server`
- [ ] Publish:
  - [ ] Release notes summarizing supported languages, targets, and features.
  - [ ] Reference deployment templates for the server.
  - [ ] Container images for all components.
  - [ ] Homebrew formula for macOS.
  - [ ] APT/RPM packages for Linux.

**Exit criteria**

- CI green on all tests.
- Documentation enables a new user to install CADI, run the demo suite, and understand core concepts in under an hour.
- CADI v1.0.0 artifacts (binaries, containers) published and stable.
- Security audit passed.

***

## 6. Timeline Summary

| Phase | Duration | Cumulative | Key Deliverable |
|-------|----------|------------|-----------------|
| Phase 0: Foundations | 2-3 weeks | Week 3 | Spec schemas, repo scaffold |
| Phase 0.5: Federation & Security | 1-2 weeks | Week 5 | Security model, federation design |
| Phase 1: Registry & CLI | 4-6 weeks | Week 11 | **v0.1 PoC**: Basic import/publish/fetch |
| Phase 2: Builder & Deps | 6-8 weeks | Week 19 | Build system, dependency resolution |
| Phase 2.5: LLM Layer | 2-3 weeks | Week 22 | Summaries, semantic search |
| Phase 3: Demo Suite | 6-8 weeks | Week 30 | Full todo-suite working |
| Phase 4: Provenance | 4-6 weeks | Week 36 | Verification, security |
| Phase 5: MCP | 4-6 weeks | Week 42 | LLM integration complete |
| Phase 6: Release | 2-4 weeks | Week 46 | **v1.0 Release** |

**Critical Path**: Phases 0 → 1 → 2 → 3 → 4 → 6

**Parallel Work Possible**:
- Phase 0.5 can overlap with Phase 0
- Phase 2.5 can start during Phase 2
- Phase 5 can start during Phase 4

***

## 7. Team Roles

- **Tech Lead / Architect**
  - Owns CADI spec and overall architecture.
- **Backend Engineers**
  - Implement server, registry, and CLI.
- **Build/Tooling Engineer**
  - Implements transformations, cache, build graph logic, provenance.
- **Application Engineers**
  - Implement demo suite components (web, Node, C).
- **MCP/Integration Engineer**
  - Implements MCP server and example client configurations.[1][4][5][9][2][3][7][8][6]

This `implementation.md` is intended as the project’s execution blueprint and can be used directly by engineering teams to plan sprints, allocate ownership, and start implementation.

Sources
[1] Fast, correct, reproducible builds with Nix + Bazel https://www.youtube.com/watch?v=2wI5J8XYxM8
[2] Specification - Model Context Protocol https://modelcontextprotocol.io/specification/2025-03-26
[3] Specification https://modelcontextprotocol.io/specification/2025-06-18
[4] Model Context Protocol - Wikipedia https://en.wikipedia.org/wiki/Model_Context_Protocol
[5] Introducing the Model Context Protocol https://www.anthropic.com/news/model-context-protocol
[6] Distributing WebAssembly modules using OCI registries https://radu-matei.com/blog/wasm-to-oci/
[7] Distributing WebAssembly components using OCI registries https://opensource.microsoft.com/blog/2024/09/25/distributing-webassembly-components-using-oci-registries
[8] OCI Registries https://wasmcloud.com/docs/deployment/netconf/registries/
[9] Bazel and Nix: A Migration Experience https://tweag.io/blog/2022-12-15-bazel-nix-migration-experience/
[10] Make a standard format for input and content addressed ... https://github.com/NixOS/nix/issues/7310
[11] Advice wanted: writing a build system in Nix - Help https://discourse.nixos.org/t/advice-wanted-writing-a-build-system-in-nix/23460

***

## 8. Risk Assessment and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Scope creep from feature requests | High | High | Strict v1 scope definition; defer to v1.1+ |
| Dependency resolution edge cases | Medium | High | Extensive test suite; learn from Cargo/NPM |
| Security vulnerabilities in sandbox | Medium | Critical | External security audit; bug bounty |
| LLM embedding model changes | Medium | Medium | Abstract embedding layer; support multiple models |
| Cross-compilation toolchain issues | Medium | Medium | Container-based builders as fallback |
| Performance issues with large registries | Low | High | Pagination; caching; query optimization |
| MCP spec changes | Low | Medium | Abstract MCP layer; version compatibility |

***

## 9. Success Metrics Dashboard

Track these metrics throughout development and post-release:

**Development Metrics**
- [ ] Schema coverage: All 15 schemas defined and validated
- [ ] API coverage: All endpoints implemented and tested
- [ ] Test coverage: ≥80% code coverage

**v0.1 PoC Metrics (Week 11)**
- [ ] Import/publish/fetch working end-to-end
- [ ] Basic signing working
- [ ] Demo video created

**v1.0 Release Metrics (Week 46)**
- [ ] All 7 demo suite components working
- [ ] ≥30 chunks in registry
- [ ] Semantic search returning relevant results
- [ ] LLM context reduces tokens by ≥70%
- [ ] Build cache hit rate ≥80%
- [ ] Security audit passed
- [ ] Documentation complete

**Post-Release Metrics**
- [ ] Time to first successful demo: < 30 minutes
- [ ] User-reported bugs in first month: < 10 critical
- [ ] Community contributions in first 3 months

***

## 10. Appendix: Quick Reference

### CLI Command Summary

| Command | Description |
|---------|-------------|
| `cadi init` | Initialize CADI config and optionally generate signing key |
| `cadi import <path>` | Import project and create Source CADI chunks |
| `cadi build <manifest>` | Build artifacts from manifest |
| `cadi publish` | Publish chunks to registry |
| `cadi fetch <id>` | Download chunks from registry |
| `cadi run <manifest>` | Run built artifacts |
| `cadi plan <manifest>` | Show build plan without executing |
| `cadi verify <id>` | Verify signatures and provenance |
| `cadi trust` | Manage trusted signers |
| `cadi gc` | Garbage collect local cache |
| `cadi stats` | Show efficiency metrics |

### Environment Variables

| Variable | Description |
|----------|-------------|
| `CADI_REGISTRY` | Default registry URL |
| `CADI_TOKEN` | Authentication token |
| `CADI_CACHE_DIR` | Local cache directory (default: `~/.cadi/store`) |
| `CADI_SIGNING_KEY` | Path to signing key |
| `CADI_TRUST_POLICY` | Trust policy mode (`strict`, `standard`, `permissive`) |

### Configuration File (~/.cadi/config.yaml)

```yaml
registry:
  url: "https://registry.cadi.dev"
  namespace: "github.com/myorg"
  
auth:
  token: "${CADI_TOKEN}"
  
cache:
  dir: "~/.cadi/store"
  max_size_gb: 10
  eviction_policy: "lru"
  
build:
  parallelism: 4
  prefer_representation: ["binary", "wasm", "source"]
  
security:
  signing_key: "~/.cadi/signing.key"
  trust_policy: "standard"
  verify_on_fetch: true
  sandbox_untrusted: true
  
llm:
  embedding_model: "text-embedding-3-large"
  summary_max_tokens: 500
```
