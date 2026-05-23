# repo-doctor roadmap

This roadmap is based on a quick comparison with GitHub Community Profile,
TODO Group Repolinter, OpenSSF Scorecard, and SCM security posture tools such
as Legitify.

## Positioning

`repo-doctor` should stay a local-first generic repository readiness checker
with first-class ecosystem profiles. Rust remains a first-class profile, but it
is no longer the whole product scope. The tool should complement broader tools
instead of becoming a clone of OpenSSF Scorecard or a full SCM security scanner.

## Product principles

- Keep `repo-doctor check <path>` useful offline.
- Run language-independent core checks for every repository.
- Auto-detect ecosystem checks from local files.
- Do not warn for ecosystems that are not present unless a profile is selected.
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
- [x] Split check logic into focused modules instead of growing `src/lib.rs`.
- [x] Add `--profile generic|auto|rust|node|python|go|docker|jvm|deno|bun|dotnet|php|ruby|cpp`.
- [x] Default `--profile` to `auto`.
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

### P1 - Ecosystem profiles

- [x] Detect Rust by `Cargo.toml`.
- [x] Detect Node.js by `package.json`.
- [x] Detect Python by `pyproject.toml`, `setup.py`, or `requirements.txt`.
- [x] Detect Go by `go.mod`.
- [x] Detect Docker/container projects by Dockerfile, Containerfile, Compose,
      or `.dockerignore`.
- [x] Detect JVM projects by Maven or Gradle build files.
- [x] Detect Deno, Bun, .NET, PHP, Ruby, and C/C++ projects.
- [x] Ensure `--profile generic` runs core checks only.

### P1 - Rust project hygiene

- [x] Parse `Cargo.toml` with a TOML parser instead of treating it as a file.
- [x] Check package metadata: `description`, `license` or `license-file`,
      `repository`, `readme`, `rust-version`.
- [x] Check `[package]` includes `name`, `version`, and `edition`.
- [x] Check paths referenced by `readme` and `license-file` exist.
- [x] Warn when binary crates lack `Cargo.lock`.
- [x] Detect workspace roots and member crates.
- [x] Check `.gitignore` includes Rust build artifacts such as `/target`.
- [ ] Check README includes basic install, usage, and development commands.
- [ ] Check README command examples mention the package or binary name.

### P1 - Node.js project hygiene

- [x] Parse `package.json` with `serde_json`.
- [x] Check `name`, `version`, `description`, `license`, and `repository`.
- [x] Check `scripts.test`.
- [x] Check `engines.node`.
- [x] Check for a package manager lockfile.

### P1 - Python project hygiene

- [x] Detect `pyproject.toml`, `setup.py`, or `requirements.txt`.
- [x] Parse `pyproject.toml` when present.
- [x] Check PEP 621 project metadata.
- [x] Check `build-system`.
- [x] Check for lockfile or requirements candidates.

### P1 - Go project hygiene

- [x] Detect `go.mod`.
- [x] Check module declaration.
- [x] Check Go version directive.
- [x] Check `go.sum`.

### P1 - Docker/container hygiene

- [x] Detect Dockerfile, Containerfile, Compose files, or `.dockerignore`.
- [x] Check container build file presence.
- [x] Check `.dockerignore`.
- [x] Check Compose file presence.
- [x] Warn on base images that use `:latest`.

### P1 - Java/JVM hygiene

- [x] Detect Maven or Gradle build files.
- [x] Check Maven/Gradle wrapper presence.
- [x] Check Maven `groupId`, `artifactId`, and `version`.
- [x] Check Gradle settings, group, version, and test task hints.

### P1 - Additional high-share ecosystem profiles

- [x] Deno: config, lockfile, and tasks.
- [x] Bun: lockfile, package manager metadata, package name, and test script.
- [x] .NET: solution/project files, SDK pinning, and test project hints.
- [x] PHP: Composer metadata, requirements, test script, and lockfile.
- [x] Ruby: Gemfile, Gemfile.lock, and gemspec hints.
- [x] C/C++: build system, tooling metadata, and dependency manifest hints.

### P1 - GitHub Actions local checks

- [ ] Parse workflow YAML files under `.github/workflows`.
- [x] Check CI runs on pull requests and pushes.
- [ ] Check Rust CI includes `cargo fmt --check`, `cargo clippy`, and tests.
- [x] Warn when workflow-level or job-level `permissions` is missing.
- [x] Warn on risky `pull_request_target` usage.
- [x] Warn on floating third-party action refs such as `main`, `master`, or
      `latest`.
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
