# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Before exploring, read these

This repo is **multi-context**.

- **`GLOSSARY-MAP.md`** at the repo root: lists the bounded contexts, how they relate, and which apps compose them. Read the `GLOSSARY.md` of each context relevant to the topic.
- **`docs/adr/`**: system-wide decisions. Also check the context's `adr/` (next to its `GLOSSARY.md`) for context-scoped decisions.

If a context's `adr/` doesn't exist, proceed silently; the `/domain-modeling` skill creates files lazily.

## File structure

```
/
├── GLOSSARY-MAP.md
├── contexts/
│   └── tab-library/                   ← crate mt-tab-library
│       ├── GLOSSARY.md
│       └── adr/                       ← context-specific decisions
└── docs/
    ├── adr/                           ← system-wide decisions
    └── contexts/
        └── score/                     ← no crate yet
            └── GLOSSARY.md
```

Context docs live under `docs/contexts/` until the context's crate exists; with that crate's first commit they move to `contexts/<context>/GLOSSARY.md` and `contexts/<context>/adr/`, with `GLOSSARY-MAP.md` and this file updated (ADR 0004).

A word may mean different things in different contexts (e.g. Beat in Drum Pattern vs Onset in Score). Use the term of the context you're working in.

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a test name), use the term as defined in that context's `GLOSSARY.md`. Don't drift to synonyms the glossary explicitly avoids.

If the concept you need isn't in the glossary yet, that's a signal: either you're inventing language the project doesn't use (reconsider) or there's a real gap (note it for `/domain-modeling`).

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than silently overriding:

> _Contradicts ADR-0007 (event-sourced orders), but worth reopening because…_
