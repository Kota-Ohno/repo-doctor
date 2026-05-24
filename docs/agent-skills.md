# Agent Skills Notes

Useful agent guidance for this template should stay small, concrete, and verifiable.

このテンプレートのagent向けガイダンスは、小さく、具体的で、検証可能な内容に留めます。

## Recommended Local Skills

推奨するローカルskill:

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
