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
        check_file(
            path,
            "editorconfig",
            ".editorconfig",
            ".editorconfig is present",
        ),
        check_file(
            path,
            "gitattributes",
            ".gitattributes",
            ".gitattributes is present",
        ),
        check_any_file(
            path,
            "env_example",
            &[".env.example", ".env.sample", "config/.env.example"],
            "Example environment file is present",
        ),
        check_docs_or_examples(path),
        check_secret_like_files(path),
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
        check_monorepo_workspaces(path),
    ];

    if let Some(check) = check_issue_template_frontmatter(path) {
        checks.push(check);
    }
    checks.extend(inspect_workflow_content(path));
    checks
}

fn check_monorepo_workspaces(path: &Path) -> Check {
    let roots = ["Cargo.toml", "package.json", "pyproject.toml", "go.mod"]
        .iter()
        .filter(|candidate| path.join(candidate).exists())
        .count();
    let workspace_dirs = ["crates", "packages", "apps", "services", "modules"]
        .iter()
        .filter(|candidate| path.join(candidate).is_dir())
        .count();

    if roots > 1 || workspace_dirs > 0 {
        pass(
            "monorepo_workspaces",
            "Monorepo or workspace layout hints are present",
        )
    } else {
        pass(
            "monorepo_workspaces",
            "Monorepo or workspace layout is not detected",
        )
    }
}

fn check_docs_or_examples(path: &Path) -> Check {
    if path.join("docs").is_dir() || path.join("examples").is_dir() {
        pass(
            "docs_or_examples",
            "Documentation or examples directory is present",
        )
    } else {
        warn(
            "docs_or_examples",
            "Documentation or examples directory is missing",
            "Add docs/ or examples/ for richer usage guidance.",
        )
    }
}

fn check_secret_like_files(path: &Path) -> Check {
    let candidates = [
        ".env",
        ".env.local",
        ".env.production",
        ".env.development",
        "secrets.json",
        "credentials.json",
    ];
    let found = candidates
        .iter()
        .find(|candidate| path.join(candidate).is_file());

    if let Some(found) = found {
        warn(
            "secret_like_file",
            format!("Secret-like file is present: {found}"),
            "Remove committed secret files and keep only sanitized examples such as .env.example.",
        )
        .with_location(*found, None)
    } else {
        pass(
            "secret_like_file",
            "No common secret-like files are present",
        )
    }
}

fn inspect_workflow_content(path: &Path) -> Vec<Check> {
    let workflows = read_workflows(path);
    if workflows.is_empty() {
        return Vec::new();
    }

    vec![
        check_workflow_yaml_parse(&workflows),
        check_workflow_push_and_pull_request(&workflows),
        check_workflow_permissions(&workflows),
        check_workflow_pull_request_target(&workflows),
        check_workflow_floating_action_refs(&workflows),
        check_dependency_update_config(path),
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

fn check_workflow_yaml_parse(workflows: &[(String, String)]) -> Check {
    let invalid = workflows
        .iter()
        .filter_map(|(name, contents)| {
            yaml_rust2::YamlLoader::load_from_str(contents)
                .err()
                .map(|error| format!("{name}: {error}"))
        })
        .collect::<Vec<_>>();

    if invalid.is_empty() {
        pass(
            "github_actions_yaml",
            "GitHub Actions workflows parse as YAML",
        )
        .with_documentation_url(
            "https://docs.github.com/actions/using-workflows/workflow-syntax-for-github-actions",
        )
    } else {
        let location = invalid
            .first()
            .and_then(|entry| entry.split_once(':').map(|(name, _)| name.to_owned()));
        warn(
            "github_actions_yaml",
            format!(
                "GitHub Actions workflows contain invalid YAML: {}",
                invalid.join(", ")
            ),
            "Fix workflow syntax so GitHub Actions can load the files.",
        )
        .with_documentation_url(
            "https://docs.github.com/actions/using-workflows/workflow-syntax-for-github-actions",
        )
        .with_location(
            location.unwrap_or_else(|| ".github/workflows".to_owned()),
            None,
        )
    }
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

fn check_dependency_update_config(path: &Path) -> Check {
    let has_dependabot = path.join(".github/dependabot.yml").exists()
        || path.join(".github/dependabot.yaml").exists();
    let has_renovate = [
        "renovate.json",
        "renovate.json5",
        ".github/renovate.json",
        ".github/renovate.json5",
    ]
    .iter()
    .any(|candidate| path.join(candidate).exists());

    if has_dependabot || has_renovate {
        pass(
            "dependency_update_config",
            "Dependency update configuration is present",
        )
    } else {
        warn(
            "dependency_update_config",
            "Dependabot or Renovate configuration is missing",
            "Add .github/dependabot.yml or a Renovate config to keep dependencies current.",
        )
    }
}

fn check_issue_template_frontmatter(path: &Path) -> Option<Check> {
    let templates_dir = path.join(".github/ISSUE_TEMPLATE");
    let Ok(entries) = templates_dir.read_dir() else {
        return None;
    };

    let invalid = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let is_markdown = path
                .extension()
                .is_some_and(|extension| extension == "md" || extension == "markdown");
            if !is_markdown {
                return None;
            }

            let contents = std::fs::read_to_string(&path).ok()?;
            let has_frontmatter = contents.trim_start().starts_with("---");
            let has_name = contents
                .lines()
                .any(|line| line.trim_start().starts_with("name:"));
            let has_about = contents
                .lines()
                .any(|line| line.trim_start().starts_with("about:"));

            (!has_frontmatter || !has_name || !has_about).then(|| path.display().to_string())
        })
        .collect::<Vec<_>>();

    if invalid.is_empty() {
        Some(pass(
            "issue_template_frontmatter",
            "Issue template frontmatter is populated",
        ))
    } else {
        Some(
            warn(
                "issue_template_frontmatter",
                format!(
                    "Issue templates have missing frontmatter fields: {}",
                    invalid.join(", ")
                ),
                "Add YAML frontmatter with at least name and about to each issue template.",
            )
            .with_location(
                invalid
                    .first()
                    .cloned()
                    .unwrap_or_else(|| ".github/ISSUE_TEMPLATE".to_owned()),
                None,
            ),
        )
    }
}
