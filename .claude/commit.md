# Commit convention

style: conventional
body: when-non-obvious
granularity: atomic
issue-refs: when-available
attribution: keep

## Changelog

path: <app-dir>/CHANGELOG.json
tag-format: <app>/v{major}.{minor}.{patch}

### end-users

who: Musicians using one of the repo's apps or CLIs (TabFinder and its CLIs, Drum Practice, later apps) on Windows, Linux or Android.
notices: new or changed screens, controls, filters, file formats read, CLI flags and output, platform support, fixes to bugs they could hit in a released version.
never: refactors, tests, CI, build tooling, docs-only changes, internal shared crates/modules with no visible effect, wayfinder/research material.
categories: added, changed, fixed, removed, breaking
languages: en, de, fr

## House rules

- One changelog per app at `<app-dir>/CHANGELOG.json`; a change shared by several apps gets an entry in each app's changelog it visibly affects. No app directories exist yet → no changelog entries.
- Release tags are per app: `<app>/vMAJOR.MINOR.PATCH`. Rollover and releases work on one app's changelog and that app's tags only.
- Scope = the app or bounded context touched (e.g. `feat(tabfinder): ...`); repo-wide changes take no scope.
