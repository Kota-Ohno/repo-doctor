use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

mod checks;
mod core;
mod profiles;
mod report;

pub use profiles::Profile;
pub use report::RunOutput;

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

        /// Profile to run.
        #[arg(long, value_enum, default_value_t = Profile::Auto)]
        profile: Profile,

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
    Markdown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum FailOn {
    Warn,
}

pub fn run(cli: Cli) -> Result<RunOutput> {
    match cli.command {
        Command::Check {
            path,
            format,
            profile,
            fail_on,
        } => check_repository(&path, format, profile, fail_on),
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
    profile: Profile,
    fail_on: Option<FailOn>,
) -> Result<RunOutput> {
    validate_repository_path(path)?;

    let report = inspect_repository(path, profile);
    let exit_code = if fail_on == Some(FailOn::Warn) && report.has_warnings() {
        1
    } else {
        0
    };

    let text = match format {
        OutputFormat::Text => report.format_text(),
        OutputFormat::Json => serde_json::to_string_pretty(&report)?,
        OutputFormat::Markdown => report.format_markdown(),
    };

    Ok(RunOutput { text, exit_code })
}

pub fn inspect_repository(path: &Path, profile: Profile) -> report::Report {
    let mut checks = core::inspect(path);
    let selected_profiles = profiles::resolve(path, profile);
    checks.extend(profiles::inspect(path, &selected_profiles));

    report::Report::new(path.to_path_buf(), selected_profiles, checks)
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
    fn auto_profile_runs_rust_checks_when_cargo_toml_exists() {
        let temp_dir = tempfile::tempdir().unwrap();
        write_rust_fixture(temp_dir.path());

        let output =
            check_repository(temp_dir.path(), OutputFormat::Text, Profile::Auto, None).unwrap();

        assert!(output.text.contains("[PASS] readme: README is present"));
        assert!(output.text.contains("Profiles: rust"));
        assert!(output.text.contains("Summary:"));
        assert!(
            output
                .text
                .contains("[PASS] rust_cargo_name: Cargo package `name` is set")
        );
    }

    #[test]
    fn generic_profile_skips_rust_checks() {
        let temp_dir = tempfile::tempdir().unwrap();
        write_rust_fixture(temp_dir.path());

        let output =
            check_repository(temp_dir.path(), OutputFormat::Text, Profile::Generic, None).unwrap();

        assert!(output.text.contains("[PASS] readme: README is present"));
        assert!(output.text.contains("Profiles: none"));
        assert!(!output.text.contains("rust_cargo_name"));
    }

    #[test]
    fn explicit_rust_profile_runs_rust_checks() {
        let temp_dir = tempfile::tempdir().unwrap();
        write_rust_fixture(temp_dir.path());

        let output =
            check_repository(temp_dir.path(), OutputFormat::Text, Profile::Rust, None).unwrap();

        assert!(output.text.contains("[PASS] rust_cargo_name"));
    }

    #[test]
    fn empty_repository_only_runs_core_checks_in_auto_mode() {
        let temp_dir = tempfile::tempdir().unwrap();

        let output =
            check_repository(temp_dir.path(), OutputFormat::Text, Profile::Auto, None).unwrap();

        assert!(output.text.contains("[WARN] readme"));
        assert!(!output.text.contains("rust_"));
        assert!(!output.text.contains("node_"));
        assert!(!output.text.contains("python_"));
        assert!(!output.text.contains("go_"));
    }

    #[test]
    fn explicit_node_profile_runs_node_checks() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(
            temp_dir.path().join("package.json"),
            r#"{
  "name": "demo",
  "version": "0.1.0",
  "description": "Demo package",
  "license": "MIT",
  "repository": "https://example.com/demo",
  "scripts": { "test": "node --test" },
  "engines": { "node": ">=20" }
}"#,
        )
        .unwrap();
        fs::write(temp_dir.path().join("package-lock.json"), "{}\n").unwrap();

        let output =
            check_repository(temp_dir.path(), OutputFormat::Text, Profile::Node, None).unwrap();

        assert!(output.text.contains("[PASS] node_name"));
        assert!(output.text.contains("[PASS] node_lockfile"));
    }

    #[test]
    fn explicit_python_profile_runs_python_checks() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(
            temp_dir.path().join("pyproject.toml"),
            r#"[project]
name = "demo"
version = "0.1.0"
description = "Demo package"
readme = "README.md"
license = "MIT"

[build-system]
requires = ["setuptools"]
"#,
        )
        .unwrap();
        fs::write(temp_dir.path().join("README.md"), "# Demo\n").unwrap();
        fs::write(temp_dir.path().join("uv.lock"), "\n").unwrap();

        let output =
            check_repository(temp_dir.path(), OutputFormat::Text, Profile::Python, None).unwrap();

        assert!(output.text.contains("[PASS] python_name"));
        assert!(output.text.contains("[PASS] python_build_system"));
    }

    #[test]
    fn explicit_go_profile_runs_go_checks() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(
            temp_dir.path().join("go.mod"),
            "module example.com/demo\n\ngo 1.22\n",
        )
        .unwrap();
        fs::write(temp_dir.path().join("go.sum"), "\n").unwrap();

        let output =
            check_repository(temp_dir.path(), OutputFormat::Text, Profile::Go, None).unwrap();

        assert!(output.text.contains("[PASS] go_module"));
        assert!(output.text.contains("[PASS] go_version"));
    }

    #[test]
    fn rejects_missing_repository_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let missing_path = temp_dir.path().join("missing");

        let error =
            check_repository(&missing_path, OutputFormat::Text, Profile::Auto, None).unwrap_err();

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

        let error =
            check_repository(&file_path, OutputFormat::Text, Profile::Auto, None).unwrap_err();

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

        let output =
            check_repository(temp_dir.path(), OutputFormat::Text, Profile::Auto, None).unwrap();

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

        let output =
            check_repository(temp_dir.path(), OutputFormat::Text, Profile::Auto, None).unwrap();

        assert!(
            output
                .text
                .contains("[WARN] github_actions: Missing .github/workflows/*.yml or *.yaml")
        );
    }

    #[test]
    fn fail_on_warn_sets_nonzero_exit_code() {
        let temp_dir = tempfile::tempdir().unwrap();

        let output = check_repository(
            temp_dir.path(),
            OutputFormat::Text,
            Profile::Auto,
            Some(FailOn::Warn),
        )
        .unwrap();

        assert_eq!(output.exit_code, 1);
    }

    #[test]
    fn json_output_keeps_schema_version() {
        let temp_dir = tempfile::tempdir().unwrap();
        write_rust_fixture(temp_dir.path());

        let output =
            check_repository(temp_dir.path(), OutputFormat::Json, Profile::Auto, None).unwrap();
        let value: serde_json::Value = serde_json::from_str(&output.text).unwrap();

        assert_eq!(value.get("schema_version").unwrap(), 1);
        assert_eq!(value.get("selected_profiles").unwrap()[0], "rust");
        assert!(value.get("summary").unwrap().is_object());
        assert!(value.get("checks").unwrap().is_array());
        assert!(output.text.contains("\"id\": \"rust_cargo_name\""));
    }

    #[test]
    fn markdown_output_contains_summary_and_table() {
        let temp_dir = tempfile::tempdir().unwrap();
        write_rust_fixture(temp_dir.path());

        let output =
            check_repository(temp_dir.path(), OutputFormat::Markdown, Profile::Auto, None).unwrap();

        assert!(output.text.contains("# repo-doctor report"));
        assert!(output.text.contains("- Profiles: rust"));
        assert!(
            output
                .text
                .contains("| Status | Rule | Message | Remediation |")
        );
        assert!(output.text.contains("| PASS | `rust_cargo_name` |"));
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

    fn write_rust_fixture(path: &Path) {
        fs::write(path.join("README.md"), "# Test\n").unwrap();
        fs::write(path.join("LICENSE"), "MIT\n").unwrap();
        fs::write(path.join(".gitignore"), "/target\n").unwrap();
        fs::write(
            path.join("Cargo.toml"),
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
        fs::write(path.join("Cargo.lock"), "# lock\n").unwrap();
    }
}
