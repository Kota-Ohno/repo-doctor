# Release Management

Use this checklist before cutting or publishing a release.

releaseを切る前、または公開する前にこのchecklistを使います。

## Version And Tags

Versionとtag:

- Keep `Cargo.toml`, `Cargo.lock`, `packaging/npm/package.json`, and the
  default version in `action.yml` aligned.
- Release tags use `vMAJOR.MINOR.PATCH`, for example `v0.1.1`.
- `release-plz` owns normal version bumps, changelog updates, GitHub releases,
  and tags for merged release PRs.
- Manual tags should point at the commit that contains the matching version
  bump and changelog entry.

日本語要約:

- `Cargo.toml`, `Cargo.lock`, `packaging/npm/package.json`, `action.yml` のdefault versionを揃えます。
- release tagは `vMAJOR.MINOR.PATCH` 形式です。例: `v0.1.1`
- 通常のversion bump、changelog更新、GitHub release、tag作成は `release-plz` に任せます。
- manual tagは、対応するversion bumpとchangelog entryを含むcommitを指すようにします。

## Preflight

Run the local distribution preflight before pushing a release tag:

release tagをpushする前に、local distribution preflightを実行します。

```bash
scripts/profile-smoke.sh
scripts/distribution-smoke.sh
scripts/release-preflight.sh
```

When validating an already-created local tag:

作成済みのlocal tagを検証する場合:

```bash
scripts/release-preflight.sh --require-tag
```

The smoke scripts exercise generated profile fixtures, CI snippets, install
surfaces, the npm wrapper, and optional Docker usage before the heavier release
preflight runs. The preflight builds a release binary, creates local release-like assets,
verifies checksums, exercises `scripts/install.sh` through `file://` assets,
exercises the npm wrapper installer, checks version consistency, and runs
`repo-doctor guard`.

smoke scriptは、重いrelease preflightの前にprofile fixture、CI snippet、install surface、npm wrapper、任意のDocker利用を確認します。このpreflightはrelease binaryをbuildし、local release相当のassetsを作成し、checksumを検証し、`file://` assets経由で `scripts/install.sh` とnpm wrapper installerを実行し、version整合性を確認し、`repo-doctor guard` を実行します。

If Node.js is installed outside `PATH`, pass it explicitly:

Node.jsが `PATH` 外にある場合は明示します。

```bash
NODE=/path/to/node scripts/release-preflight.sh
```

## Publish Flow

公開手順:

1. Merge the release PR produced by `release-plz`.
2. Confirm the `vMAJOR.MINOR.PATCH` tag exists.
3. Confirm `.github/workflows/release.yml` uploads archives and `.sha256`
   files for every supported release target.
4. Confirm `.github/workflows/container.yml` publishes GHCR tags for the same
   release.
5. Run install smoke tests from a clean directory:

```bash
tmp="$(mktemp -d)"
curl -fsSL https://raw.githubusercontent.com/Kota-Ohno/repo-doctor/main/scripts/install.sh | sh -s -- --version v0.1.1 --dir "$tmp/bin"
"$tmp/bin/repo-doctor" --version
"$tmp/bin/repo-doctor" check --format compact
```

日本語要約:

1. `release-plz` が作成したrelease PRをmergeします。
2. `vMAJOR.MINOR.PATCH` tagが存在することを確認します。
3. `.github/workflows/release.yml` が全supported targetのarchiveと `.sha256` をuploadしたことを確認します。
4. `.github/workflows/container.yml` が同じreleaseのGHCR tagをpublishしたことを確認します。
5. clean directoryからinstall smoke testを実行します。

## Retry And Rollback

Retryとrollback:

- Re-run the release workflow with the target tag if binary assets are missing.
- Re-run the container workflow with the target tag if GHCR images are missing.
- Do not move a published tag unless the release is known to be unusable and no
  package manager has consumed it.
- If an install path fails, fix the script or package wrapper first, then
  publish a patch release.

日本語要約:

- binary assetが足りない場合は、対象tagでrelease workflowを再実行します。
- GHCR imageが足りない場合は、対象tagでcontainer workflowを再実行します。
- 公開済みtagは、releaseが利用不能で、かつpackage managerに取り込まれていないと分かる場合を除き、動かしません。
- install pathが失敗する場合はscriptまたはpackage wrapperを先に直し、patch releaseを公開します。
