# AI Usage Specification / AI向け利用仕様

`repo-doctor` is designed to be operated by coding agents as well as humans.
Agents should prefer machine-readable commands first, then use markdown docs as
fallback context.

`repo-doctor` は人間だけでなくcoding agentが操作する前提で設計します。agentはまず機械可読なコマンドを優先し、必要に応じてMarkdown docsを補助情報として使います。

## Discovery / 発見

Use these commands before deciding how to operate the repository:

リポジトリへの操作方針を決める前に、次のコマンドで能力と手順を発見します。

```bash
repo-doctor spec --format json
repo-doctor recipes --format markdown
repo-doctor list-profiles
repo-doctor list-rules
repo-doctor config-explain
```

`spec --format json` is the primary machine-readable contract for agents. It
lists commands, supported profiles, rule IDs, output contracts, and recipes.

`spec --format json` はagent向けの主要な機械可読contractです。commands、supported profiles、rule IDs、output contracts、recipesを列挙します。

## Standard Agent Loop / 標準Agent Loop

```bash
repo-doctor check --format summary
repo-doctor recipes --format markdown
repo-doctor agent-guide --format markdown
repo-doctor guard --fail-on warn
```

Use `check` to understand repository readiness. Use `guard` before finishing a
coding task because it includes diff-aware checks for AI-generated changes.

`check` はrepository readinessの把握に使います。coding taskを完了する前には、AI生成差分向けの検査を含む `guard` を使います。

## AGENTS.md Generation / AGENTS.md生成

```bash
repo-doctor agent-guide --format markdown >> AGENTS.md
repo-doctor guard --warnings-only
```

`agent-guide` detects repository profiles and emits verification commands and
behavior constraints suitable for `AGENTS.md`.

`agent-guide` はrepository profileを検出し、`AGENTS.md` に貼れる検証コマンドと行動制約を出力します。

## CI / CI導入

```bash
repo-doctor ci --template generic > .github/workflows/repo-doctor.yml
repo-doctor ci --guard > .github/workflows/repo-doctor-guard.yml
```

The readiness workflow checks repository hygiene. The guard workflow checks
AI/VibeCoding risks in Git diffs.

readiness workflowはrepository hygieneを確認します。guard workflowはGit差分内のAI/VibeCodingリスクを確認します。

## Completion Criteria / 完了条件

An agent should not consider repository-changing work complete until this
command succeeds or every warning has a documented rationale:

agentは、次のコマンドが成功するか、すべてのwarningに明示的な理由が付くまで、リポジトリ変更作業を完了扱いにしないでください。

```bash
repo-doctor guard --fail-on warn
```
