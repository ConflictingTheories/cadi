# CADI Constraint DSL (CADL) v2.0

## Purpose

A minimal, deterministic encoding for expressing:
1. **Interface contracts** - Syntactic AND semantic compatibility
2. **Capability constraints** - Platform/runtime requirements
3. **Behavioral contracts** - Effects, resources, state machines
4. **Cross-language ABI** - Memory ownership, encoding, calling conventions
5. **Composition rules** - Value-level compatibility and invariants
6. **Selection policies** - How to choose between equivalent chunks

## Design Principles

- **Brevity**: Express constraints in <100 chars per rule
- **Determinism**: Same constraints always yield same resolution
- **Composability**: Constraints combine via logical operators
- **Human-readable**: Understandable without full spec
- **Safety-first**: Catch incompatibilities at build-time, not runtime

---

## Core Syntax (Extended)

### 1. Full Interface Declaration with Contracts

```cadl
@interface VideoCodec {
  // Syntactic signature
  encode(frame: ImageData, quality: u8) -> Blob
  decode(data: Blob) -> ImageData
  
  // SEMANTIC CONTRACT (new in v2)
  @contract {
    // What this interface actually does
    codec: "h264"                    // Concrete codec format
    profile: "baseline" | "main" | "high"
    container: "mp4"
    
    // Behavioral guarantees
    ensures: [
      "decode(encode(x, q)) ≈ x",    // Round-trip property
      "quality ∈ [0, 100]",          // Input domain
      "higher_quality => larger_size" // Monotonic property
    ]
    
    // Performance characteristics
    complexity: {
      encode: "O(width * height)",
      decode: "O(width * height)"
    }
  }
  
  // EFFECT SYSTEM (new in v2)
  @effects {
    concurrency: "thread_safe"       // Thread safety guarantee
    io: []                           // No IO operations
    memory: "bounded(width*height*4)" // Memory bound
    
    // State mutations
    mutates: []                      // Pure function
    reads: ["gpu.context"]           // GPU state dependency
    
    // Blocking behavior
    blocking: "no"                   // Non-blocking
    async: "no"                      // Synchronous execution
  }
  
  // CROSS-LANGUAGE ABI (new in v2)
  @abi {
    // String handling
    string_encoding: "utf8"
    
    // Memory ownership
    memory_ownership: {
      "frame": "borrowed",           // Input is borrowed (caller owns)
      "return": "owned"              // Output is owned (callee transfers)
    }
    
    // Error handling
    error_model: "exception"         // Throws exceptions
    error_types: ["InvalidFormat", "OutOfMemory"]
    
    // Numeric types
    numeric: {
      endianness: "little",
      int_size: 32,
      pointer_size: 64
    }
    
    // Calling convention
    calling_convention: "c"          // C calling convention
  }
  
  // DATA FORMAT CONTRACT (new in v2)
  @data_format {
    input: {
      ImageData: {
        format: "RGBA",
        value_range: [0, 255],       // uint8 range
        color_space: "sRGB",
        layout: "packed"             // No padding
      }
    }
    
    output: {
      Blob: {
        format: "mp4/h264",
        schema: "iso14496-10",       // H.264 spec
        max_size: "input.size * 2"   // Size bound
      }
    }
  }
  
  // RESOURCE CONTRACTS (new in v2)
  @resources {
    // Preconditions
    requires: [
      "frame.width <= 16384",
      "frame.height <= 16384",
      "frame.size <= 100MB",
      "quality ∈ [0, 100]"
    ]
    
    // Resource usage
    memory: {
      peak: "frame.size * 3",        // Peak memory usage
      allocation_model: "stack+heap"
    }
    
    cpu_time: "O(frame.size)",
    gpu_time: "O(frame.size)",
    
    // GPU requirements
    gpu: {
      required: "yes",
      min_memory: "512MB",
      shader_model: "webgl1"
    }
  }
  
  // STATE MACHINE (new in v2)
  @protocol {
    // Stateless for this interface
    stateless: "yes"
    
    // If stateful, would specify:
    // states: ["Init", "Ready", "Processing", "Done"]
    // transitions: ["Init --setup()--> Ready", ...]
  }
  
  // NUMERICAL CONTRACTS (new in v2)
  @numerical {
    precision: "f32",                // Float32 internally
    
    error_bounds: {
      absolute_error: "<= 1e-5",
      relative_error: "<= 1e-3"
    }
    
    special_values: {
      nan: "error",                  // NaN is an error
      inf: "error",                  // Infinity is an error
      denormals: "flush_to_zero"
    }
    
    deterministic: "platform_dependent" // May vary by GPU
  }
  
  // OBSERVABILITY (new in v2)
  @observability {
    logging: {
      events: ["encode_start", "encode_complete", "encode_error"]
    }
    
    metrics: {
      counters: ["frames_encoded", "errors"],
      histograms: ["encode_duration_ms", "output_size_bytes"]
    }
    
    tracing: {
      enabled: "yes",
      spans: ["encode", "gpu_transfer", "compression"]
    }
  }
  
  // SECURITY (new in v2)
  @permissions {
    allow: ["gpu.compute", "memory.allocate(max:1GB)"]
    deny: ["network.*", "filesystem.*", "device.*"]
    
    sandbox: {
      isolation: "wasm",
      network: "none",
      filesystem: "none"
    }
  }
}
```

