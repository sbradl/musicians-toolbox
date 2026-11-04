# Static analysis and linting

Checks that read the code without running it: linters, formatters, type checkers, analyzers, architecture and dead-code checks. Task: `lint`.

## Research categories

- **Linter** — bug patterns, best practices, style rules (ESLint with typescript-eslint, Roslyn analyzers, Ruff).
- **Formatter** — one canonical layout (Prettier, Biome, `dotnet format`, Ruff format).
- **Type strictness** — the compiler's or type checker's strict modes (`tsconfig` `strict`, `<Nullable>enable</Nullable>`, `TreatWarningsAsErrors`, `mypy --strict`).
- **Complexity and duplication** — complexity limits per function, copy-paste detection.
- **Architecture rules** — allowed dependencies between modules and layers, no cycles (dependency-cruiser, ArchUnitNET, NetArchTest, import-linter).
- **Dead code and unused dependencies** — unreferenced exports, files, and packages (knip, deptry).
- **API spec linting** — only when the stack ships an OpenAPI or AsyncAPI spec (Spectral).

## Rounds

Both rounds. Tools that overlap across categories (Biome against ESLint plus Prettier) say so in their option description.

## Record

```markdown
| Stack | Category | Tool | Ruleset |
| --- | --- | --- | --- |

Layers: <layer> → <layers it may depend on>, one line each   (architecture rules)
Run: `<lint task command>`
```

## Apply

- **Formatters** run in check mode only; reformatting the whole repo is the user's call.
- **Type strictness** lives in the project files (`tsconfig.json`, `Directory.Build.props`), not a separate tool config.
- **Architecture rules** start with the generic ones — no cycles, no orphaned modules. Layer rules encode the user's intent, so propose the layering you read from the directory structure and ask the user to confirm or correct it in plain chat before writing them; a tool that expresses rules as tests (ArchUnitNET) gets them as one test file.
