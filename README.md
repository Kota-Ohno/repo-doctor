# repo-doctor

A small Rust CLI that checks repository hygiene.

`repo-doctor` inspects a repository directory and reports whether basic project
files are present. It currently checks for:

- `README.md`
- a license file
- `.gitignore`
- `Cargo.toml`
- `.github/workflows`

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
