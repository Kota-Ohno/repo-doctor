# repo-doctor roadmap

This roadmap is based on a quick comparison with GitHub Community Profile,
TODO Group Repolinter, OpenSSF Scorecard, and SCM security posture tools such
as Legitify.

## Positioning

`repo-doctor` should stay a local-first repository readiness checker for Rust
projects. It should complement broader tools instead of becoming a clone of
OpenSSF Scorecard or a full SCM security scanner.

## Product principles

- Keep `repo-doctor check <path>` useful offline.
- Prefer explainable structural checks over subjective prose judgments.
- Use stable rule IDs and machine-readable output from the start.
- Default to warnings for context-dependent community checks.
- Add GitHub API checks only behind an explicit remote mode.
- Integrate mature scanners instead of reimplementing deep vulnerability,
  secret, or license analysis.

## Backlog

### P0 - Tighten the local checker

- [x] Add regression tests that lock down check order, rule IDs, and JSON fields.
- [x] Add a JSON schema version or documented compatibility policy.
- [x] Add severity to checks.
- [x] Add remediation text to each check.
- [x] Add nonzero exit codes with `--fail-on warn`.
- [x] Validate the target path exists and is a directory.
- [x] Keep JSON output stable and document the report schema.
- [ ] Split check logic into focused modules instead of growing `src/lib.rs`.
- [x] Support common README names such as `README.md`, `README`, and `README.txt`.
- [x] Support common license names such as `LICENSE`, `LICENSE.md`, and `LICENSE.txt`.
- [x] Check that `.github/workflows` contains at least one `.yml` or `.yaml` file.

### P1 - Community health checks

- [x] Check for `CONTRIBUTING.md` in root, `docs`, or `.github`.
- [x] Check for `CODE_OF_CONDUCT.md` in root, `docs`, or `.github`.
- [x] Check for `SECURITY.md` in root, `docs`, or `.github`.
- [x] Check for issue templates under `.github/ISSUE_TEMPLATE`.
- [x] Check for `.github/pull_request_template.md` or equivalent templates.
- [x] Check for `CHANGELOG.md` or release notes.
- [ ] Validate issue template frontmatter enough to catch empty placeholders.

### P1 - Rust project hygiene

- [x] Parse `Cargo.toml` with a TOML parser instead of treating it as a file.
- [x] Check package metadata: `description`, `license` or `license-file`,
      `repository`, `readme`, `rust-version`.
- [x] Check `[package]` includes `name`, `version`, and `edition`.
- [x] Check paths referenced by `readme` and `license-file` exist.
- [x] Warn when binary crates lack `Cargo.lock`.
- [ ] Detect workspace roots and member crates.
- [x] Check `.gitignore` includes Rust build artifacts such as `/target`.
- [ ] Check README includes basic install, usage, and development commands.
- [ ] Check README command examples mention the package or binary name.

### P1 - GitHub Actions local checks

- [ ] Parse workflow YAML files under `.github/workflows`.
- [ ] Check CI runs on pull requests and pushes to the default branch.
- [ ] Check Rust CI includes `cargo fmt --check`, `cargo clippy`, and tests.
- [ ] Warn when workflow-level or job-level `permissions` is missing.
- [ ] Warn on risky `pull_request_target` usage.
- [ ] Warn on unpinned third-party actions.
- [ ] Detect Dependabot or Renovate configuration.
- [ ] Keep workflow content checks heuristic and warning-oriented until YAML
      parsing is mature.

### P2 - Configuration and profiles

- [ ] Add `repo-doctor.toml`.
- [ ] Allow rules to be disabled with a required rationale.
- [ ] Allow severity overrides by rule ID.
- [ ] Add profiles: `rust-cli`, `rust-lib`, `oss`, `internal`, `strict`.
- [ ] Add `repo-doctor init` to generate a starter config.

### P2 - Automation outputs

- [ ] Add `--format markdown` for PR comments and human reports.
- [ ] Add GitHub Actions annotation output.
- [ ] Add SARIF output after rule IDs and locations are stable.
- [ ] Add file paths and line numbers where parsers provide them.

### P3 - Remote GitHub checks

- [ ] Add an explicit `repo-doctor github owner/repo` command.
- [ ] Use GitHub community profile API as an optional comparison source.
- [ ] Check default branch protection status.
- [ ] Check repository description and topics.
- [ ] Check repository security features where the token has permission.
- [ ] Check recent maintenance activity.
- [ ] Add OpenSSF Scorecard comparison or link-out mode.

## Out of scope for now

- Full OpenSSF Scorecard reimplementation.
- Custom secret scanning.
- Custom vulnerability database scanning.
- Deep dependency license compatibility analysis.
- Organization-wide policy enforcement.

## References

- GitHub Community Profile:
  https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/about-community-profiles-for-public-repositories
- OpenSSF Scorecard:
  https://github.com/ossf/scorecard
- TODO Group Repolinter:
  https://github.com/todogroup/repolinter
- Legitify:
  https://github.com/Legit-Labs/legitify
