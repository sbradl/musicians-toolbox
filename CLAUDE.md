# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Status

Walking skeleton of TabFinder on Linux desktop and Android. Architecture in `docs/adr/`: bridge (0002), UI/core line (0003), repo layout and build (0004), engineering bar (0005). Follow ADR 0004's layout when adding code.

Android follows the desktop: reuse the desktop's Rust adapters and wiring; Kotlin only for what the platform forces (SAF picker, grants, DocumentsContract).

## Commands

Toolchain via mise: `mise run setup` once (installs tools, enables `.githooks/`). Tasks live in `mise.toml`; the checks and their rulesets are recorded in `.claude/engineering-rules.md`.

- `mise run check`: lint + test + security, what CI runs on every push
- `mise run lint`: rustfmt, clippy, cargo-deny bans (layer rules), Opengrep own rules, ktlint (pre-commit hook)
- `mise run test`: `cargo test` + `./gradlew check` (Compose tests against the real native lib; pre-commit hook)
- `mise run security`: Opengrep packs, Trivy vulns + secrets, licence checks (pre-push hook)
- `mise run fmt`: format Rust and Kotlin
- `mise run run:tabfinder`: start the desktop app
- `mise run android:tabfinder`: install and start the app on the running emulator or device
- `mise run test:android`: instrumented Compose tests on the running emulator (not part of `check`)
- Android SDK: `ANDROID_HOME` (default `~/Android/Sdk`) needs NDK 28.2.13676358, platform 36 and build-tools; `check` verifies the APK has no `INTERNET` permission and 16 KB aligned native libraries
- Single Rust test file: `cargo test -p mt-tabfinder-presentation --test library_screen`
- Single Kotlin test: `./gradlew :apps:tabfinder:ui:shared:jvmTest --tests '*LibraryScreenTest*'`
- After changing Kotlin dependencies: `./gradlew dependencies --write-locks` per project (Trivy reads `gradle.lockfile`)

Gradle builds the FFI crate with `cargo zigbuild` against glibc 2.31 (`checkGlibcFloor` guards it) and generates UniFFI bindings into `build/` (`build-logic/src/main/kotlin/mt.uniffi.gradle.kts`).

## Working rules (ADR 0005)

- No production code without a failing test: use the `/tdd` skill. The user confirms the requirement and the seams first; review via `/code-review`.
- Any lint or compiler warning: ask the user whether to fix the code or loosen the rule. Never silence or loosen it yourself.
- Ask the user before adding any dependency.
- Don't write comments where code can say it; a needed comment is a cue to refactor.

## Readable code

Code and tests read like **prose**: a human coming from C# understands every line without docs, while the code stays idiomatic Rust and Kotlin.

- Name things in the context glossary's words (`docs/agents/domain.md`). The domain speaks only Library and Song; files, paths and formats stay inside the adapter that touches them.
- Domain types over strings and tuples (`LibraryLocation`, `Song`).
- One level of abstraction per function: high-level functions read as a sequence of named steps; the details live in small helpers below them.
- Functions stay under 20 lines (clippy stops Rust at 25).
- Plain control flow: `match`, `if`/`else`, `let … else` and early returns in place of `map_or_else`-style closure chains.
- Return values in place of output parameters.
- A blank line before every `return`, implicit return (Rust tail expression, Kotlin value-returning lambda's last expression) and `if` statement, unless it opens its block.
- Tests tell the musician's story in the arrange / act / assert shape: test names state the behaviour, and a harness speaks the user's verbs (`tabfinder.musician_picks_folder("/music")`, `tabfinder.assert_status("1 Song")`). Fakes and plumbing live in `tests/support/`. Test data uses invented Song titles, never real songs or artists.
- When a lint makes code noisy, extract a method first: test assertions go into `#[track_caller]` assert helpers in place of `expect`. Loosening stays the user's call (working rules above).
- `redundant_pub_crate` and `unreachable_pub` conflict in private modules: declare a type shared across private modules in the crate root (like `Texts` in `presentation/src/lib.rs`), and expose test helpers through `pub mod support;` with `#[must_use]`.

## Agent skills

### Issue tracker

GitHub Issues on `sbradl/musicians-toolbox` via `gh`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five labels (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Multi-context: root `GLOSSARY-MAP.md` lists the bounded contexts; each has a `GLOSSARY.md` (+ `adr/`): `contexts/<context>/` once its crate exists, else `docs/contexts/<context>/`. System-wide decisions in `docs/adr/`. See `docs/agents/domain.md`.
