# Enforcement

Where the other areas' checks run — git hooks, CI, both — and how dependencies stay current. Runs last: it wires the tasks the other areas created. Task: `check`.

## Research categories

- **Git hook manager** — runs tasks on commit or push, versioned in the repo (Lefthook, Husky, pre-commit). Include the no-tool option: a tracked `.githooks/` directory enabled with `git config core.hooksPath`.
- **Dependency updates** — opens pull requests for new dependency versions (Renovate, Dependabot).

## Rounds

Both rounds. For dependency updates, the ruleset is the update policy: Renovate's strictest preset (`config:best-practices` — pinned versions and digests) `(Recommended)`, then `config:recommended`.

## Questions

**Where checks run** — per task the other areas created, one multi-select question (four tasks per call): `pre-commit hook`, `pre-push hook`, `CI`. Recommend fast, file-scoped checks (`lint` on staged files, secret scanning) for `pre-commit`, `test` for `pre-push`, and every task for `CI`. Tasks that need a running app (`dast`, `e2e`) and slow ones (`mutation`) recommend `CI` only.

**CI platform** — detect it (`.github/workflows/`, `.gitlab-ci.yml`, `azure-pipelines.yml`) and confirm; nothing detected → single-select: GitHub Actions, GitLab CI, Azure Pipelines.

**Code scanning upload** — GitHub Actions only, when a picked tool emits SARIF: ask whether CI uploads SARIF to GitHub code scanning. Say in the question that it is free for public repos and needs GitHub Advanced Security on private ones.

## Record

```markdown
| Task | pre-commit | pre-push | CI |
| --- | --- | --- | --- |

Hook manager: <tool>
CI: <platform> → `<workflow file>`; SARIF upload: <yes | no>
Dependency updates: <tool>, <preset>
Run everything: `<check task command>`
```

## Apply

- **`check`** runs every other area's task except those needing a running app; it is what CI runs and what a developer runs before pushing.
- **Hooks** — install the hook manager, configure the picked tasks, add enabling the hooks to the repo's setup task (`mise run setup` or the stack's equivalent), and enable them in this clone.
- **CI** — one workflow running each CI task as its own step, so a failure names its check; install tools the same way the developer setup does (mise's action when the repo uses mise).
- **Dependency updates** — write the tool's config with the picked preset, grouping updates per stack.
