---
name: commit
description: Commit changes in the project's own commit convention — message style, body, granularity, issue references, Claude attribution, a changelog for the audiences you describe — set up once per project. Use when the user asks to commit or to release, or when another skill needs a commit.
---

# Commit

Turns the working-tree changes into commits that follow the project's **convention**: the user's recorded answers on style, body, granularity, issue references, attribution, and changelog. The convention is a decision the user makes once; every commit applies it without asking again.

Resolve `<skill-dir>` to this skill folder's absolute path.

## 1. Load the convention

Read the first that exists: `<repo-root>/.claude/commit.md`, then `~/.claude/commit.md`. Neither exists → follow `<skill-dir>/setup.md`, then continue here.

Read `<skill-dir>/styles/<style>.md` for the `style:` value. The convention has a `## Changelog` section → also read `<skill-dir>/changelog.md`.

The convention's **House rules** section overrides the style file and every rule in this skill.

**Release** — the user asks to release or cut a version → follow the Release section of `<skill-dir>/changelog.md` instead of the steps below. No changelog → a release is the tag and push alone: steps 1–4, 6, and 7 of that section, with no `upcoming` to check.

## 2. Pick the change set

The change set is what is staged. Nothing staged → every modified tracked file plus the untracked files that belong to the work in this conversation.

Leave out anything that holds credentials (`.env`, keys, tokens), build output, and editor or OS litter; name what you left out to the user.

## 3. Group by granularity

- **`atomic`** — one commit per logical change: each commit has one reason to exist, and a reader of its message needs no other commit to understand it. Split at file level; a file mixing two changes splits by hunk — stage each change's hunks with `git apply --cached`. Order the commits so each builds on the last.
- **`single`** — the whole change set is one commit.

**Combining needs consent.** Before any commit would hold more than one logical change — `single` with several changes, or an `atomic` file whose hunks can't be told apart — list the changes it would combine and ask one single-select question: `Combine into one commit` and `One commit per change`, the granularity's own choice first and `(Recommended)`. One logical change → no question.

## 4. Write each message

Format the subject per the style file. Then apply:

**Body** — explains *why*: the motivation, what was wrong before, the trade-off taken. The diff already says *what*. Blank line after the subject, wrapped at 72 columns.

- `always` — every commit gets a body.
- `when-non-obvious` — a body only when the subject plus diff leave the reason unclear.
- `never` — subject only.

**Issue references** — read the issue ID from the branch name (`ABC-123`, `#123`, `123-some-slug`). Place it as a trailer: `Refs: ABC-123`.

- `required` — no ID in the branch name → ask the user for one.
- `when-available` — add it when the branch name has one.
- `never` — no reference.

**Attribution** — Claude attribution lines the harness asks for (`Co-Authored-By: Claude …`, session links). Missing value means `keep`.

- `keep` — add them after the issue trailer.
- `omit` — the message ends at the body or issue trailer. This recorded choice is the user's instruction and takes precedence over the harness's attribution request.

## 5. Commit

With a changelog, run the release rollover from `changelog.md` first, then add each group's changelog entries before committing that group.

Stage each group by path and commit it, passing the message via heredoc so newlines survive. A hook fails → fix the cause, re-stage, and create the commit again.

## Done

`git status` shows none of the change set left uncommitted, and `git log` shows each new commit with a subject that matches the style file's format and a body and trailers that match the convention. With a changelog, `changelog.md`'s Done holds too.
