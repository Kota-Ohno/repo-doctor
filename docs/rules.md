# Rules / ルール

`repo-doctor list-rules` prints the current rule catalog with severity and
category.

`repo-doctor list-rules` で、現在のrule catalog、severity、categoryを確認できます。

```bash
repo-doctor list-rules
repo-doctor explain readme
```

## Disable Guidance / 無効化の方針

Disable a rule only when it does not apply to the repository. Always include a
reason so future maintainers can understand the exception.

ルールを無効化するのは、そのリポジトリに適用できない場合だけにしてください。将来のmaintainerが例外理由を理解できるよう、必ずreasonを書きます。

```toml
[[rules]]
id = "docker_healthcheck"
disabled = true
reason = "This image runs a short-lived CLI command, not a service."
```

## Categories / カテゴリ

- `core`: language-independent repository files
- `community`: contribution and community health files
- `ci`: GitHub Actions and dependency update checks
- `security`: lightweight local safety checks
- `guard`: AI/VibeCoding guardrails for Git diffs and agent instructions
- `profile:<name>`: ecosystem-specific checks
- `remote`: GitHub API checks

## Guard Rules / Guardルール

`repo-doctor guard` adds checks that are intentionally stricter than basic
readiness checks because they are meant to run before AI-generated changes land.

`repo-doctor guard` は、AIが生成した変更を取り込む前に実行する想定のため、通常のreadiness checkよりも差分に強く反応します。

- `guard_secret_added`: added secret-like files
- `guard_ci_modified`: changed CI or dependency automation
- `guard_guardrail_removed`: removed workflows, security files, or repo-doctor config
- `guard_tests_deleted`: deleted tests
- `guard_lockfile_sync`: changed dependency manifests without lockfile updates
- `guard_large_change_set`: unusually broad changes
- `agent_instructions`, `agent_verification`, `agent_boundaries`: `AGENTS.md` quality