### 2. Implementation Declaration

```cadl
@impl VideoCodec.webgl1 {
  chunk_id: "chunk:sha256:abc123..."
  
  // Platform capabilities required
  caps: [webgl>=1.0, gpu.memory>=512MB]
  
  // Interface provided
  provides: VideoCodec
  
  // CONTRACT IMPLEMENTATION (new in v2)
  @contract_impl {
    codec: "h264",
    profile: "baseline",              // Specific profile
    container: "mp4"
    // All ensures/complexity inherited from interface
  }
  
  @effects_impl {
    concurrency: "thread_safe",       // Matches interface
    io: [],
    memory: "bounded(1GB)",           // More specific bound
    
    // Implementation-specific details
    gpu_usage: "high",
    power_consumption: "medium"
  }
  
  @abi_impl {
    // Language-specific mapping
    language: "typescript",
    
    signature: "encode(frame: ImageData, quality: number): Blob",
    
    memory_ownership: {
      "frame": "borrowed",            // Matches interface
      "return": "owned"
    }
    
    error_model: "exception",
    error_mapping: {
      "InvalidFormat": "TypeError",
      "OutOfMemory": "RangeError"
    }
  }
  
  @data_format_impl {
    // Exactly matches interface requirements
    input: { ImageData: { format: "RGBA", value_range: [0, 255] } },
    output: { Blob: { format: "mp4/h264" } }
  }
  
  @resources_impl {
    // Implementation satisfies interface requirements
    memory: { peak: "frame.size * 2.5" },  // Better than interface
    cpu_time: "O(frame.size * 1.2)",
    gpu_time: "O(frame.size * 0.8)"
  }
  
  @numerical_impl {
    precision: "f32",                 // Matches interface
    error_bounds: {
      absolute_error: "<= 1e-6",      // Better than interface
      relative_error: "<= 1e-4"
    }
  }
  
  // VERSION (new in v2)
  @version {
    semver: "1.2.3",
    compatible_with: ["1.0.0", "1.1.0", "1.2.0"],
    breaking_changes_from: ["0.9.0"]
  }
}

@impl VideoCodec.webgl2 {
  chunk_id: "chunk:sha256:def456..."
  caps: [webgl>=2.0, gpu.shader.compute]
  provides: VideoCodec
  
  @contract_impl {
    codec: "h264",                    // SAME as webgl1
    profile: "main",                  // DIFFERENT - higher profile
    container: "mp4"
    
    // Compatibility mode
    compat_modes: {
      webgl1: {
        profile: "baseline",          // Can downgrade to baseline
        caps: [webgl>=1.0]
      }
    }
  }
  
  // ... rest similar to webgl1
}
```

### 3. Cross-Chunk Constraints (new in v2)

```cadl
@constraints {
  // MATCHING CONTRACTS
  // If using encoder, must use matching decoder
  match_contract {
    if uses(H264_Encoder) {
      require(H264_Decoder)
      
      assert(
        Encoder.@contract.codec == Decoder.@contract.codec,
        Encoder.@contract.profile == Decoder.@contract.profile,
        Encoder.@contract.container == Decoder.@contract.container
      )
    }
  }
  
  // DATA FORMAT COMPATIBILITY
  // Serializer and deserializer must use same protocol
  match_data_format {
    if uses(Serializer) {
      require(Deserializer)
      
      assert(
        Serializer.@data_format.protocol == Deserializer.@data_format.protocol,
        Serializer.@data_format.version == Deserializer.@data_format.version
      )
    }
  }
  
  // COMPOSITION COMPATIBILITY
  // Adjacent components in pipeline must have compatible data formats
  composition {
    for (A, B) in pipeline_edges {
      assert(
        A.@data_format.output compatible_with B.@data_format.input
      )
    }
  }
  
  // ABI COMPATIBILITY
  // All components sharing memory must have same ABI
  abi_consistency {
    if exists(components.*.@abi.shared_memory) {
      assert(
        all_equal(components.*.@abi.endianness),
        all_equal(components.*.@abi.pointer_size),
        all_equal(components.*.@abi.struct_alignment)
      )
    }
  }
  
  // RESOURCE BUDGETS
  // Total resource usage must not exceed budget
  resource_budget {
    assert(
      sum(components.*.@resources.memory.peak) <= platform.memory.available,
      sum(components.*.@resources.gpu_time) <= platform.gpu.time_budget
    )
  }
  
  // SECURITY BOUNDARIES
  // Untrusted components must be sandboxed
  security {
    for component in components {
      if component.trust_level == "untrusted" {
        require(component.@permissions.sandbox.isolation != "none")
      }
    }
  }
}
```

