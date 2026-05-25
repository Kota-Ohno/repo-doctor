use std::collections::{BTreeSet, HashSet};
use std::path::Path;
use std::process::Command;

use crate::profiles::Profile;
use crate::report::{Check, pass, warn};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ChangedFile {
    status: ChangeStatus,
    path: String,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum ChangeStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

pub(crate) fn inspect(path: &Path, base: Option<&str>, profiles: &[Profile]) -> Vec<Check> {
    let mut checks = Vec::new();
    let changes = changed_files(path, base);

    let Ok(changes) = changes else {
        return vec![warn(
            "guard_git_diff",
            "Git diff information is unavailable",
            "Run guard mode inside a Git worktree, or pass a valid --base ref.",
        )];
    };

    checks.push(check_diff_available(&changes));
    checks.push(check_large_change_set(&changes));
    checks.push(check_secret_like_additions(&changes));
    checks.push(check_ci_changes(&changes));
    checks.push(check_removed_guardrail_files(&changes));
    checks.push(check_deleted_tests(&changes));
    checks.push(check_manifest_lockfile_sync(&changes));
    checks.push(check_source_changes_have_tests(path, &changes));
    checks.push(check_build_or_task_changes(&changes));
    checks.push(check_generated_artifact_additions(&changes));
    checks.extend(check_agent_instructions(path, profiles));
    checks
}

fn changed_files(path: &Path, base: Option<&str>) -> std::io::Result<Vec<ChangedFile>> {
    let mut changes = HashSet::new();
    let mut saw_successful_git_command = false;

    if let Some(base) = base {
        let output = Command::new("git")
            .args(["-C"])
            .arg(path)
            .args([
                "diff",
                "--name-status",
                "--find-renames",
                &format!("{base}...HEAD"),
            ])
            .output()?;
        if output.status.success() {
            saw_successful_git_command = true;
            parse_name_status(&String::from_utf8_lossy(&output.stdout), &mut changes);
        }
    }

    for args in [
        &["diff", "--name-status", "--find-renames"][..],
        &["diff", "--cached", "--name-status", "--find-renames"][..],
    ] {
        let output = Command::new("git")
            .args(["-C"])
            .arg(path)
            .args(args)
            .output()?;
        if output.status.success() {
            saw_successful_git_command = true;
            parse_name_status(&String::from_utf8_lossy(&output.stdout), &mut changes);
        }
    }

    let output = Command::new("git")
        .args(["-C"])
        .arg(path)
        .args(["status", "--porcelain"])
        .output()?;
    if output.status.success() {
        saw_successful_git_command = true;
        parse_porcelain_status(&String::from_utf8_lossy(&output.stdout), &mut changes);
    }

    let output = Command::new("git")
        .args(["-C"])
        .arg(path)
        .args(["ls-files", "--others", "--exclude-standard"])
        .output()?;
    if output.status.success() {
        saw_successful_git_command = true;
        for file in String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| !line.trim().is_empty())
        {
            changes.insert(ChangedFile {
                status: ChangeStatus::Added,
                path: normalize_path(file),
            });
        }
    }

    if !saw_successful_git_command {
        return Err(std::io::Error::other("not a Git worktree"));
    }

    let mut changes = changes.into_iter().collect::<Vec<_>>();
    changes.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(changes)
}

fn parse_name_status(output: &str, changes: &mut HashSet<ChangedFile>) {
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let fields = line.split('\t').collect::<Vec<_>>();
        let Some(status_field) = fields.first() else {
            continue;
        };
        let status = match status_field.chars().next() {
            Some('A') => ChangeStatus::Added,
            Some('D') => ChangeStatus::Deleted,
            Some('R') => ChangeStatus::Renamed,
            Some('M') | Some('C') | Some('T') => ChangeStatus::Modified,
            _ => ChangeStatus::Modified,
        };
        let path = if matches!(status, ChangeStatus::Renamed) && fields.len() >= 3 {
            fields[2]
        } else if fields.len() >= 2 {
            fields[1]
        } else {
            continue;
        };
        changes.insert(ChangedFile {
            status,
            path: normalize_path(path),
        });
    }
}

fn parse_porcelain_status(output: &str, changes: &mut HashSet<ChangedFile>) {
    for line in output.lines().filter(|line| line.len() >= 4) {
        let status_code = &line[..2];
        let path = line[3..].split(" -> ").last().unwrap_or(&line[3..]);
        let status = if status_code.contains('D') {
            ChangeStatus::Deleted
        } else if status_code == "??" || status_code.contains('A') {
            ChangeStatus::Added
        } else if status_code.contains('R') {
            ChangeStatus::Renamed
        } else {
            ChangeStatus::Modified
        };
        changes.insert(ChangedFile {
            status,
            path: normalize_path(path),
        });
    }
}

