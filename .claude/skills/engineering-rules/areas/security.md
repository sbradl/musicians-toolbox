# Security analysis

Checks for insecure code, vulnerable or non-compliant dependencies, misconfigured infrastructure, leaked secrets, and attackable running apps. Task: `security`; `dast` for the running-app scan.

## Research categories

- **SAST** — insecure code patterns. Include Opengrep and the rule packs available to it.
- **Dependency vulnerabilities** — known CVEs in direct and transitive dependencies. Include Trivy (`trivy fs`) and the stack's native audit (`npm audit`, `dotnet list package --vulnerable`).
- **License compliance** — dependency licenses checked against an allow-list. Include Trivy's license scanner.
- **SBOM** — a software bill of materials as a build artifact. Include Syft and Trivy.
- **Infrastructure misconfiguration** — infrastructure stacks only: Dockerfiles, Compose, Terraform, Helm, Kubernetes. Include Trivy (`trivy config`).
- **Container image scanning** — infrastructure stacks with a Dockerfile: vulnerabilities in the built image's OS packages and layers. Include Trivy (`trivy image`) and Grype.
- **Secret scanning** — credentials in the working tree and the git history. Include Gitleaks, TruffleHog, and Trivy's secret scanner.
- **DAST** — only for stacks serving HTTP: attacks the running app from outside. Include ZAP.

## Rounds

Both rounds. Rulesets here are mostly rule packs (strictest = the broadest set) and severity thresholds (strictest = fail on the lowest severity). Two categories rank differently:

- **License compliance** — the allow-list: permissive only (MIT, Apache-2.0, BSD, ISC) `(Recommended)`, then adding weak copyleft (LGPL, MPL), then adding strong copyleft (GPL, AGPL).
- **SBOM** — not a check, so round 2 picks the format: CycloneDX `(Recommended)` or SPDX.

## Record

```markdown
| Stack | Category | Tool | Ruleset |
| --- | --- | --- | --- |

License allow-list: <SPDX ids>
SBOM: <format> → `<output path>`
Run: `<security task command>`; `<dast task command>` against a running app
```

## Apply

- **Secret scanning** runs once over the full git history as well as the working tree. A secret found anywhere → tell the user at once, naming file and commit: the secret is compromised and must be rotated; rewriting history does not un-leak it.
- **SBOM** joins the `security` task as an artifact step: it writes the file and never fails the task.
- **Container image scanning** builds the image in the task, then scans it.
- **DAST** gets its own `dast` task taking the target URL, because it needs a running app. Its strictest ruleset (ZAP's full active scan) attacks the target — the task must refuse any URL but a local or staging one the user names.
