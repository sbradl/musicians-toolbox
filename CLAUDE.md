# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Status

Walking skeleton of TabFinder on Linux desktop. Architecture in `docs/adr/`: bridge (0002), UI/core line (0003), repo layout and build (0004), engineering bar (0005). Follow ADR 0004's layout when adding code.

## Commands

Toolchain via mise (`mise install` once). All tasks live in `mise.toml`; CI runs `check`.

- `mise run check`: lint + test, what CI runs on every push
- `mise run lint`: `cargo fmt --check`, clippy (`-D warnings`), ktlint
- `mise run fmt`: format Rust and Kotlin
- `mise run test`: `cargo test --workspace` + `./gradlew check` (Compose tests against the real native lib)
- `mise run run:tabfinder`: start the desktop app
- Single Rust test file: `cargo test -p mt-tabfinder-presentation --test library_screen`
- Single Kotlin test: `./gradlew :apps:tabfinder:ui:shared:jvmTest --tests '*LibraryScreenTest*'`

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
