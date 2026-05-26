use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::profiles::{self, Profile};
use crate::report::{self, Check, Severity, warn};

pub(crate) const STARTER_CONFIG: &str = r#"# repo-doctor configuration
# profiles is used only when the CLI profile is left as auto.
# profiles = ["auto"]

# Policy presets tune generic rules without changing the report schema.
# presets = ["oss"]
# Shape presets reduce expected noise for common project types:
# python-app, python-lib, node-app, node-lib, php-app, php-package,
# ruby-app, ruby-gem, cpp-app, cpp-lib, docker-service, docker-job.
# For private/internal repositories, `presets = ["internal"]` suppresses
# public community-file requirements while keeping core checks active.
# AI/VibeCoding guardrails do not require a preset; run
# `repo-doctor guard --fail-on warn`.

# File-based findings with matching locations are removed after checking.
# exclude_paths = ["vendor/*", "generated/*"]

# Rules can be disabled only with a rationale.
# [[rules]]
# id = "changelog"
# disabled = true
# reason = "Internal service with release notes in deployment tickets."

# Severity can be overridden per rule.
# [[rules]]
# id = "code_of_conduct"
# severity = "info"
"#;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct Config {
    profiles: Option<Vec<Profile>>,
    #[serde(default)]
    presets: Vec<Preset>,
    #[serde(default)]
    exclude_paths: Vec<String>,
    #[serde(default)]
    rules: Vec<RuleConfig>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Preset {
    RustCli,
    RustLib,
    NodeApp,
    NodeLib,
    PythonApp,
    PythonLib,
    GoModule,
    JvmApp,
    JvmLib,
    DotnetApp,
    DotnetLib,
    PhpApp,
    PhpPackage,
    RubyApp,
    RubyGem,
    CppApp,
    CppLib,
    SwiftPackage,
    KotlinApp,
    DockerService,
    DockerJob,
    Oss,
    Internal,
    Strict,
    Vibe,
}

#[derive(Debug, Deserialize)]
struct RuleConfig {
    id: String,
    #[serde(default)]
    disabled: bool,
    reason: Option<String>,
    severity: Option<SeverityOverride>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SeverityOverride {
    Info,
    Warning,
}

impl Config {
    pub(crate) fn selected_profiles(&self, path: &Path, cli_profile: Profile) -> Vec<Profile> {
        if cli_profile != Profile::Auto {
            return profiles::resolve(path, cli_profile);
        }

        let mut selected = self.preset_profiles();
        let Some(configured) = &self.profiles else {
            if selected.is_empty() {
                return profiles::resolve(path, cli_profile);
            }
            selected.extend(profiles::resolve(path, cli_profile));
            return dedupe_profiles(selected);
        };

        for profile in configured {
            match profile {
                Profile::Auto => selected.extend(profiles::resolve(path, Profile::Auto)),
                Profile::Generic => {}
                explicit => selected.push(*explicit),
            }
        }

        dedupe_profiles(selected)
    }

    pub(crate) fn selected_profiles_with_explicit(
        &self,
        path: &Path,
        cli_profile: Profile,
        explicit_profiles: &[Profile],
    ) -> Vec<Profile> {
        if explicit_profiles.is_empty() {
            self.selected_profiles(path, cli_profile)
        } else {
            profiles::resolve_many(path, explicit_profiles)
        }
    }

    pub(crate) fn apply(&self, checks: Vec<Check>) -> Vec<Check> {
        let disabled = self.disabled_rules_with_reason();
        let preset_disabled = self.preset_disabled_rules();
        let invalid_disabled = self.disabled_rules_without_reason();
        let severity_overrides = self.severity_overrides();

        let mut configured = checks
            .into_iter()
            .filter(|check| !disabled.contains(check.id()) && !preset_disabled.contains(check.id()))
            .filter(|check| !self.is_excluded(check))
            .map(|mut check| {
                if let Some(severity) = severity_overrides.get(check.id()) {
                    check.set_severity(*severity);
                }
                check
            })
            .collect::<Vec<_>>();

        configured.extend(invalid_disabled.into_iter().map(|id| {
            let mut check = warn(
                "config_disabled_rule_reason",
                format!("Rule `{id}` is disabled without a rationale"),
                "Add a non-empty reason to the disabled rule or remove the disabled flag.",
            );
            if let Some(severity) = severity_overrides.get(check.id()) {
                check.set_severity(*severity);
            }
            check
        }));

        configured
    }

