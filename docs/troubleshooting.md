# Troubleshooting / トラブルシュート

## `repo-doctor` is not found

`repo-doctor` が見つからない場合は、install先のdirectoryを `PATH` に追加してください。

```bash
export PATH="$HOME/.local/bin:$PATH"
```

On Windows PowerShell:

Windows PowerShellの場合:

```powershell
$env:PATH = "$HOME\.repo-doctor\bin;$env:PATH"
repo-doctor --version
```

## Install directory permission denied

If the install directory is not writable, install into a user-owned directory.
Avoid `sudo` unless you intentionally want a system-wide install.

install先directoryに書き込めない場合は、ユーザー所有のdirectoryへ導入してください。system-wide installを意図している場合を除き、`sudo` は避けます。

```bash
curl -fsSL https://raw.githubusercontent.com/Kota-Ohno/repo-doctor/main/scripts/install.sh | sh -s -- --dir "$HOME/.local/bin"
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

If `repo-doctor github-auth-doctor` reports `gh_cli=missing`, install GitHub
CLI first. If it reports `gh_auth=failed`, run `gh auth login`. If repository
or API checks are unavailable after login, confirm that the token can read the
target repository. Private repositories usually need the `repo` scope. Public
read checks can often work with `public_repo`. Setup commands such as
`repo-doctor github-setup` may require repository admin access.

`repo-doctor github-auth-doctor` が `gh_cli=missing` を返す場合は、先にGitHub CLIを導入します。`gh_auth=failed` の場合は `gh auth login` を実行します。login後もrepository/API checksがunavailableの場合は、tokenが対象repositoryを読めるか確認してください。private repositoryでは通常 `repo` scopeが必要です。public repositoryの読み取りcheckは `public_repo` で足りることがあります。`repo-doctor github-setup` のような設定変更コマンドはrepository admin権限を必要とする場合があります。

```bash
gh auth refresh -s repo
repo-doctor github-auth-doctor
```

## GitHub API permission denied

Branch protection and vulnerability alert APIs may be hidden when the token
lacks permission, even if normal repository metadata is visible.

通常のrepository metadataが見えていても、token権限が足りない場合はbranch protectionやvulnerability alert APIが見えないことがあります。

```bash
repo-doctor github-auth-doctor
gh auth refresh -s repo
```

## GitHub Action cannot find a release asset

指定した `version` のrelease assetが存在するか確認してください。まずは `version: latest` で試せます。

```yaml
- uses: Kota-Ohno/repo-doctor@v0.1.1
  with:
    version: latest
    fail-on: warn
```
