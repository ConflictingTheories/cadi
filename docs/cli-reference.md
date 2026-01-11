# CADI CLI Reference

Complete reference for all CADI command-line interface commands.

## Global Options

```
--config <path>     Path to config file (default: ~/.config/cadi/config.yaml)
--verbose, -v       Enable verbose output
--quiet, -q         Suppress non-error output
--help, -h          Show help
--version           Show version
```

## Commands

### `cadi init`

Initialize a new CADI project.

```bash
cadi init [path] [options]
```

**Arguments:**
- `path` - Directory to initialize (default: current directory)

**Options:**
- `--name <name>` - Project name
- `--template <template>` - Project template (minimal, library, application)

**Example:**
```bash
cadi init my-project --template library
```

---

### `cadi import`

Import source files as a CADI chunk.

```bash
cadi import <path> [options]
```

**Arguments:**
- `path` - Path to source files or directory

**Options:**
- `--language <lang>` - Source language (rust, python, typescript, etc.)
- `--name <name>` - Chunk name
- `--recursive` - Include subdirectories

**Example:**
```bash
cadi import ./src --language rust --name my-library
```

---

### `cadi build`

Build the manifest for a target platform.

```bash
cadi build [options]
```

**Options:**
- `--manifest <path>` - Path to manifest file (default: cadi.yaml)
- `--target <name>` - Build target name
- `--prefer <rep>` - Preferred representation (source, ir, blob)

**Example:**
```bash
cadi build --target web --prefer ir
```

---

### `cadi publish`

Publish chunks to a registry.

```bash
cadi publish [options]
```

**Options:**
- `--chunks <ids>` - Specific chunk IDs to publish (comma-separated)
- `--sign` - Sign chunks with your key
- `--registry <url>` - Target registry URL

**Example:**
```bash
cadi publish --sign
```

---

### `cadi fetch`

Fetch chunks from a registry.

```bash
cadi fetch <chunk_or_manifest> [options]
```

**Arguments:**
- `chunk_or_manifest` - Chunk ID or manifest path

**Options:**
- `--tier <tier>` - Storage tier preference (hot, warm, cold)
- `--verify` - Verify hash after fetching

**Example:**
```bash
cadi fetch chunk:sha256:abc123... --verify
```

---

### `cadi run`

Run a built application.

```bash
cadi run [options]
```

**Options:**
- `--manifest <path>` - Path to manifest file
- `--target <name>` - Target to run
- `--port <port>` - Port for server applications
- `--detach` - Run in background
- `--sandbox` - Run in sandboxed environment

**Example:**
```bash
cadi run --target server --port 8080
```

---

### `cadi plan`

Show the build plan without executing.

```bash
cadi plan [options]
```

**Options:**
- `--manifest <path>` - Path to manifest file
- `--target <name>` - Target to plan for
- `--verbose` - Show detailed plan

**Example:**
```bash
cadi plan --target web --verbose
```

---

### `cadi verify`

Verify chunk integrity and provenance.

```bash
cadi verify <chunk_or_manifest> [options]
```

**Arguments:**
- `chunk_or_manifest` - Chunk ID or manifest path

**Options:**
- `--rebuild` - Attempt to rebuild from source
- `--verbose` - Show verification details

**Example:**
```bash
cadi verify chunk:sha256:abc123... --rebuild
```

---

### `cadi trust`

Manage trusted publishers and trust policies.

```bash
cadi trust <subcommand>
```

**Subcommands:**

#### `cadi trust add`
Add a trusted publisher.
```bash
cadi trust add <publisher_id> [options]
--level <level>   Trust level (full, verified, limited)
```

#### `cadi trust remove`
Remove a trusted publisher.
```bash
cadi trust remove <publisher_id>
```

#### `cadi trust list`
List trusted publishers.
```bash
cadi trust list
```

#### `cadi trust policy`
Show or set trust policy.
```bash
cadi trust policy [options]
--show            Show current policy
--set <key=value> Set policy value
```

---

### `cadi gc`

Garbage collect local cache.

```bash
cadi gc [options]
```

**Options:**
- `--status` - Show cache status only
- `--dry-run` - Show what would be removed
- `--aggressive` - More aggressive cleanup
- `--pin <id>` - Pin a chunk (prevent GC)
- `--unpin <id>` - Unpin a chunk

**Example:**
```bash
cadi gc --dry-run
```

---

### `cadi stats`

Show usage statistics.

```bash
cadi stats [options]
```

**Options:**
- `--period <period>` - Time period (day, week, month, all)
- `--detailed` - Show detailed breakdown
- `--export <format>` - Export format (json, csv)

**Example:**
```bash
cadi stats --period week --detailed
```

---

### `cadi demo`

Run example demonstrations.

```bash
cadi demo <name> [options]
```

**Arguments:**
- `name` - Demo name (todo-suite, hello-world, etc.)

**Options:**
- `--target <target>` - Specific target to demo
- `--no-build` - Skip building, use cached
- `--clean` - Clean before building

**Example:**
```bash
cadi demo todo-suite --target web
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CADI_CONFIG` | Config file path | `~/.config/cadi/config.yaml` |
| `CADI_CACHE` | Cache directory | `~/.cache/cadi` |
| `CADI_REGISTRY` | Default registry URL | `https://registry.cadi.dev` |
| `CADI_TOKEN` | Authentication token | - |
| `CADI_LOG` | Log level | `info` |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Build failed |
| 4 | Network error |
| 5 | Verification failed |
| 6 | Authentication error |
