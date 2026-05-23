use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Check repository hygiene.
    Check {
        /// Repository directory to inspect.
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Output format.
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,

        /// Exit nonzero when the report contains findings at this level.
        #[arg(long, value_enum)]
        fail_on: Option<FailOn>,
    },

    /// Generate shell completion scripts.
    Completions {
        /// Shell to generate completions for.
        shell: Shell,
    },

    /// Generate a manual page.
    Man,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum FailOn {
    Warn,
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
    checks: Vec<Check>,
}

#[derive(Debug, Serialize)]
pub struct Check {
    id: &'static str,
    status: CheckStatus,
    severity: Severity,
    message: String,
    remediation: String,
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

pub fn run(cli: Cli) -> Result<RunOutput> {
    match cli.command {
        Command::Check {
            path,
            format,
            fail_on,
        } => check_repository(&path, format, fail_on),
        Command::Completions { shell } => Ok(RunOutput {
            text: completions(shell)?,
            exit_code: 0,
        }),
        Command::Man => Ok(RunOutput {
            text: man_page()?,
            exit_code: 0,
        }),
    }
}

pub fn check_repository(
    path: &Path,
    format: OutputFormat,
    fail_on: Option<FailOn>,
) -> Result<RunOutput> {
    validate_repository_path(path)?;

    let report = inspect_repository(path);
    let exit_code = if fail_on == Some(FailOn::Warn) && report.has_warnings() {
        1
    } else {
        0
    };

    let text = match format {
        OutputFormat::Text => format_text_report(&report),
        OutputFormat::Json => serde_json::to_string_pretty(&report)?,
    };

    Ok(RunOutput { text, exit_code })
}

pub fn inspect_repository(path: &Path) -> Report {
    let checks = vec![
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
        check_file(path, "cargo_toml", "Cargo.toml", "Cargo.toml is present"),
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

    Report {
        schema_version: 1,
        path: path.to_path_buf(),
        checks,
    }
}

impl Report {
    fn has_warnings(&self) -> bool {
        self.checks
            .iter()
            .any(|check| matches!(check.status, CheckStatus::Warn))
    }
}

fn validate_repository_path(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("repository path does not exist: {}", path.display());
    }

    if !path.is_dir() {
        bail!("repository path is not a directory: {}", path.display());
    }

    Ok(())
}

fn check_file(path: &Path, id: &'static str, relative: &str, pass_message: &str) -> Check {
    let candidate = path.join(relative);
    if candidate.exists() {
        Check {
            id,
            status: CheckStatus::Pass,
            severity: Severity::Info,
            message: pass_message.to_owned(),
            remediation: "No action needed.".to_owned(),
        }
    } else {
        Check {
            id,
            status: CheckStatus::Warn,
            severity: Severity::Warning,
            message: format!("Missing {relative}"),
            remediation: format!("Add {relative}."),
        }
    }
}

fn check_any_file(
    path: &Path,
    id: &'static str,
    candidates: &[&'static str],
    pass_message: &str,
) -> Check {
    if candidates
        .iter()
        .any(|candidate| path.join(candidate).exists())
    {
        Check {
            id,
            status: CheckStatus::Pass,
            severity: Severity::Info,
            message: pass_message.to_owned(),
            remediation: "No action needed.".to_owned(),
        }
    } else {
        Check {
            id,
            status: CheckStatus::Warn,
            severity: Severity::Warning,
            message: format!("Missing one of {}", candidates.join(", ")),
            remediation: format!("Add one of {}.", candidates.join(", ")),
        }
    }
}

fn check_workflows(path: &Path, id: &'static str, pass_message: &str) -> Check {
    let workflows_dir = path.join(".github/workflows");
    let has_workflow = workflows_dir
        .read_dir()
        .map(|entries| {
            entries.filter_map(Result::ok).any(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "yml" || extension == "yaml")
            })
        })
        .unwrap_or(false);

    if has_workflow {
        Check {
            id,
            status: CheckStatus::Pass,
            severity: Severity::Info,
            message: pass_message.to_owned(),
            remediation: "No action needed.".to_owned(),
        }
    } else {
        Check {
            id,
            status: CheckStatus::Warn,
            severity: Severity::Warning,
            message: "Missing .github/workflows/*.yml or *.yaml".to_owned(),
            remediation: "Add at least one GitHub Actions workflow file under .github/workflows."
                .to_owned(),
        }
    }
}

fn check_directory_has_file(
    path: &Path,
    id: &'static str,
    relative: &str,
    pass_message: &str,
) -> Check {
    let candidate = path.join(relative);
    let has_file = candidate
        .read_dir()
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .any(|entry| entry.path().is_file())
        })
        .unwrap_or(false);

    if has_file {
        Check {
            id,
            status: CheckStatus::Pass,
            severity: Severity::Info,
            message: pass_message.to_owned(),
            remediation: "No action needed.".to_owned(),
        }
    } else {
        Check {
            id,
            status: CheckStatus::Warn,
            severity: Severity::Warning,
            message: format!("Missing files under {relative}"),
            remediation: format!("Add at least one file under {relative}."),
        }
    }
}

fn format_text_report(report: &Report) -> String {
    let mut lines = vec![format!("Repository: {}", report.path.display())];

    for check in &report.checks {
        let marker = match check.status {
            CheckStatus::Pass => "PASS",
            CheckStatus::Warn => "WARN",
        };

        lines.push(format!("[{marker}] {}: {}", check.id, check.message));
    }

    lines.join("\n")
}

pub fn completions(shell: Shell) -> Result<String> {
    let mut command = Cli::command();
    let name = command.get_name().to_owned();
    let mut buffer = Vec::new();

    clap_complete::generate(shell, &mut command, name, &mut buffer);

    Ok(String::from_utf8(buffer)?)
}

