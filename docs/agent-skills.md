# Agent Skills Notes

Useful agent guidance for this template should stay small, concrete, and verifiable.

## Recommended Local Skills

### repo-doctor

Use when checking repository readiness, adding AI/VibeCoding guardrails,
installing repo-doctor in another repository, or deciding what an agent should
do before finishing repository-changing work.

- The packaged Codex-compatible skill is `skills/repo-doctor/SKILL.md`.
- Local checks only require the `repo-doctor` binary and repository read access.
- Start with `repo-doctor spec --format json` to discover commands, profiles,
  rules, output contracts, and recipes.
- Use `repo-doctor recipes --format markdown` to choose the right workflow.
- Use `repo-doctor check --format compact` for the first readiness signal.
- Use `repo-doctor agent-guide --format markdown` when `AGENTS.md` is missing
  or too generic.
- Run `repo-doctor github-auth-doctor` before remote GitHub checks or setup.
- Confirm repository admin permission before running remote setup changes.
- Treat `repo-doctor guard --fail-on warn` as the completion gate for
  repository-changing AI work.

### Rust CLI Maintainer

Use when adding or changing CLI behavior.

- Read `Cargo.toml`, `src/main.rs`, `src/lib.rs`, and relevant files under `tests/`.
- Keep executable setup in `main.rs`; put behavior in library code.
- Verify with `cargo fmtc`, `cargo lint`, and `cargo nt`.
- Check user-visible output with integration tests.

### Dependency Auditor

Use when adding or upgrading dependencies.

- Prefer standard library or existing dependencies first.
- If adding a dependency, explain why it is worth the maintenance cost.
- Run `cargo audit-deps` and `cargo deny check`.
- Update `deny.toml` only when the license or source policy is understood.

### Template Smoke Tester

Use when editing `scripts/new-from-template.sh`, `Cargo.toml`, or project naming behavior.

- Generate a fresh project under `~/src/template-smoke-cli`.
- Verify package name, crate import name, integration tests, and `cargo run`.
- Remove the smoke project after verification.

## Karpathy Guidelines

This template intentionally includes the community "andrej-karpathy-skills" style guardrails:

- Think before coding.
- Prefer the simplest sufficient solution.
- Keep changes surgical.
- Drive work by verifiable goals.

Use these as behavior constraints, then apply the repository-specific `AGENTS.md` sections for actual commands and file ownership.

## Imported Ideas

- Use `AGENTS.md` as repo-specific wayfinding, not a generic manifesto.
- Give agents concrete commands, file ownership boundaries, and completion criteria.
- For autonomous loops, define the editable files, frozen files, metric, and keep/discard rule.
- Keep "behavioral" rules short: think first, prefer simplicity, make surgical changes, verify with tests.
