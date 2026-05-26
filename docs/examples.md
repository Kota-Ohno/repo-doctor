# Examples / 例

Copy-paste starting points for common repositories.

よくあるリポジトリ種別ごとの導入例です。

## Generic OSS

```bash
repo-doctor suggest
repo-doctor ci --template generic > .github/workflows/repo-doctor.yml
repo-doctor check --fail-on warn
```

## VibeCoding Guardrails / VibeCodingガードレール

```bash
repo-doctor spec --format json
repo-doctor recipes --format markdown
repo-doctor agent-guide --format markdown
repo-doctor agent-guide --profiles rust,node --format markdown
repo-doctor guard --fail-on warn
repo-doctor guard --base origin/main --format github --fail-on warn
repo-doctor ci --guard > .github/workflows/repo-doctor-guard.yml
```

Use this when AI agents are making repository changes. It checks the full
repository and adds diff-aware warnings for newly added secret-like files,
removed tests, changed CI guardrails, dependency manifests without lockfile
updates, source changes without test updates, generated/binary artifacts, build
or task definition changes, and weak or missing `AGENTS.md` instructions.

AI agentがリポジトリを変更する場合に使います。リポジトリ全体の確認に加えて、secret-like fileの追加、test削除、CI guardrail変更、lockfile未更新、test更新なしのsource変更、生成物/binary artifact、build/task定義変更、弱い/不足した `AGENTS.md` を差分ベースで警告します。

## Rust CLI

```bash
repo-doctor check --profile rust --format summary
repo-doctor ci --template rust > .github/workflows/repo-doctor.yml
```

## Node / Frontend

```bash
repo-doctor check --profile node --format summary
repo-doctor check --profile frontend --warnings-only
repo-doctor ci --template node > .github/workflows/repo-doctor.yml
```

## Monorepo / 明示的な複数profile

Use `--profiles` when auto-detection misses an ecosystem or when a monorepo
needs a fixed set of checks:

auto-detectionがecosystemを拾いきれない場合や、monorepoで固定のcheck setを使いたい場合は `--profiles` を使います。

```bash
repo-doctor check --profiles rust,node,docker --format summary
repo-doctor guard --profiles rust,node,docker --fail-on warn
```

## Python Package

```bash
repo-doctor check --profile python --format summary
repo-doctor ci --template python > .github/workflows/repo-doctor.yml
```

## Go Module

```bash
repo-doctor check --profile go --format summary
repo-doctor ci --template go > .github/workflows/repo-doctor.yml
```

## Other Ecosystems / その他のecosystem

```bash
repo-doctor check --profile deno --format summary
repo-doctor check --profile bun --format summary
repo-doctor check --profile jvm --format summary
repo-doctor check --profile dotnet --format summary
repo-doctor check --profile php --format summary
repo-doctor check --profile ruby --format summary
repo-doctor check --profile swift --format summary
repo-doctor check --profile cpp --format summary
repo-doctor check --profile iac --format summary
repo-doctor check --profile docs --format summary
```

Generate starter CI with the matching template:

対応するtemplateでstarter CIを生成できます。

```bash
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
```

## Dockerized App

```bash
repo-doctor check --profile docker --format summary
docker run --rm -v "$PWD:/repo" ghcr.io/kota-ohno/repo-doctor:main check /repo
```

## Gradual Adoption / 段階導入

```bash
repo-doctor preflight
repo-doctor baseline > repo-doctor-baseline.json
repo-doctor check --baseline repo-doctor-baseline.json --fail-on warn
```

## Warning Triage / warningの切り分け

If a warning is expected for the project shape, prefer a preset or a documented
rule disable over ignoring it silently:

project形状としてwarningが想定通りの場合は、黙って無視せずpresetまたは理由付きrule disableを使います。

```toml
presets = ["python-lib"]

[[rules]]
id = "python_lockfile"
disabled = true
reason = "Library package intentionally does not commit a lockfile."
```
