use std::path::PathBuf;

use serde::Serialize;
use serde_json::json;

use crate::profiles::Profile;

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

    pub fn format_github_annotations(&self) -> String {
        let warnings = self
            .checks
            .iter()
            .filter(|check| matches!(check.status, CheckStatus::Warn))
            .collect::<Vec<_>>();

        if warnings.is_empty() {
            return "repo-doctor: no warnings".to_owned();
        }

        warnings
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
            .collect::<Vec<_>>()
            .join("\n")
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
                    "locations": [
                        {
                            "physicalLocation": {
                                "artifactLocation": {
                                    "uri": self.path.display().to_string(),
                                },
                            },
                        },
                    ],
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

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct Check {
    pub(crate) id: &'static str,
    pub(crate) status: CheckStatus,
    pub(crate) severity: Severity,
    pub(crate) message: String,
    pub(crate) remediation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) documentation_url: Option<&'static str>,
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
}

#[derive(Debug, Serialize)]
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
