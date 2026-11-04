# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Status

No code yet. Architecture is decided in `docs/adr/`: bridge (0002), UI/core line (0003), repo layout and build (0004), engineering bar (0005). Follow ADR 0004's layout when adding code. Add commands here once they exist.

## Working rules (ADR 0005)

- No production code without failing tests. Order: user confirms the requirement → write failing tests for it → a fresh subagent reviews the tests against the requirement only → write code until green.
- Any lint or compiler warning: ask the user whether to fix the code or loosen the rule. Never silence or loosen it yourself.
- Ask the user before adding any dependency.
- Don't write comments where code can say it; a needed comment is a cue to refactor.

## Agent skills

### Issue tracker

GitHub Issues on `sbradl/musicians-toolbox` via `gh`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five labels (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Multi-context: root `GLOSSARY-MAP.md` lists the bounded contexts; each has `docs/contexts/<context>/GLOSSARY.md` (+ `adr/`). System-wide decisions in `docs/adr/`. See `docs/agents/domain.md`.
