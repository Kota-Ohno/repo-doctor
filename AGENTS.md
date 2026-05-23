# AGENTS.md

This file gives coding agents the project-specific context needed to work safely in this repository.

## Karpathy Guidelines

These behavioral rules are adapted from the community "andrej-karpathy-skills" guidelines. They are not a substitute for the project-specific rules below.

### 1. Think Before Coding

- Do not assume silently.
- State assumptions when they affect implementation.
- If multiple interpretations are plausible, call that out before choosing one.
- If something is unclear enough to change the result, ask before editing.

### 2. Simplicity First

- Write the smallest code that solves the requested problem.
- Do not add speculative features, abstractions, configuration, or flexibility.
- Prefer direct Rust code over clever generic machinery.
- If the solution grows large, re-check whether the task can be solved more simply.

### 3. Surgical Changes

- Touch only files needed for the request.
- Match existing style and structure.
- Do not refactor adjacent code unless the request requires it.
- Remove only unused code introduced by your own change.
- Mention unrelated cleanup opportunities instead of applying them.

### 4. Goal-Driven Execution

- Convert the request into a verifiable outcome.
- For fixes, prefer a failing test or reproduction before changing behavior.
- For feature changes, add or update tests that describe the user-visible result.
- Keep working until the relevant verification commands pass, or explain the blocker.

## Project Shape

- This is a Rust CLI template.
- `src/main.rs` is the thin executable entry point.
- `src/lib.rs` contains application logic and unit tests.
- `tests/` contains integration tests for CLI behavior.
- `.github/workflows/ci.yml` defines the expected CI checks.
- `scripts/new-from-template.sh` creates a new project from this template.

## Standard Commands

Run these before considering a change complete:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run
cargo test --doc
cargo audit
cargo deny check
typos
taplo fmt --check
```

Short aliases are also available:

```bash
cargo fmtc
cargo lint
cargo nt
cargo audit-deps
cargo docs
```

## Development Rules

- Keep `main.rs` small. Move behavior into `lib.rs` or focused modules.
- Prefer explicit error handling with `anyhow::Result` at app boundaries.
- Add or update tests when behavior changes.
- Keep CLI output stable unless the task explicitly asks to change it.
- Keep the template generic. Do not add app-specific business logic.
- Avoid new dependencies unless they materially simplify the implementation.
- If a dependency is added, verify `cargo deny check` still passes.

## Agent Guardrails

- Think before coding: identify the target behavior, likely files, and verification command.
- Keep changes surgical: avoid unrelated formatting, renames, or refactors.
- Favor simple code over abstractions that are not yet needed.
- Surface uncertainty. If requirements conflict, pause and ask rather than guessing.
- Do not publish, push, release, or change repository visibility without explicit user confirmation.
- Do not edit generated build output such as `target/`.

## Template Validation

When changing template-generation behavior, test both the template and a generated project:

```bash
rm -rf ~/src/template-smoke-cli
scripts/new-from-template.sh template-smoke-cli
cd ~/src/template-smoke-cli
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run
cargo test --doc
cargo run -- greet --name Kota
```
