# Installation

`repo-doctor` is intended to run without a local Rust toolchain.

## GitHub Action

```yaml
- uses: actions/checkout@v6
- uses: Kota-Ohno/repo-doctor@v0.1.0
  with:
    args: check --fail-on warn
```

## Install Script

Linux and macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/Kota-Ohno/repo-doctor/main/scripts/install.sh | sh
```

Windows PowerShell:

```powershell
iwr https://raw.githubusercontent.com/Kota-Ohno/repo-doctor/main/scripts/install.ps1 -UseB | iex
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
