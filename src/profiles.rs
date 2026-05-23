use std::path::Path;

use clap::ValueEnum;
use serde_json::Value as JsonValue;

use crate::report::{Check, pass, warn};

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Profile {
    Auto,
    Generic,
    Rust,
    Node,
    Python,
    Go,
}

pub(crate) fn resolve(path: &Path, profile: Profile) -> Vec<Profile> {
    match profile {
        Profile::Auto => detect(path),
        Profile::Generic => Vec::new(),
        selected => vec![selected],
    }
}

fn detect(path: &Path) -> Vec<Profile> {
    let mut profiles = Vec::new();

    if path.join("Cargo.toml").exists() {
        profiles.push(Profile::Rust);
    }
    if path.join("package.json").exists() {
        profiles.push(Profile::Node);
    }
    if path.join("pyproject.toml").exists()
        || path.join("setup.py").exists()
        || path.join("requirements.txt").exists()
    {
        profiles.push(Profile::Python);
    }
    if path.join("go.mod").exists() {
        profiles.push(Profile::Go);
    }

    profiles
}

pub(crate) fn inspect(path: &Path, profiles: &[Profile]) -> Vec<Check> {
    let mut checks = Vec::new();

    for profile in profiles {
        match profile {
            Profile::Rust => checks.extend(inspect_rust(path)),
            Profile::Node => checks.extend(inspect_node(path)),
            Profile::Python => checks.extend(inspect_python(path)),
            Profile::Go => checks.extend(inspect_go(path)),
            Profile::Auto | Profile::Generic => {}
        }
    }

    checks
}

fn inspect_rust(path: &Path) -> Vec<Check> {
    let cargo_toml_path = path.join("Cargo.toml");
    let manifest = match std::fs::read_to_string(&cargo_toml_path) {
        Ok(contents) => contents,
        Err(_) => {
            return vec![warn(
                "rust_cargo_toml",
                "Cargo.toml is missing",
                "Add Cargo.toml or use --profile generic for language-independent checks only.",
            )];
        }
    };

    let parsed = match toml::from_str::<toml::Value>(&manifest) {
        Ok(value) => value,
        Err(error) => {
            return vec![warn(
                "rust_cargo_toml_parse",
                format!("Cargo.toml could not be parsed: {error}"),
                "Fix Cargo.toml so it is valid TOML.",
            )];
        }
    };

    let package = parsed.get("package").and_then(toml::Value::as_table);
    let Some(package) = package else {
        return vec![warn(
            "rust_cargo_package",
            "Cargo.toml is missing [package]",
            "Add a [package] section or run repo-doctor from a package crate.",
        )];
    };

    let mut checks = vec![
        check_toml_string(package, "rust_cargo_name", "name", "Cargo package"),
        check_toml_string(package, "rust_cargo_version", "version", "Cargo package"),
        check_toml_string(package, "rust_cargo_edition", "edition", "Cargo package"),
        check_toml_string(
            package,
            "rust_cargo_rust_version",
            "rust-version",
            "Cargo package",
        ),
        check_toml_string(
            package,
            "rust_cargo_description",
            "description",
            "Cargo package",
        ),
        check_toml_string(
            package,
            "rust_cargo_repository",
            "repository",
            "Cargo package",
        ),
        check_toml_string(package, "rust_cargo_readme", "readme", "Cargo package"),
        check_rust_license(package),
        check_toml_path(path, package, "rust_cargo_readme_path", "readme"),
        check_toml_path(
            path,
            package,
            "rust_cargo_license_file_path",
            "license-file",
        ),
        check_rust_workspace(&parsed),
        check_cargo_lock(path),
        check_gitignore_target(path),
    ];

    checks.retain(|check| !check.id.is_empty());
    checks
}

