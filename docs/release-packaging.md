# Release Packaging / リリース配布

Release assets are produced by `.github/workflows/release.yml`.

release assetは `.github/workflows/release.yml` で生成されます。

After a release:

release後に確認すること:

- Check that binary archives and `.sha256` files exist.
- Update Homebrew and Scoop checksums from the release assets.
- Publish or update npm wrapper metadata when the package is ready.
- Confirm GHCR image tags from `.github/workflows/container.yml`.

Current scaffold files:

- `packaging/homebrew/repo-doctor.rb`
- `packaging/scoop/repo-doctor.json`
- `packaging/npm/`
- `packaging/winget/README.md`
- `packaging/binstall/README.md`

Helper:

```bash
scripts/update-packaging-checksums.sh v0.1.1
```
