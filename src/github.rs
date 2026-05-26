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
        "description,repositoryTopics,defaultBranchRef,isArchived,pushedAt,licenseInfo,homepageUrl",
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
    checks.push(check_dependabot_config(repo));
    checks.push(check_actions_permissions(repo));
    checks.push(check_repository_rulesets(repo));
    checks.extend(check_community_profile(repo));
    checks.push(check_scorecard_link(repo));
    checks.push(check_latest_release(repo));

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

pub(crate) fn setup(
    repo: &str,
    topics: &[String],
    homepage: Option<&str>,
    branch_protection: bool,
) -> Result<Vec<String>> {
    validate_repo(repo)?;
    let mut actions = Vec::new();

    if !topics.is_empty() {
        let mut args = vec!["repo", "edit", repo];
        for topic in topics {
            args.push("--add-topic");
            args.push(topic.as_str());
        }
        gh_unit(&args)?;
        actions.push(format!("set {} topic(s)", topics.len()));
    }

    if let Some(homepage) = homepage {
        gh_unit(&["repo", "edit", repo, "--homepage", homepage])?;
        actions.push("set homepage".to_owned());
    }

    gh_unit(&[
        "api",
        "-X",
        "PUT",
        &format!("repos/{repo}/vulnerability-alerts"),
        "--silent",
    ])?;
    actions.push("enabled vulnerability alerts".to_owned());

    if branch_protection {
        let view = gh_json(&["repo", "view", repo, "--json", "defaultBranchRef"])?;
        let branch = view
            .get("defaultBranchRef")
            .and_then(|branch| branch.get("name"))
            .and_then(JsonValue::as_str)
            .unwrap_or("main");
        gh_unit(&[
            "api",
            "-X",
            "PUT",
            &format!("repos/{repo}/branches/{branch}/protection"),
            "-H",
            "Accept: application/vnd.github+json",
            "-F",
            "required_status_checks=null",
            "-F",
            "enforce_admins=false",
            "-F",
            "required_pull_request_reviews=null",
            "-F",
            "restrictions=null",
            "--silent",
        ])
        .with_context(|| {
            format!(
                "failed to enable branch protection for {repo}/{branch}; private repositories may require GitHub Pro, public visibility, repository admin access, or organization policy access"
            )
        })?;
        actions.push(format!("enabled branch protection for {branch}"));
    }

    Ok(actions)
}

pub(crate) fn setup_plan(
    repo: &str,
    topics: &[String],
    homepage: Option<&str>,
    branch_protection: bool,
) -> Result<Vec<String>> {
    validate_repo(repo)?;
    let mut actions = Vec::new();
    if !topics.is_empty() {
        actions.push(format!(
            "would set {} topic(s): {}",
            topics.len(),
            topics.join(", ")
        ));
    }
    if let Some(homepage) = homepage {
        actions.push(format!("would set homepage: {homepage}"));
    }
    actions.push("would enable vulnerability alerts".to_owned());
    if branch_protection {
        actions.push("would enable default branch protection".to_owned());
    }
    Ok(actions)
}

pub(crate) fn auth_doctor() -> Result<String> {
    let mut lines = vec!["repo-doctor gh auth doctor".to_owned()];
    if !gh_status(&["--version"]) {
        lines.push("gh_cli=missing".to_owned());
        lines.push("fix=install GitHub CLI from https://cli.github.com/".to_owned());
        lines.push("fix=after installation, run `gh auth login`".to_owned());
        return Ok(lines.join("\n"));
    }
    lines.push("gh_cli=ok".to_owned());

    if gh_status(&["auth", "status"]) {
        lines.push("gh_auth=ok".to_owned());
    } else {
        lines.push("gh_auth=failed".to_owned());
        lines.push("fix=run `gh auth login` and retry remote checks".to_owned());
        return Ok(lines.join("\n"));
    }

    for (name, args) in [
        ("repo_view", vec!["repo", "view", "--json", "name"]),
        ("api_user", vec!["api", "user", "--silent"]),
    ] {
        if gh_status(&args) {
            lines.push(format!("{name}=ok"));
        } else {
            lines.push(format!("{name}=unavailable"));
        }
    }
    lines.push("recommended_scopes=repo for private repositories; public_repo is enough for many public read checks".to_owned());
    lines.push(
        "admin_scope=required for repo-doctor github-setup and some branch protection changes"
            .to_owned(),
    );
    lines.push("note=branch protection and security settings may require repository admin access or organization policy access".to_owned());
    Ok(lines.join("\n"))
}

