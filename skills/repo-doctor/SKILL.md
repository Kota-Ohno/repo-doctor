---
name: repo-doctor
description: Install and run repo-doctor to assess repository readiness, then summarize warnings and likely fixes.
---

# repo-doctor

Use this skill when a user asks whether a repository is ready to publish,
whether project metadata is complete, or how to improve repository hygiene.

## Workflow

1. Prefer an existing `repo-doctor` binary on `PATH`.
2. If it is missing, install the latest release with `scripts/install.sh` or
   `scripts/install.ps1`.
3. Run `repo-doctor check --format compact` first for a quick signal.
4. Run `repo-doctor check --warnings-only` when warnings exist.
5. For GitHub repository settings, run `repo-doctor github owner/repo` only when
   the user wants remote checks and the `gh` CLI is authenticated.
6. Summarize warnings as actionable tasks. Do not modify files unless the user
   asks you to fix them.

## Useful Commands

```bash
repo-doctor check --format compact
repo-doctor check --warnings-only
repo-doctor check --format markdown
repo-doctor explain readme
repo-doctor config-validate repo-doctor.toml
```
