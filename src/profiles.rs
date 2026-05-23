use std::path::Path;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::report::{Check, pass, warn};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    Auto,
    Generic,
    Rust,
    Node,
    Python,
    Go,
    Docker,
    Jvm,
    Deno,
    Bun,
    Dotnet,
    Php,
    Ruby,
    Cpp,
    Swift,
    Kotlin,
}

impl Profile {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Profile::Auto => "auto",
            Profile::Generic => "generic",
            Profile::Rust => "rust",
            Profile::Node => "node",
            Profile::Python => "python",
            Profile::Go => "go",
            Profile::Docker => "docker",
            Profile::Jvm => "jvm",
            Profile::Deno => "deno",
            Profile::Bun => "bun",
            Profile::Dotnet => "dotnet",
            Profile::Php => "php",
            Profile::Ruby => "ruby",
            Profile::Cpp => "cpp",
            Profile::Swift => "swift",
            Profile::Kotlin => "kotlin",
        }
    }
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
    if has_container_files(path) {
        profiles.push(Profile::Docker);
    }
    if has_jvm_files(path) {
        profiles.push(Profile::Jvm);
    }
    if has_deno_files(path) {
        profiles.push(Profile::Deno);
    }
    if has_bun_files(path) {
        profiles.push(Profile::Bun);
    }
    if has_dotnet_files(path) {
        profiles.push(Profile::Dotnet);
    }
    if has_php_files(path) {
        profiles.push(Profile::Php);
    }
    if has_ruby_files(path) {
        profiles.push(Profile::Ruby);
    }
    if has_cpp_files(path) {
        profiles.push(Profile::Cpp);
    }
    if has_swift_files(path) {
        profiles.push(Profile::Swift);
    }
    if has_kotlin_files(path) {
        profiles.push(Profile::Kotlin);
    }

    profiles
}

fn has_container_files(path: &Path) -> bool {
    [
        "Dockerfile",
        "Containerfile",
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
        ".dockerignore",
    ]
    .iter()
    .any(|candidate| path.join(candidate).exists())
}

fn has_jvm_files(path: &Path) -> bool {
    [
        "pom.xml",
        "build.gradle",
        "build.gradle.kts",
        "settings.gradle",
        "settings.gradle.kts",
    ]
    .iter()
    .any(|candidate| path.join(candidate).exists())
}

fn has_deno_files(path: &Path) -> bool {
    ["deno.json", "deno.jsonc", "deno.lock"]
        .iter()
        .any(|candidate| path.join(candidate).exists())
}

fn has_bun_files(path: &Path) -> bool {
    ["bun.lock", "bun.lockb", "bunfig.toml"]
        .iter()
        .any(|candidate| path.join(candidate).exists())
        || package_manager_starts_with(path, "bun")
}

fn has_dotnet_files(path: &Path) -> bool {
    ["Directory.Build.props", "global.json"]
        .iter()
        .any(|candidate| path.join(candidate).exists())
        || root_has_extension(path, "sln")
        || root_has_extension(path, "csproj")
        || root_has_extension(path, "fsproj")
        || root_has_extension(path, "vbproj")
}

fn has_php_files(path: &Path) -> bool {
    path.join("composer.json").exists() || path.join("composer.lock").exists()
}

fn has_ruby_files(path: &Path) -> bool {
    path.join("Gemfile").exists()
        || path.join("Gemfile.lock").exists()
        || root_has_extension(path, "gemspec")
}

fn has_cpp_files(path: &Path) -> bool {
    ["CMakeLists.txt", "Makefile", "meson.build", "configure.ac"]
        .iter()
        .any(|candidate| path.join(candidate).exists())
        || ["c", "cc", "cpp", "cxx", "h", "hpp"]
            .iter()
            .any(|extension| root_has_extension(path, extension))
}

fn has_swift_files(path: &Path) -> bool {
    path.join("Package.swift").exists() || path.join("Package.resolved").exists()
}

fn has_kotlin_files(path: &Path) -> bool {
    path.join("build.gradle.kts").exists()
        || path.join("settings.gradle.kts").exists()
        || path.join("src/main/kotlin").exists()
        || root_has_extension(path, "kt")
        || root_has_extension(path, "kts")
}