fn normalize_path(path: &str) -> String {
    path.trim().trim_matches('"').replace('\\', "/")
}

fn check_diff_available(changes: &[ChangedFile]) -> Check {
    if changes.is_empty() {
        pass("guard_git_diff", "No Git changes detected for guard mode")
    } else {
        pass(
            "guard_git_diff",
            format!("Guard mode inspected {} changed file(s)", changes.len()),
        )
    }
}

fn check_large_change_set(changes: &[ChangedFile]) -> Check {
    const LARGE_CHANGE_THRESHOLD: usize = 100;
    if changes.len() <= LARGE_CHANGE_THRESHOLD {
        return pass(
            "guard_large_change_set",
            "Changed file count is within the guard threshold",
        );
    }

    warn(
        "guard_large_change_set",
        format!(
            "Guard mode found {} changed files, which is larger than the review threshold",
            changes.len()
        ),
        "Split broad generated or mechanical changes from hand-written changes before review.",
    )
}

fn check_secret_like_additions(changes: &[ChangedFile]) -> Check {
    let risky = changes
        .iter()
        .filter(|change| matches!(change.status, ChangeStatus::Added | ChangeStatus::Renamed))
        .filter(|change| is_secret_like_path(&change.path))
        .map(|change| change.path.as_str())
        .collect::<Vec<_>>();

    if risky.is_empty() {
        return pass(
            "guard_secret_added",
            "No newly added secret-like files were detected",
        );
    }

    warn(
        "guard_secret_added",
        format!("Secret-like files were added: {}", risky.join(", ")),
        "Remove committed secrets and keep only sanitized examples such as .env.example.",
    )
    .with_location(risky[0], None)
}

fn is_secret_like_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    let name = lower.rsplit('/').next().unwrap_or(&lower);
    matches!(
        name,
        ".env"
            | ".env.local"
            | ".env.production"
            | "secrets.json"
            | "secret.json"
            | "credentials.json"
            | "id_rsa"
            | "id_dsa"
    ) || lower.ends_with(".pem")
        || lower.ends_with(".key")
        || lower.ends_with(".p12")
}

fn check_ci_changes(changes: &[ChangedFile]) -> Check {
    let ci_changes = changes
        .iter()
        .filter(|change| {
            change.path.starts_with(".github/workflows/")
                || change.path == ".github/dependabot.yml"
                || change.path == ".github/dependabot.yaml"
                || change.path == "renovate.json"
                || change.path == ".github/renovate.json"
        })
        .map(|change| change.path.as_str())
        .collect::<Vec<_>>();

    if ci_changes.is_empty() {
        return pass(
            "guard_ci_modified",
            "CI and dependency update guardrails were not changed",
        );
    }

    warn(
        "guard_ci_modified",
        format!(
            "CI or dependency update files changed: {}",
            ci_changes.join(", ")
        ),
        "Review workflow permissions, triggers, and quality gates whenever automation changes.",
    )
    .with_location(ci_changes[0], None)
}

fn check_removed_guardrail_files(changes: &[ChangedFile]) -> Check {
    let removed = changes
        .iter()
        .filter(|change| matches!(change.status, ChangeStatus::Deleted))
        .filter(|change| is_guardrail_file(&change.path))
        .map(|change| change.path.as_str())
        .collect::<Vec<_>>();

    if removed.is_empty() {
        return pass(
            "guard_guardrail_removed",
            "No repository guardrail files were removed",
        );
    }

    warn(
        "guard_guardrail_removed",
        format!(
            "Repository guardrail files were removed: {}",
            removed.join(", ")
        ),
        "Restore removed guardrails or document the replacement in the same change.",
    )
    .with_location(removed[0], None)
}

fn is_guardrail_file(path: &str) -> bool {
    matches!(
        path,
        "SECURITY.md"
            | ".github/SECURITY.md"
            | "CODEOWNERS"
            | ".github/CODEOWNERS"
            | ".github/dependabot.yml"
            | ".github/dependabot.yaml"
            | "renovate.json"
            | ".github/renovate.json"
            | "repo-doctor.toml"
    ) || path.starts_with(".github/workflows/")
}

