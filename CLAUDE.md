# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Status

No code yet. Architecture is decided in `docs/adr/`: bridge (0002), UI/core line (0003), repo layout and build (0004). Follow ADR 0004's layout when adding code. Add commands here once they exist.

## Agent skills

### Issue tracker

GitHub Issues on `sbradl/musicians-toolbox` via `gh`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five labels (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Multi-context: root `GLOSSARY-MAP.md` lists the bounded contexts; each has `docs/contexts/<context>/GLOSSARY.md` (+ `adr/`). System-wide decisions in `docs/adr/`. See `docs/agents/domain.md`.