pub(crate) fn inspect(path: &Path, profiles: &[Profile]) -> Vec<Check> {
    let mut checks = Vec::new();

    for profile in profiles {
        match profile {
            Profile::Rust => checks.extend(inspect_rust(path)),
            Profile::Node => checks.extend(inspect_node(path)),
            Profile::Python => checks.extend(inspect_python(path)),
            Profile::Go => checks.extend(inspect_go(path)),
            Profile::Docker => checks.extend(inspect_docker(path)),
            Profile::Jvm => checks.extend(inspect_jvm(path)),
            Profile::Deno => checks.extend(inspect_deno(path)),
            Profile::Bun => checks.extend(inspect_bun(path)),
            Profile::Dotnet => checks.extend(inspect_dotnet(path)),
            Profile::Php => checks.extend(inspect_php(path)),
            Profile::Ruby => checks.extend(inspect_ruby(path)),
            Profile::Cpp => checks.extend(inspect_cpp(path)),
            Profile::Swift => checks.extend(inspect_swift(path)),
            Profile::Kotlin => checks.extend(inspect_kotlin(path)),
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
        check_rust_readme_commands(path, package),
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

fn check_rust_readme_commands(path: &Path, package: &toml::map::Map<String, toml::Value>) -> Check {
    let Some(readme) = read_readme(path) else {
        return warn(
            "rust_readme_commands",
            "README could not be read for Rust command examples",
            "Add a readable README with cargo test and usage examples.",
        );
    };

    let package_name = package
        .get("name")
        .and_then(toml::Value::as_str)
        .unwrap_or_default();
    let mentions_usage = !package_name.is_empty() && readme.contains(package_name);
    let mentions_tests = readme.contains("cargo test") || readme.contains("cargo nextest");

    if mentions_usage && mentions_tests {
        pass(
            "rust_readme_commands",
            "README documents Rust usage and test commands",
        )
    } else {
        warn(
            "rust_readme_commands",
            "README is missing Rust usage or test commands",
            "Document how to run the crate and include cargo test or cargo nextest.",
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
        inspect_legacy_python(path)
    }
}

fn inspect_legacy_python(path: &Path) -> Vec<Check> {
    vec![
        warn(
            "python_pyproject",
            "pyproject.toml is missing",
            "Add pyproject.toml for Python project metadata.",
        ),
        check_legacy_python_setup(path),
        check_legacy_python_requirements(path),
        check_python_tests(path),
    ]
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
    checks.push(check_python_tests(path));
    checks.retain(|check| !check.id.is_empty());
    checks
}

fn check_legacy_python_setup(path: &Path) -> Check {
    if path.join("setup.py").exists() || path.join("setup.cfg").exists() {
        pass("python_legacy_setup", "Legacy Python setup file is present")
    } else {
        warn(
            "python_legacy_setup",
            "Legacy Python setup file is missing",
            "Add setup.py/setup.cfg or migrate project metadata to pyproject.toml.",
        )
    }
}

fn check_legacy_python_requirements(path: &Path) -> Check {
    match std::fs::read_to_string(path.join("requirements.txt")) {
        Ok(contents) if !contents.trim().is_empty() => {
            pass("python_requirements", "requirements.txt is present")
        }
        Ok(_) => warn(
            "python_requirements",
            "requirements.txt is empty",
            "Add dependencies or remove the empty requirements.txt file.",
        ),
        Err(_) => warn(
            "python_requirements",
            "requirements.txt is missing",
            "Add requirements.txt or a lockfile for legacy Python projects.",
        ),
    }
}

fn check_python_tests(path: &Path) -> Check {
    if path.join("tests").exists() || path.join("test").exists() {
        pass("python_tests", "Python test directory is present")
    } else {
        warn(
            "python_tests",
            "Python test directory is missing",
            "Add tests/ or test/ so automated test discovery is clear.",
        )
    }
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

fn inspect_docker(path: &Path) -> Vec<Check> {
    vec![
        check_container_build_file(path),
        check_dockerignore(path),
        check_compose_file(path),
        check_dockerfile_latest_tag(path),
    ]
}

fn check_container_build_file(path: &Path) -> Check {
    if path.join("Dockerfile").exists() || path.join("Containerfile").exists() {
        pass("docker_build_file", "Container build file is present")
    } else {
        warn(
            "docker_build_file",
            "Container build file is missing",
            "Add Dockerfile or Containerfile, or use --profile generic if this is not a containerized project.",
        )
    }
}

fn check_dockerignore(path: &Path) -> Check {
    if path.join(".dockerignore").exists() {
        pass("dockerignore", ".dockerignore is present")
    } else {
        warn(
            "dockerignore",
            ".dockerignore is missing",
            "Add .dockerignore to keep build contexts small and avoid copying unwanted files.",
        )
    }
}

fn check_compose_file(path: &Path) -> Check {
    let candidates = [
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
    ];

    if candidates
        .iter()
        .any(|candidate| path.join(candidate).exists())
    {
        pass("docker_compose", "Compose file is present")
    } else {
        warn(
            "docker_compose",
            "Compose file is missing",
            "Add a compose file if local multi-service development is expected.",
        )
    }
}

fn check_dockerfile_latest_tag(path: &Path) -> Check {
    let build_file = ["Dockerfile", "Containerfile"]
        .iter()
        .map(|candidate| path.join(candidate))
        .find(|candidate| candidate.exists());

    let Some(build_file) = build_file else {
        return empty_check();
    };

    let contents = match std::fs::read_to_string(&build_file) {
        Ok(contents) => contents,
        Err(_) => {
            return warn(
                "docker_base_image_pin",
                "Container build file could not be read",
                "Make the container build file readable.",
            );
        }
    };

    let uses_latest = contents
        .lines()
        .map(str::trim)
        .any(|line| line.to_ascii_lowercase().starts_with("from ") && line.contains(":latest"));

    if uses_latest {
        warn(
            "docker_base_image_pin",
            "Container base image uses :latest",
            "Pin base image tags to a specific version instead of :latest.",
        )
    } else {
        pass(
            "docker_base_image_pin",
            "Container base image tags avoid :latest",
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
        documentation_url: None,
        location: None,
    }
}

fn inspect_deno(path: &Path) -> Vec<Check> {
    let config = ["deno.json", "deno.jsonc"]
        .iter()
        .map(|candidate| path.join(candidate))
        .find(|candidate| candidate.exists());

    let mut checks = vec![check_deno_config(config.as_deref()), check_deno_lock(path)];

    if let Some(config) = config {
        checks.push(check_deno_tasks(&config));
    }

    checks
}

fn check_deno_config(config: Option<&Path>) -> Check {
    if config.is_some() {
        pass("deno_config", "Deno config is present")
    } else {
        warn(
            "deno_config",
            "Deno config is missing",
            "Add deno.json or deno.jsonc.",
        )
    }
}

fn check_deno_lock(path: &Path) -> Check {
    if path.join("deno.lock").exists() {
        pass("deno_lock", "deno.lock is present")
    } else {
        warn(
            "deno_lock",
            "deno.lock is missing",
            "Run Deno with lockfile generation enabled and commit deno.lock.",
        )
    }
}

fn check_deno_tasks(config: &Path) -> Check {
    let contents = match std::fs::read_to_string(config) {
        Ok(contents) => contents,
        Err(_) => {
            return warn(
                "deno_tasks",
                "Deno config could not be read",
                "Make deno.json or deno.jsonc readable.",
            );
        }
    };

    if contents.contains("\"tasks\"") {
        pass("deno_tasks", "Deno tasks are configured")
    } else {
        warn(
            "deno_tasks",
            "Deno tasks are missing",
            "Add a tasks section to deno.json or deno.jsonc.",
        )
    }
}

fn inspect_bun(path: &Path) -> Vec<Check> {
    let package = read_json(path.join("package.json"));

    let mut checks = vec![check_bun_lockfile(path), check_bun_package_manager(path)];
    if let Some(package) = package {
        checks.extend([
            check_json_string(&package, "bun_package_name", "name", "package.json"),
            check_node_test_script_with_id(&package, "bun_test_script"),
        ]);
    } else {
        checks.push(warn(
            "bun_package_json",
            "package.json is missing",
            "Add package.json for Bun package metadata.",
        ));
    }

    checks
}

fn check_bun_lockfile(path: &Path) -> Check {
    if path.join("bun.lock").exists() || path.join("bun.lockb").exists() {
        pass("bun_lockfile", "Bun lockfile is present")
    } else {
        warn(
            "bun_lockfile",
            "Bun lockfile is missing",
            "Run bun install and commit bun.lock or bun.lockb.",
        )
    }
}

fn check_bun_package_manager(path: &Path) -> Check {
    if package_manager_starts_with(path, "bun") {
        pass("bun_package_manager", "packageManager is set to Bun")
    } else {
        warn(
            "bun_package_manager",
            "packageManager is not set to Bun",
            "Set packageManager to a Bun version in package.json.",
        )
    }
}

fn check_node_test_script_with_id(package: &JsonValue, id: &'static str) -> Check {
    if package
        .get("scripts")
        .and_then(|scripts| scripts.get("test"))
        .and_then(JsonValue::as_str)
        .is_some_and(|script| !script.trim().is_empty())
    {
        pass(id, "package.json scripts.test is set")
    } else {
        warn(
            id,
            "package.json scripts.test is missing",
            "Add a `test` script to package.json.",
        )
    }
}

fn inspect_dotnet(path: &Path) -> Vec<Check> {
    vec![
        check_dotnet_project_or_solution(path),
        check_dotnet_global_json(path),
        check_dotnet_test_project(path),
    ]
}

fn check_dotnet_project_or_solution(path: &Path) -> Check {
    if root_has_extension(path, "sln")
        || root_has_extension(path, "csproj")
        || root_has_extension(path, "fsproj")
        || root_has_extension(path, "vbproj")
    {
        pass("dotnet_project", ".NET solution or project file is present")
    } else {
        warn(
            "dotnet_project",
            ".NET solution or project file is missing",
            "Add a .sln, .csproj, .fsproj, or .vbproj file.",
        )
    }
}

fn check_dotnet_global_json(path: &Path) -> Check {
    if path.join("global.json").exists() {
        pass(
            "dotnet_global_json",
            ".NET SDK version is pinned with global.json",
        )
    } else {
        warn(
            "dotnet_global_json",
            ".NET SDK version is not pinned",
            "Add global.json to pin the .NET SDK version.",
        )
    }
}

fn check_dotnet_test_project(path: &Path) -> Check {
    let has_test_project = root_file_names(path).iter().any(|name| {
        name.ends_with("Tests.csproj")
            || name.ends_with(".Tests.csproj")
            || name.ends_with("Test.csproj")
            || name.ends_with(".Test.csproj")
    });

    if has_test_project {
        pass("dotnet_tests", ".NET test project is present")
    } else {
        warn(
            "dotnet_tests",
            ".NET test project is missing",
            "Add a test project such as Project.Tests.csproj.",
        )
    }
}

fn inspect_php(path: &Path) -> Vec<Check> {
    let composer = match read_json(path.join("composer.json")) {
        Some(value) => value,
        None => {
            return vec![warn(
                "php_composer_json",
                "composer.json is missing or invalid",
                "Add a valid composer.json.",
            )];
        }
    };

    vec![
        check_json_string(&composer, "php_name", "name", "composer.json"),
        check_json_string(&composer, "php_description", "description", "composer.json"),
        check_json_present(&composer, "php_license", "license", "composer.json"),
        check_json_present(&composer, "php_require", "require", "composer.json"),
        check_php_test_script(&composer),
        check_php_lockfile(path),
    ]
}

fn check_php_test_script(composer: &JsonValue) -> Check {
    if composer
        .get("scripts")
        .and_then(|scripts| scripts.get("test"))
        .is_some()
    {
        pass("php_test_script", "composer.json scripts.test is set")
    } else {
        warn(
            "php_test_script",
            "composer.json scripts.test is missing",
            "Add a test script to composer.json.",
        )
    }
}

fn check_php_lockfile(path: &Path) -> Check {
    if path.join("composer.lock").exists() {
        pass("php_composer_lock", "composer.lock is present")
    } else {
        warn(
            "php_composer_lock",
            "composer.lock is missing",
            "Run composer install or composer update and commit composer.lock for applications.",
        )
    }
}

fn inspect_ruby(path: &Path) -> Vec<Check> {
    vec![
        check_ruby_gemfile(path),
        check_ruby_lockfile(path),
        check_ruby_gemspec(path),
    ]
}

fn check_ruby_gemfile(path: &Path) -> Check {
    if path.join("Gemfile").exists() {
        pass("ruby_gemfile", "Gemfile is present")
    } else {
        warn("ruby_gemfile", "Gemfile is missing", "Add a Gemfile.")
    }
}

fn check_ruby_lockfile(path: &Path) -> Check {
    if path.join("Gemfile.lock").exists() {
        pass("ruby_gemfile_lock", "Gemfile.lock is present")
    } else {
        warn(
            "ruby_gemfile_lock",
            "Gemfile.lock is missing",
            "Run bundle install and commit Gemfile.lock for applications.",
        )
    }
}

fn check_ruby_gemspec(path: &Path) -> Check {
    if root_has_extension(path, "gemspec") {
        pass("ruby_gemspec", "Ruby gemspec is present")
    } else {
        warn(
            "ruby_gemspec",
            "Ruby gemspec is missing",
            "Add a .gemspec file for libraries, or ignore this warning for applications.",
        )
    }
}

fn inspect_cpp(path: &Path) -> Vec<Check> {
    vec![
        check_cpp_build_system(path),
        check_cpp_compile_commands(path),
        check_cpp_dependency_manifest(path),
    ]
}

fn check_cpp_build_system(path: &Path) -> Check {
    let candidates = ["CMakeLists.txt", "Makefile", "meson.build", "configure.ac"];

    if candidates
        .iter()
        .any(|candidate| path.join(candidate).exists())
    {
        pass("cpp_build_system", "C/C++ build system file is present")
    } else {
        warn(
            "cpp_build_system",
            "C/C++ build system file is missing",
            format!("Add one of {}.", candidates.join(", ")),
        )
    }
}

fn check_cpp_compile_commands(path: &Path) -> Check {
    if path.join("compile_commands.json").exists()
        || path.join("CMakePresets.json").exists()
        || path.join("CMakeUserPresets.json").exists()
    {
        pass("cpp_tooling_metadata", "C/C++ tooling metadata is present")
    } else {
        warn(
            "cpp_tooling_metadata",
            "C/C++ tooling metadata is missing",
            "Add compile_commands.json or CMakePresets.json for editor and analysis tooling.",
        )
    }
}

fn check_cpp_dependency_manifest(path: &Path) -> Check {
    if path.join("vcpkg.json").exists()
        || path.join("conanfile.txt").exists()
        || path.join("conanfile.py").exists()
    {
        pass(
            "cpp_dependency_manifest",
            "C/C++ dependency manifest is present",
        )
    } else {
        warn(
            "cpp_dependency_manifest",
            "C/C++ dependency manifest is missing",
            "Add vcpkg.json or conanfile.* when the project has external dependencies.",
        )
    }
}

fn inspect_swift(path: &Path) -> Vec<Check> {
    vec![
        check_swift_package(path),
        check_swift_package_resolved(path),
        check_swift_tests(path),
    ]
}

fn check_swift_package(path: &Path) -> Check {
    if path.join("Package.swift").exists() {
        pass("swift_package", "Swift Package.swift is present")
    } else {
        warn(
            "swift_package",
            "Swift Package.swift is missing",
            "Add Package.swift for Swift Package Manager projects.",
        )
    }
}

fn check_swift_package_resolved(path: &Path) -> Check {
    if path.join("Package.resolved").exists() {
        pass(
            "swift_package_resolved",
            "Swift Package.resolved is present",
        )
    } else {
        warn(
            "swift_package_resolved",
            "Swift Package.resolved is missing",
            "Commit Package.resolved when package dependencies are used.",
        )
    }
}

fn check_swift_tests(path: &Path) -> Check {
    if path.join("Tests").exists() {
        pass("swift_tests", "Swift Tests directory is present")
    } else {
        warn(
            "swift_tests",
            "Swift Tests directory is missing",
            "Add a Tests directory for Swift Package Manager tests.",
        )
    }
}

fn inspect_kotlin(path: &Path) -> Vec<Check> {
    let build_file = ["build.gradle.kts", "build.gradle"]
        .iter()
        .map(|candidate| path.join(candidate))
        .find(|candidate| candidate.exists());

    vec![
        check_kotlin_build_file(build_file.as_deref()),
        check_kotlin_plugin(build_file.as_deref()),
        check_kotlin_sources(path),
    ]
}

fn check_kotlin_build_file(build_file: Option<&Path>) -> Check {
    if build_file.is_some() {
        pass("kotlin_build_file", "Kotlin Gradle build file is present")
    } else {
        warn(
            "kotlin_build_file",
            "Kotlin Gradle build file is missing",
            "Add build.gradle.kts or build.gradle for Kotlin builds.",
        )
    }
}

fn check_kotlin_plugin(build_file: Option<&Path>) -> Check {
    let Some(build_file) = build_file else {
        return empty_check();
    };

    let contents = match std::fs::read_to_string(build_file) {
        Ok(contents) => contents,
        Err(_) => {
            return warn(
                "kotlin_plugin",
                "Kotlin build file could not be read",
                "Make the Gradle build file readable.",
            );
        }
    };

    if contents.contains("org.jetbrains.kotlin")
        || contents.contains("kotlin(\"")
        || contents.contains("kotlin(")
    {
        pass("kotlin_plugin", "Kotlin Gradle plugin is configured")
    } else {
        warn(
            "kotlin_plugin",
            "Kotlin Gradle plugin is missing",
            "Configure the org.jetbrains.kotlin Gradle plugin.",
        )
    }
}

fn check_kotlin_sources(path: &Path) -> Check {
    if path.join("src/main/kotlin").exists()
        || path.join("src/test/kotlin").exists()
        || root_has_extension(path, "kt")
        || root_has_extension(path, "kts")
    {
        pass("kotlin_sources", "Kotlin source path is present")
    } else {
        warn(
            "kotlin_sources",
            "Kotlin source path is missing",
            "Add src/main/kotlin or Kotlin source files.",
        )
    }
}

fn read_json(path: impl AsRef<Path>) -> Option<JsonValue> {
    let contents = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<JsonValue>(&contents).ok()
}

fn read_readme(path: &Path) -> Option<String> {
    ["README.md", "README", "README.txt"]
        .iter()
        .find_map(|candidate| std::fs::read_to_string(path.join(candidate)).ok())
}

fn package_manager_starts_with(path: &Path, prefix: &str) -> bool {
    read_json(path.join("package.json"))
        .and_then(|package| {
            package
                .get("packageManager")
                .and_then(JsonValue::as_str)
                .map(str::to_owned)
        })
        .is_some_and(|package_manager| package_manager.starts_with(prefix))
}

fn root_has_extension(path: &Path, extension: &str) -> bool {
    root_file_names(path).iter().any(|name| {
        Path::new(name)
            .extension()
            .is_some_and(|candidate| candidate == extension)
    })
}

fn root_file_names(path: &Path) -> Vec<String> {
    let Ok(entries) = path.read_dir() else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect()
}

fn inspect_jvm(path: &Path) -> Vec<Check> {
    let has_maven = path.join("pom.xml").exists();
    let has_gradle = ["build.gradle", "build.gradle.kts"]
        .iter()
        .any(|candidate| path.join(candidate).exists());

    let mut checks = vec![
        check_jvm_build_file(has_maven, has_gradle),
        check_jvm_wrapper(path, has_maven, has_gradle),
        check_jvm_settings(path, has_gradle),
    ];

    if has_maven {
        checks.extend(inspect_maven(path));
    }
    if has_gradle {
        checks.extend(inspect_gradle(path));
    }

    checks
}

fn check_jvm_build_file(has_maven: bool, has_gradle: bool) -> Check {
    if has_maven || has_gradle {
        pass("jvm_build_file", "JVM build file is present")
    } else {
        warn(
            "jvm_build_file",
            "JVM build file is missing",
            "Add pom.xml, build.gradle, or build.gradle.kts.",
        )
    }
}

fn check_jvm_wrapper(path: &Path, has_maven: bool, has_gradle: bool) -> Check {
    let has_maven_wrapper = path.join("mvnw").exists() || path.join("mvnw.cmd").exists();
    let has_gradle_wrapper = path.join("gradlew").exists() || path.join("gradlew.bat").exists();
    let wrapper_ok = (has_maven && has_maven_wrapper) || (has_gradle && has_gradle_wrapper);

    if wrapper_ok {
        pass("jvm_wrapper", "JVM build wrapper is present")
    } else {
        warn(
            "jvm_wrapper",
            "JVM build wrapper is missing",
            "Commit mvnw or gradlew so builds use a consistent tool version.",
        )
    }
}

fn check_jvm_settings(path: &Path, has_gradle: bool) -> Check {
    if !has_gradle {
        return empty_check();
    }

    if path.join("settings.gradle").exists() || path.join("settings.gradle.kts").exists() {
        pass("jvm_gradle_settings", "Gradle settings file is present")
    } else {
        warn(
            "jvm_gradle_settings",
            "Gradle settings file is missing",
            "Add settings.gradle or settings.gradle.kts.",
        )
    }
}

fn inspect_maven(path: &Path) -> Vec<Check> {
    let pom = match std::fs::read_to_string(path.join("pom.xml")) {
        Ok(contents) => contents,
        Err(_) => {
            return vec![warn(
                "jvm_maven_pom",
                "pom.xml could not be read",
                "Make pom.xml readable.",
            )];
        }
    };

    vec![
        check_xml_tag(&pom, "jvm_maven_group_id", "groupId", "Maven groupId"),
        check_xml_tag(
            &pom,
            "jvm_maven_artifact_id",
            "artifactId",
            "Maven artifactId",
        ),
        check_xml_tag(&pom, "jvm_maven_version", "version", "Maven version"),
    ]
}

fn check_xml_tag(contents: &str, id: &'static str, tag: &'static str, label: &str) -> Check {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");

    if contents.contains(&open) && contents.contains(&close) {
        pass(id, format!("{label} is set"))
    } else {
        warn(
            id,
            format!("{label} is missing"),
            format!("Set <{tag}> in pom.xml."),
        )
    }
}

fn inspect_gradle(path: &Path) -> Vec<Check> {
    let build_file = ["build.gradle", "build.gradle.kts"]
        .iter()
        .map(|candidate| path.join(candidate))
        .find(|candidate| candidate.exists());
    let Some(build_file) = build_file else {
        return Vec::new();
    };

    let contents = match std::fs::read_to_string(build_file) {
        Ok(contents) => contents,
        Err(_) => {
            return vec![warn(
                "jvm_gradle_build",
                "Gradle build file could not be read",
                "Make the Gradle build file readable.",
            )];
        }
    };

    vec![
        check_gradle_has_group(&contents),
        check_gradle_has_version(&contents),
        check_gradle_test_task(&contents),
    ]
}

fn check_gradle_has_group(contents: &str) -> Check {
    if contents.contains("group =") || contents.contains("group=") {
        pass("jvm_gradle_group", "Gradle group is set")
    } else {
        warn(
            "jvm_gradle_group",
            "Gradle group is missing",
            "Set group in the Gradle build file.",
        )
    }
}

fn check_gradle_has_version(contents: &str) -> Check {
    if contents.contains("version =") || contents.contains("version=") {
        pass("jvm_gradle_version", "Gradle version is set")
    } else {
        warn(
            "jvm_gradle_version",
            "Gradle version is missing",
            "Set version in the Gradle build file.",
        )
    }
}

fn check_gradle_test_task(contents: &str) -> Check {
    if contents.contains("test {") || contents.contains("tasks.test") {
        pass("jvm_gradle_test", "Gradle test task is configured")
    } else {
        warn(
            "jvm_gradle_test",
            "Gradle test task is not explicitly configured",
            "Configure the test task if this project has tests.",
        )
    }
}
