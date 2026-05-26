<p align="center">
  <img src="assets/icon.svg" alt="repo-doctor" width="120" height="120">
</p>

# repo-doctor

A local-first CLI that checks repository readiness.

リポジトリが公開・CI導入・継続運用に必要な基本要素を満たしているかを、ローカル優先で確認するCLIです。

`repo-doctor` inspects a repository directory and reports whether basic project
files and ecosystem-specific metadata are present. It currently checks for:

`repo-doctor` はリポジトリディレクトリを検査し、基本ファイルとエコシステム固有メタデータが揃っているかを報告します。現在の主なチェック対象は次のとおりです。

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
- frontend framework, Terraform/OpenTofu, and docs-site metadata
- AI/VibeCoding guardrails for Git diffs and `AGENTS.md`

By default, `check` runs core repository checks plus auto-detected ecosystem
profiles. Use `--profile generic` for language-independent checks only, or
select a profile explicitly. Use `--profiles rust,node,docker` to force a
comma-separated set for monorepos or imperfect auto-detection.

デフォルトの `check` は、言語非依存のcoreチェックと、自動検出されたエコシステムprofileチェックを実行します。言語非依存チェックだけを実行したい場合は `--profile generic`、特定profileを指定したい場合は `--profile rust` などを使います。monorepoや自動検出の補正では `--profiles rust,node,docker` のように複数profileを明示できます。

## Install

`repo-doctor` is distributed as a standalone binary. You do not need a Rust
toolchain to run it.

`repo-doctor` は単体バイナリとして配布されます。利用するだけならRust toolchainは不要です。

Local checks only require the `repo-doctor` binary and read access to the target
repository. Remote GitHub checks additionally require the GitHub CLI (`gh`) and
an authenticated token with access to the target repository. Setup commands that
change GitHub repository settings may require repository admin permission.

ローカルチェックに必要なのは `repo-doctor` binaryと対象リポジトリの読み取り権限だけです。remote GitHub checksには、追加でGitHub CLI (`gh`) と対象リポジトリへアクセスできる認証済みtokenが必要です。GitHub repository settingsを変更するsetup系コマンドはrepository admin権限を必要とする場合があります。

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

ファイルを作らずに試す:

```bash
repo-doctor preflight
repo-doctor suggest
repo-doctor suggest --profiles rust,node
repo-doctor init --print-config
repo-doctor ci --template generic
```

GitHub Actions:

```yaml
- uses: actions/checkout@v6
- uses: Kota-Ohno/repo-doctor@v0.1.1
  with:
    fail-on: warn
```

Docker:

```bash
docker build -t repo-doctor .
docker run --rm -v "$PWD:/repo" repo-doctor check /repo
docker run --rm -v "$PWD:/repo" ghcr.io/kota-ohno/repo-doctor:main check /repo
```

More install paths are documented in [docs/installation.md](docs/installation.md).

その他の導入方法は [docs/installation.md](docs/installation.md) にまとめています。

## Development Requirements

開発に必要なもの:

- Rust stable
- `cargo-nextest`
- `cargo-audit`
- `cargo-deny`

## Commands

主なコマンド:

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
repo-doctor check --format summary
repo-doctor guard --fail-on warn
scripts/profile-smoke.sh
scripts/distribution-smoke.sh
repo-doctor check --format html --output repo-doctor.html
repo-doctor check --config repo-doctor.toml
repo-doctor check --baseline repo-doctor-baseline.json
repo-doctor check --warnings-only
repo-doctor check --min-score 90
repo-doctor check --profile generic
repo-doctor check --profile rust
repo-doctor check --profiles rust,node,docker
repo-doctor guard --profiles rust,node --fail-on warn
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
repo-doctor check --profile frontend
repo-doctor check --profile iac
repo-doctor check --profile docs
repo-doctor check --fail-on warn
repo-doctor preflight --format json
repo-doctor github Kota-Ohno/repo-doctor
repo-doctor github Kota-Ohno/repo-doctor --warnings-only
repo-doctor github-setup Kota-Ohno/repo-doctor --topic rust --topic cli --homepage https://github.com/Kota-Ohno/repo-doctor --branch-protection
repo-doctor baseline > repo-doctor-baseline.json
repo-doctor batch repos.txt
repo-doctor suggest
repo-doctor suggest --profiles rust,node
repo-doctor spec --format json
repo-doctor recipes --format markdown
repo-doctor agent-guide --format markdown
repo-doctor agent-guide --profiles rust,node --format markdown
repo-doctor ci --template node
repo-doctor ci --template deno
repo-doctor ci --template bun
repo-doctor ci --template jvm
repo-doctor ci --template dotnet
repo-doctor ci --template php
repo-doctor ci --template ruby
repo-doctor ci --template swift
repo-doctor ci --template cpp
repo-doctor ci --template docker
repo-doctor ci --template iac
repo-doctor ci --template docs
repo-doctor ci --guard
repo-doctor explain readme
repo-doctor config-validate repo-doctor.toml
repo-doctor config-explain
repo-doctor init --print-config
repo-doctor init --interactive
repo-doctor init --full --dry-run
repo-doctor init --full --template node --yes
repo-doctor version-check
repo-doctor github-auth-doctor
repo-doctor github-setup Kota-Ohno/repo-doctor --topic rust --dry-run
repo-doctor scorecard Kota-Ohno/repo-doctor
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

