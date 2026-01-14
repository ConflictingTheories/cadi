# CADI-First Development Prompt for LLMs

You are building a new project using CADI (Content-Addressed Development Interface).
CADI's power is **semantic reuse**: find and compose existing code atoms instead of writing from scratch.

## Your Workflow

### Step 1: Plan with CADI
Before writing code, decompose the problem:
- "I need to build: [project]"
- "Breaking it down into: [components]"
- "Which of these likely exist in past projects? [reusable]"
- "Which are novel? [to generate]"

### Step 2: Write a Build Spec (not code!)
Create a `build.build-spec.yaml` that:
1. **References reusable chunks** - search by semantic intent
2. **Marks novel parts** with `generate: true`
3. **Declares dependencies** between components
4. **Defines targets** (api, worker, etc.)

Example:
```yaml
components:
  - id: auth
    query: "JWT authentication middleware for Node.js"
    # Build engine searches CADI, finds todo's JWT chunk, resolves it
  
  - id: novel_feature
    generate: true
    depends_on: [auth]
    description: "New feature only in this project"
```

### Step 3: Build & Iterate
1. Run: `cadi build blog.build-spec.yaml`
2. Engine resolves queries, generates scaffolds
3. If "needs_generation" → you provide details (summaries, not full code)
4. Engine links everything automatically

## CBS Format Quick Reference

### Reuse Pattern (50-200 tokens)
```yaml
- id: my_component
  source: chunk:sha256:abc123  # or use query below
  query: "what you need"  # Build engine finds it
  language: typescript
```

### Generate Pattern (100-300 tokens for description)
```yaml
- id: novel_feature
  generate: true
  depends_on: [auth, db]
  description: "Brief description of what it does"
  interface:
    inputs: ["userId", "data"]
    outputs: ["result"]
```

## Token Savings

**Without CADI** (your baseline):
- Read existing code: 500 tokens
- Understand patterns: 300 tokens
- Write new code: 1000 tokens
- **Total: ~1800 tokens**

**With CADI** (your target):
- Search: 50 tokens
- Review 5 results: 100 tokens
- Build spec: 200 tokens
- (Novel code only): 500 tokens
- **Total: ~850 tokens (53% savings!)**

## Constraints to Remember

- **Prefer reuse**: If something similar exists, search CADI first
- **Query semantically**: "How would a human describe this?" not keywords
- **Mark novel clearly**: `generate: true` means "I'm writing new logic"
- **Declare dependencies**: Build engine links them automatically
- **No manual imports**: Engine generates imports/scaffolding for you

## Success = Reuse + Minimal Novel Code

Target composition: **60-80% reused, 20-40% novel**
If you're writing >50% from scratch, search CADI harder!

---

## Example: Building a Blog App (CADI-First)

### Decomposition
- Need: JWT auth (✓ from todo app)
- Need: DB operations (✓ from todo app)
- Need: Error handling (? search CADI)
- Need: Blog content service (✗ novel)
- Need: Comments (✗ novel)
- Need: Analytics (✗ novel)

### Build Spec (80 lines total!)
```yaml
version: "1.0"
project:
  name: blog-app
  language: typescript

components:
  # REUSE
  - id: auth
    query: "JWT authentication for Node.js"
  - id: database
    query: "PostgreSQL CRUD operations with TypeScript"
  - id: logger
    query: "logging middleware for HTTP servers"
  
  # NOVEL (only ~40 lines each for descriptions)
  - id: content_service
    generate: true
    depends_on: [database, logger]
    description: "Create, read, update blog posts"
  
  - id: comments
    generate: true
    depends_on: [database, auth, logger]
    description: "Threaded comments with real-time sync"
  
  - id: analytics
    generate: true
    depends_on: [database]
    description: "Track views and engagement"

targets:
  - name: api
    components: [auth, database, logger, content_service, comments, analytics]
```

### Build Output
```
cadi build blog.build-spec.yaml
→ Resolves auth, database, logger via semantic search
→ Scaffolds content_service, comments, analytics
→ Generates imports automatically
→ Output: Ready-to-run project
```

**Reuse rate: 70% | Novel: 30% | Tokens: ~900**

---

## When to Adjust

### "No results for my search"
- Try broader query: "HTTP server" instead of "Express.js HTTP server"
- Fallback: Mark as `generate: true`, write it yourself

### "Found results but not quite right"
- Adjust in build spec: `version: "2.0"` to pin specific variant
- Or compose multiple chunks: `depends_on: [chunk1, chunk2]`

### "Too many dependencies"
- Review composition: Are you pulling in unused transitive deps?
- Refactor: Break into smaller targets (api, worker)

---

## Integration with Your Workflow

1. **Plan** → Decompose, search, draft build spec
2. **Write Spec** → CBS in 5-10 minutes
3. **Build** → Engine resolves + scaffolds (1-2 minutes)
4. **Develop** → Fill in novel logic with full context

Compared to:
1. Read codebase → 30 minutes (500+ tokens)
2. Copy-paste patterns → 20 minutes
3. Write glue code → 40 minutes
4. Debug wiring → 30 minutes

**CADI workflow: 15-20 minutes, 50-75% token savings**