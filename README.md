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

## Install

`repo-doctor` is distributed as a standalone binary. You do not need a Rust
toolchain to run it.

Linux and macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/Kota-Ohno/repo-doctor/main/scripts/install.sh | sh
repo-doctor check
```

Windows PowerShell:

```powershell
iwr https://raw.githubusercontent.com/Kota-Ohno/repo-doctor/main/scripts/install.ps1 -UseB | iex
repo-doctor check
```

Try without creating files:

```bash
repo-doctor suggest
repo-doctor init --print-config
repo-doctor ci --template generic
```

GitHub Actions:

```yaml
- uses: actions/checkout@v6
- uses: Kota-Ohno/repo-doctor@v0.1.0
  with:
    fail-on: warn
```

Docker:

```bash
docker build -t repo-doctor .
docker run --rm -v "$PWD:/repo" repo-doctor check /repo
```

More install paths are documented in [docs/installation.md](docs/installation.md).

## Development Requirements

- Rust stable
- `cargo-nextest`
- `cargo-audit`
- `cargo-deny`

## Commands

```bash
repo-doctor check
repo-doctor check /path/to/repo
repo-doctor check --format json
repo-doctor check --format markdown
repo-doctor check --format github
repo-doctor check --format sarif
repo-doctor check --format compact
repo-doctor check --format junit
repo-doctor check --format html
repo-doctor check --config repo-doctor.toml
repo-doctor check --baseline repo-doctor-baseline.json
repo-doctor check --warnings-only
repo-doctor check --min-score 90
repo-doctor check --profile generic
repo-doctor check --profile rust
repo-doctor check --profile node
repo-doctor check --profile python
repo-doctor check --profile go
repo-doctor check --profile docker
repo-doctor check --profile jvm
repo-doctor check --profile deno
repo-doctor check --profile bun
repo-doctor check --profile dotnet
repo-doctor check --profile php
repo-doctor check --profile ruby
repo-doctor check --profile cpp
repo-doctor check --profile swift
repo-doctor check --profile kotlin
repo-doctor check --fail-on warn
repo-doctor github Kota-Ohno/repo-doctor
repo-doctor github Kota-Ohno/repo-doctor --warnings-only
repo-doctor github-setup Kota-Ohno/repo-doctor --topic rust --topic cli --homepage https://github.com/Kota-Ohno/repo-doctor --branch-protection
repo-doctor baseline > repo-doctor-baseline.json
repo-doctor batch repos.txt
repo-doctor suggest
repo-doctor ci --template node
repo-doctor explain readme
repo-doctor config-validate repo-doctor.toml
repo-doctor init --print-config
repo-doctor init --full --dry-run
repo-doctor init --full --template node --yes
repo-doctor version-check
repo-doctor list-profiles
repo-doctor list-rules
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
repo-doctor init
repo-doctor completions bash
repo-doctor man
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
repo-doctor ci --template generic > .github/workflows/repo-doctor.yml
repo-doctor check --format github --fail-on warn
repo-doctor check --format compact --min-score 90
repo-doctor check --format junit > repo-doctor-junit.xml
repo-doctor check --format html > repo-doctor.html
```

For config authoring:

```bash
repo-doctor init --print-config
repo-doctor list-profiles
repo-doctor list-rules
repo-doctor explain readme
repo-doctor config-validate repo-doctor.toml
repo-doctor check --warnings-only
```

For incremental adoption:

```bash
repo-doctor baseline > repo-doctor-baseline.json
repo-doctor check --baseline repo-doctor-baseline.json --fail-on warn
```

## Layout

```text
AGENTS.md     coding-agent instructions with Karpathy-style guardrails
src/main.rs   CLI entry point
src/lib.rs    application logic
tests/        integration tests
docs/         project notes
```