pub fn man_page() -> Result<String> {
    let command = Cli::command();
    let man = clap_mangen::Man::new(command);
    let mut buffer = Vec::new();

    man.render(&mut buffer)?;

    Ok(String::from_utf8(buffer)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn reports_present_and_missing_files_as_text() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("README.md"), "# Test\n").unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]\n").unwrap();

        let output = check_repository(temp_dir.path(), OutputFormat::Text, None).unwrap();

        assert!(output.text.contains("[PASS] readme: README is present"));
        assert!(
            output
                .text
                .contains("[PASS] cargo_toml: Cargo.toml is present")
        );
        assert!(output.text.contains("[WARN] gitignore: Missing .gitignore"));
        assert!(
            output
                .text
                .contains("[WARN] contributing: Missing one of CONTRIBUTING.md")
        );
    }

    #[test]
    fn rejects_missing_repository_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let missing_path = temp_dir.path().join("missing");

        let error = check_repository(&missing_path, OutputFormat::Text, None).unwrap_err();

        assert_eq!(
            error.to_string(),
            format!("repository path does not exist: {}", missing_path.display())
        );
    }

    #[test]
    fn rejects_file_repository_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("README.md");
        fs::write(&file_path, "# Test\n").unwrap();

        let error = check_repository(&file_path, OutputFormat::Text, None).unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "repository path is not a directory: {}",
                file_path.display()
            )
        );
    }

    #[test]
    fn accepts_common_readme_and_license_names() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("README.txt"), "# Test\n").unwrap();
        fs::write(temp_dir.path().join("LICENSE.txt"), "MIT\n").unwrap();

        let output = check_repository(temp_dir.path(), OutputFormat::Text, None).unwrap();

        assert!(output.text.contains("[PASS] readme: README is present"));
        assert!(
            output
                .text
                .contains("[PASS] license: License file is present")
        );
    }

    #[test]
    fn warns_when_workflows_directory_has_no_workflow_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();
        fs::write(
            temp_dir.path().join(".github/workflows/README.md"),
            "# Workflows\n",
        )
        .unwrap();

        let output = check_repository(temp_dir.path(), OutputFormat::Text, None).unwrap();

        assert!(
            output
                .text
                .contains("[WARN] github_actions: Missing .github/workflows/*.yml or *.yaml")
        );
    }

    #[test]
    fn fail_on_warn_sets_nonzero_exit_code() {
        let temp_dir = tempfile::tempdir().unwrap();

        let output =
            check_repository(temp_dir.path(), OutputFormat::Text, Some(FailOn::Warn)).unwrap();

        assert_eq!(output.exit_code, 1);
    }

    #[test]
    fn json_snapshot_is_stable() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("README.md"), "# Test\n").unwrap();
        fs::write(temp_dir.path().join("LICENSE"), "MIT\n").unwrap();
        fs::write(temp_dir.path().join(".gitignore"), "/target\n").unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]\n").unwrap();
        fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();
        fs::write(
            temp_dir.path().join(".github/workflows/ci.yml"),
            "name: CI\n",
        )
        .unwrap();
        fs::write(temp_dir.path().join("CONTRIBUTING.md"), "# Contributing\n").unwrap();
        fs::write(temp_dir.path().join("SECURITY.md"), "# Security\n").unwrap();
        fs::create_dir_all(temp_dir.path().join(".github/ISSUE_TEMPLATE")).unwrap();
        fs::write(
            temp_dir.path().join(".github/ISSUE_TEMPLATE/bug_report.md"),
            "name: Bug report\n",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join(".github/pull_request_template.md"),
            "# Pull request\n",
        )
        .unwrap();

        let output = check_repository(temp_dir.path(), OutputFormat::Json, None).unwrap();
        let value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
        let checks = value.get("checks").unwrap();

        let checks = serde_json::to_string_pretty(checks).unwrap();

        insta::assert_snapshot!(checks, @r###"
        [
          {
            "id": "readme",
            "message": "README is present",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "license",
            "message": "License file is present",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "gitignore",
            "message": ".gitignore is present",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "cargo_toml",
            "message": "Cargo.toml is present",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "github_actions",
            "message": "GitHub Actions workflow file is present",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "contributing",
            "message": "Contribution guide is present",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "code_of_conduct",
            "message": "Missing one of CODE_OF_CONDUCT.md, docs/CODE_OF_CONDUCT.md, .github/CODE_OF_CONDUCT.md",
            "remediation": "Add one of CODE_OF_CONDUCT.md, docs/CODE_OF_CONDUCT.md, .github/CODE_OF_CONDUCT.md.",
            "severity": "warning",
            "status": "warn"
          },
          {
            "id": "security_policy",
            "message": "Security policy is present",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "issue_templates",
            "message": "Issue template is present",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "pull_request_template",
            "message": "Pull request template is present",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "changelog",
            "message": "Missing one of CHANGELOG.md, CHANGES.md, RELEASES.md, docs/CHANGELOG.md",
            "remediation": "Add one of CHANGELOG.md, CHANGES.md, RELEASES.md, docs/CHANGELOG.md.",
            "severity": "warning",
            "status": "warn"
          }
        ]
        "###);
    }

    #[test]
    fn generates_bash_completions() {
        let output = completions(Shell::Bash).unwrap();
        let function_name = format!("_{}", env!("CARGO_PKG_NAME"));

        assert!(output.contains(&function_name));
    }

    #[test]
    fn generates_man_page() {
        let output = man_page().unwrap();
        let package_name = env!("CARGO_PKG_NAME");
        let escaped_name = package_name.replace('-', "\\-");

        assert!(output.contains(&format!(".TH {package_name}")));
        assert!(output.contains(&escaped_name));
    }
}
