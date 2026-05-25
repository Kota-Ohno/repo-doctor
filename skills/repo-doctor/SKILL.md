---
name: repo-doctor
description: Use repo-doctor as an AI-first repository readiness and VibeCoding guardrail system with machine-readable specs, recipes, AGENTS.md guidance, and diff checks.
---

# repo-doctor

Use this skill when a user asks whether a repository is ready to publish,
whether project metadata is complete, how to improve repository hygiene, or how
to guard AI/VibeCoding changes before merge.

このskillは、リポジトリを公開できる状態か、project metadataが十分か、
repository hygieneをどう改善するか、AI/VibeCoding変更をmerge前にどう守るかを
確認するときに使います。

## Workflow

1. Prefer an existing `repo-doctor` binary on `PATH`.
2. If it is missing, install the latest release with `scripts/install.sh` or
   `scripts/install.ps1`.
3. Confirm the user has repository read access for local checks.
4. Run `repo-doctor spec --format json` to learn the tool capabilities,
   supported profiles, stable output contracts, rules, and recipes.
5. Run `repo-doctor recipes --format markdown` and choose the recipe matching
   the user goal.
6. Run `repo-doctor check --format compact` first for a quick readiness signal.
7. Run `repo-doctor agent-guide --format markdown` when `AGENTS.md` is missing
   or too generic.
8. Before finishing any repository-changing work, run
   `repo-doctor guard --fail-on warn`.
9. Run `repo-doctor check --warnings-only` or
   `repo-doctor guard --warnings-only` when warnings exist.
10. For GitHub repository settings, run `repo-doctor github-auth-doctor` first.
    Continue with `repo-doctor github owner/repo` only when the user wants
    remote checks and the `gh` CLI is installed and authenticated.
11. For remote setup changes, confirm the user has repository admin permission
    or equivalent access.
12. Summarize warnings as actionable tasks. Do not modify files unless the user
    asks you to fix them.

日本語workflow:

1. まず `PATH` 上の既存 `repo-doctor` binaryを使います。
2. 見つからない場合は `scripts/install.sh` または `scripts/install.ps1` で
   latest releaseを導入します。
3. local checksのためにrepository read accessがあることを確認します。
4. `repo-doctor spec --format json` で capabilities、profiles、output
   contracts、rules、recipesを確認します。
5. `repo-doctor recipes --format markdown` でユーザーの目的に合うrecipeを選びます。
6. 最初に `repo-doctor check --format compact` でreadinessの概要を確認します。
7. `AGENTS.md` がない、または一般的すぎる場合は
   `repo-doctor agent-guide --format markdown` を実行します。
8. リポジトリを変更する作業を終える前に
   `repo-doctor guard --fail-on warn` を実行します。
9. warningがある場合は `repo-doctor check --warnings-only` または
   `repo-doctor guard --warnings-only` で絞り込みます。
10. GitHub repository settingsでは、まず `repo-doctor github-auth-doctor` を実行します。
    ユーザーがremote checksを求め、`gh` CLIが導入済みかつauthenticated済みの場合だけ
    `repo-doctor github owner/repo` を使います。
11. remote setup変更では、repository adminまたは同等の権限があることを確認します。
12. warningはactionable taskとして要約します。修正はユーザーが依頼した場合だけ行います。

## Completion Gate

Repository-changing work is not complete until this succeeds, or every warning
has a concrete rationale:

```bash
repo-doctor guard --fail-on warn
```

リポジトリを変更する作業は、このコマンドが成功するか、すべてのwarningに具体的な理由が付くまで完了扱いにしません。

## Useful Commands

```bash
repo-doctor spec --format json
repo-doctor recipes --format markdown
repo-doctor agent-guide --format markdown
repo-doctor check --format compact
repo-doctor check --warnings-only
repo-doctor github-auth-doctor
repo-doctor guard --fail-on warn
repo-doctor guard --warnings-only
repo-doctor check --format markdown
repo-doctor guard --format sarif --output repo-doctor-guard.sarif
repo-doctor ci --guard
repo-doctor explain readme
repo-doctor config-validate repo-doctor.toml
```

## Output Preference

Prefer JSON or compact output when making automated decisions. Use Markdown for
user-facing summaries and AGENTS.md snippets.

自動判断にはJSONまたはcompact outputを優先します。ユーザー向け要約や
`AGENTS.md` に貼る内容にはMarkdownを使います。