fn check_deleted_tests(changes: &[ChangedFile]) -> Check {
    let deleted_tests = changes
        .iter()
        .filter(|change| matches!(change.status, ChangeStatus::Deleted))
        .filter(|change| is_test_path(&change.path))
        .map(|change| change.path.as_str())
        .collect::<Vec<_>>();

    if deleted_tests.is_empty() {
        return pass("guard_tests_deleted", "No test files were deleted");
    }

    warn(
        "guard_tests_deleted",
        format!("Test files were deleted: {}", deleted_tests.join(", ")),
        "Keep coverage or replace deleted tests with equivalent checks in the same change.",
    )
    .with_location(deleted_tests[0], None)
}

fn is_test_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.starts_with("tests/")
        || lower.contains("/tests/")
        || lower.ends_with("_test.go")
        || lower.ends_with(".test.ts")
        || lower.ends_with(".test.tsx")
        || lower.ends_with(".spec.ts")
        || lower.ends_with(".spec.tsx")
        || lower.ends_with("_test.py")
        || lower.ends_with("test.rs")
}

fn check_manifest_lockfile_sync(changes: &[ChangedFile]) -> Check {
    let changed_paths = changed_non_deleted_paths(changes);
    let mut unsynced = Vec::new();

    for rule in manifest_lockfile_rules() {
        let changed_manifests = changed_paths
            .iter()
            .filter(|path| rule.matches_manifest(path))
            .copied()
            .collect::<Vec<_>>();
        for manifest in changed_manifests {
            let dir = parent_dir(manifest);
            if !changed_paths
                .iter()
                .any(|path| rule.matches_lockfile_for_dir(path, dir))
            {
                unsynced.push(manifest);
            }
        }
    }

    if unsynced.is_empty() {
        return pass(
            "guard_lockfile_sync",
            "Changed manifests have matching lockfile updates when expected",
        );
    }

    warn(
        "guard_lockfile_sync",
        format!(
            "Dependency manifests changed without matching lockfile updates: {}",
            unsynced.join(", ")
        ),
        "Update the relevant lockfile in the same change, or document why the project intentionally has none.",
    )
    .with_location(unsynced[0], None)
}

struct LockfileRule {
    manifests: &'static [&'static str],
    lockfiles: &'static [&'static str],
}

impl LockfileRule {
    fn matches_manifest(&self, path: &str) -> bool {
        self.manifests
            .iter()
            .any(|manifest| path == *manifest || path.ends_with(&format!("/{manifest}")))
    }

    fn matches_lockfile_for_dir(&self, path: &str, dir: &str) -> bool {
        self.lockfiles
            .iter()
            .any(|lockfile| path == join_path(dir, lockfile))
    }
}

fn manifest_lockfile_rules() -> Vec<LockfileRule> {
    vec![
        LockfileRule {
            manifests: &["Cargo.toml"],
            lockfiles: &["Cargo.lock"],
        },
        LockfileRule {
            manifests: &["package.json"],
            lockfiles: &[
                "package-lock.json",
                "npm-shrinkwrap.json",
                "yarn.lock",
                "pnpm-lock.yaml",
                "bun.lock",
                "bun.lockb",
            ],
        },
        LockfileRule {
            manifests: &["pyproject.toml", "Pipfile"],
            lockfiles: &["uv.lock", "poetry.lock", "Pipfile.lock", "pdm.lock"],
        },
        LockfileRule {
            manifests: &["requirements.txt"],
            lockfiles: &["requirements.lock", "requirements.txt"],
        },
        LockfileRule {
            manifests: &["go.mod"],
            lockfiles: &["go.sum"],
        },
        LockfileRule {
            manifests: &["deno.json", "deno.jsonc"],
            lockfiles: &["deno.lock"],
        },
        LockfileRule {
            manifests: &["composer.json"],
            lockfiles: &["composer.lock"],
        },
        LockfileRule {
            manifests: &["Gemfile"],
            lockfiles: &["Gemfile.lock"],
        },
        LockfileRule {
            manifests: &["Package.swift"],
            lockfiles: &["Package.resolved"],
        },
        LockfileRule {
            manifests: &["packages.config", "Directory.Packages.props"],
            lockfiles: &["packages.lock.json"],
        },
        LockfileRule {
            manifests: &["main.tf", "providers.tf", "versions.tf"],
            lockfiles: &[".terraform.lock.hcl"],
        },
    ]
}

