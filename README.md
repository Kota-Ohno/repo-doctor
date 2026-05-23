# repo-doctor

A local-first CLI that checks repository readiness.

`repo-doctor` inspects a repository directory and reports whether basic project
files and ecosystem-specific metadata are present. It currently checks for:

- `README.md`
- a license file
- `.gitignore`
- `.github/workflows`
- community health files such as contributing, security, issue, PR, conduct,
  and changelog files
- Rust package metadata when `Cargo.toml` is detected
- Node.js package metadata when `package.json` is detected
- Python package metadata when `pyproject.toml`, `setup.py`, or
  `requirements.txt` is detected
- Go module metadata when `go.mod` is detected

By default, `check` runs core repository checks plus auto-detected ecosystem
profiles. Use `--profile generic` for language-independent checks only, or
select a profile explicitly.

## Requirements

- Rust stable
- `cargo-nextest`
- `cargo-audit`
- `cargo-deny`

## Commands

```bash
cargo run -- check
cargo run -- check /path/to/repo
cargo run -- check --format json
cargo run -- check --profile generic
cargo run -- check --profile rust
cargo run -- check --profile node
cargo run -- check --profile python
cargo run -- check --profile go
cargo run -- check --fail-on warn
cargo c
cargo lint
cargo fmtc
cargo nt
cargo docs
cargo audit-deps
cargo deny check
typos
taplo fmt --check
```

Generate shell completions or a man page:

```bash
cargo run -- completions bash
cargo run -- man
```

## JSON Output

`--format json` emits `schema_version: 1`. Fields may be added in later minor
versions, but existing rule IDs and field meanings should remain stable within
schema version 1.

## Layout

```text
AGENTS.md     coding-agent instructions with Karpathy-style guardrails
src/main.rs   CLI entry point
src/lib.rs    application logic
tests/        integration tests
docs/         project notes
```
