# Conventional Commits

Subject: `<type>(<scope>)<!>: <description>` — machine-readable, so changelogs and version bumps can be derived from history.

- **type** — the kind of change:
  - `feat` — a new capability for the user
  - `fix` — a bug fix
  - `docs` — documentation only
  - `refactor` — restructuring with behaviour unchanged
  - `perf` — a performance improvement
  - `test` — adding or correcting tests
  - `build` — build system or dependencies
  - `ci` — CI configuration
  - `style` — formatting, whitespace
  - `chore` — anything else that touches no source or test behaviour
  - `revert` — reverts an earlier commit
- **scope** — optional, in parentheses: the area touched (module, package, component). Reuse scopes already in `git log`.
- **`!`** — before the colon when the change breaks a public interface. Also add a `BREAKING CHANGE: <what breaks and how to migrate>` trailer.
- **description** — imperative mood, lowercase first letter, no trailing period; the whole subject within 72 characters.

```
feat(auth): add password reset via email

Users locked out had no self-service path and filed support tickets.

Refs: ABC-123
```
