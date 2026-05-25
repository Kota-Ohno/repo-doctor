# Troubleshooting / トラブルシュート

## `repo-doctor` is not found

`repo-doctor` が見つからない場合は、install先のdirectoryを `PATH` に追加してください。

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Checksum mismatch

Downloadが途中で壊れているか、release assetが更新中の可能性があります。再実行しても失敗する場合は、対象versionを固定してください。

```bash
curl -fsSL https://raw.githubusercontent.com/Kota-Ohno/repo-doctor/main/scripts/install.sh | sh -s -- --version v0.1.1
```

## Docker socket permission denied

Docker socket権限がない場合は、ユーザーを `docker` groupに追加し、shell/WSLを再起動してください。

```bash
sudo usermod -aG docker "$USER"
```

## `gh` authentication fails

remote checksには `gh` CLIの認証が必要です。

```bash
gh auth login
repo-doctor github-auth-doctor
```

## GitHub Action cannot find a release asset

指定した `version` のrelease assetが存在するか確認してください。まずは `version: latest` で試せます。

```yaml
- uses: Kota-Ohno/repo-doctor@v0.1.1
  with:
    version: latest
    fail-on: warn
```
