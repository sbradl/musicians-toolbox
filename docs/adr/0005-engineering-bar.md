---
status: accepted
---

# Engineering bar

This ADR sets the policies. Concrete tools and rulesets are picked and wired with `/engineering-rules` once the first `Cargo.toml` and Gradle build exist, recorded in `.claude/engineering-rules.md`.

- **No production code without a failing test, everywhere, following the `/tdd` skill.** The user confirms the requirement and the seams under test first; then red → green one test at a time, in vertical slices. Refactoring and the independent check of tests and code against the requirement happen in `/code-review` (Standards + Spec, run by subagents).
- **Tests**: plain Rust tests on use cases (`application/`, fake ports) and presenters (`presentation/`, intent in → ViewModel out), named in the context's language, given/when/then in code. No Gherkin; the old apps' scenarios are a checklist, not files. Kotlin: Compose smoke tests (a ViewModel fixture renders, input sends the right intent, every interactive element has a label) and instrumented tests for Android driven adapters. No screenshot tests until a shared design system exists.
- **Coverage 100 %** in `contexts/`, `application/`, `presentation/`; exclusions only marked in code with a reason. No gate for `adapters/` and `ui/`.
- **Mutation testing** is slow and only finds missing tests or code beyond the requirements: on demand, and required before a release for Library reorganization and the Score format readers.
- **Fuzzing**: one `cargo-fuzz` target per format reader and per persisted format we read back; Linux only; 60 s per target in CI when `contexts/score/` changes, longer before releases. Every crash becomes a regression test. Seed corpus only from freely licensed files we make, never copyrighted songs.
- **Lints and formatting**: zero warnings, strictest rulesets, formatter defaults, `unsafe` forbidden outside FFI. On any finding the agent asks the user whether to fix the code or loosen the rule; a loosened rule is changed centrally with a reason, never silenced locally on the agent's own call.
- **No comments where code can say it.** Names, types and tests carry the meaning; a needed comment is a refactoring cue. Exceptions: the reason on a local lint allow, `// SAFETY:` on FFI `unsafe`, a pointer to an external spec. No TODOs, no commented-out code.
- **Offline**: no network access, no telemetry, no update check; no `INTERNET` permission on Android; network crates banned in the core layers. A feature needing network gets its own ADR.
- **No logging** in the apps. Panics crash with the platform's default behaviour. Performance is measured with benchmarks and platform profilers, not built-in output.
- **Performance**: benchmarks for critical paths (library scan and parse, ViewModel with 10k songs, filtering) run on demand; no CI gate (shared runners are too noisy). The ADR 0003 budget is checked on a low-end device before releases. A field regression gets a benchmark before its fix.
- **Dependencies**: licence allowlist MIT, Apache-2.0, BSD-2/3-Clause, ISC, Zlib, Unicode-3.0, MPL-2.0; anything else fails. Known advisories fail; exceptions need a reason and an expiry date. Third-party notices are generated per app and shipped. The agent asks the user before adding any dependency.
- **Accessibility**: every interactive element labelled (texts from Rust), full keyboard use on desktop, no screen-reader support (tabs are read visually, so the app is of no use without sight), layouts survive 200 % font scale, WCAG AA contrast, colour never the only carrier of information.
- **Manual checks** are automated wherever possible (keyboard-only and 200 % text as Compose tests, packages started in CI). Only what needs human judgement (real system pickers, launching a tab editor) goes into a wizard, `apps/<app>/release-checks.sh`, that does the setup and asks pass/fail; never a checklist to read. The device budget gets an automated benchmark. A bug found by hand gets an automated test where possible.
- **Not adopted**: habit-hooks (`check` plus asking on every finding covers it; revisit if agents repeat the same mistakes).
