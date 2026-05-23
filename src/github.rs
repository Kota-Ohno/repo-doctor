use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::Value as JsonValue;

use crate::report::{Check, Report, pass, warn};

pub(crate) fn inspect(repo: &str) -> Result<Report> {
    validate_repo(repo)?;

    let view = gh_json(&[
        "repo",
        "view",
        repo,
        "--json",
        "description,repositoryTopics,defaultBranchRef,isArchived,pushedAt",
    ])?;
    let mut checks = inspect_view_payload(repo, &view);

    let default_branch = view
        .get("defaultBranchRef")
        .and_then(|branch| branch.get("name"))
        .and_then(JsonValue::as_str);
    if let Some(default_branch) = default_branch {
        checks.push(check_branch_protection(repo, default_branch));
    }

    checks.push(check_vulnerability_alerts(repo));

    Ok(Report::new(PathBuf::from(repo), Vec::new(), checks))
}

fn validate_repo(repo: &str) -> Result<()> {
    let parts = repo.split('/').collect::<Vec<_>>();
    if parts.len() != 2 || parts.iter().any(|part| part.trim().is_empty()) {
        bail!("GitHub repository must be in owner/name form");
    }

    Ok(())
}

fn gh_json(args: &[&str]) -> Result<JsonValue> {
    let output = Command::new("gh")
        .args(args)
        .output()
        .context("failed to execute gh CLI")?;

    if !output.status.success() {
        bail!(
            "gh command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    serde_json::from_slice(&output.stdout).context("failed to parse gh JSON output")
}

fn gh_status(args: &[&str]) -> bool {
    Command::new("gh")
        .args(args)
        .output()
        .is_ok_and(|output| output.status.success())
}

fn inspect_view_payload(repo: &str, view: &JsonValue) -> Vec<Check> {
    vec![
        pass(
            "github_remote_repo",
            format!("GitHub repository {repo} is reachable"),
        ),
        check_description(view),
        check_topics(view),
        check_default_branch(view),
        check_archived(view),
        check_recent_activity(view),
    ]
}

fn check_description(view: &JsonValue) -> Check {
    if view
        .get("description")
        .and_then(JsonValue::as_str)
        .is_some_and(|description| !description.trim().is_empty())
    {
        pass(
            "github_remote_description",
            "GitHub repository description is set",
        )
    } else {
        warn(
            "github_remote_description",
            "GitHub repository description is missing",
            "Set a concise repository description on GitHub.",
        )
    }
}

fn check_topics(view: &JsonValue) -> Check {
    let has_topics = view
        .get("repositoryTopics")
        .and_then(JsonValue::as_array)
        .is_some_and(|topics| !topics.is_empty());

    if has_topics {
        pass("github_remote_topics", "GitHub repository topics are set")
    } else {
        warn(
            "github_remote_topics",
            "GitHub repository topics are missing",
            "Add repository topics so the project is discoverable.",
        )
    }
}

fn check_default_branch(view: &JsonValue) -> Check {
    if view
        .get("defaultBranchRef")
        .and_then(|branch| branch.get("name"))
        .and_then(JsonValue::as_str)
        .is_some_and(|name| !name.trim().is_empty())
    {
        pass(
            "github_remote_default_branch",
            "GitHub default branch is available",
        )
    } else {
        warn(
            "github_remote_default_branch",
            "GitHub default branch is unavailable",
            "Ensure the repository has a default branch.",
        )
    }
}

fn check_archived(view: &JsonValue) -> Check {
    if view
        .get("isArchived")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false)
    {
        warn(
            "github_remote_archived",
            "GitHub repository is archived",
            "Unarchive the repository if it is actively maintained.",
        )
    } else {
        pass(
            "github_remote_archived",
            "GitHub repository is not archived",
        )
    }
}

fn check_recent_activity(view: &JsonValue) -> Check {
    if view
        .get("pushedAt")
        .and_then(JsonValue::as_str)
        .is_some_and(|pushed_at| !pushed_at.trim().is_empty())
    {
        pass(
            "github_remote_recent_activity",
            "GitHub repository has push activity metadata",
        )
    } else {
        warn(
            "github_remote_recent_activity",
            "GitHub repository push activity is unavailable",
            "Check whether the repository has recent maintenance activity.",
        )
    }
}

fn check_branch_protection(repo: &str, branch: &str) -> Check {
    let endpoint = format!("repos/{repo}/branches/{branch}/protection");
    if gh_status(&["api", &endpoint, "--silent"]) {
        pass(
            "github_remote_branch_protection",
            "Default branch protection is enabled",
        )
    } else {
        warn(
            "github_remote_branch_protection",
            "Default branch protection is not enabled or not visible",
            "Enable branch protection for the default branch, or run with a token that can read it.",
        )
    }
}

fn check_vulnerability_alerts(repo: &str) -> Check {
    let endpoint = format!("repos/{repo}/vulnerability-alerts");
    if gh_status(&["api", &endpoint, "--silent"]) {
        pass(
            "github_remote_vulnerability_alerts",
            "Dependabot vulnerability alerts are enabled",
        )
    } else {
        warn(
            "github_remote_vulnerability_alerts",
            "Dependabot vulnerability alerts are disabled or not visible",
            "Enable Dependabot alerts, or run with a token that can read security settings.",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_payload_checks_description_topics_and_branch() {
        let view = serde_json::json!({
            "description": "Repository checker",
            "repositoryTopics": [{ "name": "cli" }],
            "defaultBranchRef": { "name": "main" },
            "isArchived": false,
            "pushedAt": "2026-05-23T00:00:00Z"
        });

        let checks = inspect_view_payload("owner/repo", &view);

        assert!(
            checks
                .iter()
                .any(|check| check.id() == "github_remote_description")
        );
        assert!(
            checks
                .iter()
                .any(|check| check.id() == "github_remote_topics")
        );
        assert!(
            checks
                .iter()
                .any(|check| check.id() == "github_remote_default_branch")
        );
    }

    #[test]
    fn rejects_invalid_repo_names() {
        assert!(validate_repo("owner").is_err());
        assert!(validate_repo("owner/repo").is_ok());
    }
}
