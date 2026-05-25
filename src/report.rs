use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde::Serialize;
use serde_json::json;

use crate::profiles::Profile;

pub struct RuleInfo {
    pub id: &'static str,
    pub severity: &'static str,
    pub category: &'static str,
    pub description: &'static str,
}

const fn rule(id: &'static str, category: &'static str, description: &'static str) -> RuleInfo {
    RuleInfo {
        id,
        severity: "warning",
        category,
        description,
    }
}

pub fn known_rules() -> Vec<RuleInfo> {
    vec![
        RuleInfo {
            id: "readme",
            severity: "warning",
            category: "core",
            description: "README is present",
        },
        RuleInfo {
            id: "license",
            severity: "warning",
            category: "core",
            description: "License file is present",
        },
        RuleInfo {
            id: "gitignore",
            severity: "warning",
            category: "core",
            description: ".gitignore is present",
        },
        RuleInfo {
            id: "editorconfig",
            severity: "warning",
            category: "core",
            description: ".editorconfig is present",
        },
        RuleInfo {
            id: "gitattributes",
            severity: "warning",
            category: "core",
            description: ".gitattributes is present",
        },
        RuleInfo {
            id: "env_example",
            severity: "warning",
            category: "core",
            description: "Example environment file is present",
        },
        RuleInfo {
            id: "docs_or_examples",
            severity: "warning",
            category: "core",
            description: "Documentation or examples directory is present",
        },
        RuleInfo {
            id: "secret_like_file",
            severity: "warning",
            category: "security",
            description: "Common secret-like files are absent",
        },
        RuleInfo {
            id: "monorepo_workspaces",
            severity: "warning",
            category: "core",
            description: "Monorepo or workspace layout hints are reported",
        },
        RuleInfo {
            id: "github_actions",
            severity: "warning",
            category: "ci",
            description: "GitHub Actions workflow is present",
        },
        RuleInfo {
            id: "github_actions_yaml",
            severity: "warning",
            category: "ci",
            description: "GitHub Actions workflow YAML parses",
        },
        RuleInfo {
            id: "github_actions_triggers",
            severity: "warning",
            category: "ci",
            description: "GitHub Actions run on push and pull_request",
        },
        RuleInfo {
            id: "github_actions_permissions",
            severity: "warning",
            category: "ci",
            description: "GitHub Actions permissions are declared",
        },
        RuleInfo {
            id: "github_actions_floating_refs",
            severity: "warning",
            category: "ci",
            description: "GitHub Actions avoid floating action refs",
        },
        rule(
            "github_actions_pull_request_target",
            "ci",
            "GitHub Actions avoid dangerous pull_request_target triggers",
        ),
        rule(
            "github_actions_rust_ci",
            "ci",
            "GitHub Actions include Rust fmt, clippy, and test commands",
        ),
        RuleInfo {
            id: "dependency_update_config",
            severity: "warning",
            category: "ci",
            description: "Dependabot or Renovate config is present",
        },
        RuleInfo {
            id: "contributing",
            severity: "warning",
            category: "community",
            description: "Contribution guide is present",
        },
        RuleInfo {
            id: "code_of_conduct",
            severity: "warning",
            category: "community",
            description: "Code of conduct is present",
        },
        RuleInfo {
            id: "security_policy",
            severity: "warning",
            category: "community",
            description: "Security policy is present",
        },
        RuleInfo {
            id: "issue_templates",
            severity: "warning",
            category: "community",
            description: "Issue template is present",
        },
        RuleInfo {
            id: "issue_template_frontmatter",
            severity: "warning",
            category: "community",
            description: "Issue template frontmatter is populated",
        },
        RuleInfo {
            id: "pull_request_template",
            severity: "warning",
            category: "community",
            description: "Pull request template is present",
        },
        RuleInfo {
            id: "changelog",
            severity: "warning",
            category: "community",
            description: "Changelog or release notes are present",
        },
        rule(
            "config_disabled_rule_reason",
            "config",
            "Disabled rules include a rationale",
        ),
        rule(
            "guard_git_diff",
            "guard",
            "Guard mode can inspect Git changes",
        ),
        rule(
            "guard_large_change_set",
            "guard",
            "Guard mode flags unusually large change sets",
        ),
        rule(
            "guard_secret_added",
            "guard",
            "Guard mode blocks newly added secret-like files",
        ),
        rule(
            "guard_ci_modified",
            "guard",
            "Guard mode highlights CI and dependency automation changes",
        ),
        rule(
            "guard_guardrail_removed",
            "guard",
            "Guard mode detects removed repository guardrail files",
        ),
        rule(
            "guard_tests_deleted",
            "guard",
            "Guard mode detects deleted test files",
        ),
        rule(
            "guard_lockfile_sync",
            "guard",
            "Guard mode detects manifest changes without lockfile updates",
        ),
        rule(
            "agent_instructions",
            "guard",
            "Agent instructions are present",
        ),
        rule(
            "agent_verification",
            "guard",
            "Agent instructions document verification commands",
        ),
        rule(
            "agent_boundaries",
            "guard",
            "Agent instructions document editing boundaries",
        ),
        RuleInfo {
            id: "rust_cargo_name",
            severity: "warning",
            category: "profile:rust",
            description: "Cargo package name is set",
        },
        RuleInfo {
            id: "rust_cargo_version",
            severity: "warning",
            category: "profile:rust",
            description: "Cargo package version is set",
        },
        RuleInfo {
            id: "rust_cargo_edition",
            severity: "warning",
            category: "profile:rust",
            description: "Cargo package edition is set",
        },
        rule("rust_cargo_toml", "profile:rust", "Cargo.toml is present"),
        rule(
            "rust_cargo_toml_parse",
            "profile:rust",
            "Cargo.toml parses as TOML",
        ),
        rule(
            "rust_cargo_package",
            "profile:rust",
            "Cargo.toml package or workspace metadata is present",
        ),
        rule(
            "rust_cargo_rust_version",
            "profile:rust",
            "Cargo package rust-version is set",
        ),
        rule(
            "rust_cargo_description",
            "profile:rust",
            "Cargo package description is set",
        ),
        rule(
            "rust_cargo_repository",
            "profile:rust",
            "Cargo package repository is set",
        ),
        rule(
            "rust_cargo_readme",
            "profile:rust",
            "Cargo package readme metadata is set",
        ),
        rule(
            "rust_cargo_license",
            "profile:rust",
            "Cargo package license metadata is set",
        ),
        rule(
            "rust_cargo_readme_path",
            "profile:rust",
            "Cargo package readme path exists",
        ),
        rule(
            "rust_cargo_license_file_path",
            "profile:rust",
            "Cargo package license-file path exists",
        ),
        rule(
            "rust_workspace",
            "profile:rust",
            "Cargo workspace configuration is valid",
        ),
        RuleInfo {
            id: "rust_cargo_lock",
            severity: "warning",
            category: "profile:rust",
            description: "Cargo.lock is present",
        },
        rule(
            "rust_gitignore_target",
            "profile:rust",
            ".gitignore excludes Rust build output",
        ),
        RuleInfo {
            id: "rust_readme_commands",
            severity: "warning",
            category: "profile:rust",
            description: "README documents Rust usage and test commands",
        },
        RuleInfo {
            id: "rust_toolchain",
            severity: "warning",
            category: "profile:rust",
            description: "Rust toolchain pin is present",
        },
        RuleInfo {
            id: "rust_tooling_config",
            severity: "warning",
            category: "profile:rust",
            description: "Rust rustfmt and clippy configs are present",
        },
        RuleInfo {
            id: "node_name",
            severity: "warning",
            category: "profile:node",
            description: "package.json name is set",
        },
        rule(
            "node_package_json",
            "profile:node",
            "package.json is present",
        ),
        rule(
            "node_package_json_parse",
            "profile:node",
            "package.json parses as JSON",
        ),
        rule(
            "node_version",
            "profile:node",
            "package.json version is set",
        ),
        rule(
            "node_description",
            "profile:node",
            "package.json description is set",
        ),
        rule(
            "node_license",
            "profile:node",
            "package.json license is set",
        ),
        rule(
            "node_repository",
            "profile:node",
            "package.json repository metadata is set",
        ),
        RuleInfo {
            id: "node_test_script",
            severity: "warning",
            category: "profile:node",
            description: "package.json scripts.test is set",
        },
        rule(
            "node_engines",
            "profile:node",
            "package.json engines.node is set",
        ),
        rule(
            "node_lockfile",
            "profile:node",
            "Node package manager lockfile is present",
        ),
        RuleInfo {
            id: "node_typescript_config",
            severity: "warning",
            category: "profile:node",
            description: "TypeScript compiler configuration is present when applicable",
        },
        RuleInfo {
            id: "node_lint_format",
            severity: "warning",
            category: "profile:node",
            description: "Node lint and format commands are configured",
        },
        RuleInfo {
            id: "python_project_metadata",
            severity: "warning",
            category: "profile:python",
            description: "pyproject.toml has project metadata",
        },
        rule(
            "python_pyproject",
            "profile:python",
            "pyproject.toml is present when expected",
        ),
        rule(
            "python_pyproject_parse",
            "profile:python",
            "pyproject.toml parses as TOML",
        ),
        rule(
            "python_name",
            "profile:python",
            "Python project name is set",
        ),
        rule(
            "python_version",
            "profile:python",
            "Python project version is set",
        ),
        rule(
            "python_description",
            "profile:python",
            "Python project description is set",
        ),
        rule(
            "python_readme",
            "profile:python",
            "Python project README metadata is set",
        ),
        rule(
            "python_readme_path",
            "profile:python",
            "Python project README path exists",
        ),
        rule(
            "python_legacy_setup",
            "profile:python",
            "Legacy Python setup file is present when used",
        ),
        rule(
            "python_requirements",
            "profile:python",
            "Python requirements file is present when used",
        ),
        rule(
            "python_tests",
            "profile:python",
            "Python tests directory is present",
        ),
        RuleInfo {
            id: "python_lint_format",
            severity: "warning",
            category: "profile:python",
            description: "Python lint or format tooling is configured",
        },
        RuleInfo {
            id: "python_pytest_config",
            severity: "warning",
            category: "profile:python",
            description: "Python pytest configuration is present",
        },
        rule(
            "python_license",
            "profile:python",
            "Python project license metadata is set",
        ),
        rule(
            "python_build_system",
            "profile:python",
            "Python build-system is configured",
        ),
        rule(
            "python_lockfile",
            "profile:python",
            "Python lockfile is present",
        ),
        rule("go_mod", "profile:go", "go.mod is present"),
        RuleInfo {
            id: "go_module",
            severity: "warning",
            category: "profile:go",
            description: "go.mod module declaration is present",
        },
        rule("go_version", "profile:go", "go.mod go version is present"),
        rule("go_sum", "profile:go", "go.sum is present"),
        RuleInfo {
            id: "go_ci_commands",
            severity: "warning",
            category: "profile:go",
            description: "Go CI includes test, vet, and formatting commands",
        },
        RuleInfo {
            id: "docker_build_file",
            severity: "warning",
            category: "profile:docker",
            description: "Container build file is present",
        },
        RuleInfo {
            id: "dockerignore",
            severity: "warning",
            category: "profile:docker",
            description: ".dockerignore is present",
        },
        RuleInfo {
            id: "docker_compose",
            severity: "warning",
            category: "profile:docker",
            description: "Compose file is present when local multi-service development is expected",
        },
        RuleInfo {
            id: "docker_base_image_pin",
            severity: "warning",
            category: "profile:docker",
            description: "Container base image tags avoid :latest",
        },
        RuleInfo {
            id: "docker_healthcheck",
            severity: "warning",
            category: "profile:docker",
            description: "Container HEALTHCHECK is configured",
        },
        RuleInfo {
            id: "docker_non_root_user",
            severity: "warning",
            category: "profile:docker",
            description: "Container switches to a configured USER",
        },
        rule("jvm_build_file", "profile:jvm", "JVM build file is present"),
        rule("jvm_maven_pom", "profile:jvm", "Maven pom.xml is valid"),
        rule(
            "jvm_gradle_build",
            "profile:jvm",
            "Gradle build file is present",
        ),
        rule(
            "jvm_gradle_settings",
            "profile:jvm",
            "Gradle settings file is present",
        ),
        rule(
            "jvm_gradle_group",
            "profile:jvm",
            "Gradle group is configured",
        ),
        rule(
            "jvm_gradle_version",
            "profile:jvm",
            "Gradle version is configured",
        ),
        rule(
            "jvm_gradle_test",
            "profile:jvm",
            "Gradle test task is configured",
        ),
        rule("jvm_wrapper", "profile:jvm", "JVM build wrapper is present"),
        rule("deno_config", "profile:deno", "Deno config is present"),
        rule("deno_lock", "profile:deno", "deno.lock is present"),
        rule("deno_tasks", "profile:deno", "Deno tasks are configured"),
        rule(
            "bun_package_json",
            "profile:bun",
            "package.json is present for Bun projects",
        ),
        rule("bun_package_name", "profile:bun", "Bun package name is set"),
        rule("bun_test_script", "profile:bun", "Bun test script is set"),
        rule("bun_lockfile", "profile:bun", "Bun lockfile is present"),
        rule(
            "bun_package_manager",
            "profile:bun",
            "packageManager is set to Bun",
        ),
        rule(
            "dotnet_project",
            "profile:dotnet",
            ".NET project or solution file is present",
        ),
        rule(
            "dotnet_global_json",
            "profile:dotnet",
            ".NET global.json SDK pin is present",
        ),
        rule("dotnet_tests", "profile:dotnet", ".NET tests are present"),
        rule(
            "php_composer_json",
            "profile:php",
            "composer.json is present",
        ),
        rule("php_name", "profile:php", "Composer package name is set"),
        rule(
            "php_description",
            "profile:php",
            "Composer package description is set",
        ),
        rule("php_license", "profile:php", "Composer license is set"),
        rule(
            "php_require",
            "profile:php",
            "Composer requirements are set",
        ),
        rule(
            "php_composer_lock",
            "profile:php",
            "composer.lock is present",
        ),
        rule(
            "php_test_script",
            "profile:php",
            "Composer test script is set",
        ),
        rule("ruby_gemfile", "profile:ruby", "Gemfile is present"),
        rule(
            "ruby_gemfile_lock",
            "profile:ruby",
            "Gemfile.lock is present",
        ),
        rule("ruby_gemspec", "profile:ruby", "Ruby gemspec is present"),
        rule(
            "cpp_build_system",
            "profile:cpp",
            "C/C++ build system file is present",
        ),
        rule(
            "cpp_dependency_manifest",
            "profile:cpp",
            "C/C++ dependency manifest is present",
        ),
        rule(
            "cpp_tooling_metadata",
            "profile:cpp",
            "C/C++ tooling metadata is present",
        ),
        rule("swift_package", "profile:swift", "Package.swift is present"),
        rule(
            "swift_package_resolved",
            "profile:swift",
            "Package.resolved is present",
        ),
        rule("swift_tests", "profile:swift", "Swift tests are present"),
        rule(
            "kotlin_build_file",
            "profile:kotlin",
            "Kotlin build file is present",
        ),
        rule(
            "kotlin_plugin",
            "profile:kotlin",
            "Kotlin plugin is configured",
        ),
        rule(
            "kotlin_sources",
            "profile:kotlin",
            "Kotlin sources are present",
        ),
        RuleInfo {
            id: "frontend_framework",
            severity: "warning",
            category: "profile:frontend",
            description: "Frontend framework metadata is present",
        },
        RuleInfo {
            id: "frontend_build_script",
            severity: "warning",
            category: "profile:frontend",
            description: "Frontend build script is configured",
        },
        RuleInfo {
            id: "frontend_source_dir",
            severity: "warning",
            category: "profile:frontend",
            description: "Frontend source directory is present",
        },
        RuleInfo {
            id: "iac_terraform_files",
            severity: "warning",
            category: "profile:iac",
            description: "Terraform/OpenTofu files are present",
        },
        RuleInfo {
            id: "iac_lockfile",
            severity: "warning",
            category: "profile:iac",
            description: "IaC provider lockfile is present",
        },
        RuleInfo {
            id: "iac_ci_validate",
            severity: "warning",
            category: "profile:iac",
            description: "IaC formatting or validation is present in CI",
        },
        RuleInfo {
            id: "docs_site_config",
            severity: "warning",
            category: "profile:docs",
            description: "Docs site configuration is present",
        },
        RuleInfo {
            id: "docs_site_content",
            severity: "warning",
            category: "profile:docs",
            description: "Docs site content directory is present",
        },
        RuleInfo {
            id: "docs_site_ci",
            severity: "warning",
            category: "profile:docs",
            description: "Docs site build is present in CI",
        },
        RuleInfo {
            id: "github_remote_branch_protection",
            severity: "warning",
            category: "remote",
            description: "Default branch protection is enabled",
        },
        rule(
            "github_remote_repo",
            "remote",
            "GitHub repository metadata is available",
        ),
        rule(
            "github_remote_description",
            "remote",
            "GitHub repository description is set",
        ),
        rule(
            "github_remote_topics",
            "remote",
            "GitHub repository topics are set",
        ),
        rule(
            "github_remote_default_branch",
            "remote",
            "GitHub default branch is set",
        ),
        rule(
            "github_remote_archived",
            "remote",
            "GitHub repository is not archived",
        ),
        rule(
            "github_remote_recent_activity",
            "remote",
            "GitHub repository has recent activity",
        ),
        rule(
            "github_remote_license",
            "remote",
            "GitHub license is detected",
        ),
        rule("github_remote_homepage", "remote", "GitHub homepage is set"),
        rule(
            "github_remote_vulnerability_alerts",
            "remote",
            "GitHub vulnerability alerts are enabled",
        ),
        rule(
            "github_remote_community_profile",
            "remote",
            "GitHub community profile is available",
        ),
        rule(
            "github_remote_community_health",
            "remote",
            "GitHub community health score is high enough",
        ),
        rule(
            "github_remote_community_files",
            "remote",
            "GitHub community files are detected",
        ),
        rule(
            "github_remote_scorecard_link",
            "remote",
            "OpenSSF Scorecard link is available",
        ),
        rule(
            "github_remote_latest_release",
            "remote",
            "GitHub latest release is present",
        ),
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

    pub fn warning_baseline(&self) -> Vec<BaselineWarning> {
        self.checks
            .iter()
            .filter(|check| matches!(check.status, CheckStatus::Warn))
            .map(|check| BaselineWarning {
                id: check.id.to_owned(),
                message: Some(check.message.clone()),
                location: check
                    .location
                    .as_ref()
                    .map(|location| location.path.clone()),
            })
            .collect()
    }

    pub fn suppress_baseline_warnings(&self, baseline: &[BaselineWarning]) -> Self {
        let checks = self
            .checks
            .iter()
            .filter(|check| {
                !matches!(check.status, CheckStatus::Warn)
                    || !baseline.iter().any(|entry| entry.matches(check))
            })
            .cloned()
            .collect();

        Self::new(self.path.clone(), self.selected_profiles.clone(), checks)
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

    pub fn format_summary(&self) -> String {
        let mut lines = vec![
            format!("repo-doctor {}", self.path.display()),
            format!(
                "profiles={} pass={} warn={} total={} score={}",
                format_profiles(&self.selected_profiles),
                self.summary.pass,
                self.summary.warn,
                self.summary.total,
                self.summary.score
            ),
        ];

        let warnings = self.warning_details();
        if warnings.is_empty() {
            lines.push(
                "next=add repo-doctor to CI with `repo-doctor ci --template generic`".to_owned(),
            );
        } else {
            lines.push("warnings:".to_owned());
            for warning in warnings.iter().take(8) {
                lines.push(format!("- {}: {}", warning.id, warning.message));
                lines.push(format!("  fix: {}", warning.remediation));
            }
            if warnings.len() > 8 {
                lines.push(format!(
                    "- ... {} more; run `repo-doctor check --warnings-only`",
                    warnings.len() - 8
                ));
            }
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BaselineWarning {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

impl BaselineWarning {
    fn matches(&self, check: &Check) -> bool {
        if self.id != check.id {
            return false;
        }
        if self
            .message
            .as_deref()
            .is_some_and(|message| message != check.message)
        {
            return false;
        }
        if let Some(location) = self.location.as_deref() {
            return check
                .location
                .as_ref()
                .is_some_and(|check_location| check_location.path == location);
        }
        true
    }
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

impl std::fmt::Display for Severity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => formatter.write_str("info"),
            Severity::Warning => formatter.write_str("warning"),
        }
    }
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