    pub(crate) fn explain_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if !self.presets.is_empty() {
            lines.push(format!(
                "presets={}",
                self.presets
                    .iter()
                    .map(|preset| match preset {
                        Preset::RustCli => "rust-cli",
                        Preset::RustLib => "rust-lib",
                        Preset::NodeApp => "node-app",
                        Preset::NodeLib => "node-lib",
                        Preset::PythonApp => "python-app",
                        Preset::PythonLib => "python-lib",
                        Preset::GoModule => "go-module",
                        Preset::JvmApp => "jvm-app",
                        Preset::JvmLib => "jvm-lib",
                        Preset::DotnetApp => "dotnet-app",
                        Preset::DotnetLib => "dotnet-lib",
                        Preset::PhpApp => "php-app",
                        Preset::PhpPackage => "php-package",
                        Preset::RubyApp => "ruby-app",
                        Preset::RubyGem => "ruby-gem",
                        Preset::CppApp => "cpp-app",
                        Preset::CppLib => "cpp-lib",
                        Preset::SwiftPackage => "swift-package",
                        Preset::KotlinApp => "kotlin-app",
                        Preset::DockerService => "docker-service",
                        Preset::DockerJob => "docker-job",
                        Preset::Oss => "oss",
                        Preset::Internal => "internal",
                        Preset::Strict => "strict",
                        Preset::Vibe => "vibe",
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !self.exclude_paths.is_empty() {
            lines.push(format!("exclude_paths={}", self.exclude_paths.join(", ")));
        }
        let disabled = self.disabled_rules_with_reason();
        if !disabled.is_empty() {
            lines.push(format!(
                "disabled_rules={}",
                disabled.into_iter().collect::<Vec<_>>().join(", ")
            ));
        }
        let overrides = self.severity_overrides();
        if !overrides.is_empty() {
            lines.push(format!(
                "severity_overrides={}",
                overrides
                    .into_iter()
                    .map(|(id, severity)| format!("{id}:{severity}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if lines.is_empty() {
            lines.push("settings=defaults".to_owned());
        }
        lines
    }

    fn disabled_rules_with_reason(&self) -> HashSet<&str> {
        self.rules
            .iter()
            .filter(|rule| rule.disabled)
            .filter(|rule| {
                rule.reason
                    .as_deref()
                    .is_some_and(|reason| !reason.trim().is_empty())
            })
            .map(|rule| rule.id.as_str())
            .collect()
    }

    fn preset_profiles(&self) -> Vec<Profile> {
        self.presets
            .iter()
            .filter_map(|preset| match preset {
                Preset::RustCli | Preset::RustLib => Some(Profile::Rust),
                Preset::NodeApp | Preset::NodeLib => Some(Profile::Node),
                Preset::PythonApp | Preset::PythonLib => Some(Profile::Python),
                Preset::GoModule => Some(Profile::Go),
                Preset::JvmApp | Preset::JvmLib => Some(Profile::Jvm),
                Preset::DotnetApp | Preset::DotnetLib => Some(Profile::Dotnet),
                Preset::PhpApp | Preset::PhpPackage => Some(Profile::Php),
                Preset::RubyApp | Preset::RubyGem => Some(Profile::Ruby),
                Preset::CppApp | Preset::CppLib => Some(Profile::Cpp),
                Preset::SwiftPackage => Some(Profile::Swift),
                Preset::KotlinApp => Some(Profile::Kotlin),
                Preset::DockerService | Preset::DockerJob => Some(Profile::Docker),
                Preset::Oss | Preset::Internal | Preset::Strict | Preset::Vibe => None,
            })
            .collect()
    }

    fn preset_disabled_rules(&self) -> HashSet<&'static str> {
        let mut disabled = HashSet::new();

        for preset in &self.presets {
            match preset {
                Preset::RustLib => {
                    disabled.insert("rust_cargo_lock");
                }
                Preset::NodeLib => {
                    disabled.insert("node_lockfile");
                }
                Preset::PythonLib => {
                    disabled.insert("python_lockfile");
                    disabled.insert("python_pytest_config");
                }
                Preset::JvmLib => {
                    disabled.insert("jvm_gradle_test");
                }
                Preset::DotnetLib => {
                    disabled.insert("dotnet_global_json");
                }
                Preset::PhpPackage => {
                    disabled.insert("php_composer_lock");
                }
                Preset::RubyApp => {
                    disabled.insert("ruby_gemspec");
                }
                Preset::RubyGem => {
                    disabled.insert("ruby_gemfile_lock");
                }
                Preset::CppLib => {
                    disabled.insert("cpp_dependency_manifest");
                }
                Preset::DockerJob => {
                    disabled.insert("docker_healthcheck");
                    disabled.insert("docker_compose");
                }
                Preset::Internal => {
                    disabled.insert("code_of_conduct");
                    disabled.insert("issue_templates");
                    disabled.insert("pull_request_template");
                    disabled.insert("changelog");
                }
                Preset::RustCli
                | Preset::NodeApp
                | Preset::PythonApp
                | Preset::GoModule
                | Preset::JvmApp
                | Preset::DotnetApp
                | Preset::PhpApp
                | Preset::CppApp
                | Preset::SwiftPackage
                | Preset::KotlinApp
                | Preset::DockerService
                | Preset::Oss
                | Preset::Strict
                | Preset::Vibe => {}
            }
        }

        disabled
    }

    fn disabled_rules_without_reason(&self) -> Vec<&str> {
        self.rules
            .iter()
            .filter(|rule| rule.disabled)
            .filter(|rule| {
                rule.reason
                    .as_deref()
                    .is_none_or(|reason| reason.trim().is_empty())
            })
            .map(|rule| rule.id.as_str())
            .collect()
    }

    fn severity_overrides(&self) -> BTreeMap<&str, Severity> {
        self.rules
            .iter()
            .filter_map(|rule| {
                let severity = match rule.severity? {
                    SeverityOverride::Info => Severity::Info,
                    SeverityOverride::Warning => Severity::Warning,
                };
                Some((rule.id.as_str(), severity))
            })
            .collect()
    }

    fn is_excluded(&self, check: &Check) -> bool {
        let Some(location) = check.location_path() else {
            return false;
        };

        self.exclude_paths
            .iter()
            .any(|pattern| simple_path_match(pattern, location))
    }
}

pub(crate) fn load(path: &Path, explicit_path: Option<&Path>) -> Result<Config> {
    let config_path = match explicit_path {
        Some(path) => path.to_path_buf(),
        None => path.join("repo-doctor.toml"),
    };

    if explicit_path.is_none() && !config_path.exists() {
        return Ok(Config::default());
    }

    let contents = std::fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read config: {}", config_path.display()))?;
    toml::from_str(&contents)
        .with_context(|| format!("failed to parse config: {}", config_path.display()))
}

pub(crate) fn validate(path: &Path) -> Result<Vec<String>> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config: {}", path.display()))?;
    let config = toml::from_str::<Config>(&contents)
        .with_context(|| format!("failed to parse config: {}", path.display()))?;
    let known_rules = report::known_rules()
        .iter()
        .map(|rule| rule.id)
        .collect::<HashSet<_>>();
    let mut findings = Vec::new();

    for rule in &config.rules {
        if !known_rules.contains(rule.id.as_str()) {
            findings.push(format!("unknown rule id: {}", rule.id));
        }
        if rule.disabled
            && rule
                .reason
                .as_deref()
                .is_none_or(|reason| reason.trim().is_empty())
        {
            findings.push(format!("disabled rule lacks rationale: {}", rule.id));
        }
    }

    Ok(findings)
}

fn dedupe_profiles(profiles: Vec<Profile>) -> Vec<Profile> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for profile in profiles {
        if seen.insert(profile.name()) {
            deduped.push(profile);
        }
    }

    deduped
}

fn simple_path_match(pattern: &str, path: &str) -> bool {
    let pattern = pattern.trim();
    if pattern == path {
        return true;
    }

    if let Some(prefix) = pattern.strip_suffix("/*") {
        return path == prefix || path.starts_with(&format!("{prefix}/"));
    }

    if let Some(suffix) = pattern.strip_prefix("*.") {
        return path.ends_with(&format!(".{suffix}"));
    }

    false
}
