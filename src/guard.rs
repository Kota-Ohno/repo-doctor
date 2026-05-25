use std::collections::{BTreeSet, HashSet};
use std::path::Path;
use std::process::Command;

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

pub(crate) fn inspect(path: &Path, base: Option<&str>) -> Vec<Check> {
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
    checks.extend(check_agent_instructions(path));
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
    let changed_paths = changes
        .iter()
        .filter(|change| !matches!(change.status, ChangeStatus::Deleted))
        .map(|change| change.path.as_str())
        .collect::<BTreeSet<_>>();
    let mut unsynced = Vec::new();

    for (manifest, lockfiles) in manifest_lockfile_rules() {
        if changed_paths.contains(manifest)
            && !lockfiles.iter().any(|lock| changed_paths.contains(lock))
        {
            unsynced.push(manifest);
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

fn manifest_lockfile_rules() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("Cargo.toml", vec!["Cargo.lock"]),
        (
            "package.json",
            vec![
                "package-lock.json",
                "npm-shrinkwrap.json",
                "yarn.lock",
                "pnpm-lock.yaml",
                "bun.lock",
                "bun.lockb",
            ],
        ),
        (
            "pyproject.toml",
            vec!["uv.lock", "poetry.lock", "Pipfile.lock", "pdm.lock"],
        ),
        ("go.mod", vec!["go.sum"]),
        ("composer.json", vec!["composer.lock"]),
        ("Gemfile", vec!["Gemfile.lock"]),
    ]
}

fn check_agent_instructions(path: &Path) -> Vec<Check> {
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

    checks
}
