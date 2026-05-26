# AI Usage Specification

`repo-doctor` is designed to be operated by coding agents as well as humans.
Agents should prefer machine-readable commands first, then use markdown docs as
fallback context.

## Discovery

Use these commands before deciding how to operate the repository:

```bash
repo-doctor spec --format json
repo-doctor recipes --format markdown
repo-doctor list-profiles
repo-doctor list-rules
repo-doctor config-explain
```

`spec --format json` is the primary machine-readable contract for agents. It
lists commands, supported profiles, rule IDs, output contracts, and recipes.

## Environment Preflight

Agents should distinguish local checks from remote GitHub operations. Local
checks need only `repo-doctor` and repository read access. Remote checks need
`gh` and authentication. Remote setup changes may need repository admin access.

```bash
repo-doctor --version
repo-doctor preflight --format json
repo-doctor github-auth-doctor
```

If `github-auth-doctor` reports missing `gh`, failed authentication, or
unavailable API access, report that as the blocker before attempting remote
checks or setup.

## Skill Usage

This repository ships a Codex-compatible skill at
`skills/repo-doctor/SKILL.md`. Agents that support local skills should load that
skill when the user asks about repository readiness, VibeCoding guardrails,
AI-safe repository changes, or repo-doctor adoption.

Recommended skill workflow:

```bash
repo-doctor spec --format json
repo-doctor recipes --format markdown
repo-doctor check --format compact
repo-doctor agent-guide --format markdown
repo-doctor agent-guide --profiles rust,node --format markdown
repo-doctor guard --fail-on warn
```

The skill's completion gate is the same as the general AI loop:

```bash
repo-doctor guard --fail-on warn
```

## Standard Agent Loop

```bash
repo-doctor check --format summary
repo-doctor preflight --format json
repo-doctor recipes --format markdown
repo-doctor agent-guide --format markdown
repo-doctor agent-guide --profiles rust,node --format markdown
repo-doctor guard --fail-on warn
```

Use `check` to understand repository readiness. Use `guard` before finishing a
coding task because it includes diff-aware checks for AI-generated changes.

## AGENTS.md Generation

```bash
repo-doctor agent-guide --format markdown >> AGENTS.md
repo-doctor agent-guide --profiles rust,node --format markdown >> AGENTS.md
repo-doctor guard --warnings-only
```

`agent-guide` detects repository profiles and emits verification commands and
behavior constraints suitable for `AGENTS.md`. Use `--profiles` when a monorepo
or unusual layout needs an explicit profile set.

## CI

```bash
repo-doctor ci --template generic > .github/workflows/repo-doctor.yml
repo-doctor ci --guard > .github/workflows/repo-doctor-guard.yml
```

## Smoke Checks

Before finishing repo-doctor profile, rule, CI-template, install, or release
work, agents should run:

```bash
scripts/profile-smoke.sh
scripts/distribution-smoke.sh
```

The readiness workflow checks repository hygiene. The guard workflow checks
AI/VibeCoding risks in Git diffs.

## Completion Criteria

An agent should not consider repository-changing work complete until this
command succeeds or every warning has a documented rationale:

```bash
repo-doctor guard --fail-on warn
```
