# Agent Skills Notes

Useful agent guidance for this template should stay small, concrete, and verifiable.

## Recommended Local Skills

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
