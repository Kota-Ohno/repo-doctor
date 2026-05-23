use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug)]
pub struct RunOutput {
    pub text: String,
    pub exit_code: i32,
}

#[derive(Debug, Serialize)]
pub struct Report {
    schema_version: u16,
    path: PathBuf,
    checks: Vec<Check>,
}

impl Report {
    pub fn new(path: PathBuf, checks: Vec<Check>) -> Self {
        Self {
            schema_version: 1,
            path,
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

        for check in &self.checks {
            let marker = match check.status {
                CheckStatus::Pass => "PASS",
                CheckStatus::Warn => "WARN",
            };

            lines.push(format!("[{marker}] {}: {}", check.id, check.message));
        }

        lines.join("\n")
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
