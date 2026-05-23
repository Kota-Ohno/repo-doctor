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

    checks.extend(inspect_rust_project(path));

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
        pass(id, pass_message)
    } else {
        warn(
            id,
            format!("Missing {relative}"),
            format!("Add {relative}."),
        )
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
        pass(id, pass_message)
    } else {
        warn(
            id,
            format!("Missing one of {}", candidates.join(", ")),
            format!("Add one of {}.", candidates.join(", ")),
        )
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
        pass(id, pass_message)
    } else {
        warn(
            id,
            "Missing .github/workflows/*.yml or *.yaml",
            "Add at least one GitHub Actions workflow file under .github/workflows.",
        )
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
        pass(id, pass_message)
    } else {
        warn(
            id,
            format!("Missing files under {relative}"),
            format!("Add at least one file under {relative}."),
        )
    }
}

fn inspect_rust_project(path: &Path) -> Vec<Check> {
    let cargo_toml_path = path.join("Cargo.toml");
    let manifest = match std::fs::read_to_string(&cargo_toml_path) {
        Ok(contents) => contents,
        Err(_) => return Vec::new(),
    };

    let parsed = match toml::from_str::<toml::Value>(&manifest) {
        Ok(value) => value,
        Err(error) => {
            return vec![warn(
                "cargo_toml_parse",
                format!("Cargo.toml could not be parsed: {error}"),
                "Fix Cargo.toml so it is valid TOML.",
            )];
        }
    };

    let package = parsed.get("package").and_then(toml::Value::as_table);
    let Some(package) = package else {
        return vec![warn(
            "cargo_package",
            "Cargo.toml is missing [package]",
            "Add a [package] section or run repo-doctor from a package crate.",
        )];
    };

    let mut checks = vec![
        check_manifest_string(package, "cargo_name", "name"),
        check_manifest_string(package, "cargo_version", "version"),
        check_manifest_string(package, "cargo_edition", "edition"),
        check_manifest_string(package, "cargo_rust_version", "rust-version"),
        check_manifest_string(package, "cargo_description", "description"),
        check_manifest_string(package, "cargo_repository", "repository"),
        check_manifest_string(package, "cargo_readme", "readme"),
        check_cargo_license(package),
        check_manifest_path(path, package, "cargo_readme_path", "readme"),
        check_manifest_path(path, package, "cargo_license_file_path", "license-file"),
        check_cargo_lock(path),
        check_gitignore_target(path),
    ];

    checks.retain(|check| !check.id.is_empty());
    checks
}

fn check_manifest_string(
    package: &toml::map::Map<String, toml::Value>,
    id: &'static str,
    key: &'static str,
) -> Check {
    if package
        .get(key)
        .and_then(toml::Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
    {
        pass(id, format!("Cargo package `{key}` is set"))
    } else {
        warn(
            id,
            format!("Cargo package `{key}` is missing"),
            format!("Set `package.{key}` in Cargo.toml."),
        )
    }
}

fn check_cargo_license(package: &toml::map::Map<String, toml::Value>) -> Check {
    let has_license = ["license", "license-file"].iter().any(|key| {
        package
            .get(*key)
            .and_then(toml::Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
    });

    if has_license {
        pass("cargo_license", "Cargo package license metadata is set")
    } else {
        warn(
            "cargo_license",
            "Cargo package license metadata is missing",
            "Set `package.license` or `package.license-file` in Cargo.toml.",
        )
    }
}

fn check_manifest_path(
    path: &Path,
    package: &toml::map::Map<String, toml::Value>,
    id: &'static str,
    key: &'static str,
) -> Check {
    let Some(relative) = package.get(key).and_then(toml::Value::as_str) else {
        return Check {
            id: "",
            status: CheckStatus::Pass,
            severity: Severity::Info,
            message: String::new(),
            remediation: String::new(),
        };
    };

    if path.join(relative).exists() {
        pass(id, format!("Cargo package `{key}` path exists"))
    } else {
        warn(
            id,
            format!("Cargo package `{key}` path does not exist: {relative}"),
            format!("Create {relative} or update `package.{key}` in Cargo.toml."),
        )
    }
}

fn check_cargo_lock(path: &Path) -> Check {
    if path.join("Cargo.lock").exists() {
        pass("cargo_lock", "Cargo.lock is present")
    } else {
        warn(
            "cargo_lock",
            "Cargo.lock is missing",
            "Commit Cargo.lock for binary applications.",
        )
    }
}

fn check_gitignore_target(path: &Path) -> Check {
    let gitignore = match std::fs::read_to_string(path.join(".gitignore")) {
        Ok(contents) => contents,
        Err(_) => {
            return warn(
                "gitignore_target",
                ".gitignore could not be read",
                "Add a readable .gitignore containing /target.",
            );
        }
    };

    if gitignore
        .lines()
        .map(str::trim)
        .any(|line| matches!(line, "target" | "target/" | "/target" | "/target/"))
    {
        pass("gitignore_target", ".gitignore excludes Rust build output")
    } else {
        warn(
            "gitignore_target",
            ".gitignore does not exclude Rust build output",
            "Add /target to .gitignore.",
        )
    }
}

fn pass(id: &'static str, message: impl Into<String>) -> Check {
    Check {
        id,
        status: CheckStatus::Pass,
        severity: Severity::Info,
        message: message.into(),
        remediation: "No action needed.".to_owned(),
    }
}

fn warn(id: &'static str, message: impl Into<String>, remediation: impl Into<String>) -> Check {
    Check {
        id,
        status: CheckStatus::Warn,
        severity: Severity::Warning,
        message: message.into(),
        remediation: remediation.into(),
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
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
rust-version = "1.95"
description = "Test package"
repository = "https://example.com/test"
readme = "README.md"
license = "MIT"
"#,
        )
        .unwrap();

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
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
rust-version = "1.95"
description = "Test package"
repository = "https://example.com/test"
readme = "README.md"
license = "MIT"
"#,
        )
        .unwrap();
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
          },
          {
            "id": "cargo_name",
            "message": "Cargo package `name` is set",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "cargo_version",
            "message": "Cargo package `version` is set",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "cargo_edition",
            "message": "Cargo package `edition` is set",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "cargo_rust_version",
            "message": "Cargo package `rust-version` is set",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "cargo_description",
            "message": "Cargo package `description` is set",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "cargo_repository",
            "message": "Cargo package `repository` is set",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "cargo_readme",
            "message": "Cargo package `readme` is set",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "cargo_license",
            "message": "Cargo package license metadata is set",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "cargo_readme_path",
            "message": "Cargo package `readme` path exists",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
          },
          {
            "id": "cargo_lock",
            "message": "Cargo.lock is missing",
            "remediation": "Commit Cargo.lock for binary applications.",
            "severity": "warning",
            "status": "warn"
          },
          {
            "id": "gitignore_target",
            "message": ".gitignore excludes Rust build output",
            "remediation": "No action needed.",
            "severity": "info",
            "status": "pass"
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