---

## Complete Example: Messaging P2P App (Updated)

```cadl
app MessagingP2P {
  version: "1.0.0"
  
  // Platform capabilities
  @platform {
    caps: [
      webgl>=1.0,
      webrtc.datachannel,
      wasm.mvp,
      indexeddb
    ]
    
    exclude: [webgl2, wasm.threads]
    
    resources: {
      memory: { total: "2GB", available: "1.5GB" },
      gpu: { memory: "512MB", time_budget: "16ms" },
      cpu: { cores: 4, time_budget: "100ms" }
    }
  }
  
  // Component dependencies
  @components {
    // Video codec with full contracts
    VideoCodec: {
      interface: @interface VideoCodec {
        encode(ImageData, u8) -> Blob
        decode(Blob) -> ImageData
        
        @contract {
          codec: "h264",
          profile: "baseline",
          container: "mp4"
        }
        
        @effects {
          concurrency: "thread_safe",
          memory: "bounded(1GB)"
        }
        
        @abi {
          string_encoding: "utf8",
          memory_ownership: { return: "owned" }
        }
        
        @data_format {
          output: { Blob: { format: "mp4/h264" } }
        }
      }
      
      impls: [
        webgl1[webgl>=1.0]{priority:required},
        software[cpu.simd]{priority:fallback}
      ]
      
      reject: [webgl2_pure]
    }
    
    // Encryption with security contracts
    Crypto: {
      interface: @interface Encryptor {
        encrypt(data: Blob, key: Key) -> Blob
        decrypt(data: Blob, key: Key) -> Blob
        
        @contract {
          algorithm: "aes256gcm",
          key_size: 256,
          
          ensures: [
            "decrypt(encrypt(x, k), k) == x",
            "constant_time(encrypt)",        // Timing attack resistant
            "constant_time(decrypt)"
          ]
        }
        
        @effects {
          concurrency: "thread_safe",
          timing: "constant"                 // CRITICAL for security
        }
        
        @permissions {
          allow: ["crypto.subtle", "memory.allocate(max:100MB)"],
          deny: ["network.*", "filesystem.*"]
        }
      }
      
      impls: [
        wasm_subtle[wasm.mvp,timing:constant]{priority:required},
        webcrypto[crypto.subtle,timing:constant]{priority:high}
      ]
      
      require: [timing:constant]             // MUST be constant-time
    }
    
    // P2P transport with state machine
    P2PTransport: {
      interface: @interface Transport {
        connect(peer_id: String) -> Connection
        send(conn: Connection, data: Blob) -> Result
        receive(conn: Connection) -> Blob
        disconnect(conn: Connection) -> void
        
        @contract {
          protocol: "webrtc_datachannel",
          reliable: "yes",
          ordered: "yes"
        }
        
        @effects {
          concurrency: "thread_safe",
          io: ["network.send", "network.receive"],
          async: "yes"
        }
        
        @protocol {
          states: ["Disconnected", "Connecting", "Connected", "Failed"],
          initial: "Disconnected",
          
          transitions: [
            "Disconnected --connect()--> Connecting",
            "Connecting --on_open--> Connected",
            "Connecting --on_error--> Failed",
            "Connected --send()--> Connected",
            "Connected --receive()--> Connected",
            "Connected --disconnect()--> Disconnected",
            "Failed --*--> ERROR"
          ]
        }
        
        @resources {
          memory: { peak: "10MB per connection" },
          network: { bandwidth: "1Mbps per connection" }
        }
      }
      
      impls: [
        webrtc[webrtc.datachannel]{priority:required}
      ]
    }
    
    // Screen capture with capability refinement
    ScreenCapture: {
      interface: @interface Capture {
        start() -> Stream
        stop(stream: Stream) -> void
        
        @contract {
          format: "video/vp8",
          fps: 30,
          resolution: "1920x1080"
        }
        
        @effects {
          io: ["device.screen"],
          async: "yes"
        }
        
        @permissions {
          allow: ["navigator.mediaDevices.getDisplayMedia"],
          user_consent: "required"
        }
        
        @runtime_caps {
          // Capabilities can change at runtime
          monitoring: {
            watch: ["screen.available", "permissions.granted"]
          },
          
          degradation: {
            "screen.available == false": {
              action: "switch_to_fallback",
              fallback: "stub_capture"
            },
            "permissions.denied": {
              action: "error",
              message: "Screen capture requires user permission"
            }
          }
        }
      }
      
      impls: [
        browser[navigator.mediaDevices.getDisplayMedia]{priority:high},
        electron[desktopCapturer]{priority:high},
        stub[]{priority:fallback}
      ]
    }
  }
  
  // Cross-component constraints
  @constraints {
    // Video codec and screen capture must use compatible formats
    video_compat {
      assert(
        ScreenCapture.@contract.format compatible_with VideoCodec.@contract.container
      )
    }
    
    // Encryption and decryption must match
    crypto_match {
      assert(
        all(Crypto.impls, impl => 
          impl.@contract.algorithm == "aes256gcm" &&
          impl.@effects.timing == "constant"
        )
      )
    }
    
    // If VideoCodec uses GPU, ensure ScreenCapture doesn't overload
    resource_balance {
      if VideoCodec.selected.uses_gpu {
        assert(ScreenCapture.@contract.fps <= 30)
      }
    }
    
    // All components must fit in resource budget
    resource_budget {
      assert(
        sum(components.*.@resources.memory.peak) <= platform.resources.memory.available
      )
    }
    
    // Security: untrusted components must be sandboxed
    security {
      for component in [VideoCodec, Crypto] {
        assert(component.@permissions.sandbox.isolation == "wasm")
      }
    }
  }
  
  // Build targets
  @targets {
    dev: {
      prefer: "source",
      optimizations: "off",
      debug: "on"
    }
    
    prod: {
      prefer: "binary",
      optimizations: "aggressive",
      debug: "off",
      
      // Production-specific constraints
      require: [
        "all(components.*.@effects.timing == 'constant' OR security_ok)",
        "all(components.*.@permissions.sandbox != 'none')"
      ]
    }
  }
}
```

