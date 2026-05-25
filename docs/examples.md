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
repo-doctor guard --fail-on warn
repo-doctor guard --base origin/main --format github --fail-on warn
repo-doctor ci --guard > .github/workflows/repo-doctor-guard.yml
```

Use this when AI agents are making repository changes. It checks the full
repository and adds diff-aware warnings for newly added secret-like files,
removed tests, changed CI guardrails, dependency manifests without lockfile
updates, and weak or missing `AGENTS.md` instructions.

AI agentがリポジトリを変更する場合に使います。リポジトリ全体の確認に加えて、secret-like fileの追加、test削除、CI guardrail変更、lockfile未更新、弱い/不足した `AGENTS.md` を差分ベースで警告します。

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

## Dockerized App

```bash
repo-doctor check --profile docker --format summary
docker run --rm -v "$PWD:/repo" ghcr.io/kota-ohno/repo-doctor:main check /repo
```

## Gradual Adoption / 段階導入

```bash
repo-doctor baseline > repo-doctor-baseline.json
repo-doctor check --baseline repo-doctor-baseline.json --fail-on warn
```