fn gh_unit(args: &[&str]) -> Result<()> {
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

    Ok(())
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
        check_remote_license(view),
        check_remote_homepage(view),
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

fn check_remote_license(view: &JsonValue) -> Check {
    if view
        .get("licenseInfo")
        .is_some_and(|license| !license.is_null())
    {
        pass(
            "github_remote_license",
            "GitHub repository license metadata is available",
        )
    } else {
        warn(
            "github_remote_license",
            "GitHub repository license metadata is missing",
            "Add a recognizable license file so GitHub can classify repository licensing.",
        )
    }
}

fn check_remote_homepage(view: &JsonValue) -> Check {
    if view
        .get("homepageUrl")
        .and_then(JsonValue::as_str)
        .is_some_and(|homepage| !homepage.trim().is_empty())
    {
        pass(
            "github_remote_homepage",
            "GitHub repository homepage URL is set",
        )
    } else {
        warn(
            "github_remote_homepage",
            "GitHub repository homepage URL is missing",
            "Set a homepage URL when the project has docs, crates, packages, or a product page.",
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
            "Enable branch protection for the default branch. Private repositories may require GitHub Pro, public visibility, repository admin access, or organization policy access.",
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

fn check_dependabot_config(repo: &str) -> Check {
    let endpoint = format!("repos/{repo}/contents/.github/dependabot.yml");
    if gh_status(&["api", &endpoint, "--silent"])
        || gh_status(&[
            "api",
            &format!("repos/{repo}/contents/.github/dependabot.yaml"),
            "--silent",
        ])
    {
        pass(
            "github_remote_dependabot_config",
            "Dependabot configuration is present",
        )
    } else {
        warn(
            "github_remote_dependabot_config",
            "Dependabot configuration is missing or not visible",
            "Add .github/dependabot.yml or confirm dependency updates are handled elsewhere.",
        )
    }
}

fn check_actions_permissions(repo: &str) -> Check {
    let endpoint = format!("repos/{repo}/actions/permissions");
    let Ok(permissions) = gh_json(&["api", &endpoint]) else {
        return warn(
            "github_remote_actions_permissions",
            "GitHub Actions permissions are unavailable",
            "Run with a token that can read Actions settings, or inspect repository Actions permissions in GitHub.",
        );
    };

    check_actions_permissions_payload(&permissions)
}

fn check_actions_permissions_payload(permissions: &JsonValue) -> Check {
    let enabled = permissions
        .get("enabled")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);
    let default_permissions = permissions
        .get("default_workflow_permissions")
        .and_then(JsonValue::as_str)
        .unwrap_or("");

    if enabled && default_permissions == "read" {
        pass(
            "github_remote_actions_permissions",
            "GitHub Actions are enabled with read-only default token permissions",
        )
    } else if enabled {
        warn(
            "github_remote_actions_permissions",
            format!("GitHub Actions default token permissions are `{default_permissions}`"),
            "Prefer read-only default workflow permissions and grant write permissions per workflow job.",
        )
    } else {
        warn(
            "github_remote_actions_permissions",
            "GitHub Actions are disabled",
            "Enable GitHub Actions if this repository should run CI.",
        )
    }
}

fn check_repository_rulesets(repo: &str) -> Check {
    let endpoint = format!("repos/{repo}/rulesets");
    let Ok(rulesets) = gh_json(&["api", &endpoint]) else {
        return warn(
            "github_remote_rulesets",
            "Repository rulesets are unavailable",
            "Run with a token that can read rulesets, or inspect branch/tag rulesets in GitHub.",
        );
    };

    check_repository_rulesets_payload(&rulesets)
}

fn check_repository_rulesets_payload(rulesets: &JsonValue) -> Check {
    if rulesets
        .as_array()
        .is_some_and(|rulesets| !rulesets.is_empty())
    {
        pass(
            "github_remote_rulesets",
            "Repository rulesets are configured",
        )
    } else {
        warn(
            "github_remote_rulesets",
            "Repository rulesets are not configured",
            "Use branch protection or repository rulesets to protect important branches and tags.",
        )
    }
}

fn check_community_profile(repo: &str) -> Vec<Check> {
    let endpoint = format!("repos/{repo}/community/profile");
    let Ok(profile) = gh_json(&["api", &endpoint]) else {
        return vec![warn(
            "github_remote_community_profile",
            "GitHub community profile is unavailable",
            "Run with a token that can read the community profile API, or inspect the repository community profile in GitHub.",
        )];
    };

    vec![
        check_community_health_percentage(&profile),
        check_community_profile_files(&profile),
    ]
}

fn check_community_health_percentage(profile: &JsonValue) -> Check {
    let health_percentage = profile
        .get("health_percentage")
        .and_then(JsonValue::as_u64)
        .unwrap_or(0);

    if health_percentage >= 80 {
        pass(
            "github_remote_community_health",
            format!("GitHub community profile health is {health_percentage}%"),
        )
    } else {
        warn(
            "github_remote_community_health",
            format!("GitHub community profile health is {health_percentage}%"),
            "Review the GitHub community profile and fill missing community files.",
        )
    }
}

fn check_community_profile_files(profile: &JsonValue) -> Check {
    let files = profile.get("files").and_then(JsonValue::as_object);
    let has_readme = files
        .and_then(|files| files.get("readme"))
        .is_some_and(|value| !value.is_null());
    let has_license = files
        .and_then(|files| files.get("license"))
        .is_some_and(|value| !value.is_null());

    if has_readme && has_license {
        pass(
            "github_remote_community_files",
            "GitHub community profile sees README and license",
        )
    } else {
        warn(
            "github_remote_community_files",
            "GitHub community profile is missing README or license",
            "Ensure GitHub recognizes the README and license files.",
        )
    }
}

fn check_scorecard_link(repo: &str) -> Check {
    pass(
        "github_remote_scorecard_link",
        format!("OpenSSF Scorecard link: https://scorecard.dev/viewer/?uri=github.com/{repo}"),
    )
    .with_documentation_url("https://scorecard.dev/")
}

fn check_latest_release(repo: &str) -> Check {
    let endpoint = format!("repos/{repo}/releases/latest");
    if gh_status(&["api", &endpoint, "--silent"]) {
        pass(
            "github_remote_latest_release",
            "GitHub latest release is available",
        )
    } else {
        warn(
            "github_remote_latest_release",
            "GitHub latest release is missing",
            "Create releases when users need stable installable versions.",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::CheckStatus;

    #[test]
    fn view_payload_checks_description_topics_and_branch() {
        let view = serde_json::json!({
            "description": "Repository checker",
            "repositoryTopics": [{ "name": "cli" }],
            "defaultBranchRef": { "name": "main" },
            "isArchived": false,
            "pushedAt": "2026-05-23T00:00:00Z",
            "licenseInfo": { "spdxId": "MIT" },
            "homepageUrl": "https://example.com"
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

    #[test]
    fn actions_permissions_payload_reports_read_only_default_as_pass() {
        let check = check_actions_permissions_payload(&serde_json::json!({
            "enabled": true,
            "default_workflow_permissions": "read"
        }));

        assert!(matches!(check.status, CheckStatus::Pass));
        assert_eq!(check.id(), "github_remote_actions_permissions");
    }

    #[test]
    fn actions_permissions_payload_warns_on_write_default() {
        let check = check_actions_permissions_payload(&serde_json::json!({
            "enabled": true,
            "default_workflow_permissions": "write"
        }));

        assert!(matches!(check.status, CheckStatus::Warn));
        assert!(check.message.contains("write"));
    }

    #[test]
    fn rulesets_payload_reports_empty_list_as_warning() {
        let check = check_repository_rulesets_payload(&serde_json::json!([]));

        assert!(matches!(check.status, CheckStatus::Warn));
        assert_eq!(check.id(), "github_remote_rulesets");
    }

    #[test]
    fn rulesets_payload_reports_configured_rulesets_as_pass() {
        let check = check_repository_rulesets_payload(&serde_json::json!([
            { "name": "main" }
        ]));

        assert!(matches!(check.status, CheckStatus::Pass));
    }
}
