# cargo-binstall

`cargo-binstall` can install binary crates when release assets match its target
conventions. `repo-doctor` release assets use this pattern:

```text
repo-doctor-{target}.tar.gz
repo-doctor-{target}.zip
```

Supported targets:

- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

Until published metadata is added, prefer the install scripts or GitHub Action.
