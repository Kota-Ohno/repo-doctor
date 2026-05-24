use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::json;

use crate::profiles::Profile;

pub struct RuleInfo {
    pub id: &'static str,
    pub severity: &'static str,
    pub description: &'static str,
}

pub fn known_rules() -> &'static [RuleInfo] {
    &[
        RuleInfo {
            id: "readme",
            severity: "warning",
            description: "README is present",
        },
        RuleInfo {
            id: "license",
            severity: "warning",
            description: "License file is present",
        },
        RuleInfo {
            id: "gitignore",
            severity: "warning",
            description: ".gitignore is present",
        },
        RuleInfo {
            id: "editorconfig",
            severity: "warning",
            description: ".editorconfig is present",
        },
        RuleInfo {
            id: "gitattributes",
            severity: "warning",
            description: ".gitattributes is present",
        },
        RuleInfo {
            id: "env_example",
            severity: "warning",
            description: "Example environment file is present",
        },
        RuleInfo {
            id: "docs_or_examples",
            severity: "warning",
            description: "Documentation or examples directory is present",
        },
        RuleInfo {
            id: "secret_like_file",
            severity: "warning",
            description: "Common secret-like files are absent",
        },
        RuleInfo {
            id: "github_actions",
            severity: "warning",
            description: "GitHub Actions workflow is present",
        },
        RuleInfo {
            id: "github_actions_yaml",
            severity: "warning",
            description: "GitHub Actions workflow YAML parses",
        },
        RuleInfo {
            id: "github_actions_triggers",
            severity: "warning",
            description: "GitHub Actions run on push and pull_request",
        },
        RuleInfo {
            id: "github_actions_permissions",
            severity: "warning",
            description: "GitHub Actions permissions are declared",
        },
        RuleInfo {
            id: "github_actions_floating_refs",
            severity: "warning",
            description: "GitHub Actions avoid floating action refs",
        },
        RuleInfo {
            id: "dependency_update_config",
            severity: "warning",
            description: "Dependabot or Renovate config is present",
        },
        RuleInfo {
            id: "contributing",
            severity: "warning",
            description: "Contribution guide is present",
        },
        RuleInfo {
            id: "code_of_conduct",
            severity: "warning",
            description: "Code of conduct is present",
        },
        RuleInfo {
            id: "security_policy",
            severity: "warning",
            description: "Security policy is present",
        },
        RuleInfo {
            id: "issue_templates",
            severity: "warning",
            description: "Issue template is present",
        },
        RuleInfo {
            id: "issue_template_frontmatter",
            severity: "warning",
            description: "Issue template frontmatter is populated",
        },
        RuleInfo {
            id: "pull_request_template",
            severity: "warning",
            description: "Pull request template is present",
        },
        RuleInfo {
            id: "changelog",
            severity: "warning",
            description: "Changelog or release notes are present",
        },
        RuleInfo {
            id: "rust_cargo_name",
            severity: "warning",
            description: "Cargo package name is set",
        },
        RuleInfo {
            id: "rust_cargo_version",
            severity: "warning",
            description: "Cargo package version is set",
        },
        RuleInfo {
            id: "rust_cargo_edition",
            severity: "warning",
            description: "Cargo package edition is set",
        },
        RuleInfo {
            id: "rust_cargo_lock",
            severity: "warning",
            description: "Cargo.lock is present",
        },
        RuleInfo {
            id: "rust_readme_commands",
            severity: "warning",
            description: "README documents Rust usage and test commands",
        },
        RuleInfo {
            id: "rust_toolchain",
            severity: "warning",
            description: "Rust toolchain pin is present",
        },
        RuleInfo {
            id: "rust_tooling_config",
            severity: "warning",
            description: "Rust rustfmt and clippy configs are present",
        },
        RuleInfo {
            id: "node_name",
            severity: "warning",
            description: "package.json name is set",
        },
        RuleInfo {
            id: "node_test_script",
            severity: "warning",
            description: "package.json scripts.test is set",
        },
        RuleInfo {
            id: "node_typescript_config",
            severity: "warning",
            description: "TypeScript compiler configuration is present when applicable",
        },
        RuleInfo {
            id: "node_lint_format",
            severity: "warning",
            description: "Node lint and format commands are configured",
        },
        RuleInfo {
            id: "python_project_metadata",
            severity: "warning",
            description: "pyproject.toml has project metadata",
        },
        RuleInfo {
            id: "python_lint_format",
            severity: "warning",
            description: "Python lint or format tooling is configured",
        },
        RuleInfo {
            id: "python_pytest_config",
            severity: "warning",
            description: "Python pytest configuration is present",
        },
        RuleInfo {
            id: "go_module",
            severity: "warning",
            description: "go.mod module declaration is present",
        },
        RuleInfo {
            id: "go_ci_commands",
            severity: "warning",
            description: "Go CI includes test, vet, and formatting commands",
        },
        RuleInfo {
            id: "docker_build_file",
            severity: "warning",
            description: "Container build file is present",
        },
        RuleInfo {
            id: "dockerignore",
            severity: "warning",
            description: ".dockerignore is present",
        },
        RuleInfo {
            id: "docker_compose",
            severity: "warning",
            description: "Compose file is present when local multi-service development is expected",
        },
        RuleInfo {
            id: "docker_base_image_pin",
            severity: "warning",
            description: "Container base image tags avoid :latest",
        },
        RuleInfo {
            id: "docker_healthcheck",
            severity: "warning",
            description: "Container HEALTHCHECK is configured",
        },
        RuleInfo {
            id: "docker_non_root_user",
            severity: "warning",
            description: "Container switches to a configured USER",
        },
        RuleInfo {
            id: "github_remote_branch_protection",
            severity: "warning",
            description: "Default branch protection is enabled",
        },
    ]
}

