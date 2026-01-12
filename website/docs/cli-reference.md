# CADI CLI Reference

Complete reference for all CADI command-line interface commands.

## Global Options

```
--config <path>     Path to config file (default: ~/.cadi/config.yaml)
--verbose, -v       Enable verbose output
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

Import a project and create Source CADI chunks.

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

Build artifacts from a manifest.

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
- `--no-sign` - Disable signing (signing is enabled by default if key is configured)
- `--registry <url>` - Target registry URL
- `--dry-run` - Perform a trial run without uploading

**Example:**
```bash
cadi publish --sign
```

---

### `cadi query`

Query the registry for chunks matching a pattern or criteria.

```bash
cadi query <pattern> [options]
```

**Arguments:**
- `pattern` - Search pattern or chunk ID

**Options:**
- `--language <lang>` - Filter by language
- `--limit <num>` - Limit results (default: 10)
- `--json` - Output in JSON format

**Example:**
```bash
cadi query "auth middleware" --language rust
```

---

### `cadi fetch`

Fetch chunks from a registry to local cache.

```bash
cadi fetch <chunk_or_manifest> [options]
```

**Arguments:**
- `chunk_or_manifest` - Chunk ID or manifest path

**Options:**
- `--verify` - Verify hash after fetching
- `--output <dir>` - Output directory (default: local cache)

**Example:**
```bash
cadi fetch chunk:sha256:abc123... --verify
```

---

### `cadi run`

Run a built application or specific chunk.

```bash
cadi run [options]
```

**Options:**
- `--manifest <path>` - Path to manifest file
- `--target <name>` - Target to run
- `--chunk <id>` - Run a specific chunk ID directly
- `--sandbox` - Run in sandboxed environment
- `--env <key=val>` - Set environment variable

**Example:**
```bash
cadi run --target server --env PORT=8080
```

---

### `cadi validate`

Validate a CADL file against the specification.

```bash
cadi validate <path> [options]
```

**Arguments:**
- `path` - Path to .cadl file

**Options:**
- `--json` - Output validation report in JSON
- `--strict` - Enable strict validation mode

**Example:**
```bash
cadi validate my-interface.cadl
```

---

### `cadi scrape`

Scrape and chunk repositories or files.

```bash
cadi scrape <url_or_path> [options]
```

**Arguments:**
- `url_or_path` - URL of a repository or local file path

**Options:**
- `--strategy <name>` - Chunking strategy (file, semantic, fixed)
- `--output-dir <dir>` - Where to save generated chunks

**Example:**
```bash
cadi scrape https://github.com/my/repo --strategy semantic
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
- `add` - Add a trusted publisher
- `remove` - Remove a trusted publisher
- `list` - List trusted publishers
- `policy` - Show or set trust policy

**Example:**
```bash
cadi trust add publisher:abc123 --level full
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

**Example:**
```bash
cadi gc --dry-run
```

---

### `cadi stats`

Show usage and efficiency statistics.

```bash
cadi stats [options]
```

**Options:**
- `--detailed` - Show detailed breakdown
- `--json` - Output in JSON format

**Example:**
```bash
cadi stats --detailed
```

---

### `cadi demo`

Run example demonstrations.

```bash
cadi demo <name> [options]
```

**Arguments:**
- `name` - Demo name (todo-suite)

**Options:**
- `--target <target>` - Specific target to demo
- `--clean` - Clean before building

**Example:**
```bash
cadi demo todo-suite --target web
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CADI_CONFIG` | Config file path | `~/.cadi/config.yaml` |
| `CADI_CACHE` | Cache directory | `~/.cadi/store` |
| `CADI_REGISTRY` | Default registry URL | `https://registry.cadi.dev` |
| `CADI_TOKEN` | Authentication token | - |
| `CADI_LOG` | Log level (tracing) | `cadi=info` |

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