---

## Value-Level Compatibility (new in v2)

### Composition Type System

```cadl
@value_types {
  // Define rich value types
  ImageData_uint8 {
    format: "RGBA",
    value_range: [0, 255],
    value_type: "u8",
    color_space: "sRGB"
  }
  
  ImageData_float {
    format: "RGBA",
    value_range: [0.0, 1.0],
    value_type: "f32",
    color_space: "linear"
  }
  
  // Automatic conversion rules
  @conversions {
    ImageData_float -> ImageData_uint8 {
      method: "multiply_and_clamp",
      formula: "clamp(x * 255.0, 0, 255)"
    }
    
    ImageData_uint8 -> ImageData_float {
      method: "divide",
      formula: "x / 255.0"
    }
  }
}

// Use in interface
@interface ColorNormalizer {
  process(input: ImageData_uint8) -> ImageData_float
  
  @data_format {
    input: { ImageData_uint8 },
    output: { ImageData_float }
  }
}

@interface VideoEncoder {
  encode(input: ImageData_uint8) -> Blob
  
  @data_format {
    input: { ImageData_uint8 },
    output: { Blob }
  }
}

// Composition check
@composition {
  pipeline = ColorNormalizer >> VideoEncoder
  
  // Type checker catches mismatch:
  // ColorNormalizer outputs ImageData_float
  // VideoEncoder expects ImageData_uint8
  // ERROR: Type mismatch, need converter
  
  // Fixed:
  pipeline = ColorNormalizer >> FloatToUint8Converter >> VideoEncoder
  // OK: Types align
}
```

---

## Interface Versioning (new in v2)

