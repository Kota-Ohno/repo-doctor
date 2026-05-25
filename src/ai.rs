use std::path::Path;

use anyhow::Result;
use serde_json::json;

use crate::config;
use crate::profiles::Profile;
use crate::report::{self, RunOutput};
use crate::{AiDocFormat, validate_repository_path};

pub(crate) fn spec(format: AiDocFormat) -> Result<RunOutput> {
    let profiles = Profile::catalog()
        .iter()
        .map(|(name, detection)| json!({ "name": name, "auto_detection": detection }))
        .collect::<Vec<_>>();
    let rules = report::known_rules()
        .iter()
        .map(|rule| {
            json!({
                "id": rule.id,
                "severity": rule.severity,
                "category": rule.category,
                "description": rule.description,
            })
        })
        .collect::<Vec<_>>();
    let recipes = recipe_catalog_json();
    let spec = json!({
        "schema_version": 1,
        "tool": "repo-doctor",
        "positioning": "local-first repository readiness and AI/VibeCoding guardrail checker",
        "default_mode": "repo-doctor check uses core checks plus auto-detected ecosystem profiles",
        "commands": [
            {"name": "check", "purpose": "repository readiness checks", "machine_outputs": ["json", "sarif", "junit", "github", "compact"]},
            {"name": "guard", "purpose": "AI/VibeCoding guardrails over readiness checks plus Git diffs", "machine_outputs": ["json", "sarif", "junit", "github", "compact"]},
            {"name": "baseline", "purpose": "capture existing warnings for incremental adoption"},
            {"name": "ci", "purpose": "generate GitHub Actions workflows"},
            {"name": "github", "purpose": "remote GitHub posture checks through gh CLI"},
            {"name": "suggest", "purpose": "human and agent friendly next actions"},
            {"name": "spec", "purpose": "machine-readable product specification"},
            {"name": "recipes", "purpose": "task recipes for agents"},
            {"name": "agent-guide", "purpose": "AGENTS.md-ready guidance for detected profiles"}
        ],
        "profiles": profiles,
        "rules": rules,
        "recipes": recipes,
        "recommended_ai_loop": [
            "repo-doctor spec --format json",
            "repo-doctor recipes --format markdown",
            "repo-doctor agent-guide --format markdown >> AGENTS.md",
            "repo-doctor guard --fail-on warn"
        ],
        "stable_contracts": {
            "json_schema_version": 1,
            "stable_fields": ["schema_version", "path", "selected_profiles", "summary", "checks"],
            "stable_check_fields": ["id", "status", "severity", "message", "remediation"]
        }
    });

    Ok(RunOutput {
        text: format_ai_doc(format, &spec, spec_markdown(), spec_text())?,
        exit_code: 0,
    })
}

pub(crate) fn recipes(format: AiDocFormat) -> Result<RunOutput> {
    let recipes = json!({
        "schema_version": 1,
        "recipes": recipe_catalog_json(),
    });

    Ok(RunOutput {
        text: format_ai_doc(format, &recipes, recipes_markdown(), recipes_text())?,
        exit_code: 0,
    })
}

pub(crate) fn agent_guide(path: &Path, format: AiDocFormat, profile: Profile) -> Result<RunOutput> {
    validate_repository_path(path)?;
    let config = config::load(path, None)?;
    let selected_profiles = config.selected_profiles(path, profile);
    let profiles = selected_profiles
        .iter()
        .map(|profile| profile.name())
        .collect::<Vec<_>>();
    let commands = selected_profiles
        .iter()
        .flat_map(|profile| profile_verification_commands(*profile).iter().copied())
        .collect::<Vec<_>>();
    let unique_commands = dedupe_strings(commands);
    let value = json!({
        "schema_version": 1,
        "path": path.display().to_string(),
        "selected_profiles": profiles,
        "purpose": "AGENTS.md guidance for AI coding agents",
        "required_behavior": [
            "Keep changes scoped to the requested task",
            "Do not remove guardrails without replacing them",
            "Update tests with source changes or document why existing coverage is sufficient",
            "Run repo-doctor guard --fail-on warn before finishing"
        ],
        "verification_commands": unique_commands,
        "completion_gate": "repo-doctor guard --fail-on warn"
    });

    Ok(RunOutput {
        text: format_ai_doc(
            format,
            &value,
            agent_guide_markdown(&selected_profiles, &unique_commands),
            agent_guide_text(&selected_profiles, &unique_commands),
        )?,
        exit_code: 0,
    })
}

