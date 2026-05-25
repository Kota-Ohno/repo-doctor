# Agent Skills Notes

Useful agent guidance for this template should stay small, concrete, and verifiable.

このテンプレートのagent向けガイダンスは、小さく、具体的で、検証可能な内容に留めます。

## Recommended Local Skills

推奨するローカルskill:

### repo-doctor

Use when checking repository readiness, adding AI/VibeCoding guardrails,
installing repo-doctor in another repository, or deciding what an agent should
do before finishing repository-changing work.

repository readinessの確認、AI/VibeCoding guardrailsの導入、別リポジトリへのrepo-doctor導入、またはagentがリポジトリ変更作業を完了する前に何を確認すべきか判断するときに使います。

- The packaged Codex-compatible skill is `skills/repo-doctor/SKILL.md`.
- Local checks only require the `repo-doctor` binary and repository read access.
- Start with `repo-doctor spec --format json` to discover commands, profiles,
  rules, output contracts, and recipes.
- Use `repo-doctor recipes --format markdown` to choose the right workflow.
- Use `repo-doctor check --format compact` for the first readiness signal.
- Use `repo-doctor agent-guide --format markdown` when `AGENTS.md` is missing
  or too generic.
- Run `repo-doctor github-auth-doctor` before remote GitHub checks or setup.
- Confirm repository admin permission before running remote setup changes.
- Treat `repo-doctor guard --fail-on warn` as the completion gate for
  repository-changing AI work.

日本語要約:

- Codex互換skillは `skills/repo-doctor/SKILL.md` に同梱しています。
- local checksに必要なのは `repo-doctor` binaryとrepository read accessだけです。
- まず `repo-doctor spec --format json` で commands、profiles、rules、output contracts、recipesを発見します。
- `repo-doctor recipes --format markdown` で目的に合うworkflowを選びます。
- 最初のreadiness確認には `repo-doctor check --format compact` を使います。
- `AGENTS.md` がない、または一般的すぎる場合は `repo-doctor agent-guide --format markdown` を使います。
- remote GitHub checksやsetupの前に `repo-doctor github-auth-doctor` を実行します。
- remote setup変更の前にrepository admin権限を確認します。
- AIによるリポジトリ変更作業の完了条件として `repo-doctor guard --fail-on warn` を扱います。

### Rust CLI Maintainer

Use when adding or changing CLI behavior.

CLIの挙動を追加・変更するときに使います。

- Read `Cargo.toml`, `src/main.rs`, `src/lib.rs`, and relevant files under `tests/`.
- Keep executable setup in `main.rs`; put behavior in library code.
- Verify with `cargo fmtc`, `cargo lint`, and `cargo nt`.
- Check user-visible output with integration tests.

### Dependency Auditor

Use when adding or upgrading dependencies.

依存関係を追加・更新するときに使います。

- Prefer standard library or existing dependencies first.
- If adding a dependency, explain why it is worth the maintenance cost.
- Run `cargo audit-deps` and `cargo deny check`.
- Update `deny.toml` only when the license or source policy is understood.

### Template Smoke Tester

Use when editing `scripts/new-from-template.sh`, `Cargo.toml`, or project naming behavior.

`scripts/new-from-template.sh`、`Cargo.toml`、project namingの挙動を変更するときに使います。

- Generate a fresh project under `~/src/template-smoke-cli`.
- Verify package name, crate import name, integration tests, and `cargo run`.
- Remove the smoke project after verification.

## Karpathy Guidelines

This template intentionally includes the community "andrej-karpathy-skills" style guardrails:

このテンプレートには、communityの "andrej-karpathy-skills" style guardrails を意図的に含めています。

- Think before coding.
- Prefer the simplest sufficient solution.
- Keep changes surgical.
- Drive work by verifiable goals.

Use these as behavior constraints, then apply the repository-specific `AGENTS.md` sections for actual commands and file ownership.

これらを行動制約として使い、実際のコマンドや編集範囲はリポジトリ固有の `AGENTS.md` に従います。
## Imported Ideas

- Use `AGENTS.md` as repo-specific wayfinding, not a generic manifesto.
- Give agents concrete commands, file ownership boundaries, and completion criteria.
- For autonomous loops, define the editable files, frozen files, metric, and keep/discard rule.
- Keep "behavioral" rules short: think first, prefer simplicity, make surgical changes, verify with tests.

日本語要約:

- `AGENTS.md` は一般論ではなく、このリポジトリ固有の道案内として扱います。
- agentには具体的なコマンド、編集してよい範囲、完了条件を与えます。
- 自律ループでは、編集対象、固定対象、評価指標、採用/破棄ルールを明確にします。
- 行動ルールは短く保ちます。考えてから書く、十分に単純な解を選ぶ、変更を小さくする、テストで検証する、という方針です。
