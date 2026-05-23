use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::profiles::{self, Profile};
use crate::report::{Check, Severity, warn};

pub(crate) const STARTER_CONFIG: &str = r#"# repo-doctor configuration
# profiles is used only when the CLI profile is left as auto.
# profiles = ["auto"]

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
    rules: Vec<RuleConfig>,
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

        let Some(configured) = &self.profiles else {
            return profiles::resolve(path, cli_profile);
        };

        let mut selected = Vec::new();
        for profile in configured {
            match profile {
                Profile::Auto => selected.extend(profiles::resolve(path, Profile::Auto)),
                Profile::Generic => {}
                explicit => selected.push(*explicit),
            }
        }

        dedupe_profiles(selected)
    }

    pub(crate) fn apply(&self, checks: Vec<Check>) -> Vec<Check> {
        let disabled = self.disabled_rules_with_reason();
        let invalid_disabled = self.disabled_rules_without_reason();
        let severity_overrides = self.severity_overrides();

        let mut configured = checks
            .into_iter()
            .filter(|check| !disabled.contains(check.id()))
            .map(|mut check| {
                if let Some(severity) = severity_overrides.get(check.id()) {
                    check.set_severity(*severity);
                }
                check
            })
            .collect::<Vec<_>>();

        configured.extend(invalid_disabled.into_iter().map(|id| {
            warn(
                "config_disabled_rule_reason",
                format!("Rule `{id}` is disabled without a rationale"),
                "Add a non-empty reason to the disabled rule or remove the disabled flag.",
            )
        }));

        configured
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
