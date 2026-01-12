# CADI Java Atomizer

Example extension that demonstrates how to create a language atomizer for CADI.

## Overview

This extension provides Java language support for CADI by:
- Extracting atomic chunks from Java source code (classes, methods)
- Resolving Java import statements
- Supporting Java-specific syntax and patterns

## Building

```bash
cargo build --release
```

This will create a dynamic library that can be loaded by CADI.

## Usage

1. Copy the built library to your CADI extensions directory:
   ```bash
   cp target/release/libextension.so ~/.cadi/extensions/java-atomizer/
   cp extension.toml ~/.cadi/extensions/java-atomizer/
   ```

2. CADI will automatically load the extension on startup.

## Extension Structure

```
java-atomizer/
├── extension.toml    # Extension manifest
├── libextension.so   # Compiled extension library
└── src/
    └── lib.rs        # Extension implementation
```

## Implementation Details

The Java atomizer uses regex patterns to:
- Extract class definitions and their bodies
- Extract method definitions within classes
- Resolve import statements

Each extracted atom includes:
- Unique ID based on language and type
- Content type (`text/java`)
- Metadata (language, type, name, class)
- Dependencies (parent classes/methods)
- Content hash for deduplication

## Testing

```java
public class Example {
    private String name;

    public Example(String name) {
        this.name = name;
    }

    public String getName() {
        return name;
    }
}
```

This would be atomized into:
- Class atom: `Example` class definition
- Method atoms: constructor and `getName()` method