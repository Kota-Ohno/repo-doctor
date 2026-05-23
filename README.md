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
- Container metadata when Docker or Compose files are detected
- JVM metadata when Maven or Gradle files are detected
- Deno and Bun metadata for TypeScript/JavaScript runtimes
- .NET, PHP, Ruby, C/C++, Swift, and Kotlin project metadata

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
cargo run -- check --format markdown
cargo run -- check --format github
cargo run -- check --format sarif
cargo run -- check --format compact
cargo run -- check --format junit
cargo run -- check --config repo-doctor.toml
cargo run -- check --warnings-only
cargo run -- check --min-score 90
cargo run -- check --profile generic
cargo run -- check --profile rust
cargo run -- check --profile node
cargo run -- check --profile python
cargo run -- check --profile go
cargo run -- check --profile docker
cargo run -- check --profile jvm
cargo run -- check --profile deno
cargo run -- check --profile bun
cargo run -- check --profile dotnet
cargo run -- check --profile php
cargo run -- check --profile ruby
cargo run -- check --profile cpp
cargo run -- check --profile swift
cargo run -- check --profile kotlin
cargo run -- check --fail-on warn
cargo run -- github Kota-Ohno/repo-doctor
cargo run -- github Kota-Ohno/repo-doctor --warnings-only
cargo run -- list-profiles
cargo run -- list-rules
cargo test
cargo clippy --all-targets --all-features -- -D warnings
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
cargo run -- init
cargo run -- completions bash
cargo run -- man
```

## Configuration

`repo-doctor.toml` is loaded from the checked repository root by default.
Use `repo-doctor init` to create a starter config.

```toml
profiles = ["auto"]
presets = ["oss"]
exclude_paths = ["vendor/*", "generated/*"]

[[rules]]
id = "changelog"
disabled = true
reason = "Release notes are tracked outside this repository."

[[rules]]
id = "code_of_conduct"
severity = "info"
```

## JSON Output

`--format json` emits `schema_version: 1`. Fields may be added in later minor
versions, but existing rule IDs and field meanings should remain stable within
schema version 1. See [docs/report-contract.md](docs/report-contract.md).

## Remote GitHub Checks

`repo-doctor github owner/repo` runs optional remote checks through the `gh` CLI.
It is separate from local `check` so local repository checks stay offline-first.

## Usage Patterns

For CI quality gates, combine a machine-readable output with an exit policy:

```bash
repo-doctor check --format github --fail-on warn
repo-doctor check --format compact --min-score 90
repo-doctor check --format junit > repo-doctor-junit.xml
```

For config authoring:

```bash
repo-doctor list-profiles
repo-doctor list-rules
repo-doctor check --warnings-only
```

## Layout

```text
AGENTS.md     coding-agent instructions with Karpathy-style guardrails
src/main.rs   CLI entry point
src/lib.rs    application logic
tests/        integration tests
docs/         project notes
```