fn check_source_changes_have_tests(path: &Path, changes: &[ChangedFile]) -> Check {
    let source_changes = changes
        .iter()
        .filter(|change| !matches!(change.status, ChangeStatus::Deleted))
        .filter(|change| is_source_path(&change.path))
        .map(|change| change.path.as_str())
        .collect::<Vec<_>>();

    if source_changes.is_empty() {
        return pass(
            "guard_source_tests",
            "No source changes require test changes",
        );
    }

    let test_changed = changes.iter().any(|change| {
        !matches!(change.status, ChangeStatus::Deleted)
            && (is_test_path(&change.path) || changed_file_contains_test(path, &change.path))
    });
    if test_changed {
        return pass(
            "guard_source_tests",
            "Source changes include matching test updates",
        );
    }

    warn(
        "guard_source_tests",
        format!(
            "Source files changed without matching test changes: {}",
            source_changes.join(", ")
        ),
        "Add or update tests in the same change, or document why existing coverage is sufficient.",
    )
    .with_location(source_changes[0], None)
}

fn is_source_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    if lower.starts_with("docs/")
        || lower.starts_with("tests/")
        || lower.contains("/tests/")
        || lower.starts_with("examples/")
        || lower.starts_with("vendor/")
        || lower.contains("/vendor/")
    {
        return false;
    }

    [
        ".rs", ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".py", ".go", ".java", ".kt", ".kts",
        ".cs", ".fs", ".vb", ".php", ".rb", ".c", ".cc", ".cpp", ".cxx", ".h", ".hpp", ".swift",
        ".tf", ".tfvars",
    ]
    .iter()
    .any(|extension| lower.ends_with(extension))
}

fn changed_file_contains_test(root: &Path, path: &str) -> bool {
    let Ok(contents) = std::fs::read_to_string(root.join(path)) else {
        return false;
    };
    let lower = contents.to_ascii_lowercase();
    lower.contains("#[test]")
        || lower.contains("describe(")
        || lower.contains("it(")
        || lower.contains("test(")
        || lower.contains("def test_")
        || lower.contains("func test")
        || lower.contains("@test")
}

fn check_build_or_task_changes(changes: &[ChangedFile]) -> Check {
    let changed = changes
        .iter()
        .filter(|change| !matches!(change.status, ChangeStatus::Deleted))
        .filter(|change| is_build_or_task_path(&change.path))
        .map(|change| change.path.as_str())
        .collect::<Vec<_>>();

    if changed.is_empty() {
        return pass(
            "guard_build_logic_modified",
            "Build, package, and task definitions were not changed",
        );
    }

    warn(
        "guard_build_logic_modified",
        format!("Build or task definitions changed: {}", changed.join(", ")),
        "Review install, build, publish, and test command changes carefully before merging AI-generated edits.",
    )
    .with_location(changed[0], None)
}

fn is_build_or_task_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    let name = lower.rsplit('/').next().unwrap_or(&lower);
    matches!(
        name,
        "package.json"
            | "deno.json"
            | "deno.jsonc"
            | "bunfig.toml"
            | "cargo.toml"
            | "pyproject.toml"
            | "setup.py"
            | "go.mod"
            | "pom.xml"
            | "build.gradle"
            | "build.gradle.kts"
            | "composer.json"
            | "gemfile"
            | "rakefile"
            | "cmakelists.txt"
            | "makefile"
            | "meson.build"
            | "package.swift"
            | "global.json"
            | "directory.build.props"
            | "directory.packages.props"
    )
}

fn check_generated_artifact_additions(changes: &[ChangedFile]) -> Check {
    let added = changes
        .iter()
        .filter(|change| matches!(change.status, ChangeStatus::Added | ChangeStatus::Renamed))
        .filter(|change| is_generated_or_binary_artifact(&change.path))
        .map(|change| change.path.as_str())
        .collect::<Vec<_>>();

    if added.is_empty() {
        return pass(
            "guard_generated_artifact_added",
            "No generated, vendor, or binary artifacts were added",
        );
    }

    warn(
        "guard_generated_artifact_added",
        format!("Generated or binary artifacts were added: {}", added.join(", ")),
        "Keep generated output, vendored dependencies, and binaries out of review unless explicitly intended.",
    )
    .with_location(added[0], None)
}

fn is_generated_or_binary_artifact(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.starts_with("dist/")
        || lower.starts_with("build/")
        || lower.starts_with("target/")
        || lower.starts_with("vendor/")
        || lower.contains("/vendor/")
        || lower.contains("/node_modules/")
        || lower.ends_with(".min.js")
        || lower.ends_with(".wasm")
        || lower.ends_with(".exe")
        || lower.ends_with(".dll")
        || lower.ends_with(".so")
        || lower.ends_with(".dylib")
        || lower.ends_with(".zip")
        || lower.ends_with(".tar.gz")
        || lower.ends_with(".jar")
        || lower.ends_with(".class")
        || lower.ends_with(".pyc")
}

