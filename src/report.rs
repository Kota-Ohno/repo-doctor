use std::path::PathBuf;

use serde::Serialize;

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
