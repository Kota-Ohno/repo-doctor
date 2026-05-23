use std::path::{Path, PathBuf};

use anyhow::Result;
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

#[derive(Debug, Serialize)]
pub struct Report {
    path: PathBuf,
    checks: Vec<Check>,
}

#[derive(Debug, Serialize)]
pub struct Check {
    id: &'static str,
    status: CheckStatus,
    message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Warn,
}

pub fn run(cli: Cli) -> Result<String> {
    match cli.command {
        Command::Check { path, format } => check_repository(&path, format),
        Command::Completions { shell } => completions(shell),
        Command::Man => man_page(),
    }
}

pub fn check_repository(path: &Path, format: OutputFormat) -> Result<String> {
    let report = inspect_repository(path);

    match format {
        OutputFormat::Text => Ok(format_text_report(&report)),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(&report)?),
    }
}

pub fn inspect_repository(path: &Path) -> Report {
    let checks = vec![
        check_file(path, "readme", "README.md", "README is present"),
        check_any_file(
            path,
            "license",
            &["LICENSE", "LICENSE.md", "LICENSE-MIT", "LICENSE-APACHE"],
            "License file is present",
        ),
        check_file(path, "gitignore", ".gitignore", ".gitignore is present"),
        check_file(path, "cargo_toml", "Cargo.toml", "Cargo.toml is present"),
        check_file(
            path,
            "github_actions",
            ".github/workflows",
            "GitHub Actions workflow directory is present",
        ),
    ];

    Report {
        path: path.to_path_buf(),
        checks,
    }
}

fn check_file(path: &Path, id: &'static str, relative: &str, pass_message: &str) -> Check {
    let candidate = path.join(relative);
    if candidate.exists() {
        Check {
            id,
            status: CheckStatus::Pass,
            message: pass_message.to_owned(),
        }
    } else {
        Check {
            id,
            status: CheckStatus::Warn,
            message: format!("Missing {relative}"),
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
            message: pass_message.to_owned(),
        }
    } else {
        Check {
            id,
            status: CheckStatus::Warn,
            message: format!("Missing one of {}", candidates.join(", ")),
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

        let output = check_repository(temp_dir.path(), OutputFormat::Text).unwrap();

        assert!(output.contains("[PASS] readme: README is present"));
        assert!(output.contains("[PASS] cargo_toml: Cargo.toml is present"));
        assert!(output.contains("[WARN] gitignore: Missing .gitignore"));
    }

    #[test]
    fn json_snapshot_is_stable() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("README.md"), "# Test\n").unwrap();
        fs::write(temp_dir.path().join("LICENSE"), "MIT\n").unwrap();
        fs::write(temp_dir.path().join(".gitignore"), "/target\n").unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]\n").unwrap();
        fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();

        let output = check_repository(temp_dir.path(), OutputFormat::Json).unwrap();
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        let checks = value.get("checks").unwrap();

        let checks = serde_json::to_string_pretty(checks).unwrap();

        insta::assert_snapshot!(checks, @r###"
        [
          {
            "id": "readme",
            "message": "README is present",
            "status": "pass"
          },
          {
            "id": "license",
            "message": "License file is present",
            "status": "pass"
          },
          {
            "id": "gitignore",
            "message": ".gitignore is present",
            "status": "pass"
          },
          {
            "id": "cargo_toml",
            "message": "Cargo.toml is present",
            "status": "pass"
          },
          {
            "id": "github_actions",
            "message": "GitHub Actions workflow directory is present",
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