```cadl
// Version 1.0
@interface PaymentProcessor@1.0 {
  charge(amount: u64, currency: String) -> Receipt
  
  @version {
    semver: "1.0.0",
    stable: "yes"
  }
}

// Version 2.0 - Added metadata parameter
@interface PaymentProcessor@2.0 {
  charge(amount: u64, currency: String, metadata: Map?) -> Receipt
  //                                     ^^^^^^^^ optional
  
  @version {
    semver: "2.0.0",
    stable: "yes",
    
    extends: PaymentProcessor@1.0,
    
    compatibility: {
      backward: {
        "1.0": "compatible",
        strategy: "default_parameter(metadata, {})"
      },
      forward: {
        "1.0": "incompatible"  // v1.0 impls can't serve v2.0 clients
      }
    },
    
    migration: {
      from: "1.0",
      to: "2.0",
      automatic: "yes",
      migration_chunk: "chunk:sha256:migration123..."
    }
  }
}

// Version resolution
@build {
  deps: [
    PaymentProcessor@^2.0  // Accepts 2.0, 2.1, but not 3.0
  ]
  
  // Resolves to:
  // - v2.0 impl if available
  // - OR v1.0 impl with automatic upgrade wrapper
}
```

---

## Decompilation Support (new in v2)

```cadl
@decompilation {
  source: "legacy_code.c"
  
  // Inferred interface through static analysis
  @inferred_interface {
    signature: "void* process_data(void*, size_t, int)",
    confidence: 0.85,
    
    // Inferred contracts (need validation)
    @inferred_contract {
      method: "static_analysis + fuzzing",
      samples: 10000,
      
      contracts: [
        { rule: "input != NULL", confidence: 0.99 },
        { rule: "len > 0", confidence: 0.95 },
        { rule: "flags in [0,1,2,4]", confidence: 0.92 }
      ]
    }
    
    @inferred_effects {
      io: ["filesystem.read"],  // Detected file I/O
      memory: "allocates",      // Detected malloc calls
      confidence: 0.88
    }
    
    @inferred_abi {
      memory_ownership: {
        output: "caller_must_free",  // Detected malloc without free
        confidence: 0.90
      },
      error_handling: "returns_null_on_error",
      confidence: 0.85
    }
  }
  
  // Human annotations to complete the picture
  @human_annotations {
    validated_contracts: [
      "input != NULL",
      "len > 0"
    ],
    
    added_contracts: [
      "len <= 1MB",  // Human adds size limit
      "flags & 0xF8 == 0"  // Only bottom 5 bits used
    ],
    
    abi_clarifications: {
      memory_ownership: {
        output: "caller_must_free",
        error_case: "returns NULL, no allocation"
      },
      
      error_handling: "returns_null_on_error",
      
      flags_meaning: {
        0: "default",
        1: "compress",
        2: "encrypt",
        4: "validate",
        8: "streaming"  // Human documents flags
      }
    }
  }
  
  // Final validated interface
  @validated_interface {
    signature: "Result<Data, Error> process_data(data: Data, flags: Flags)",
    
    @contract {
      ensures: [
        "data.len > 0 && data.len <= 1MB",
        "flags & 0xF8 == 0"
      ]
    }
    
    @abi {
      memory_ownership: { output: "owned" },
      error_model: "result_type"
    }
  }
}
```

---

## Enhanced Resolution Algorithm (v2)

### Step 1: Parse and Validate Contracts

```
For each dependency:
  1. Parse interface signature
  2. Parse ALL contracts (@contract, @effects, @abi, @data_format, etc.)
  3. Validate contract completeness
  4. Check for contract conflicts
```

### Step 2: Filter by Capability AND Contract Match

```
For each dependency:
  available_chunks = chunks where:
    - chunk.caps ⊆ global.caps  (existing)
    - chunk.@contract matches interface.@contract  (NEW)
    - chunk.@effects matches interface.@effects  (NEW)
    - chunk.@abi matches interface.@abi  (NEW)
```

### Step 3: Validate Cross-Chunk Constraints

```
For all @constraints:
  - Check contract matching (encoder/decoder, serializer/deserializer)
  - Check composition compatibility (value types align)
  - Check ABI consistency (shared memory components)
  - Check resource budgets (sum <= available)
  - Check security boundaries (sandboxing)
```

### Step 4: Validate Composition Safety

```
For each pipeline edge (A -> B):
  assert(A.@data_format.output compatible_with B.@data_format.input)
  
  If incompatible:
    - Search for converter chunk
    - OR report error with suggestion
```

### Step 5: Select by Priority + Deterministic Tiebreaker

```
Priority order: required > high > medium > low > fallback
Tiebreaker: lexicographic sort of chunk_id
```

**Example Resolution with Contracts**:

```cadl
// Both provide VideoCodec
VideoCodec@webgl1 {
  @contract { codec: "h264", profile: "baseline" }
  @effects { concurrency: "thread_safe" }
  priority: high
}

VideoCodec@av1 {
  @contract { codec: "av1", profile: "main" }  // DIFFERENT codec!
  @effects { concurrency: "thread_safe" }
  priority: high
}

// Application requires H.264
@app {
  VideoCodec: {
    @contract { codec: "h264" }  // Must be H.264
  }
}

// Resolution:
// 1. Both match capability check
// 2. webgl1 matches contract (h264 ✓)
// 3. av1 REJECTED (av1 ≠ h264) ✗
// 4. SELECT: VideoCodec@webgl1
```

