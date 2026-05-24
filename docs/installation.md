# Installation

`repo-doctor` is intended to run without a local Rust toolchain.

## Fastest Path

Try it locally without writing files:

```bash
repo-doctor suggest
repo-doctor check --format compact
```

Add CI without hand-writing YAML:

```bash
mkdir -p .github/workflows
repo-doctor ci --template generic > .github/workflows/repo-doctor.yml
```

Adopt gradually in an existing repository:

```bash
repo-doctor baseline > repo-doctor-baseline.json
repo-doctor check --baseline repo-doctor-baseline.json --fail-on warn
```

## GitHub Action

```yaml
- uses: actions/checkout@v6
- uses: Kota-Ohno/repo-doctor@v0.1.0
  with:
    fail-on: warn
```

Common inputs:

- `path`: repository path to check, default `.`
- `profile`: `auto`, `generic`, `rust`, `node`, `python`, `go`, and other supported profiles
- `format`: output format, default `github`
- `fail-on`: set to `warn` for a strict quality gate
- `config`: optional config file
- `baseline`: optional baseline JSON
- `args`: full argument override for advanced use

## Install Script

Linux and macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/Kota-Ohno/repo-doctor/main/scripts/install.sh | sh
curl -fsSL https://raw.githubusercontent.com/Kota-Ohno/repo-doctor/main/scripts/install.sh | sh -s -- --version v0.1.0 --dir ~/.local/bin
```

Windows PowerShell:

```powershell
iwr https://raw.githubusercontent.com/Kota-Ohno/repo-doctor/main/scripts/install.ps1 -UseB | iex
.\install.ps1 -Version v0.1.0 -InstallDir "$HOME\.repo-doctor\bin"
```

Set `REPO_DOCTOR_VERSION=v0.1.0` or `REPO_DOCTOR_INSTALL_DIR=/path/to/bin` to
customize installation.

## Docker

```bash
docker build -t repo-doctor .
docker run --rm -v "$PWD:/repo" repo-doctor check /repo --fail-on warn
```

## pre-commit

```yaml
repos:
  - repo: https://github.com/Kota-Ohno/repo-doctor
    rev: v0.1.0
    hooks:
      - id: repo-doctor
```

## Package Managers

Packaging scaffolds live under `packaging/`:

- `packaging/homebrew/repo-doctor.rb`
- `packaging/scoop/repo-doctor.json`
- `packaging/winget/README.md`
- `packaging/npm/`

The Homebrew and Scoop manifests contain `TODO` checksums until the first real
release assets are published.
