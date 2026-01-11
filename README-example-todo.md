# CADI Todo App Builder

This script demonstrates CADI's chunk reuse capabilities by creating new todo apps with different visual styles while reusing the same core logic.

## Usage

```bash
./example-todo.sh
```

The script will:
1. Prompt you to choose an aesthetic
2. Create a new todo app with that styling
3. Use CADI to build it, reusing cached chunks
4. Serve the app to prove it works

## Available Aesthetics

- **cyberpunk**: Neon green on black with glow effects
- **minimalist**: Clean white and black design
- **sunset**: Warm orange and brown tones
- **ocean**: Deep blue with glass effects

## What It Demonstrates

- ✅ **Chunk Reuse**: The `todo-core` chunk (containing all the todo logic) is reused from cache
- ✅ **Minimal Generation**: Only CSS styling is generated for each new aesthetic
- ✅ **CADI Build System**: Shows how CADI pulls dependencies and builds incrementally
- ✅ **Registry Integration**: Publishes new chunks and reuses existing ones

## Example Output

```
Build Plan:
  → todo-core (prefer: ["source:rust"]) [cached]    ← REUSED!
  → ocean-todo (prefer: ["bundle:esm"]) [build]     ← NEW STYLING ONLY

Executing build...
  ✓ todo-core (using cache)                          ← NO REBUILD NEEDED
  → Building ocean-todo...                           ← ONLY STYLE CHANGES
  ✓ ocean-todo built successfully
```

This proves that CADI enables true code reuse - the core business logic stays the same while only presentation layers change.