fn check_toml_string(
    table: &toml::map::Map<String, toml::Value>,
    id: &'static str,
    key: &'static str,
    label: &str,
) -> Check {
    if table
        .get(key)
        .and_then(toml::Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
    {
        pass(id, format!("{label} `{key}` is set"))
    } else {
        warn(
            id,
            format!("{label} `{key}` is missing"),
            format!("Set `{key}` in the manifest metadata."),
        )
    }
}

fn check_rust_license(package: &toml::map::Map<String, toml::Value>) -> Check {
    let has_license = ["license", "license-file"].iter().any(|key| {
        package
            .get(*key)
            .and_then(toml::Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
    });

    if has_license {
        pass(
            "rust_cargo_license",
            "Cargo package license metadata is set",
        )
    } else {
        warn(
            "rust_cargo_license",
            "Cargo package license metadata is missing",
            "Set `package.license` or `package.license-file` in Cargo.toml.",
        )
    }
}

fn check_toml_path(
    path: &Path,
    table: &toml::map::Map<String, toml::Value>,
    id: &'static str,
    key: &'static str,
) -> Check {
    let Some(relative) = table.get(key).and_then(toml::Value::as_str) else {
        return empty_check();
    };

    if path.join(relative).exists() {
        pass(id, format!("Manifest `{key}` path exists"))
    } else {
        warn(
            id,
            format!("Manifest `{key}` path does not exist: {relative}"),
            format!("Create {relative} or update `{key}` in the manifest."),
        )
    }
}

fn check_rust_workspace(manifest: &toml::Value) -> Check {
    let workspace = manifest.get("workspace").and_then(toml::Value::as_table);
    let Some(workspace) = workspace else {
        return pass("rust_workspace", "Cargo manifest is a package crate");
    };

    if workspace
        .get("members")
        .and_then(toml::Value::as_array)
        .is_some_and(|members| !members.is_empty())
    {
        pass("rust_workspace", "Cargo workspace members are configured")
    } else {
        warn(
            "rust_workspace",
            "Cargo workspace has no members",
            "Set `workspace.members` in Cargo.toml or remove the empty workspace section.",
        )
    }
}

fn check_cargo_lock(path: &Path) -> Check {
    if path.join("Cargo.lock").exists() {
        pass("rust_cargo_lock", "Cargo.lock is present")
    } else {
        warn(
            "rust_cargo_lock",
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
                "rust_gitignore_target",
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
        pass(
            "rust_gitignore_target",
            ".gitignore excludes Rust build output",
        )
    } else {
        warn(
            "rust_gitignore_target",
            ".gitignore does not exclude Rust build output",
            "Add /target to .gitignore.",
        )
    }
}

fn inspect_node(path: &Path) -> Vec<Check> {
    let package_path = path.join("package.json");
    let package = match std::fs::read_to_string(&package_path) {
        Ok(contents) => contents,
        Err(_) => {
            return vec![warn(
                "node_package_json",
                "package.json is missing",
                "Add package.json or use --profile generic for language-independent checks only.",
            )];
        }
    };

    let parsed = match serde_json::from_str::<JsonValue>(&package) {
        Ok(value) => value,
        Err(error) => {
            return vec![warn(
                "node_package_json_parse",
                format!("package.json could not be parsed: {error}"),
                "Fix package.json so it is valid JSON.",
            )];
        }
    };

    vec![
        check_json_string(&parsed, "node_name", "name", "package.json"),
        check_json_string(&parsed, "node_version", "version", "package.json"),
        check_json_string(&parsed, "node_description", "description", "package.json"),
        check_json_string(&parsed, "node_license", "license", "package.json"),
        check_json_present(&parsed, "node_repository", "repository", "package.json"),
        check_node_test_script(&parsed),
        check_node_engines(&parsed),
        check_node_lockfile(path),
    ]
}

fn check_json_string(value: &JsonValue, id: &'static str, key: &'static str, label: &str) -> Check {
    if value
        .get(key)
        .and_then(JsonValue::as_str)
        .is_some_and(|text| !text.trim().is_empty())
    {
        pass(id, format!("{label} `{key}` is set"))
    } else {
        warn(
            id,
            format!("{label} `{key}` is missing"),
            format!("Set `{key}` in {label}."),
        )
    }
}

fn check_json_present(
    value: &JsonValue,
    id: &'static str,
    key: &'static str,
    label: &str,
) -> Check {
    if value.get(key).is_some() {
        pass(id, format!("{label} `{key}` is set"))
    } else {
        warn(
            id,
            format!("{label} `{key}` is missing"),
            format!("Set `{key}` in {label}."),
        )
    }
}

fn check_node_test_script(package: &JsonValue) -> Check {
    if package
        .get("scripts")
        .and_then(|scripts| scripts.get("test"))
        .and_then(JsonValue::as_str)
        .is_some_and(|script| !script.trim().is_empty())
    {
        pass("node_test_script", "package.json scripts.test is set")
    } else {
        warn(
            "node_test_script",
            "package.json scripts.test is missing",
            "Add a `test` script to package.json.",
        )
    }
}

fn check_node_engines(package: &JsonValue) -> Check {
    if package
        .get("engines")
        .and_then(|engines| engines.get("node"))
        .and_then(JsonValue::as_str)
        .is_some_and(|engine| !engine.trim().is_empty())
    {
        pass("node_engines", "package.json engines.node is set")
    } else {
        warn(
            "node_engines",
            "package.json engines.node is missing",
            "Set `engines.node` in package.json.",
        )
    }
}

fn check_node_lockfile(path: &Path) -> Check {
    let candidates = [
        "package-lock.json",
        "npm-shrinkwrap.json",
        "yarn.lock",
        "pnpm-lock.yaml",
        "bun.lockb",
    ];

    if candidates
        .iter()
        .any(|candidate| path.join(candidate).exists())
    {
        pass("node_lockfile", "Node package manager lockfile is present")
    } else {
        warn(
            "node_lockfile",
            "Node package manager lockfile is missing",
            format!("Add one of {}.", candidates.join(", ")),
        )
    }
}

fn inspect_python(path: &Path) -> Vec<Check> {
    if path.join("pyproject.toml").exists() {
        inspect_pyproject(path)
    } else {
        vec![warn(
            "python_pyproject",
            "pyproject.toml is missing",
            "Add pyproject.toml for Python project metadata.",
        )]
    }
}

fn inspect_pyproject(path: &Path) -> Vec<Check> {
    let pyproject = match std::fs::read_to_string(path.join("pyproject.toml")) {
        Ok(contents) => contents,
        Err(_) => {
            return vec![warn(
                "python_pyproject",
                "pyproject.toml could not be read",
                "Add a readable pyproject.toml.",
            )];
        }
    };

    let parsed = match toml::from_str::<toml::Value>(&pyproject) {
        Ok(value) => value,
        Err(error) => {
            return vec![warn(
                "python_pyproject_parse",
                format!("pyproject.toml could not be parsed: {error}"),
                "Fix pyproject.toml so it is valid TOML.",
            )];
        }
    };

    let project = parsed.get("project").and_then(toml::Value::as_table);
    let mut checks = Vec::new();

    if let Some(project) = project {
        checks.extend([
            check_toml_string(project, "python_name", "name", "Python project"),
            check_toml_string(project, "python_version", "version", "Python project"),
            check_toml_string(
                project,
                "python_description",
                "description",
                "Python project",
            ),
            check_toml_string(project, "python_readme", "readme", "Python project"),
            check_python_license(project),
            check_toml_path(path, project, "python_readme_path", "readme"),
        ]);
    } else {
        checks.push(warn(
            "python_project_metadata",
            "pyproject.toml is missing [project]",
            "Add PEP 621 [project] metadata to pyproject.toml.",
        ));
    }

    checks.push(check_python_build_system(&parsed));
    checks.push(check_python_lockfile(path));
    checks.retain(|check| !check.id.is_empty());
    checks
}

fn check_python_license(project: &toml::map::Map<String, toml::Value>) -> Check {
    if project.get("license").is_some() || project.get("license-files").is_some() {
        pass("python_license", "Python project license metadata is set")
    } else {
        warn(
            "python_license",
            "Python project license metadata is missing",
            "Set `project.license` or `project.license-files` in pyproject.toml.",
        )
    }
}

fn check_python_build_system(pyproject: &toml::Value) -> Check {
    if pyproject.get("build-system").is_some() {
        pass("python_build_system", "Python build-system is configured")
    } else {
        warn(
            "python_build_system",
            "Python build-system is missing",
            "Add a [build-system] section to pyproject.toml.",
        )
    }
}

fn check_python_lockfile(path: &Path) -> Check {
    let candidates = ["uv.lock", "poetry.lock", "pdm.lock", "requirements.txt"];

    if candidates
        .iter()
        .any(|candidate| path.join(candidate).exists())
    {
        pass(
            "python_lockfile",
            "Python dependency lock or requirements file is present",
        )
    } else {
        warn(
            "python_lockfile",
            "Python dependency lock or requirements file is missing",
            format!("Add one of {}.", candidates.join(", ")),
        )
    }
}

fn inspect_go(path: &Path) -> Vec<Check> {
    let go_mod = match std::fs::read_to_string(path.join("go.mod")) {
        Ok(contents) => contents,
        Err(_) => {
            return vec![warn(
                "go_mod",
                "go.mod is missing",
                "Add go.mod or use --profile generic for language-independent checks only.",
            )];
        }
    };

    vec![
        check_go_module(&go_mod),
        check_go_version(&go_mod),
        check_go_sum(path),
    ]
}

fn check_go_module(go_mod: &str) -> Check {
    if go_mod
        .lines()
        .map(str::trim)
        .any(|line| line.starts_with("module "))
    {
        pass("go_module", "go.mod module declaration is present")
    } else {
        warn(
            "go_module",
            "go.mod module declaration is missing",
            "Add a module declaration to go.mod.",
        )
    }
}

fn check_go_version(go_mod: &str) -> Check {
    if go_mod
        .lines()
        .map(str::trim)
        .any(|line| line.starts_with("go "))
    {
        pass("go_version", "go.mod go version is present")
    } else {
        warn(
            "go_version",
            "go.mod go version is missing",
            "Add a go version directive to go.mod.",
        )
    }
}

fn check_go_sum(path: &Path) -> Check {
    if path.join("go.sum").exists() {
        pass("go_sum", "go.sum is present")
    } else {
        warn(
            "go_sum",
            "go.sum is missing",
            "Run `go mod tidy` and commit go.sum when dependencies are present.",
        )
    }
}

fn empty_check() -> Check {
    Check {
        id: "",
        status: crate::report::CheckStatus::Pass,
        severity: crate::report::Severity::Info,
        message: String::new(),
        remediation: String::new(),
    }
}
