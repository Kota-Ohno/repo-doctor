# Contributing

## Development

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run
cargo test --doc
cargo audit
cargo deny check
typos
taplo fmt --check
```

If `just` is installed, run:

```bash
just check
```

## Pull Requests

- Keep changes focused.
- Add or update tests for behavior changes.
- Update README or AGENTS.md when workflows change.
- Run the verification commands before opening a PR.

## Template Changes

Changes to `scripts/new-from-template.sh`, `Cargo.toml`, crate naming, or test layout must validate a generated project:

```bash
just smoke-template
```
