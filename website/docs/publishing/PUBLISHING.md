# Publishing CADI to crates.io

This guide explains how to publish all CADI crates to crates.io.

## Prerequisites

1. Create a crates.io account at https://crates.io
2. Get your API token: https://crates.io/me (copy from "API Tokens")
3. Configure cargo: `cargo login <token>`

## Publication Order

Crates must be published in dependency order:

1. **cadi-core** (no internal dependencies)
2. **cadi-builder** (depends on cadi-core)
3. **cadi-registry** (depends on cadi-core)
4. **cadi-scraper** (depends on cadi-core, cadi-registry)
5. **cadi** (depends on all above)

## Pre-Publication Checklist

- [ ] All crates have version 1.0.0
- [ ] All crates have README.md
- [ ] All crates have proper Cargo.toml metadata
- [ ] License file exists (LICENSE in root)
- [ ] No local path dependencies (except in workspace)
- [ ] `cargo check` passes
- [ ] `cargo test` passes (if tests exist)
- [ ] `cargo build --release` produces no errors
- [ ] All public APIs have documentation

## Publishing Steps

### 1. Login to crates.io

```bash
cargo login
# Paste your API token when prompted
```

### 2. Verify Each Crate

```bash
# For each crate in order
cd internal/cadi-core
cargo publish --dry-run
cargo publish

# Wait for it to appear on crates.io before publishing dependents
# Check: https://crates.io/crates/cadi-core
```

### 3. Publish cadi-core

```bash
cd /Users/kderbyma/Desktop/cadi/internal/cadi-core
cargo publish --dry-run  # Should complete successfully
cargo publish            # Publish to crates.io
```

Wait for confirmation and check https://crates.io/crates/cadi-core

### 4. Update cadi-builder Dependencies

After cadi-core is published, update `cadi-builder/Cargo.toml`:

```toml
[dependencies]
cadi-core = "1.0"  # Use published version instead of path
```

### 5. Publish cadi-builder

```bash
cd /Users/kderbyma/Desktop/cadi/internal/cadi-builder
cargo publish --dry-run
cargo publish
```

### 6. Publish cadi-registry

```bash
cd /Users/kderbyma/Desktop/cadi/internal/cadi-registry
cargo publish --dry-run
cargo publish
```

### 7. Publish cadi-scraper

```bash
cd /Users/kderbyma/Desktop/cadi/internal/cadi-scraper
cargo publish --dry-run
cargo publish
```

### 8. Publish cadi CLI

Finally, publish the CLI which depends on all others:

```bash
cd /Users/kderbyma/Desktop/cadi/cmd/cadi
cargo publish --dry-run
cargo publish
```

## After Publication

### Installation

Users can now install with:

```bash
cargo install cadi
```

### Using as Libraries

```toml
[dependencies]
cadi-core = "1.0"
cadi-builder = "1.0"
cadi-registry = "1.0"
cadi-scraper = "1.0"
```

### Documentation

Documentation automatically appears at:
- https://docs.rs/cadi-core
- https://docs.rs/cadi-builder
- https://docs.rs/cadi-registry
- https://docs.rs/cadi-scraper
- https://docs.rs/cadi

(May take a few minutes to build)

## Troubleshooting

### "Crate already exists"

Each version can only be published once. To publish again:

1. Bump the version in Cargo.toml
2. Update CHANGELOG.md
3. Commit and tag
4. Publish new version

### "Cannot find crate dependency"

Wait 1-2 minutes after publishing a crate before publishing dependents. The index needs to update.

### "Unauthorized"

Run `cargo login` again and paste your current API token.

### Local Path Dependencies

If you see "local path dependency" errors, ensure all Cargo.toml files use published crates:

```bash
grep -r "path = " internal/*/Cargo.toml cmd/*/Cargo.toml
```

Should only show workspace paths during development.

## Updating crates.io Entries

To update crate metadata on crates.io without republishing:

1. Edit Cargo.toml
2. Update on crates.io dashboard: https://crates.io/crates/cadi-core/settings

## Version Management

After v1.0.0, follow semantic versioning:

- **1.0.x** - Patch releases (bug fixes only)
- **1.1.0** - Minor releases (new features, backwards compatible)
- **2.0.0** - Major releases (breaking changes)

## Continuous Publishing

For CI/CD automation, see `.github/workflows/` for GitHub Actions setup.

Example workflow step:

```yaml
- name: Publish to crates.io
  env:
    CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_TOKEN }}
  run: |
    cd internal/cadi-core && cargo publish
    cd ../cadi-builder && cargo publish
    cd ../cadi-registry && cargo publish
    cd ../cadi-scraper && cargo publish
    cd ../../cmd/cadi && cargo publish
```

## Support

For crates.io issues, visit https://crates.io

For CADI-specific issues, see https://github.com/ConflictingTheories/cadi