---

## Sanity Check DSL (Extended)

```cadl
@check MessagingP2P {
  // Existing checks
  assert(all(components.*.impls.length > 0))
  assert(none(components.*.caps intersect platform.exclude))
  
  // NEW: Contract validation
  contract_validation {
    for component in components {
      for impl in component.impls {
        // Signature matches
        assert(impl.signature == component.interface.signature)
        
        // Contract matches (NEW)
        assert(impl.@contract matches component.interface.@contract)
        
        // Effects match (NEW)
        assert(impl.@effects satisfies component.interface.@effects)
        
        // ABI matches (NEW)
        assert(impl.@abi compatible_with component.interface.@abi)
        
        // Data formats match (NEW)
        assert(impl.@data_format matches component.interface.@data_format)
      }
    }
  }
  
  // NEW: Cross-component constraints
  cross_component {
    assert(satisfiable(constraints))
    
    // Check all constraint rules
    for rule in constraints {
      assert(evaluate(rule) == true)
    }
  }
  
  // NEW: Composition validation
  composition {
    for (A, B) in pipeline_edges {
      assert(A.@data_format.output compatible_with B.@data_format.input)
    }
  }
  
  // NEW: Resource budget validation
  resources {
    total_memory = sum(components.*.@resources.memory.peak)
    assert(total_memory <= platform.resources.memory.available)
    
    total_gpu_time = sum(components.*.@resources.gpu_time)
    assert(total_gpu_time <= platform.resources.gpu.time_budget)
  }
  
  // NEW: Security validation
  security {
    for component in components {
      if component.trust_level == "untrusted" {
        assert(component.@permissions.sandbox.isolation != "none")
      }
    }
  }
  
  // NEW: ABI consistency
  abi_consistency {
    if exists(components.*.@abi.shared_memory) {
      assert(all_equal(components.*.@abi.endianness))
      assert(all_equal(components.*.@abi.pointer_size))
    }
  }
  
  // Dependency graph is acyclic
  assert(acyclic(dependency_graph))
}
```

**Example Output**:

```
Running sanity checks for MessagingP2P...

✓ All components have viable implementations (4/4)
✓ No excluded capabilities required
✓ Interface signatures match (8/8)
✓ Contract validation passed (8/8)
  ✓ Semantic contracts match
  ✓ Effect contracts match
  ✓ ABI contracts match
  ✓ Data format contracts match
✓ Cross-component constraints satisfied (5/5)
  ✓ VideoCodec contracts match
  ✓ Crypto timing=constant enforced
  ✓ Resource budgets OK (1.2GB / 1.5GB available)
✓ Composition safety verified (3/3 edges)
  ✓ ColorNormalizer -> Converter -> VideoEncoder
✓ Security requirements met (2/2)
  ✓ Sandboxing enabled for untrusted components
✓ ABI consistency verified
  ✓ All shared memory components use little-endian, 64-bit pointers
✓ Dependency graph is acyclic

BUILD MANIFEST VALID ✓
Selected chunks:
  - VideoCodec@webgl1 (chunk:sha256:abc123...)
    Contract: h264/baseline/mp4, thread_safe, bounded(1GB)
  - Crypto@wasm_subtle (chunk:sha256:def456...)
    Contract: aes256gcm, timing=constant, sandboxed
  - P2PTransport@webrtc (chunk:sha256:789ghi...)
    Contract: webrtc_datachannel, reliable, ordered
  - ScreenCapture@browser (chunk:sha256:012jkl...)
    Contract: vp8@30fps, requires_permission

Estimated build time: 4m 32s
Total memory usage: 1.2GB / 1.5GB available
GPU time: 12ms / 16ms budget
```

---

## Binary Encoding (Extended for v2)

