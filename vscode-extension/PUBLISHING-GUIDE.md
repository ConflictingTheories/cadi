# Publishing CADI VS Code Extension to Marketplace

## Prerequisites

1. **VS Code Marketplace Publisher Account**
   - Go to https://marketplace.visualstudio.com/
   - Sign in with your Microsoft account
   - Create a publisher profile (use "ConflictingTheories" as the publisher ID)

2. **Personal Access Token (PAT)**
   - Go to https://dev.azure.com/ and sign in
   - Navigate to User Settings → Personal Access Tokens
   - Create a new token with "Marketplace → Manage" scope
   - Copy the token (keep it secure!)

## Publishing Steps

### Step 1: Install VSCE CLI (if not already installed)
```bash
npm install -g @vscode/vsce
```

### Step 2: Login to VS Code Marketplace
```bash
vsce login ConflictingTheories
# When prompted, enter your Personal Access Token
```

### Step 3: Verify Login
```bash
vsce verify-pat ConflictingTheories
```

### Step 4: Publish the Extension
```bash
cd vscode-extension
vsce publish
```

### Step 5: Alternative - Publish Specific Version
```bash
# Patch version (2.0.1)
vsce publish patch

# Minor version (2.1.0)
vsce publish minor

# Major version (3.0.0)
vsce publish major
```

## Automated Publishing (Optional)

### GitHub Actions Setup

Create `.github/workflows/publish.yml`:

```yaml
name: Publish Extension

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'

    - name: Install dependencies
      run: npm ci
      working-directory: vscode-extension

    - name: Package extension
      run: npm run package
      working-directory: vscode-extension

    - name: Publish extension
      run: npx vsce publish -p ${{ secrets.VSCE_PAT }}
      working-directory: vscode-extension
      env:
        VSCE_PAT: ${{ secrets.VSCE_PAT }}
```

### Add Publishing Scripts to package.json

The following scripts are already added:
- `npm run package` - Creates .vsix file
- `npm run publish:patch` - Publish patch version
- `npm run publish:minor` - Publish minor version
- `npm run publish:major` - Publish major version

## Extension Details

- **Publisher**: ConflictingTheories
- **Name**: cadi
- **Display Name**: CADI - Content-Addressed Development Interface
- **Version**: 2.0.0
- **Categories**: Other
- **VS Code Engine**: ^1.74.0

## Marketplace Listing

The extension will appear on the marketplace with:
- Description: "VS Code integration for CADI - the agentic development platform"
- Features listed in README.md
- Icon: cadi-icon.png (128x128)
- Repository: https://github.com/ConflictingTheories/cadi

## Post-Publishing

1. **Verify Installation**: Search for "CADI" in VS Code extensions
2. **Check Marketplace Page**: Visit https://marketplace.visualstudio.com/items?itemName=ConflictingTheories.cadi
3. **Update Documentation**: Ensure README and marketplace description are accurate
4. **Monitor Reviews**: Respond to user feedback and issues

## Troubleshooting

### Common Issues:

1. **"Publisher not found"**
   - Ensure publisher ID matches exactly: "ConflictingTheories"
   - Verify publisher exists on marketplace

2. **"Invalid token"**
   - Regenerate PAT with correct scopes
   - Ensure token hasn't expired

3. **"Extension already exists"**
   - Increment version number in package.json
   - Or use version-specific publish commands

4. **Icon issues**
   - Ensure icon is PNG format, 128x128 pixels
   - Path should be "assets/cadi-icon.png"

### Verification Commands:

```bash
# Check extension info
vsce show ConflictingTheories.cadi

# List publisher extensions
vsce list ConflictingTheories

# Validate package
vsce package --no-dependencies
```

## Next Steps

1. Complete the marketplace publisher account setup
2. Generate and securely store your PAT
3. Run the publishing commands
4. Verify the extension appears in the marketplace
5. Announce the release to the CADI community