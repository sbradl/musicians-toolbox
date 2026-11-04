# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Status

The repo is new. It holds only a README with the project name ("musicians-toolbox"). There's no code, build system, tests or tooling yet. Update this file with commands and architecture once they exist.

## Agent skills

### Issue tracker

GitHub Issues on `sbradl/musicians-toolbox` via `gh`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five labels (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Multi-context: root `GLOSSARY-MAP.md` lists the bounded contexts; each has `docs/contexts/<context>/GLOSSARY.md` (+ `adr/`). System-wide decisions in `docs/adr/`. See `docs/agents/domain.md`.
