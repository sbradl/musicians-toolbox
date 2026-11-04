# Testing strategy

How the project proves its behaviour: which kinds of test exist, how each is written, and how strictly the suite itself is held to account. Tasks: `test`, `coverage`, `acceptance`, `mutation`, `e2e`, `contract`.

## Research categories

- **Unit test framework** — the stack's runner and assertion library (xUnit, NUnit, Vitest, pytest).
- **Coverage** — line and branch coverage with a failing threshold (coverlet, Vitest's v8 provider, coverage.py).
- **Mutation testing** — mutates the code and checks the suite notices (Stryker.NET, StrykerJS, mutmut, PIT, cargo-mutants).
- **Property-based testing** — generated inputs against stated invariants (fast-check, CsCheck, FsCheck, Hypothesis).
- **Gherkin runner** — runs `.feature` files against step definitions in this stack's test runner (Reqnroll for .NET, Cucumber.js, playwright-bdd, QuickPickle for Vitest). Researched up front; used only when the user picks Gherkin.
- **End-to-end** — only for stacks with a UI or an HTTP API: drives the running app (Playwright).
- **Contract testing** — only for stacks that call or serve other services: consumer-driven contracts (Pact).

## Rounds

Both rounds per category. Rulesets here are mostly thresholds — strictest = the highest: coverage `100% lines and branches` `(Recommended)` then `90%`, `80%`; mutation score break threshold likewise. For end-to-end, the ruleset is the browser matrix: all engines the tool supports `(Recommended)`, then Chromium only.

## Questions

**Unit test naming** — single-select:

- **Behaviour sentence** `(Recommended)` — what a user of the code can do or expect, in their vocabulary: `customer earns a free coffee after ten stamps`.
- **`Method_Scenario_Expected`** — `AddStamp_TenthStamp_GrantsReward`.
- **Given/When/Then** — `Given_nine_stamps_When_stamped_Then_reward_granted`.

**Acceptance tests** — single-select:

- **`gherkin`** — feature files in Given/When/Then, readable by non-developers, bound to step definitions.
- **`coded`** — acceptance tests written in the test framework itself, named after the behaviour they prove.
- **`both`** — each for a different kind of behaviour.

`gherkin` or `both` → round 1 and 2 over the Gherkin runner candidates. `both` → ask which behaviour goes where: single-select with `Gherkin for behaviour agreed with stakeholders, coded for the rest` and `Gherkin for user-facing behaviour, coded for technical and integration behaviour`; the free-text answer states another split.

**Property-based testing** — when a library is picked, ask where it is required: single-select with `Pure functions with a stateable invariant (round-trips, ordering, idempotence)` `(Recommended)` and `Wherever a test would otherwise list example inputs`; the free-text answer states another rule.

## Record

```markdown
| Stack | Category | Tool | Ruleset |
| --- | --- | --- | --- |

Unit test names: <convention>
Acceptance tests: <gherkin | coded | both>; split: <rule>        (both)
Property-based tests required for: <rule>
Run: `test [filter]` · `coverage <file>` · `acceptance` · `MUTATE_FILES="<paths>" mutation` · `e2e` · `contract`
```

## Apply

- **`test`** runs the unit tests with the coverage threshold enforced; given an argument, only the tests it matches — a test file or a test name — without the threshold, which only a full run can meet.
- **`coverage`** takes one source file and reports its line and branch coverage under the full suite; a report, no threshold.
- **`acceptance`** — `gherkin` or `both`: install the runner and run every feature file; `coded`: run the acceptance tests by the test runner's filter for them (category, folder, or name pattern), and record that filter. Add no example tests or features; the rulebook records the convention, the next feature's tests follow it.
- **`mutation`** runs with the picked break threshold; with `MUTATE_FILES` set (space-separated paths), it mutates only those files. A full run can take long — report its duration with the score.
- **`e2e`** configures the tool to start the app itself where it can (Playwright's `webServer`); otherwise it needs a running app.
- **`contract`** — install the library and wire the task; contracts come from the first consumer test written against it. A contract broker is a server — note it in the rulebook if the user wants one; set none up.
- **`CLAUDE.md`** — write a `Testing` section into the project's `CLAUDE.md` (replace an existing one) naming the exact commands to run the full suite, one test file or test by name, coverage for one file, and mutation testing scoped by `MUTATE_FILES` — agents look for test commands there.
