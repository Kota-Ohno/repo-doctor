# Installation

`repo-doctor` is intended to run without a local Rust toolchain.

`repo-doctor` は、利用者側にローカルRust toolchainを要求しない導入を前提にしています。

## Fastest Path

Try it locally without writing files:

ファイルを書き込まずにローカルで試す:

```bash
repo-doctor preflight
repo-doctor suggest
repo-doctor suggest --profiles rust,node
repo-doctor check --format compact
repo-doctor check --format summary
repo-doctor guard --fail-on warn
```

## Requirements / 必要なもの

Local checks only need the `repo-doctor` binary. A Rust toolchain is not
required unless you are developing repo-doctor itself.

ローカルチェックに必要なのは `repo-doctor` binaryだけです。repo-doctor自体を開発する場合を除き、Rust toolchainは不要です。

| Use case | Required tools | Permissions |
| --- | --- | --- |
| Local `check`, `guard`, `suggest`, `init` | `repo-doctor` on `PATH` | Read access to the repository directory |
| Install script on Linux/macOS | `sh`, `curl`, `tar`, `sha256sum` or `shasum` | Write access to the install directory, default `~/.local/bin` |
| Install script on Windows | PowerShell, `Invoke-WebRequest`, `tar` | Write access to the install directory, default `$HOME\.repo-doctor\bin` |
| Docker usage | Docker CLI/daemon | Permission to run Docker and mount the repository directory |
| GitHub Actions | GitHub Actions enabled | Workflow permission to read repository contents |
| Remote GitHub checks | GitHub CLI `gh` | `gh auth login`; repository read access |
| Remote setup changes | GitHub CLI `gh` | Repository admin or equivalent permission |

For private repositories, GitHub branch protection can also require GitHub Pro
or public visibility even when the token is authenticated.

| 用途 | 必要なコマンド | 権限 |
| --- | --- | --- |
| ローカルの `check`, `guard`, `suggest`, `init` | `PATH` 上の `repo-doctor` | 対象リポジトリdirectoryの読み取り権限 |
| Linux/macOS install script | `sh`, `curl`, `tar`, `sha256sum` または `shasum` | install先directoryへの書き込み権限。defaultは `~/.local/bin` |
| Windows install script | PowerShell, `Invoke-WebRequest`, `tar` | install先directoryへの書き込み権限。defaultは `$HOME\.repo-doctor\bin` |
| Docker利用 | Docker CLI/daemon | Docker実行権限とrepository directoryのmount権限 |
| GitHub Actions | GitHub Actions有効化 | repository contentsの読み取りworkflow権限 |
| Remote GitHub checks | GitHub CLI `gh` | `gh auth login` 済み、repository read access |
| Remote setup変更 | GitHub CLI `gh` | repository adminまたは同等の権限 |

private repositoryのbranch protectionは、tokenが認証済みでもGitHub Proまたはpublic visibilityが必要な場合があります。

Before remote GitHub checks, run:

remote GitHub checksの前に実行します。

```bash
gh auth status
repo-doctor github-auth-doctor
```

Add CI without hand-writing YAML:

YAMLを手書きせずにCIを追加する:

```bash
mkdir -p .github/workflows
repo-doctor ci --template generic > .github/workflows/repo-doctor.yml
repo-doctor ci --guard > .github/workflows/repo-doctor-guard.yml
```

Ecosystem templates are available for `rust`, `node`, `python`, `go`, `deno`,
`bun`, `jvm`, `dotnet`, `php`, `ruby`, `swift`, `cpp`, `docker`, `iac`, and
`docs`.

ecosystem別templateとして `rust`, `node`, `python`, `go`, `deno`, `bun`,
`jvm`, `dotnet`, `php`, `ruby`, `swift`, `cpp`, `docker`, `iac`, `docs` を利用できます。

Adopt gradually in an existing repository:

既存リポジトリに段階導入する:

```bash
repo-doctor baseline > repo-doctor-baseline.json
repo-doctor check --baseline repo-doctor-baseline.json --fail-on warn
```

## GitHub Action

```yaml
- uses: actions/checkout@v6
- uses: Kota-Ohno/repo-doctor@v0.1.1
  with:
    fail-on: warn
```

VibeCoding guardrail workflow:

VibeCoding向けのガードレールworkflow:

```yaml
- uses: actions/checkout@v6
  with:
    fetch-depth: 0
- uses: Kota-Ohno/repo-doctor@v0.1.1
  with:
    args: guard --base origin/main --format github --fail-on warn
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
curl -fsSL https://raw.githubusercontent.com/Kota-Ohno/repo-doctor/main/scripts/install.sh | sh -s -- --version v0.1.1 --dir ~/.local/bin
```

Windows PowerShell:

```powershell
iwr https://raw.githubusercontent.com/Kota-Ohno/repo-doctor/main/scripts/install.ps1 -UseB | iex
.\install.ps1 -Version v0.1.1 -InstallDir "$HOME\.repo-doctor\bin"
```

Set `REPO_DOCTOR_VERSION=v0.1.1` or `REPO_DOCTOR_INSTALL_DIR=/path/to/bin` to
customize installation.

`REPO_DOCTOR_VERSION=v0.1.1` や `REPO_DOCTOR_INSTALL_DIR=/path/to/bin` を指定すると、導入するversionやinstall先を変更できます。

If the install directory is not writable, choose a user-writable directory
instead of using `sudo`:

install先directoryに書き込めない場合は、`sudo` ではなくユーザーが書き込めるdirectoryを指定します。

```bash
curl -fsSL https://raw.githubusercontent.com/Kota-Ohno/repo-doctor/main/scripts/install.sh | sh -s -- --dir "$HOME/.local/bin"
export PATH="$HOME/.local/bin:$PATH"
```

```powershell
iwr https://raw.githubusercontent.com/Kota-Ohno/repo-doctor/main/scripts/install.ps1 -UseB | iex
$env:PATH = "$HOME\.repo-doctor\bin;$env:PATH"
```

## Docker

```bash
docker run --rm -v "$PWD:/repo" ghcr.io/kota-ohno/repo-doctor:main check /repo --fail-on warn
```

## pre-commit

```yaml
repos:
  - repo: https://github.com/Kota-Ohno/repo-doctor
    rev: v0.1.1
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

The Homebrew and Scoop manifests are updated from published release checksum
assets.

HomebrewとScoopのmanifestは、公開済みrelease checksum assetsから更新します。