fn check_agent_instructions(path: &Path, profiles: &[Profile]) -> Vec<Check> {
    let mut checks = Vec::new();
    let agent_path = path.join("AGENTS.md");
    if !agent_path.exists() {
        return vec![
            warn(
                "agent_instructions",
                "AGENTS.md is missing",
                "Add AGENTS.md with repository-specific guidance for coding agents.",
            ),
            warn(
                "agent_verification",
                "Agent verification commands are not documented",
                "Document the exact commands agents must run before finishing work.",
            ),
            warn(
                "agent_boundaries",
                "Agent editing boundaries are not documented",
                "Document files or areas agents may edit, must avoid, or must ask before changing.",
            ),
            warn(
                "agent_profile_verification",
                "Agent profile-specific verification commands are not documented",
                "Document verification commands for each detected ecosystem profile.",
            ),
        ];
    }

    let contents = std::fs::read_to_string(&agent_path).unwrap_or_default();
    checks.push(pass("agent_instructions", "AGENTS.md is present"));

    let lower = contents.to_ascii_lowercase();
    if [
        "cargo test",
        "npm test",
        "pytest",
        "go test",
        "verify",
        "verification",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        checks.push(pass(
            "agent_verification",
            "AGENTS.md documents verification expectations",
        ));
    } else {
        checks.push(
            warn(
                "agent_verification",
                "AGENTS.md does not document concrete verification commands",
                "Add exact test, lint, or smoke-test commands agents should run.",
            )
            .with_location("AGENTS.md", None),
        );
    }

    if [
        "do not",
        "never",
        "avoid",
        "ask before",
        "ownership",
        "editable",
        "scope",
        "boundary",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        checks.push(pass(
            "agent_boundaries",
            "AGENTS.md documents editing boundaries",
        ));
    } else {
        checks.push(
            warn(
                "agent_boundaries",
                "AGENTS.md does not document editing boundaries",
                "Add repository-specific ownership, frozen files, or ask-before-changing rules.",
            )
            .with_location("AGENTS.md", None),
        );
    }

    checks.push(check_agent_profile_verification(&lower, profiles));
    checks
}

fn check_agent_profile_verification(contents: &str, profiles: &[Profile]) -> Check {
    let missing = profiles
        .iter()
        .filter(|profile| {
            !profile_verification_terms(**profile)
                .iter()
                .any(|term| contents.contains(term))
        })
        .map(|profile| profile.name())
        .collect::<Vec<_>>();

    if missing.is_empty() {
        return pass(
            "agent_profile_verification",
            "AGENTS.md documents verification commands for detected profiles",
        );
    }

    warn(
        "agent_profile_verification",
        format!(
            "AGENTS.md lacks verification commands for detected profiles: {}",
            missing.join(", ")
        ),
        "Add profile-specific commands such as cargo test, npm test, pytest, go test, docker build, terraform validate, or docs builds.",
    )
    .with_location("AGENTS.md", None)
}

fn profile_verification_terms(profile: Profile) -> &'static [&'static str] {
    match profile {
        Profile::Rust => &["cargo test", "cargo clippy"],
        Profile::Node | Profile::Frontend => {
            &["npm test", "pnpm test", "yarn test", "npm run build"]
        }
        Profile::Python => &["pytest", "python -m pytest"],
        Profile::Go => &["go test"],
        Profile::Docker => &["docker build", "hadolint"],
        Profile::Jvm | Profile::Kotlin => &["mvn test", "gradle test", "./gradlew test"],
        Profile::Deno => &["deno test", "deno task"],
        Profile::Bun => &["bun test"],
        Profile::Dotnet => &["dotnet test"],
        Profile::Php => &["composer test", "vendor/bin/phpunit", "phpunit"],
        Profile::Ruby => &["bundle exec", "rake test", "rspec"],
        Profile::Cpp => &["ctest", "cmake --build", "make test"],
        Profile::Swift => &["swift test"],
        Profile::Iac => &["terraform validate", "tofu validate", "terraform fmt"],
        Profile::Docs => &[
            "mkdocs build",
            "mdbook build",
            "docusaurus build",
            "vitepress build",
        ],
        Profile::Auto | Profile::Generic => &[],
    }
}

fn changed_non_deleted_paths(changes: &[ChangedFile]) -> BTreeSet<&str> {
    changes
        .iter()
        .filter(|change| !matches!(change.status, ChangeStatus::Deleted))
        .map(|change| change.path.as_str())
        .collect()
}

fn parent_dir(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(dir, _)| dir)
}

fn join_path(dir: &str, file: &str) -> String {
    if dir.is_empty() {
        file.to_owned()
    } else {
        format!("{dir}/{file}")
    }
}