シェル補完やman pageも生成できます。

```bash
repo-doctor init
repo-doctor completions bash
repo-doctor man
```

## Configuration

`repo-doctor.toml` is loaded from the checked repository root by default.
Use `repo-doctor init` to create a starter config.

`repo-doctor.toml` は、デフォルトではチェック対象リポジトリのrootから読み込まれます。スターター設定は `repo-doctor init` で作成できます。ファイルを書かずに内容だけ見たい場合は `repo-doctor init --print-config` を使います。

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

`--format json` は `schema_version: 1` を出力します。minor releaseでフィールドが追加されることはありますが、schema version 1 の範囲では既存rule IDと既存フィールドの意味を安定させます。詳細は [docs/report-contract.md](docs/report-contract.md) を参照してください。

## Remote GitHub Checks

`repo-doctor github owner/repo` runs optional remote checks through the `gh` CLI.
It is separate from local `check` so local repository checks stay offline-first.

`repo-doctor github owner/repo` は `gh` CLI 経由で任意のremote GitHub checksを実行します。ローカルの `check` はoffline-firstのままにするため、remote checksは明示コマンドに分離しています。

## Usage Patterns

For CI quality gates, combine a machine-readable output with an exit policy:

CIのquality gateでは、機械可読な出力と終了コードポリシーを組み合わせます。

```bash
repo-doctor ci --template generic > .github/workflows/repo-doctor.yml
repo-doctor check --format github --fail-on warn
repo-doctor check --format compact --min-score 90
repo-doctor check --format junit > repo-doctor-junit.xml
repo-doctor check --format html > repo-doctor.html
repo-doctor check --format summary
```

For AI/VibeCoding guardrails, use `guard`. It runs normal readiness checks and
adds Git-diff checks for newly added secret-like files, CI guardrail changes,
removed tests, manifest changes without lockfile updates, oversized changes, and
`AGENTS.md` quality. It also flags source changes without test updates, build or
task definition changes, and generated/vendor/binary artifacts across the
supported ecosystem profiles.

AI/VibeCoding のガードレールとして使う場合は `guard` を使います。通常のreadiness checkに加えて、Git差分から、secret-like fileの追加、CI guardrail変更、test削除、lockfile未更新、巨大差分、`AGENTS.md` の品質を確認します。さらに、対応済みecosystem profile全体で、test更新なしのsource変更、build/task定義変更、生成物/vendor/binary artifactも警告します。

```bash
repo-doctor guard --fail-on warn
repo-doctor guard --base origin/main --format github --fail-on warn
repo-doctor ci --guard > .github/workflows/repo-doctor-guard.yml
```

For AI agents, the machine-readable operating contract is documented in
[docs/ai.md](docs/ai.md), and a Codex-compatible local skill is included at
[skills/repo-doctor/SKILL.md](skills/repo-doctor/SKILL.md).

AI agent向けの機械可読な操作仕様は [docs/ai.md](docs/ai.md) にまとめています。Codex互換のlocal skillも [skills/repo-doctor/SKILL.md](skills/repo-doctor/SKILL.md) に同梱しています。

For config authoring:

設定を書くとき:

```bash
repo-doctor init --print-config
repo-doctor list-profiles
repo-doctor list-rules
repo-doctor explain readme
repo-doctor config-validate repo-doctor.toml
repo-doctor config-explain
repo-doctor check --warnings-only
```

For incremental adoption:

既存リポジトリへ段階導入するとき:

```bash
repo-doctor baseline > repo-doctor-baseline.json
repo-doctor check --baseline repo-doctor-baseline.json --fail-on warn
```

More examples:

その他の例:

- [docs/ai.md](docs/ai.md)
- [docs/examples.md](docs/examples.md)
- [docs/release.md](docs/release.md)
- [docs/troubleshooting.md](docs/troubleshooting.md)
- [docs/rules.md](docs/rules.md)
- [docs/release-packaging.md](docs/release-packaging.md)

## Layout

リポジトリ構成:

```text
AGENTS.md     coding-agent instructions with Karpathy-style guardrails
src/main.rs   CLI entry point
src/lib.rs    application logic
tests/        integration tests
docs/         project notes
```
