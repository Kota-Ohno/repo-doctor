# winget packaging

`winget` packages are submitted to `microsoft/winget-pkgs` after a release is
available. The release workflow publishes the Windows zip and checksum that the
manifest should reference.

Expected metadata:

- PackageIdentifier: `KotaOhno.repo-doctor`
- PackageName: `repo-doctor`
- Publisher: `Kota Ono`
- License: `MIT OR Apache-2.0`
- InstallerType: `zip`
- NestedInstallerType: `portable`
- NestedInstallerFiles: `repo-doctor.exe`

Use `wingetcreate update KotaOhno.repo-doctor --urls <release-zip-url>` once a
real release asset exists.
