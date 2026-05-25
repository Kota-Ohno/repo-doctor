# Installation

`repo-doctor` is intended to run without a local Rust toolchain.

`repo-doctor` は、利用者側にローカルRust toolchainを要求しない導入を前提にしています。

## Fastest Path

Try it locally without writing files:

ファイルを書き込まずにローカルで試す:

```bash
repo-doctor suggest
repo-doctor check --format compact
repo-doctor check --format summary
```

Add CI without hand-writing YAML:

YAMLを手書きせずにCIを追加する:

```bash
mkdir -p .github/workflows
repo-doctor ci --template generic > .github/workflows/repo-doctor.yml
```

Adopt gradually in an existing repository:

既存リポジトリに段階導入する:

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

よく使う入力:

- `path`: repository path to check, default `.`
- `profile`: `auto`, `generic`, `rust`, `node`, `python`, `go`, and other supported profiles
- `format`: output format, default `github`
- `fail-on`: set to `warn` for a strict quality gate
- `config`: optional config file
- `baseline`: optional baseline JSON
- `args`: full argument override for advanced use
- `html-report`, `junit-report`, `sarif-report`: optional report files

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

`REPO_DOCTOR_VERSION=v0.1.0` や `REPO_DOCTOR_INSTALL_DIR=/path/to/bin` を指定すると、導入するversionやinstall先を変更できます。

## Docker

```bash
docker run --rm -v "$PWD:/repo" ghcr.io/kota-ohno/repo-doctor:main check /repo --fail-on warn
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

package manager向けの雛形は `packaging/` にあります。

- `packaging/homebrew/repo-doctor.rb`
- `packaging/scoop/repo-doctor.json`
- `packaging/winget/README.md`
- `packaging/npm/`
- `packaging/binstall/README.md`

The Homebrew and Scoop manifests contain `TODO` checksums until the first real
release assets are published.

HomebrewとScoopのmanifestは、実際のrelease assetsが公開されるまでchecksumを `TODO` のままにしています。