fn format_ai_doc(
    format: AiDocFormat,
    json_value: &serde_json::Value,
    markdown: String,
    text: String,
) -> Result<String> {
    Ok(match format {
        AiDocFormat::Json => serde_json::to_string_pretty(json_value)?,
        AiDocFormat::Markdown => markdown,
        AiDocFormat::Text => text,
    })
}

fn recipe_catalog_json() -> Vec<serde_json::Value> {
    recipe_catalog()
        .iter()
        .map(|recipe| {
            json!({
                "id": recipe.id,
                "goal": recipe.goal,
                "when_to_use": recipe.when_to_use,
                "commands": recipe.commands,
                "success": recipe.success,
            })
        })
        .collect()
}

struct Recipe {
    id: &'static str,
    goal: &'static str,
    when_to_use: &'static str,
    commands: &'static [&'static str],
    success: &'static str,
}

fn recipe_catalog() -> &'static [Recipe] {
    &[
        Recipe {
            id: "inspect-readiness",
            goal: "Understand repository readiness without changing files",
            when_to_use: "Start of an AI coding session or repository triage",
            commands: &[
                "repo-doctor check --format summary",
                "repo-doctor check --format json",
                "repo-doctor suggest",
            ],
            success: "Warnings are understood and mapped to follow-up changes.",
        },
        Recipe {
            id: "vibecoding-guard",
            goal: "Block risky AI-generated changes before merge",
            when_to_use: "Before finishing any AI coding task",
            commands: &[
                "repo-doctor guard --fail-on warn",
                "repo-doctor guard --format sarif --output repo-doctor-guard.sarif",
            ],
            success: "guard exits zero, or every warning has an explicit fix or rationale.",
        },
        Recipe {
            id: "ci-adoption",
            goal: "Install repo-doctor in GitHub Actions",
            when_to_use: "Repository does not already run repo-doctor in CI",
            commands: &[
                "mkdir -p .github/workflows",
                "repo-doctor ci --template generic > .github/workflows/repo-doctor.yml",
                "repo-doctor ci --guard > .github/workflows/repo-doctor-guard.yml",
            ],
            success: "CI runs readiness and guardrail checks on pull_request and push.",
        },
        Recipe {
            id: "incremental-adoption",
            goal: "Adopt without blocking on existing repository debt",
            when_to_use: "Existing repository has many current warnings",
            commands: &[
                "repo-doctor baseline > repo-doctor-baseline.json",
                "repo-doctor check --baseline repo-doctor-baseline.json --fail-on warn",
            ],
            success: "Existing warnings are baselined and new warnings fail the gate.",
        },
        Recipe {
            id: "agent-instructions",
            goal: "Create AI-readable repository instructions",
            when_to_use: "AGENTS.md is missing or too generic",
            commands: &[
                "repo-doctor agent-guide --format markdown",
                "repo-doctor guard --warnings-only",
            ],
            success: "AGENTS.md includes profile-specific verification commands and boundaries.",
        },
        Recipe {
            id: "remote-github-posture",
            goal: "Check remote GitHub repository settings",
            when_to_use: "Repository is public or near release",
            commands: &[
                "repo-doctor github-auth-doctor",
                "repo-doctor github owner/repo --format summary",
                "repo-doctor scorecard owner/repo",
            ],
            success: "Remote metadata, security settings, and branch protection are visible.",
        },
    ]
}

fn spec_markdown() -> String {
    [
        "# repo-doctor AI specification",
        "",
        "`repo-doctor` is a local-first repository readiness and AI/VibeCoding guardrail checker.",
        "",
        "Use `repo-doctor check` for repository readiness and `repo-doctor guard` before finishing AI-generated changes.",
        "",
        "Machine-readable outputs: JSON, SARIF, JUnit, GitHub annotations, and compact summaries.",
        "",
        "Recommended AI loop:",
        "",
        "```bash",
        "repo-doctor spec --format json",
        "repo-doctor recipes --format markdown",
        "repo-doctor agent-guide --format markdown",
        "repo-doctor guard --fail-on warn",
        "```",
        "",
        "日本語: AIはまず `spec` で能力を把握し、`recipes` で作業手順を選び、`agent-guide` を `AGENTS.md` に反映し、最後に `guard --fail-on warn` で差分を止めます。",
    ]
    .join("\n")
}

