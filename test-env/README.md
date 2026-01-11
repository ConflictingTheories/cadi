# Local CADI Test Environment

This directory contains test projects and configurations for local CADI development.

## Structure

```
test-env/
├── README.md           # This file
├── config/             # Local configuration
│   └── config.yaml     # CADI config pointing to local server
├── test-project/       # Sample project for testing
└── registry-data/      # Local registry storage
```

## Running Locally

1. Start the local registry server:
   ```bash
   cd /Users/kderbyma/Desktop/cadi
   cargo run -p cadi-server
   ```

2. In another terminal, run CADI commands:
   ```bash
   cd test-env/test-project
   ../../target/release/cadi init
   ```

## Configuration

The local configuration in `config/config.yaml` points to `http://localhost:8080` 
for all registry operations.