#[derive(Debug)]
pub struct RunOutput {
    pub text: String,
    pub exit_code: i32,
}

#[derive(Debug, Serialize)]
pub struct Report {
    schema_version: u16,
    path: PathBuf,
    selected_profiles: Vec<Profile>,
    summary: Summary,
    checks: Vec<Check>,
}

impl Report {
    pub fn new(path: PathBuf, selected_profiles: Vec<Profile>, checks: Vec<Check>) -> Self {
        let summary = Summary::from_checks(&checks);

        Self {
            schema_version: 1,
            path,
            selected_profiles,
            summary,
            checks,
        }
    }

    pub fn score(&self) -> u8 {
        self.summary.score
    }

    pub fn warning_count(&self) -> usize {
        self.summary.warn
    }

    pub fn warning_ids(&self) -> Vec<&'static str> {
        self.checks
            .iter()
            .filter(|check| matches!(check.status, CheckStatus::Warn))
            .map(|check| check.id)
            .collect()
    }

    pub fn warning_details(&self) -> Vec<WarningDetail<'_>> {
        self.checks
            .iter()
            .filter(|check| matches!(check.status, CheckStatus::Warn))
            .map(|check| WarningDetail {
                id: check.id,
                message: &check.message,
                remediation: &check.remediation,
            })
            .collect()
    }

    pub fn suppress_warning_ids(&self, ignored_ids: &[String]) -> Self {
        let checks = self
            .checks
            .iter()
            .filter(|check| {
                !matches!(check.status, CheckStatus::Warn)
                    || !ignored_ids.iter().any(|ignored| ignored == check.id)
            })
            .cloned()
            .collect();

        Self::new(self.path.clone(), self.selected_profiles.clone(), checks)
    }

    pub fn warnings_only(&self) -> Self {
        let checks = self
            .checks
            .iter()
            .filter(|check| matches!(check.status, CheckStatus::Warn))
            .cloned()
            .collect();

        Self::new(self.path.clone(), self.selected_profiles.clone(), checks)
    }

    pub fn has_warnings(&self) -> bool {
        self.checks
            .iter()
            .any(|check| matches!(check.status, CheckStatus::Warn))
    }

    pub fn format_text(&self) -> String {
        let mut lines = vec![format!("Repository: {}", self.path.display())];
        lines.push(format!(
            "Profiles: {}",
            format_profiles(&self.selected_profiles)
        ));
        lines.push(format!(
            "Summary: {} passed, {} warnings, score {}",
            self.summary.pass, self.summary.warn, self.summary.score
        ));

        for check in &self.checks {
            let marker = match check.status {
                CheckStatus::Pass => "PASS",
                CheckStatus::Warn => "WARN",
            };

            lines.push(format!("[{marker}] {}: {}", check.id, check.message));
        }

        lines.join("\n")
    }

    pub fn format_compact(&self) -> String {
        format!(
            "repo-doctor path={} profiles={} pass={} warn={} total={} score={}",
            self.path.display(),
            format_profiles(&self.selected_profiles),
            self.summary.pass,
            self.summary.warn,
            self.summary.total,
            self.summary.score
        )
    }

    pub fn format_markdown(&self) -> String {
        let mut lines = vec![
            "# repo-doctor report".to_owned(),
            String::new(),
            format!("- Repository: `{}`", self.path.display()),
            format!("- Profiles: {}", format_profiles(&self.selected_profiles)),
            format!(
                "- Summary: {} passed, {} warnings, score {}",
                self.summary.pass, self.summary.warn, self.summary.score
            ),
            String::new(),
            "| Status | Rule | Message | Remediation |".to_owned(),
            "| --- | --- | --- | --- |".to_owned(),
        ];

        for check in &self.checks {
            lines.push(format!(
                "| {} | `{}` | {} | {} |",
                check.status.markdown_label(),
                check.id,
                escape_markdown_table_cell(&check.message),
                escape_markdown_table_cell(&check.remediation)
            ));
        }

        lines.join("\n")
    }

    pub fn format_html(&self) -> String {
        let rows = self
            .checks
            .iter()
            .map(|check| {
                format!(
                    "<tr class=\"{}\"><td>{}</td><td><code>{}</code></td><td>{}</td><td>{}</td></tr>",
                    check.status.html_class(),
                    check.status.markdown_label(),
                    escape_html(check.id),
                    escape_html(&check.message),
                    escape_html(&check.remediation)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>repo-doctor report</title>
  <style>
    body {{ font-family: system-ui, sans-serif; margin: 2rem; color: #202124; }}
    table {{ border-collapse: collapse; width: 100%; }}
    th, td {{ border-bottom: 1px solid #ddd; padding: 0.5rem; text-align: left; vertical-align: top; }}
    .pass td:first-child {{ color: #137333; font-weight: 700; }}
    .warn td:first-child {{ color: #b06000; font-weight: 700; }}
    code {{ background: #f1f3f4; padding: 0.1rem 0.25rem; border-radius: 4px; }}
  </style>
</head>
<body>
  <h1>repo-doctor report</h1>
  <p><strong>Repository:</strong> {}</p>
  <p><strong>Profiles:</strong> {}</p>
  <p><strong>Summary:</strong> {} passed, {} warnings, score {}</p>
  <table>
    <thead><tr><th>Status</th><th>Rule</th><th>Message</th><th>Remediation</th></tr></thead>
    <tbody>
{}
    </tbody>
  </table>
</body>
</html>"#,
            escape_html(&self.path.display().to_string()),
            escape_html(&format_profiles(&self.selected_profiles)),
            self.summary.pass,
            self.summary.warn,
            self.summary.score,
            rows
        )
    }

    pub fn format_github_annotations(&self) -> String {
        let warnings = self
            .checks
            .iter()
            .filter(|check| matches!(check.status, CheckStatus::Warn))
            .collect::<Vec<_>>();

        if warnings.is_empty() {
            return "repo-doctor: no warnings".to_owned();
        }

        let mut lines = warnings
            .iter()
            .map(|check| {
                format!(
                    "::warning title={}::{}",
                    escape_github_annotation(check.id),
                    escape_github_annotation(&format!(
                        "{} Remediation: {}",
                        check.message, check.remediation
                    ))
                )
            })
            .collect::<Vec<_>>();
        lines.push(format!(
            "repo-doctor: {} warning(s), score {}",
            self.summary.warn, self.summary.score
        ));
        lines.join("\n")
    }

    pub fn format_junit(&self) -> String {
        let mut lines = vec![format!(
            r#"<testsuite name="repo-doctor" tests="{}" failures="{}">"#,
            self.summary.total, self.summary.warn
        )];

        for check in &self.checks {
            lines.push(format!(
                r#"  <testcase classname="repo-doctor" name="{}">"#,
                escape_xml(check.id)
            ));
            if matches!(check.status, CheckStatus::Warn) {
                lines.push(format!(
                    r#"    <failure message="{}">{}</failure>"#,
                    escape_xml(&check.message),
                    escape_xml(&check.remediation)
                ));
            }
            lines.push("  </testcase>".to_owned());
        }

        lines.push("</testsuite>".to_owned());
        lines.join("\n")
    }

    pub fn format_sarif(&self) -> serde_json::Result<String> {
        let rules = self
            .checks
            .iter()
            .map(|check| {
                json!({
                    "id": check.id,
                    "name": check.id,
                    "shortDescription": {
                        "text": check.message,
                    },
                    "help": {
                        "text": check.remediation,
                    },
                })
            })
            .collect::<Vec<_>>();
        let results = self
            .checks
            .iter()
            .filter(|check| matches!(check.status, CheckStatus::Warn))
            .map(|check| {
                json!({
                    "ruleId": check.id,
                    "level": "warning",
                    "message": {
                        "text": check.message,
                    },
                    "locations": [sarif_location(&self.path, check.location.as_ref())],
                })
            })
            .collect::<Vec<_>>();

        let sarif = json!({
            "version": "2.1.0",
            "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "repo-doctor",
                            "informationUri": "https://github.com/Kota-Ohno/repo-doctor",
                            "rules": rules,
                        },
                    },
                    "results": results,
                },
            ],
        });

        serde_json::to_string_pretty(&sarif)
    }
}

pub struct WarningDetail<'a> {
    pub id: &'static str,
    pub message: &'a str,
    pub remediation: &'a str,
}

#[derive(Clone, Debug, Serialize)]
pub struct Summary {
    pass: usize,
    warn: usize,
    total: usize,
    score: u8,
}

impl Summary {
    fn from_checks(checks: &[Check]) -> Self {
        let pass = checks
            .iter()
            .filter(|check| matches!(check.status, CheckStatus::Pass))
            .count();
        let warn = checks
            .iter()
            .filter(|check| matches!(check.status, CheckStatus::Warn))
            .count();
        let total = checks.len();
        let score = (pass * 100)
            .checked_div(total)
            .map_or(100, |score| score as u8);

        Self {
            pass,
            warn,
            total,
            score,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Check {
    pub(crate) id: &'static str,
    pub(crate) status: CheckStatus,
    pub(crate) severity: Severity,
    pub(crate) message: String,
    pub(crate) remediation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) documentation_url: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) location: Option<Location>,
}

impl Check {
    pub(crate) fn id(&self) -> &'static str {
        self.id
    }

    pub(crate) fn set_severity(&mut self, severity: Severity) {
        self.severity = severity;
    }

    pub(crate) fn with_documentation_url(mut self, documentation_url: &'static str) -> Self {
        self.documentation_url = Some(documentation_url);
        self
    }

    pub(crate) fn with_location(mut self, path: impl Into<String>, line: Option<usize>) -> Self {
        self.location = Some(Location {
            path: path.into(),
            line,
        });
        self
    }

    pub(crate) fn location_path(&self) -> Option<&str> {
        self.location
            .as_ref()
            .map(|location| location.path.as_str())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Location {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Warn,
}

impl CheckStatus {
    fn markdown_label(&self) -> &'static str {
        match self {
            CheckStatus::Pass => "PASS",
            CheckStatus::Warn => "WARN",
        }
    }

    fn html_class(&self) -> &'static str {
        match self {
            CheckStatus::Pass => "pass",
            CheckStatus::Warn => "warn",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
}

pub(crate) fn pass(id: &'static str, message: impl Into<String>) -> Check {
    Check {
        id,
        status: CheckStatus::Pass,
        severity: Severity::Info,
        message: message.into(),
        remediation: "No action needed.".to_owned(),
        documentation_url: None,
        location: None,
    }
}

pub(crate) fn warn(
    id: &'static str,
    message: impl Into<String>,
    remediation: impl Into<String>,
) -> Check {
    Check {
        id,
        status: CheckStatus::Warn,
        severity: Severity::Warning,
        message: message.into(),
        remediation: remediation.into(),
        documentation_url: None,
        location: None,
    }
}

fn format_profiles(profiles: &[Profile]) -> String {
    if profiles.is_empty() {
        "none".to_owned()
    } else {
        profiles
            .iter()
            .map(|profile| profile.name())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn escape_markdown_table_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', "<br>")
}

fn escape_github_annotation(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
        .replace(':', "%3A")
        .replace(',', "%2C")
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn escape_html(value: &str) -> String {
    escape_xml(value)
}

fn sarif_location(repo_path: &Path, location: Option<&Location>) -> serde_json::Value {
    let uri = location
        .map(|location| location.path.clone())
        .unwrap_or_else(|| repo_path.display().to_string());
    let mut physical_location = json!({
        "artifactLocation": {
            "uri": uri,
        },
    });

    if let Some(line) = location.and_then(|location| location.line) {
        physical_location["region"] = json!({ "startLine": line });
    }

    json!({ "physicalLocation": physical_location })
}
