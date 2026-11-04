---
name: engineering-rules
description: Set up a project's engineering rules — static analysis, security, testing strategy, enforcement, accessibility, performance budgets, observability — by researching free tools per stack and letting the user pick tools and rulesets.
disable-model-invocation: true
---

# Engineering rules

Sets up the checks a project's code must pass and records them in `<repo-root>/.claude/engineering-rules.md` — the **rulebook** agents and skills read to know which checks gate a change. Each **area** is a file in `<skill-dir>/areas/`; its `#` heading is its name, its first paragraph says what it covers.

Resolve `<skill-dir>` to this skill folder's absolute path.

## 1. Load the rulebook

Exists → read it, then ask (multi-select) which of its areas to revisit and whether to add areas it lacks; everything else stays as recorded. Missing → start fresh.

## 2. Pin the stacks

A **stack** is one language and its toolchain in one part of the repo: language and version, framework, test runner, package manager, path, and whether it has a UI or serves HTTP. Detect them from manifests — `*.sln`/`*.csproj`/`global.json`, `package.json` with its lockfile and `tsconfig.json` (test runner from `devDependencies`), `pyproject.toml`, `go.mod`, `Cargo.toml`, `pom.xml`/`build.gradle*`. Infrastructure the repo ships — `Dockerfile`, Compose files, `*.tf`, Helm charts, Kubernetes manifests — is a stack too; the security area scans it.

Ask one multi-select question: one option per detected stack, labelled like `dotnet 9 · xunit · src/Api`, the description naming the evidence file. The free-text answer adds or corrects a stack. Nothing detected → ask for the stacks in plain chat.

## 3. Pin the task runner

Every check runs as a **task** — one per tool, plus one per area that runs all of the area's tool tasks.

- The repo uses mise (`mise.toml`, `.mise.toml`, `.config/mise.toml`, `mise/config.toml`, or `.tool-versions`) → mise.
- Otherwise ask one single-select question: **Set up mise** `(Recommended)` — one runner for every stack, pins standalone tools like Trivy and Gitleaks, gives every developer the same versions — or **Use the stacks' own runners** (`package.json` scripts, a `Makefile`).

## 4. Pick the areas

Fresh start → multi-select, one option per file in `<skill-dir>/areas/`: label = its heading, description = its first paragraph; four options per question, as many questions as the areas need. Revisit → step 1's answer is the pick.

## 5. Research

One message, one `Agent` call (`general-purpose`) per picked stack × picked area that has research categories for that stack — all in parallel. Each prompt carries:

- "Read `<skill-dir>/research.md` and follow it."
- the stack, with every field from step 2;
- the area's name and its research categories, verbatim from the area file.

The reports are the tool catalogue for the rounds; present tools only from them.

## 6. Run each area

Areas run in the order picked — except an area whose file says it runs last — each starting once its reports are in. Read `<skill-dir>/areas/<area>.md` and follow it. Area files refer to the **two rounds**:

**Round 1 — tools.** Per stack × research category, one multi-select question; the stack goes in the header. Options = that category's candidates in report order, at most four; the rest are named in the question text and reachable through the free-text answer. Each description says what the tool checks and its license, plus "already configured" when the report found its config in the repo. A tool covering several stacks (Trivy, Gitleaks, Opengrep) is asked once, naming every stack it covers. Pack four questions per call.

**Round 2 — rulesets.** For each picked tool whose report lists two or more rulesets — presets, rule packs, severity thresholds, score thresholds: one single-select question, strictest first and marked `(Recommended)`, each option's `preview` stating what it adds over the next-looser ruleset. An area file may rank a category differently; it says so. Tools with a single ruleset skip this round. Pack four questions per call.

## 7. Record

Write the rulebook; on revisit, replace only the revisited areas' sections.

```markdown
# Engineering rules

Task runner: <mise | the stacks' own runners>

## Stacks

- **<stack key>** — <language and version> · <framework> · <test runner> · <package manager> · `<path>`

## <Area heading>

<the area file's record section, filled in>
```

## 8. Apply

**Mise** — chosen in step 3 but not yet in the repo → create `mise.toml` with the stacks' runtimes in `[tools]` and a `setup` task running `mise install`, then run `mise trust` and `mise install`.

For each picked tool, from its report — the area file's Apply section overrides these steps where it differs:

1. **Install** — dev dependency of its stack; a standalone binary goes in mise's `[tools]`, pinned to the reported version, or — without mise — gets its install command recorded in the rulebook.
2. **Configure** — write its config with the picked ruleset.
3. **Wire** — a task per tool, named `<area task>:<tool>` (`lint:eslint`, `security:trivy`), and the area's task depending on all of them. An area naming several tasks (testing: `test`, `mutation`, …) puts each tool under the task it serves. In mise: `[tasks."lint:eslint"]` entries and `[tasks.lint] depends = [...]`. Without mise: the same names in the stack's runner. Each task checks without modifying files and exits non-zero on any finding.
4. **Run** each area task once. Findings in existing code are expected under the strictest rulesets: report the count per tool and fix nothing. A tool with baseline support → offer to record a baseline so only new findings fail. A task that needs a running app it cannot start itself → report it as not run, and what it needs.

Leave everything uncommitted for the user to review.

## Done

The rulebook names the task runner, lists the confirmed stacks, and has a section per picked area; every picked tool is installed, configured with its picked ruleset, wired as its own task under its area's task, and run once with its finding count reported — or reported as not run with what it needs.
