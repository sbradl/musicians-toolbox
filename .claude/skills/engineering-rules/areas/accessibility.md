# Accessibility

Checks that a UI is usable with assistive technology and meets WCAG. UI stacks only. Task: `a11y`.

## Research categories

- **Static accessibility linting** — accessibility rules on component source (eslint-plugin-jsx-a11y, eslint-plugin-vuejs-accessibility, angular-eslint template rules). Overlaps the `lint` task's linter; the rules join that linter's config.
- **Runtime accessibility testing** — rendered pages checked against WCAG (axe-core with its Playwright or Vitest integration, Pa11y).

## Rounds

Both rounds. For runtime testing, the ruleset is the WCAG conformance target: AAA plus axe best practices `(Recommended)`, then AA, then A.

## Record

```markdown
| Stack | Category | Tool | Ruleset |
| --- | --- | --- | --- |

WCAG target: <level>
Pages checked: <how the task finds them>
Run: `<a11y task command>`
```

## Apply

Static rules go into the existing linter config, so `lint` enforces them. The `a11y` task checks every route the app exposes — read them from the router or a sitemap, and record how — starting the app itself where the tool can, as `e2e` does.
