# Logging and observability

Conventions for what the running app tells its operators: structured logs, log levels, correlation, traces, metrics. Conventions, not checks — adopting a library changes application code, which is a feature to build, not setup to apply. No task.

## Research categories

- **Structured logging** — a logging library emitting structured events (Serilog, Microsoft.Extensions.Logging, pino, structlog).
- **Tracing and metrics** — OpenTelemetry's SDK and auto-instrumentation for this stack.
- **Log linting** — lint rules that enforce the conventions: no raw console or print output, no string-interpolated log messages (ESLint `no-console`, CA2254, Ruff `T20` and `G` rules). Overlaps the `lint` task's linter; the rules join that linter's config.

## Rounds

Round 1 for each category; round 2 for log linting only.

## Questions

Single-select each:

- **Log format** — structured JSON `(Recommended)`; human-readable text locally and JSON in deployed environments; plain text.
- **Sensitive data** — never log personal data or secrets, enforced by redaction in the logger `(Recommended)`; never log them, by review only.
- **Correlation** — every log line carries the trace and span id `(Recommended)`; a request id only; none.
- **Levels** — one question listing what each level means; propose: `Error` = needs a human, `Warning` = degraded but self-healing, `Information` = business events, `Debug` = developer detail, off in production `(Recommended)`.

## Record

```markdown
Logging library: <stack>: <library>
Tracing and metrics: <stack>: <OpenTelemetry packages | none>
Log format: <answer>
Sensitive data: <answer>
Correlation: <answer>
Levels: <level> = <meaning>, one line each
```

## Apply

Add the log-linting rules to the `lint` task's linter config. Install no logging or tracing library: record the picks, and the rulebook tells the next feature that touches logging what to use.
