# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Status

Walking skeleton of TabFinder on Linux desktop. Architecture in `docs/adr/`: bridge (0002), UI/core line (0003), repo layout and build (0004), engineering bar (0005). Follow ADR 0004's layout when adding code.

## Commands

Toolchain via mise: `mise run setup` once (installs tools, enables `.githooks/`). Tasks live in `mise.toml`; the checks and their rulesets are recorded in `.claude/engineering-rules.md`.

- `mise run check`: lint + test + security, what CI runs on every push
- `mise run lint`: rustfmt, clippy, cargo-deny bans (layer rules), Opengrep own rules, ktlint (pre-commit hook)
- `mise run test`: `cargo test` + `./gradlew check` (Compose tests against the real native lib; pre-commit hook)
- `mise run security`: Opengrep packs, Trivy vulns + secrets, licence checks, SBOM (pre-push hook)
- `mise run fmt`: format Rust and Kotlin
- `mise run run:tabfinder`: start the desktop app
- Single Rust test file: `cargo test -p mt-tabfinder-presentation --test library_screen`
- Single Kotlin test: `./gradlew :apps:tabfinder:ui:shared:jvmTest --tests '*LibraryScreenTest*'`
- After changing Kotlin dependencies: `./gradlew dependencies --write-locks` per project (Trivy reads `gradle.lockfile`)

Gradle builds the FFI crate with Cargo and generates UniFFI bindings into `build/` (`build-logic/src/main/kotlin/mt.uniffi.gradle.kts`).

## Working rules (ADR 0005)

- No production code without a failing test: use the `/tdd` skill. The user confirms the requirement and the seams first; review via `/code-review`.
- Any lint or compiler warning: ask the user whether to fix the code or loosen the rule. Never silence or loosen it yourself.
- Ask the user before adding any dependency.
- Don't write comments where code can say it; a needed comment is a cue to refactor.

## Agent skills

### Issue tracker

GitHub Issues on `sbradl/musicians-toolbox` via `gh`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five labels (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Multi-context: root `GLOSSARY-MAP.md` lists the bounded contexts; each has a `GLOSSARY.md` (+ `adr/`): `contexts/<context>/` once its crate exists, else `docs/contexts/<context>/`. System-wide decisions in `docs/adr/`. See `docs/agents/domain.md`.