fn spec_text() -> String {
    "repo-doctor: local-first repository readiness and AI/VibeCoding guardrail checker\nUse check for readiness, guard for AI-generated diffs, spec/recipes/agent-guide for AI-readable operation.".to_owned()
}

fn recipes_markdown() -> String {
    let mut lines = vec!["# repo-doctor AI recipes".to_owned(), String::new()];
    for recipe in recipe_catalog() {
        lines.push(format!("## {}", recipe.id));
        lines.push(format!("- Goal: {}", recipe.goal));
        lines.push(format!("- When: {}", recipe.when_to_use));
        lines.push("- Commands:".to_owned());
        lines.push("```bash".to_owned());
        lines.extend(recipe.commands.iter().map(|command| (*command).to_owned()));
        lines.push("```".to_owned());
        lines.push(format!("- Success: {}", recipe.success));
        lines.push(String::new());
    }
    lines.push(
        "日本語: AIは目的に近いrecipeを選び、commandsを順番に実行し、success条件で完了判定します。"
            .to_owned(),
    );
    lines.join("\n")
}

fn recipes_text() -> String {
    recipe_catalog()
        .iter()
        .map(|recipe| format!("{}: {}", recipe.id, recipe.goal))
        .collect::<Vec<_>>()
        .join("\n")
}

fn agent_guide_markdown(profiles: &[Profile], commands: &[&'static str]) -> String {
    let profile_names = format_profile_names(profiles);
    let mut lines = vec![
        "# AI Agent Instructions",
        "",
        "Use this section in `AGENTS.md` for repository-specific AI coding guidance.",
        "",
        "## Scope",
        "",
        "- Keep changes scoped to the requested task.",
        "- Do not remove CI, tests, security files, dependency automation, or repo-doctor config without an explicit replacement.",
        "- Update tests with source changes, or document why existing coverage is sufficient.",
        "",
        "## Detected Profiles",
        "",
        &profile_names,
        "",
        "## Required Verification",
        "",
        "```bash",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    lines.extend(commands.iter().map(|command| (*command).to_owned()));
    lines.push("repo-doctor guard --fail-on warn".to_owned());
    lines.push("```".to_owned());
    lines.push(String::new());
    lines.push("日本語: AI agentは変更範囲を小さく保ち、source変更にはtest更新を添え、最後に `repo-doctor guard --fail-on warn` を通します。".to_owned());
    lines.join("\n")
}

fn agent_guide_text(profiles: &[Profile], commands: &[&'static str]) -> String {
    format!(
        "profiles={}\ncommands={}\ncompletion=repo-doctor guard --fail-on warn",
        format_profile_names(profiles),
        commands.join(", ")
    )
}

fn format_profile_names(profiles: &[Profile]) -> String {
    if profiles.is_empty() {
        "generic".to_owned()
    } else {
        profiles
            .iter()
            .map(|profile| profile.name())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn profile_verification_commands(profile: Profile) -> &'static [&'static str] {
    match profile {
        Profile::Rust => &[
            "cargo fmt --all --check",
            "cargo clippy --all-targets --all-features -- -D warnings",
            "cargo test",
        ],
        Profile::Node | Profile::Frontend => {
            &["npm test --if-present", "npm run build --if-present"]
        }
        Profile::Python => &["python -m pytest"],
        Profile::Go => &["go test ./..."],
        Profile::Docker => &["docker build -t local/repo-doctor-check ."],
        Profile::Jvm | Profile::Kotlin => &["./gradlew test || mvn test"],
        Profile::Deno => &["deno test"],
        Profile::Bun => &["bun test"],
        Profile::Dotnet => &["dotnet test"],
        Profile::Php => &["composer test"],
        Profile::Ruby => &["bundle exec rake test || bundle exec rspec"],
        Profile::Cpp => &["cmake --build build", "ctest --test-dir build"],
        Profile::Swift => &["swift test"],
        Profile::Iac => &["terraform fmt -check", "terraform validate"],
        Profile::Docs => &["mkdocs build || mdbook build"],
        Profile::Auto | Profile::Generic => &[],
    }
}

fn dedupe_strings(values: Vec<&'static str>) -> Vec<&'static str> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    deduped
}
