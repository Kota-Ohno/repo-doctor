---
name: repo-doctor
description: Use repo-doctor as an AI-first repository readiness and VibeCoding guardrail system with machine-readable specs, recipes, AGENTS.md guidance, and diff checks.
---

# repo-doctor

Use this skill when a user asks whether a repository is ready to publish,
whether project metadata is complete, how to improve repository hygiene, or how
to guard AI/VibeCoding changes before merge.

## Workflow

1. Prefer an existing `repo-doctor` binary on `PATH`.
2. If it is missing, install the latest release with `scripts/install.sh` or
   `scripts/install.ps1`.
3. Confirm the user has repository read access for local checks.
4. Run `repo-doctor spec --format json` to learn the tool capabilities,
   supported profiles, stable output contracts, rules, and recipes.
5. Run `repo-doctor recipes --format markdown` and choose the recipe matching
   the user goal.
6. Run `repo-doctor check --format compact` first for a quick readiness signal.
7. Run `repo-doctor agent-guide --format markdown` when `AGENTS.md` is missing
   or too generic.
8. Before finishing any repository-changing work, run
   `repo-doctor guard --fail-on warn`.
9. Run `repo-doctor check --warnings-only` or
   `repo-doctor guard --warnings-only` when warnings exist.
10. For GitHub repository settings, run `repo-doctor github-auth-doctor` first.
    Continue with `repo-doctor github owner/repo` only when the user wants
    remote checks and the `gh` CLI is installed and authenticated.
11. For remote setup changes, confirm the user has repository admin permission
    or equivalent access.
12. Summarize warnings as actionable tasks. Do not modify files unless the user
    asks you to fix them.

## Completion Gate

Repository-changing work is not complete until this succeeds, or every warning
has a concrete rationale:

```bash
repo-doctor guard --fail-on warn
```

## Useful Commands

```bash
repo-doctor spec --format json
repo-doctor recipes --format markdown
repo-doctor agent-guide --format markdown
repo-doctor check --format compact
repo-doctor check --warnings-only
repo-doctor github-auth-doctor
repo-doctor guard --fail-on warn
repo-doctor guard --warnings-only
repo-doctor check --format markdown
repo-doctor guard --format sarif --output repo-doctor-guard.sarif
repo-doctor ci --guard
repo-doctor explain readme
repo-doctor config-validate repo-doctor.toml
```

## Output Preference

Prefer JSON or compact output when making automated decisions. Use Markdown for
user-facing summaries and AGENTS.md snippets.
