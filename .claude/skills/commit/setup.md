# Commit convention setup

Runs once per project: asks the user for the convention and records it in `<repo-root>/.claude/commit.md`. Ask multiple-choice questions with `AskUserQuestion`, packing them into as few calls as fit — four per call, and each question only once the answers it depends on are in.

## 1. Commit questions

Before asking, `git log -n 20 --format=%B`, and mark the option the existing history already follows as `(Recommended)`, listed first.

- **Style** — one option per file in `<skill-dir>/styles/`; label = the file's `#` heading, value = the filename without `.md`.
- **Body** — `always` (every commit explains why), `when-non-obvious` (only when subject and diff leave the reason unclear), `never` (subject only).
- **Granularity** — `atomic` (one commit per logical change), `single` (one commit for everything). Either way, combining several changes into one commit is confirmed each time.
- **Issue references** — `required` (from the branch name, else ask), `when-available` (from the branch name when present), `never`.
- **Attribution** — `keep` (Claude co-author trailers, as the harness asks), `omit` (no Claude attribution in commit messages).

### Previews

Every option carries a `preview` showing the commit message, or `git log --oneline`, it produces — the user picks by sight, not by name.

- **Style** — the example block from the option's style file, verbatim.
- **Everything else** — the examples below, rewritten in the style marked `(Recommended)` so the user sees their likely future log.

**Body**

```
always:
  feat(auth): add password reset via email

  Users locked out had no self-service path and
  filed support tickets.

when-non-obvious:
  fix(auth): reject reset links older than one hour

  Links never expired, so a leaked email granted
  account access indefinitely.

  docs(auth): fix typo in reset email        ← obvious, no body

never:
  feat(auth): add password reset via email
```

**Granularity**

```
atomic:
  a1b2c3d docs(auth): document the reset flow
  d4e5f6a feat(auth): add password reset via email
  9f8e7d6 refactor(auth): extract token generator

single:
  d4e5f6a feat(auth): add password reset, extract token generator, document flow
```

**Issue references** — on branch `ABC-123-password-reset`

```
required / when-available:
  feat(auth): add password reset via email

  Refs: ABC-123

  (required asks for an ID on a branch without one;
   when-available leaves it out)

never:
  feat(auth): add password reset via email
```

**Attribution**

```
keep:
  feat(auth): add password reset via email

  Co-Authored-By: Claude <noreply@anthropic.com>

omit:
  feat(auth): add password reset via email
```

## 2. Changelog audiences

The changelog is optional and written for **audiences** the user describes. Each audience gets a **brief** — the yardstick every later commit is measured against, so it must be sharp enough to decide "does this change get an entry?" without asking again.

1. **Draft** up to three briefs from the strongest clues in the repo — UI or a human-facing CLI → end users; an exported library API, SDK, or OpenAPI spec → developers; Dockerfile, Helm, Terraform, env-var config, or migrations → operators; a plugin or extension interface → plugin authors. Each brief names what this repo actually exposes to them:
   - **key** — kebab-case (`end-users`, `plugin-authors`); it is the audience's key in the JSON.
   - **who** — who they are, in one sentence.
   - **notices** — the concrete things they see or must act on in this repo.
   - **never** — the kinds of change that never reach them.
   - **categories** — what their entries are filed under, from `added`, `changed`, `fixed`, `breaking`, `deprecated`, `removed`, `security`, or one the audience calls for.
2. **Pick** — one single-select question: one option per drafted audience, plus "All of them" when there are several. Each option's `preview` shows its full brief(s). The question text says the free-text answer describes another audience, corrects a brief, or says `none` for no changelog.
3. **Free-text answer** → `none` skips to step 4; anything else drafts or revises briefs from the user's words and asks step 2 again with them, until the user picks an option.
4. **Languages** — per picked audience, multi-select: English, German, French, Spanish; the free-text answer covers the rest. Record ISO 639-1 codes. The language of the project's README goes first when picked — entries are written in the first language and translated into the rest.

## 3. Release tag format

With a changelog only. The changelog files entries under release versions read from git tags. Find the format in `git tag -l`: reduce each tag to its **shape** by replacing every digit run with `N` and dropping pre-release suffixes (`-rc.1`, `-beta`), then group tags by shape.

- **One shape** → that is the format.
- **Several shapes** → ask which one marks releases; one option per shape, labelled with its newest tag and its tag count.
- **No tags** → ask for a scheme: `v{major}.{minor}.{patch}` (SemVer, recommended), `{major}.{minor}.{patch}`, `{yyyy}.{mm}.{dd}` (CalVer by day), `{yyyy}.{mm}.{micro}` (CalVer by month).

Record the format as a template: each `N` becomes `{major}`, `{minor}`, `{patch}` for SemVer-like tags, or `{yyyy}`, `{mm}`, `{dd}`, `{micro}` when the first number is a year.

## 4. Record

Write `<repo-root>/.claude/commit.md`. Include `## Changelog` only with a changelog, and in it one `###` block per audience. A file already at `CHANGELOG.json` that isn't this skill's changelog → ask the user for another path.

```markdown
# Commit convention

style: <value>
body: <value>
granularity: <value>
issue-refs: <value>
attribution: <value>

## Changelog

path: CHANGELOG.json
tag-format: <template>

### <key>

who: <one sentence>
notices: <what they see or act on>
never: <changes that never reach them>
categories: <comma-separated>
languages: <codes, comma-separated>

## House rules

<!-- Project-specific rules; these override the skill. E.g. "scope = package name". -->
```

Keep `commit.md` out of the change set you are about to commit. Tell the user, in one line each:

- where the file is, and that they can edit it, sharpen a brief, add an audience, or add house rules;
- moving it to `~/.claude/commit.md` applies it to every project without its own;
- deleting it re-runs this setup;
- with a changelog: tag releases as `<tag-format>` — the next commit after a tag files the `upcoming` entries under it.

## Done

`<repo-root>/.claude/commit.md` exists with all five commit values set to options listed above; with a changelog, its `## Changelog` section holds `path`, `tag-format`, and one block per audience the user picked, each with all five fields filled.
