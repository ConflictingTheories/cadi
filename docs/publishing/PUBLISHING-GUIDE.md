# Publishing CADI to crates.io

Complete guide for publishing all CADI crates to crates.io.

## Overview

CADI consists of several publishable crates:

- **cadi-core** (library) - Core types and utilities
- **cadi-builder** (library) - Build engine and pipeline
- **cadi-registry** (library) - Registry client
- **cadi-scraper** (library) - Scraper/chunker utility
- **cadi** (binary/library) - CLI tool

These must be published in dependency order since they depend on each other.

## Prerequisites

1. **Rust and Cargo** - Latest stable version
2. **crates.io Account** - Create at https://crates.io
3. **API Token** - Get from https://crates.io/me

## Setup

### 1. Login to crates.io

```bash
cargo login <YOUR_API_TOKEN>
```

You can get your token from: https://crates.io/me

Verify login:
```bash
cargo login --check
```

### 2. Check Git Status

Make sure your git repository is clean before publishing:

```bash
git status
```

## Publishing

### Option A: Automatic Publishing (Recommended)

Use the provided publishing script which handles everything:

```bash
bash scripts/publish-to-crates-io.sh
```

The script will:
1. ✓ Check crates.io authentication
2. ✓ Verify git is clean
3. ✓ For each library crate:
   - Run `cargo package` to validate
   - Run `cargo publish --dry-run` to test
   - Ask for confirmation
   - Publish to crates.io
   - Wait for index update (60s per crate)

### Option B: Manual Publishing

If you prefer to publish manually, do so in this order:

```bash
# 1. cadi-core (no dependencies on other CADI crates)
cd internal/cadi-core
cargo publish

# Wait ~60 seconds for index to update

# 2. cadi-builder (depends on cadi-core)
cd ../cadi-builder
cargo publish

# Wait ~60 seconds

# 3. cadi-registry (depends on cadi-core)
cd ../cadi-registry
cargo publish

# Wait ~60 seconds

# 4. cadi-scraper (depends on cadi-core, cadi-registry)
cd ../cadi-scraper
cargo publish

# Wait ~60 seconds

# 5. cadi (CLI, depends on all libraries)
cd ../../cmd/cadi
cargo publish
```

## Testing Before Publishing

### Dry Run

Test publication without uploading:

```bash
cd internal/cadi-core
cargo publish --dry-run
```

### Package Validation

Validate the package without publishing:

```bash
cd internal/cadi-core
cargo package
```

### Local Testing

Test that a crate works as a dependency:

```bash
cd /tmp
cargo new test-cadi-app
cd test-cadi-app
# Edit Cargo.toml:
# [dependencies]
# cadi-core = "1.0.0"
cargo build
```

## Troubleshooting

### "Package already exists at this version"

This means the version was already published. To publish a new version:

1. Update the version in `Cargo.toml`:
   ```toml
   [package]
   version.workspace = true  # Uses workspace version
   ```

2. Update workspace version in root `Cargo.toml`:
   ```toml
   [workspace.package]
   version = "1.0.1"
   ```

3. Commit and push
4. Run publish script again

### "Cannot find {crate} on crates.io"

The dependency hasn't been published yet or the index hasn't updated. Solutions:

1. **Check dependency order** - Publish dependencies first
2. **Wait longer** - crates.io index takes up to 2 minutes to update
3. **Manually specify path** - For testing:
   ```toml
   cadi-core = { version = "1.0.0", path = "../../internal/cadi-core" }
   ```

### "Missing README.md"

Each crate needs a README.md file in its directory. They're already created in:

- `internal/cadi-core/README.md`
- `internal/cadi-builder/README.md`
- `internal/cadi-registry/README.md`
- `internal/cadi-scraper/README.md`
- `cmd/cadi/README.md`

### "License file not found"

The root LICENSE file is referenced. Make sure `LICENSE` exists:

```bash
ls -la LICENSE  # Should exist in root directory
```

## Verification

### Check Published Crates

After publishing, verify on crates.io:

- https://crates.io/crates/cadi-core
- https://crates.io/crates/cadi-builder
- https://crates.io/crates/cadi-registry
- https://crates.io/crates/cadi-scraper
- https://crates.io/crates/cadi

### Check Documentation

Documentation auto-publishes to docs.rs:

- https://docs.rs/cadi-core
- https://docs.rs/cadi-builder
- https://docs.rs/cadi-registry
- https://docs.rs/cadi-scraper
- https://docs.rs/cadi

## Publishing Checklist

Before publishing, verify:

- ✓ `cargo check` passes
- ✓ `cargo test` passes (if tests exist)
- ✓ Git repository is clean
- ✓ All crates have unique versions (usually all 1.0.0)
- ✓ All crates have descriptions
- ✓ All crates have LICENSE = "MIT"
- ✓ All crates have repository/homepage URLs
- ✓ README.md files exist
- ✓ You're logged into crates.io
- ✓ You have publish rights for all crate names

## Security Notes

### API Token

Never commit or share your crates.io API token. It's stored in:

```
~/.cargo/credentials.toml
```

### Yanking Broken Versions

If you publish a broken version, you can yank it:

```bash
cargo yank --vers 1.0.0 --crate cadi-core
```

This prevents new projects from using that version but doesn't remove existing installs.

## After Publishing

Once all crates are published:

1. **Tag Release** in git:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **Update README** with installation instructions:
   ```bash
   # Install the CADI CLI
   cargo install cadi
   ```

3. **Announce** the release on social media/forums

## Support

For issues with:

- **crates.io account**: https://crates.io/account-settings
- **crates.io publishing**: https://doc.rust-lang.org/cargo/reference/publishing.html
- **CADI specific questions**: Open an issue on https://github.com/ConflictingTheories/cadi

---

Last Updated: January 11, 2026
