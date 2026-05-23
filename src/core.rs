use std::path::Path;

use crate::checks::{check_any_file, check_directory_has_file, check_file, check_workflows};
use crate::report::{Check, pass, warn};

pub(crate) fn inspect(path: &Path) -> Vec<Check> {
    let mut checks = vec![
        check_any_file(
            path,
            "readme",
            &["README.md", "README", "README.txt"],
            "README is present",
        ),
        check_any_file(
            path,
            "license",
            &[
                "LICENSE",
                "LICENSE.md",
                "LICENSE.txt",
                "LICENSE-MIT",
                "LICENSE-APACHE",
            ],
            "License file is present",
        ),
        check_file(path, "gitignore", ".gitignore", ".gitignore is present"),
        check_workflows(
            path,
            "github_actions",
            "GitHub Actions workflow file is present",
        ),
        check_any_file(
            path,
            "contributing",
            &[
                "CONTRIBUTING.md",
                "docs/CONTRIBUTING.md",
                ".github/CONTRIBUTING.md",
            ],
            "Contribution guide is present",
        ),
        check_any_file(
            path,
            "code_of_conduct",
            &[
                "CODE_OF_CONDUCT.md",
                "docs/CODE_OF_CONDUCT.md",
                ".github/CODE_OF_CONDUCT.md",
            ],
            "Code of conduct is present",
        ),
        check_any_file(
            path,
            "security_policy",
            &["SECURITY.md", "docs/SECURITY.md", ".github/SECURITY.md"],
            "Security policy is present",
        ),
        check_directory_has_file(
            path,
            "issue_templates",
            ".github/ISSUE_TEMPLATE",
            "Issue template is present",
        ),
        check_any_file(
            path,
            "pull_request_template",
            &[
                ".github/pull_request_template.md",
                ".github/PULL_REQUEST_TEMPLATE.md",
                "docs/pull_request_template.md",
                "docs/PULL_REQUEST_TEMPLATE.md",
                "PULL_REQUEST_TEMPLATE.md",
            ],
            "Pull request template is present",
        ),
        check_any_file(
            path,
            "changelog",
            &[
                "CHANGELOG.md",
                "CHANGES.md",
                "RELEASES.md",
                "docs/CHANGELOG.md",
            ],
            "Changelog or release notes are present",
        ),
    ];

    checks.extend(inspect_workflow_content(path));
    checks
}

fn inspect_workflow_content(path: &Path) -> Vec<Check> {
    let workflows = read_workflows(path);
    if workflows.is_empty() {
        return Vec::new();
    }

    vec![
        check_workflow_push_and_pull_request(&workflows),
        check_workflow_permissions(&workflows),
        check_workflow_pull_request_target(&workflows),
        check_workflow_floating_action_refs(&workflows),
    ]
}

fn read_workflows(path: &Path) -> Vec<(String, String)> {
    let workflows_dir = path.join(".github/workflows");
    let Ok(entries) = workflows_dir.read_dir() else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let is_workflow = path
                .extension()
                .is_some_and(|extension| extension == "yml" || extension == "yaml");
            if !is_workflow {
                return None;
            }

            let name = path.file_name()?.to_string_lossy().into_owned();
            let contents = std::fs::read_to_string(path).ok()?;
            Some((name, contents))
        })
        .collect()
}

fn check_workflow_push_and_pull_request(workflows: &[(String, String)]) -> Check {
    let has_push = workflows
        .iter()
        .any(|(_, contents)| contents.lines().any(|line| line.trim_start() == "push:"));
    let has_pull_request = workflows.iter().any(|(_, contents)| {
        contents
            .lines()
            .any(|line| line.trim_start().starts_with("pull_request:"))
    });

    if has_push && has_pull_request {
        pass(
            "github_actions_triggers",
            "GitHub Actions run on push and pull_request",
        )
    } else {
        warn(
            "github_actions_triggers",
            "GitHub Actions do not run on both push and pull_request",
            "Configure at least one workflow to run on push and pull_request.",
        )
    }
}

fn check_workflow_permissions(workflows: &[(String, String)]) -> Check {
    let missing = workflows
        .iter()
        .filter(|(_, contents)| {
            !contents
                .lines()
                .any(|line| line.trim_start().starts_with("permissions:"))
        })
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();

    if missing.is_empty() {
        pass(
            "github_actions_permissions",
            "GitHub Actions permissions are declared",
        )
    } else {
        warn(
            "github_actions_permissions",
            format!(
                "GitHub Actions permissions are missing in {}",
                missing.join(", ")
            ),
            "Declare least-privilege `permissions` in each workflow.",
        )
    }
}

fn check_workflow_pull_request_target(workflows: &[(String, String)]) -> Check {
    let uses_pull_request_target = workflows.iter().any(|(_, contents)| {
        contents
            .lines()
            .any(|line| line.trim_start().starts_with("pull_request_target:"))
    });

    if uses_pull_request_target {
        warn(
            "github_actions_pull_request_target",
            "GitHub Actions use pull_request_target",
            "Avoid pull_request_target unless the workflow is explicitly hardened.",
        )
    } else {
        pass(
            "github_actions_pull_request_target",
            "GitHub Actions avoid pull_request_target",
        )
    }
}

fn check_workflow_floating_action_refs(workflows: &[(String, String)]) -> Check {
    let floating_refs = workflows
        .iter()
        .flat_map(|(name, contents)| {
            contents.lines().filter_map(move |line| {
                let trimmed = line.trim();
                let uses = trimmed.strip_prefix("- uses: ")?;
                let (_, reference) = uses.rsplit_once('@')?;
                matches!(reference, "main" | "master" | "latest").then(|| format!("{name}: {uses}"))
            })
        })
        .collect::<Vec<_>>();

    if floating_refs.is_empty() {
        pass(
            "github_actions_floating_refs",
            "GitHub Actions avoid floating action refs",
        )
    } else {
        warn(
            "github_actions_floating_refs",
            format!(
                "GitHub Actions use floating action refs: {}",
                floating_refs.join(", ")
            ),
            "Pin action refs to version tags or commit SHAs instead of main, master, or latest.",
        )
    }
}
