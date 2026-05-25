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
- `profile:<name>`: ecosystem-specific checks
- `remote`: GitHub API checks