```
┌─────────────────────────────────────────────────────────────────┐
│ CADL Binary Format v2.0                                          │
├─────────────────────────────────────────────────────────────────┤
│ Header (8 bytes)                                                │
│   Magic: 0xCAD2 (2 bytes)                                      │
│   Version: 0x02 (1 byte)                                       │
│   Flags: 0x00 (1 byte)                                         │
│   Contract_Version: 0x01 (1 byte)                              │
│   Reserved: 0x000000 (3 bytes)                                 │
├─────────────────────────────────────────────────────────────────┤
│ Capability Vector (variable)                                    │
│   Count: u8                                                     │
│   Entries: [cap_id: u16, value: varint] * count               │
├─────────────────────────────────────────────────────────────────┤
│ Interface Signatures (variable)                                │
│   Count: u16                                                    │
│   Entries: [sig_hash: u32, param_types: bitfield] * count     │
├─────────────────────────────────────────────────────────────────┤
│ Contracts (variable) -- NEW in v2                              │
│   Count: u16                                                    │
│   Entries: [contract_record] * count                          │
│     contract_hash: u32 (4 bytes)                               │
│     contract_type: u8 (1 byte)                                 │
│       0x01 = semantic, 0x02 = effects, 0x04 = abi             │
│       0x08 = data_format, 0x10 = resources                    │
│     contract_data: variable                                    │
├─────────────────────────────────────────────────────────────────┤
│ Cross-Chunk Constraints (variable) -- NEW in v2                │
│   Count: u16                                                    │
│   Entries: [constraint_record] * count                        │
│     constraint_type: u8                                        │
│     chunk_refs: [u32] * n                                      │
│     predicate: variable                                        │
└─────────────────────────────────────────────────────────────────┘

// Example: Full VideoCodec interface with contracts
Encoded: CAD2 02 00 01 000000  03  0001 0A00  0002 1400  0003 1E00
         ^^^^  ^^  ^^  ^^  ^^^^^^  ^^  ^^^^^^^^  ^^^^^^^^  ^^^^^^^^
         Magic Ver Flg CV  Reserv  CC  Cap1      Cap2      Cap3
         
         02  A3F2B1C4  D5E6F7A8
         ^^  ^^^^^^^^  ^^^^^^^^
         SC  SigHash1  SigHash2
         
         04  B2C3D4E5  01  0F  48323634  ...
         ^^  ^^^^^^^^  ^^  ^^  ^^^^^^^^  ^^^
         CC  ConHash   Typ Flg Data...   ...
         
Total: ~120 bytes for full interface with contracts
vs ~2000 chars text representation
Compression ratio: 16.7x
```

---

## Integration with Build System

### 1. Extended Manifest

```yaml
# cadi-manifest.yaml
manifest_version: "2.0"
constraint_dsl_version: "2.0"

cadl: |
  app MessagingP2P {
    @platform {
      caps: [webgl>=1.0, webrtc, wasm.mvp]
      resources: { memory: { available: "1.5GB" } }
    }
    
    @components {
      VideoCodec: {
        interface: @interface VideoCodec {
          encode(ImageData, u8) -> Blob
          
          @contract { codec: "h264", profile: "baseline" }
          @effects { concurrency: "thread_safe", memory: "bounded(1GB)" }
          @abi { memory_ownership: { return: "owned" } }
          @data_format { output: { format: "mp4/h264" } }
        }
        
        impls: [
          webgl1[webgl>=1.0]{priority:high},
          webgl2[webgl>=2.0,compat:webgl1]{priority:low}
        ]
      }
    }
    
    @constraints {
      match_contract {
        if uses(Encoder) { require(Decoder); assert(contracts_match) }
      }
    }
  }
```

### 2. CLI Integration (Extended)

```bash
# Validate contracts
$ cadi check manifest.yaml --validate-contracts
✓ Signature validation passed (4/4)
✓ Contract validation passed (4/4)
✓ Effect validation passed (4/4)
✓ ABI validation passed (4/4)
✓ Cross-chunk constraints satisfied (3/3)

# Show detailed contract info
$ cadi inspect chunk:sha256:abc123 --show-contracts
VideoCodec@webgl1
  Signature: encode(ImageData, u8) -> Blob
  Contract:
    codec: h264
    profile: baseline
    ensures: ["decode(encode(x)) ≈ x"]
  Effects:
    concurrency: thread_safe
    memory: bounded(1GB)
  ABI:
    memory_ownership: { return: owned }
    error_model: exception

# Check composition safety
$ cadi check-composition A.chunk B.chunk
✗ ERROR: Data format mismatch
  A outputs: ImageData[0.0-1.0 float]
  B expects: ImageData[0-255 uint8]
  Suggestion: Insert FloatToUint8Converter between A and B

# Decompile legacy code
$ cadi decompile legacy.c --infer-contracts --interactive
Analyzing legacy.c...
✓ Inferred signature: void* process(void*, size_t, int)
✓ Inferred contracts (confidence: 0.85):
  - input != NULL (0.99)
  - len > 0 (0.95)
  - flags in [0,1,2,4] (0.92)

Please validate inferred contracts:
1. input != NULL [y/N]: y
2. len > 0 [y/N]: y
3. flags in [0,1,2,4] [y/N]: y

Additional constraints?
> len <= 1MB

Memory ownership of return value:
1. caller_must_free
2. callee_owns
3. shared
Choice: 1

Generated interface chunk saved to: chunk:sha256:new123...
```

### 3. Build Receipt (Extended)

```json
{
  "build_receipt_id": "sha256:...",
  "cadl_version": "2.0",
  
  "constraint_resolution": {
    "global_caps": ["webgl>=1.0", "webrtc", "wasm.mvp"],
    
    "selections": [
      {
        "component": "VideoCodec",
        "selected_chunk": "chunk:sha256:abc123",
        "selected_variant": "webgl1",
        
        "selection_reason": {
          "capability_match": true,
          "contract_match": true,
          "priority": "high"
        },
        
        "validated_contracts": {
          "signature": "✓",
          "semantic_contract": "✓ (codec:h264, profile:baseline)",
          "effects": "✓ (thread_safe, bounded)",
          "abi": "✓ (owned return, exception errors)",
          "data_format": "✓ (mp4/h264)",
          "resources": "✓ (< 1GB memory)"
        },
        
        "alternatives_rejected": [
          {
            "chunk": "chunk:sha256:def456",
            "variant": "av1",
            "reason": "contract_mismatch (codec: av1 ≠ h264)"
          }
        ]
      }
    ],
    
    "constraint_validation": {
      "signatures_valid": true,
      "contracts_valid": true,
      "effects_valid": true,
      "abi_valid": true,
      "cross_component_constraints": true,
      "composition_safe": true,
      "resource_budget_ok": true,
      "security_requirements_met": true
    }
  }
}
```

---

## Summary of v2 Enhancements

### Added Contract Types

1. **@contract** - Semantic guarantees (what it does)
2. **@effects** - Side effects, concurrency, state
3. **@abi** - Cross-language memory/calling conventions
4. **@data_format** - Wire format specifications
5. **@resources** - Memory, CPU, GPU bounds
6. **@protocol** - State machines, method ordering
7. **@numerical** - Floating-point precision
8. **@observability** - Logging, metrics, tracing
9. **@permissions** - Security boundaries
10. **@version** - Interface evolution
11. **@runtime_caps** - Dynamic capability monitoring

### Added Validation

- Contract matching (not just signature)
- Cross-chunk constraints
- Composition safety (value types)
- ABI consistency
- Resource budgets
- Security boundaries

### Improved Resolution

- Filter by contracts, not just capabilities
- Validate cross-chunk invariants
- Check composition compatibility
- Enforce resource limits
- Verify security requirements

### Developer Experience

- Earlier error detection (build-time vs runtime)
- Clear error messages with suggestions
- Decompilation support for legacy code
- Interactive contract inference
- Detailed validation reports

---

## Migration from v1 to v2

### For Existing Chunks

```cadl
// v1 (minimal)
@interface VideoCodec {
  encode(ImageData, u8) -> Blob
}

// v2 (extended) - backwards compatible
@interface VideoCodec {
  encode(ImageData, u8) -> Blob
  
  // Add contracts incrementally
  @contract { codec: "h264" }  // Start simple
  @effects { concurrency: "thread_safe" }  // Add as needed
}
```

### For Existing Manifests

```cadl
// v1 manifests still work
build {
  caps: [webgl>=1.0]
  deps: [VideoCodec@webgl1{priority:high}]
}

// But v2 adds validation
// - Warns if contracts missing
// - Suggests adding @contract, @effects, etc.
// - Can auto-infer some contracts from implementation
```

### Gradual Adoption

1. **Phase 1**: Add @contract (semantic guarantees)
2. **Phase 2**: Add @effects and @abi (interop)
3. **Phase 3**: Add @data_format (composition)
4. **Phase 4**: Add @resources, @permissions (production)

---

## Conclusion

**CADL v2** extends the constraint DSL with comprehensive contract systems that catch incompatibilities at build-time:

- ✅ **Semantic compatibility** - Not just signatures
- ✅ **Cross-language interop** - ABI, ownership, errors
- ✅ **Behavioral contracts** - Effects, resources, state
- ✅ **Composition safety** - Value-level compatibility
- ✅ **Security boundaries** - Permissions, sandboxing
- ✅ **Decompilation support** - Legacy code integration
- ✅ **LLM-friendly** - Rich behavioral context

**Brevity maintained**: Core constraints still <100 chars, full contracts ~500 chars, binary encoding ~120 bytes.

**Determinism guaranteed**: Same manifest + same registry → same build, with full validation.