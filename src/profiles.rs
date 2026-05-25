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
    Frontend,
    Iac,
    Docs,
}

impl Profile {
    pub(crate) fn catalog() -> &'static [(&'static str, &'static str)] {
        &[
            ("auto", "core checks plus auto-detected ecosystems"),
            ("generic", "core checks only"),
            ("rust", "Cargo.toml"),
            ("node", "package.json"),
            ("python", "pyproject.toml, setup.py, or requirements.txt"),
            ("go", "go.mod"),
            (
                "docker",
                "Dockerfile, Containerfile, Compose, or .dockerignore",
            ),
            ("jvm", "pom.xml or Gradle files"),
            ("deno", "deno.json, deno.jsonc, or deno.lock"),
            ("bun", "bun lockfiles or packageManager=bun"),
            ("dotnet", ".sln, .csproj, .fsproj, .vbproj, or global.json"),
            ("php", "composer.json or composer.lock"),
            ("ruby", "Gemfile, Gemfile.lock, or .gemspec"),
            (
                "cpp",
                "CMake, Make, Meson, Autotools, or C/C++ source files",
            ),
            ("swift", "Package.swift or Package.resolved"),
            ("kotlin", "Kotlin Gradle files or Kotlin source paths"),
            (
                "frontend",
                "Next.js, Vite, Astro, SvelteKit, Remix, or Nuxt metadata",
            ),
            ("iac", "Terraform or OpenTofu files"),
            (
                "docs",
                "MkDocs, Docusaurus, mdBook, or VitePress docs sites",
            ),
        ]
    }

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
            Profile::Frontend => "frontend",
            Profile::Iac => "iac",
            Profile::Docs => "docs",
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
    if has_frontend_files(path) {
        profiles.push(Profile::Frontend);
    }
    if has_iac_files(path) {
        profiles.push(Profile::Iac);
    }
    if has_docs_site_files(path) {
        profiles.push(Profile::Docs);
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

fn has_frontend_files(path: &Path) -> bool {
    if root_has_any_file(
        path,
        &[
            "next.config.js",
            "next.config.mjs",
            "vite.config.js",
            "vite.config.ts",
            "astro.config.mjs",
            "svelte.config.js",
            "nuxt.config.ts",
        ],
    ) {
        return true;
    }

    let Some(package) = read_json(path.join("package.json")) else {
        return false;
    };

    [
        "next",
        "vite",
        "astro",
        "svelte",
        "@sveltejs/kit",
        "@remix-run/dev",
        "nuxt",
    ]
    .iter()
    .any(|dependency| package_has_dependency(&package, dependency))
}

fn has_iac_files(path: &Path) -> bool {
    root_has_extension(path, "tf")
        || root_has_extension(path, "tfvars")
        || path.join(".terraform.lock.hcl").exists()
        || path.join("tofu.lock.hcl").exists()
}

fn has_docs_site_files(path: &Path) -> bool {
    root_has_any_file(
        path,
        &[
            "mkdocs.yml",
            "docusaurus.config.js",
            "docusaurus.config.ts",
            "book.toml",
            "vitepress.config.ts",
            "vitepress.config.js",
        ],
    ) || path.join("docs/.vitepress/config.ts").exists()
        || path.join("docs/.vitepress/config.js").exists()
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
            Profile::Frontend => checks.extend(inspect_frontend(path)),
            Profile::Iac => checks.extend(inspect_iac(path)),
            Profile::Docs => checks.extend(inspect_docs_site(path)),
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
        if parsed.get("workspace").is_some() {
            let mut checks = vec![
                check_rust_workspace(path, &parsed),
                check_cargo_lock(path),
                check_gitignore_target(path),
                check_rust_toolchain(path),
                check_rust_tooling_configs(path),
                check_rust_ci_commands(path),
            ];
            checks.retain(|check| !check.id.is_empty());
            return checks;
        }

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
        check_rust_workspace(path, &parsed),
        check_cargo_lock(path),
        check_gitignore_target(path),
        check_rust_readme_commands(path, package),
        check_rust_toolchain(path),
        check_rust_tooling_configs(path),
        check_rust_ci_commands(path),
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

fn check_rust_workspace(path: &Path, manifest: &toml::Value) -> Check {
    let workspace = manifest.get("workspace").and_then(toml::Value::as_table);
    let Some(workspace) = workspace else {
        return pass("rust_workspace", "Cargo manifest is a package crate");
    };

    let Some(members) = workspace.get("members").and_then(toml::Value::as_array) else {
        return warn(
            "rust_workspace",
            "Cargo workspace has no members",
            "Set `workspace.members` in Cargo.toml or remove the empty workspace section.",
        );
    };

    if members.is_empty() {
        return warn(
            "rust_workspace",
            "Cargo workspace has no members",
            "Set `workspace.members` in Cargo.toml or remove the empty workspace section.",
        );
    }

    let missing = members
        .iter()
        .filter_map(toml::Value::as_str)
        .filter(|member| !workspace_member_exists(path, member))
        .collect::<Vec<_>>();

    if missing.is_empty() {
        pass("rust_workspace", "Cargo workspace members are configured")
    } else {
        warn(
            "rust_workspace",
            format!(
                "Cargo workspace member paths are missing: {}",
                missing.join(", ")
            ),
            "Create the workspace members or update `workspace.members` in Cargo.toml.",
        )
    }
}

fn workspace_member_exists(path: &Path, member: &str) -> bool {
    if !member.contains('*') {
        return path.join(member).join("Cargo.toml").exists();
    }

    workspace_glob_matches(path, path, member)
}

fn workspace_glob_matches(root: &Path, current: &Path, member: &str) -> bool {
    let Ok(entries) = current.read_dir() else {
        return false;
    };

    entries.filter_map(Result::ok).any(|entry| {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            return false;
        }
        let Ok(relative) = entry_path.strip_prefix(root).map(Path::to_path_buf) else {
            return false;
        };
        let relative = relative.to_string_lossy().replace('\\', "/");
        (wildcard_match(member, &relative) && entry_path.join("Cargo.toml").exists())
            || workspace_glob_matches(root, &entry_path, member)
    })
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    let mut remainder = value;
    let mut first = true;

    for part in pattern.split('*') {
        if part.is_empty() {
            first = false;
            continue;
        }
        if first && !pattern.starts_with('*') {
            let Some(stripped) = remainder.strip_prefix(part) else {
                return false;
            };
            remainder = stripped;
        } else {
            let Some(index) = remainder.find(part) else {
                return false;
            };
            remainder = &remainder[index + part.len()..];
        }
        first = false;
    }

    pattern.ends_with('*') || remainder.is_empty()
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

fn check_rust_toolchain(path: &Path) -> Check {
    if path.join("rust-toolchain.toml").exists() || path.join("rust-toolchain").exists() {
        pass("rust_toolchain", "Rust toolchain pin is present")
    } else {
        warn(
            "rust_toolchain",
            "Rust toolchain pin is missing",
            "Add rust-toolchain.toml or rust-toolchain to make local and CI toolchains reproducible.",
        )
    }
}

fn check_rust_tooling_configs(path: &Path) -> Check {
    let has_rustfmt = path.join("rustfmt.toml").exists() || path.join(".rustfmt.toml").exists();
    let has_clippy = path.join("clippy.toml").exists() || path.join(".clippy.toml").exists();

    if has_rustfmt && has_clippy {
        pass(
            "rust_tooling_config",
            "Rust rustfmt and clippy configs are present",
        )
    } else {
        warn(
            "rust_tooling_config",
            "Rust rustfmt or clippy config is missing",
            "Add rustfmt.toml and clippy.toml when the project has non-default formatting or lint policy.",
        )
    }
}

fn check_rust_ci_commands(path: &Path) -> Check {
    let all_workflows = workflow_contents(path);
    if all_workflows.is_empty() {
        return pass(
            "github_actions_rust_ci",
            "Rust CI command check is not applicable",
        );
    }

    let has_fmt = all_workflows.contains("cargo fmt")
        && (all_workflows.contains("--check") || all_workflows.contains("fmtc"));
    let has_clippy = all_workflows.contains("cargo clippy") || all_workflows.contains("lint");
    let has_tests = all_workflows.contains("cargo test") || all_workflows.contains("cargo nextest");

    if has_fmt && has_clippy && has_tests {
        pass(
            "github_actions_rust_ci",
            "GitHub Actions include Rust fmt, clippy, and test commands",
        )
    } else {
        warn(
            "github_actions_rust_ci",
            "GitHub Actions are missing Rust fmt, clippy, or test commands",
            "Add cargo fmt --all --check, cargo clippy, and cargo test or cargo nextest to CI.",
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
        check_node_package_manager(path, &parsed),
        check_node_lockfile(path),
        check_typescript_config(path, &parsed),
        check_node_lint_and_format(path, &parsed),
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

fn check_node_package_manager(path: &Path, package: &JsonValue) -> Check {
    let expected = if path.join("pnpm-lock.yaml").exists() {
        Some("pnpm")
    } else if path.join("yarn.lock").exists() {
        Some("yarn")
    } else if path.join("bun.lock").exists() || path.join("bun.lockb").exists() {
        Some("bun")
    } else {
        None
    };

    let Some(expected) = expected else {
        return pass(
            "node_package_manager",
            "packageManager check is not applicable",
        );
    };

    let package_manager = package
        .get("packageManager")
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .trim();

    if package_manager.starts_with(&format!("{expected}@")) {
        pass(
            "node_package_manager",
            "package.json packageManager matches the lockfile",
        )
    } else if package_manager.is_empty() {
        warn(
            "node_package_manager",
            format!("package.json packageManager is missing for {expected}"),
            format!(
                "Set `packageManager` in package.json, for example `{expected}@<version>`, so CI setup actions can install the right package manager."
            ),
        )
    } else {
        warn(
            "node_package_manager",
            format!(
                "package.json packageManager `{package_manager}` does not match {expected} lockfile"
            ),
            format!("Set `packageManager` to a {expected} version or use the matching lockfile."),
        )
    }
}

fn check_node_lockfile(path: &Path) -> Check {
    let candidates = [
        "package-lock.json",
        "npm-shrinkwrap.json",
        "yarn.lock",
        "pnpm-lock.yaml",
        "bun.lock",
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

fn check_typescript_config(path: &Path, package: &JsonValue) -> Check {
    let has_typescript = path.join("tsconfig.json").exists()
        || package_has_dependency(package, "typescript")
        || recursive_has_extension(path, "ts");
    if !has_typescript {
        return pass(
            "node_typescript_config",
            "TypeScript config check is not applicable",
        );
    }

    let config = match read_json(path.join("tsconfig.json")) {
        Some(value) => value,
        None => {
            return warn(
                "node_typescript_config",
                "TypeScript project is missing tsconfig.json",
                "Add tsconfig.json for TypeScript compiler settings.",
            );
        }
    };
    let strict = tsconfig_strict_enabled(path, &config);

    if strict {
        pass(
            "node_typescript_config",
            "TypeScript strict mode is enabled",
        )
    } else {
        warn(
            "node_typescript_config",
            "TypeScript strict mode is not enabled",
            "Set compilerOptions.strict = true in tsconfig.json.",
        )
    }
}

fn tsconfig_strict_enabled(path: &Path, config: &JsonValue) -> bool {
    if config
        .get("compilerOptions")
        .and_then(|options| options.get("strict"))
        .and_then(JsonValue::as_bool)
        .unwrap_or(false)
    {
        return true;
    }

    config
        .get("references")
        .and_then(JsonValue::as_array)
        .is_some_and(|references| {
            references.iter().any(|reference| {
                let Some(reference_path) = reference.get("path").and_then(JsonValue::as_str) else {
                    return false;
                };
                let reference_path = path.join(reference_path);
                let mut candidates = if reference_path.extension().is_some() {
                    vec![reference_path]
                } else {
                    vec![
                        reference_path.join("tsconfig.json"),
                        reference_path.with_extension("json"),
                    ]
                };
                candidates.dedup();

                candidates.into_iter().any(|config_path| {
                    read_json(config_path)
                        .as_ref()
                        .is_some_and(|config| tsconfig_strict_enabled(path, config))
                })
            })
        })
}

fn check_node_lint_and_format(path: &Path, package: &JsonValue) -> Check {
    let has_lint = package_script(package, "lint")
        || root_has_any_file(
            path,
            &[
                "eslint.config.js",
                "eslint.config.mjs",
                ".eslintrc",
                ".eslintrc.json",
            ],
        );
    let has_format = package_script(package, "format")
        || root_has_any_file(
            path,
            &[
                ".prettierrc",
                ".prettierrc.json",
                "prettier.config.js",
                "prettier.config.mjs",
            ],
        );

    if has_lint && has_format {
        pass(
            "node_lint_format",
            "Node lint and format commands are configured",
        )
    } else {
        warn(
            "node_lint_format",
            "Node lint or format command is missing",
            "Add lint and format scripts or ESLint/Prettier config files.",
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
    checks.push(check_python_lint_format(&parsed));
    checks.push(check_python_pytest_config(path, &parsed));
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

fn check_python_lint_format(pyproject: &toml::Value) -> Check {
    let tool = pyproject.get("tool").and_then(toml::Value::as_table);
    let has_ruff = tool.is_some_and(|tool| tool.contains_key("ruff"));
    let has_black = tool.is_some_and(|tool| tool.contains_key("black"));

    if has_ruff || has_black {
        pass(
            "python_lint_format",
            "Python lint or format tooling is configured",
        )
    } else {
        warn(
            "python_lint_format",
            "Python lint or format tooling is missing",
            "Configure Ruff, Black, or another formatter/linter in pyproject.toml.",
        )
    }
}

fn check_python_pytest_config(path: &Path, pyproject: &toml::Value) -> Check {
    let has_pytest_ini = path.join("pytest.ini").exists() || path.join("tox.ini").exists();
    let has_pytest_pyproject = pyproject
        .get("tool")
        .and_then(|tool| tool.get("pytest"))
        .is_some();

    if has_pytest_ini || has_pytest_pyproject {
        pass(
            "python_pytest_config",
            "Python pytest configuration is present",
        )
    } else {
        warn(
            "python_pytest_config",
            "Python pytest configuration is missing",
            "Add pytest.ini or [tool.pytest] configuration when pytest is used.",
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
        check_go_sum(path, &go_mod),
        check_go_ci_commands(path),
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

fn check_go_sum(path: &Path, go_mod: &str) -> Check {
    let has_requirements = go_mod.lines().map(str::trim).any(|line| {
        line.starts_with("require ")
            || line == "require ("
            || (line.starts_with('\t') && !line.starts_with("\t//"))
    });
    if !has_requirements {
        return pass("go_sum", "go.sum check is not applicable");
    }

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

fn check_go_ci_commands(path: &Path) -> Check {
    let workflows = workflow_contents(path);
    if workflows.is_empty() {
        return warn(
            "go_ci_commands",
            "Go CI workflow commands are missing",
            "Add GitHub Actions steps for go test, go vet, and gofmt checking.",
        );
    }

    let has_test = workflows.contains("go test");
    let has_vet = workflows.contains("go vet");
    let has_fmt = workflows.contains("gofmt") || workflows.contains("go fmt");

    if has_test && has_vet && has_fmt {
        pass(
            "go_ci_commands",
            "Go CI includes test, vet, and formatting commands",
        )
    } else {
        warn(
            "go_ci_commands",
            "Go CI is missing test, vet, or formatting commands",
            "Add go test, go vet, and gofmt or go fmt checks to CI.",
        )
    }
}

fn inspect_docker(path: &Path) -> Vec<Check> {
    vec![
        check_container_build_file(path),
        check_dockerignore(path),
        check_compose_file(path),
        check_dockerfile_latest_tag(path),
        check_dockerfile_healthcheck(path),
        check_dockerfile_user(path),
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

fn check_dockerfile_healthcheck(path: &Path) -> Check {
    let Some(contents) = read_container_build_file(path) else {
        return empty_check();
    };

    if contents.lines().any(|line| {
        line.trim_start()
            .to_ascii_uppercase()
            .starts_with("HEALTHCHECK")
    }) {
        pass("docker_healthcheck", "Container HEALTHCHECK is configured")
    } else {
        warn(
            "docker_healthcheck",
            "Container HEALTHCHECK is missing",
            "Add a HEALTHCHECK when the image runs a long-lived service.",
        )
    }
}

fn check_dockerfile_user(path: &Path) -> Check {
    let Some(contents) = read_container_build_file(path) else {
        return empty_check();
    };

    if contents
        .lines()
        .any(|line| line.trim_start().to_ascii_uppercase().starts_with("USER "))
    {
        pass(
            "docker_non_root_user",
            "Container switches to a configured USER",
        )
    } else {
        warn(
            "docker_non_root_user",
            "Container USER directive is missing",
            "Use a non-root USER when the runtime image does not require root.",
        )
    }
}

fn read_container_build_file(path: &Path) -> Option<String> {
    ["Dockerfile", "Containerfile"]
        .iter()
        .find_map(|candidate| std::fs::read_to_string(path.join(candidate)).ok())
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

fn package_has_dependency(package: &JsonValue, name: &str) -> bool {
    ["dependencies", "devDependencies", "peerDependencies"]
        .iter()
        .any(|section| {
            package
                .get(section)
                .is_some_and(|deps| deps.get(name).is_some())
        })
}

fn package_script(package: &JsonValue, name: &str) -> bool {
    package
        .get("scripts")
        .and_then(|scripts| scripts.get(name))
        .and_then(JsonValue::as_str)
        .is_some_and(|script| !script.trim().is_empty())
}

fn root_has_any_file(path: &Path, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| path.join(candidate).exists())
}

fn recursive_has_extension(path: &Path, extension: &str) -> bool {
    let Ok(entries) = path.read_dir() else {
        return false;
    };

    entries.filter_map(Result::ok).any(|entry| {
        let entry_path = entry.path();
        if entry_path
            .file_name()
            .is_some_and(|name| name == "target" || name == "node_modules" || name == ".git")
        {
            return false;
        }
        if entry_path.is_dir() {
            recursive_has_extension(&entry_path, extension)
        } else {
            entry_path
                .extension()
                .is_some_and(|candidate| candidate == extension)
        }
    })
}

fn workflow_contents(path: &Path) -> String {
    let workflows_dir = path.join(".github/workflows");
    let Ok(entries) = workflows_dir.read_dir() else {
        return String::new();
    };

    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let is_workflow = path
                .extension()
                .is_some_and(|extension| extension == "yml" || extension == "yaml");
            is_workflow
                .then(|| std::fs::read_to_string(path).ok())
                .flatten()
        })
        .collect::<Vec<_>>()
        .join("\n")
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

fn inspect_frontend(path: &Path) -> Vec<Check> {
    let package = read_json(path.join("package.json")).unwrap_or(JsonValue::Null);
    vec![
        check_frontend_framework(path, &package),
        check_frontend_build_script(&package),
        check_frontend_source_dir(path),
    ]
}

fn check_frontend_framework(path: &Path, package: &JsonValue) -> Check {
    let frameworks = [
        "next",
        "vite",
        "astro",
        "@sveltejs/kit",
        "@remix-run/dev",
        "nuxt",
    ];
    let dependency = frameworks
        .iter()
        .find(|framework| package_has_dependency(package, framework));
    if let Some(framework) = dependency {
        pass(
            "frontend_framework",
            format!("Frontend framework dependency is present: {framework}"),
        )
    } else if has_frontend_files(path) {
        pass("frontend_framework", "Frontend framework config is present")
    } else {
        warn(
            "frontend_framework",
            "Frontend framework metadata is missing",
            "Add framework dependencies or config files such as vite.config.ts or next.config.js.",
        )
    }
}

fn check_frontend_build_script(package: &JsonValue) -> Check {
    if package_script(package, "build") {
        pass(
            "frontend_build_script",
            "Frontend build script is configured",
        )
    } else {
        warn(
            "frontend_build_script",
            "Frontend build script is missing",
            "Add a package.json build script for CI and deployment.",
        )
    }
}

fn check_frontend_source_dir(path: &Path) -> Check {
    if ["src", "app", "pages"]
        .iter()
        .any(|candidate| path.join(candidate).is_dir())
    {
        pass(
            "frontend_source_dir",
            "Frontend source directory is present",
        )
    } else {
        warn(
            "frontend_source_dir",
            "Frontend source directory is missing",
            "Add src/, app/, or pages/ depending on the framework.",
        )
    }
}

fn inspect_iac(path: &Path) -> Vec<Check> {
    vec![
        check_iac_files(path),
        check_iac_lockfile(path),
        check_iac_ci(path),
    ]
}

fn check_iac_files(path: &Path) -> Check {
    if root_has_extension(path, "tf") {
        pass(
            "iac_terraform_files",
            "Terraform/OpenTofu files are present",
        )
    } else {
        warn(
            "iac_terraform_files",
            "Terraform/OpenTofu files are missing",
            "Add .tf files or run a different profile.",
        )
    }
}

fn check_iac_lockfile(path: &Path) -> Check {
    if path.join(".terraform.lock.hcl").exists() || path.join("tofu.lock.hcl").exists() {
        pass("iac_lockfile", "IaC provider lockfile is present")
    } else {
        warn(
            "iac_lockfile",
            "IaC provider lockfile is missing",
            "Commit .terraform.lock.hcl or tofu.lock.hcl for reproducible provider versions.",
        )
    }
}

fn check_iac_ci(path: &Path) -> Check {
    let workflows = workflow_contents(path);
    if workflows.contains("terraform fmt")
        || workflows.contains("terraform validate")
        || workflows.contains("tofu fmt")
        || workflows.contains("tofu validate")
    {
        pass(
            "iac_ci_validate",
            "IaC formatting or validation is present in CI",
        )
    } else {
        warn(
            "iac_ci_validate",
            "IaC formatting or validation is missing from CI",
            "Add terraform/tofu fmt and validate commands to CI.",
        )
    }
}

fn inspect_docs_site(path: &Path) -> Vec<Check> {
    vec![
        check_docs_site_config(path),
        check_docs_site_dir(path),
        check_docs_site_ci(path),
    ]
}

fn check_docs_site_config(path: &Path) -> Check {
    if has_docs_site_files(path) {
        pass("docs_site_config", "Docs site configuration is present")
    } else {
        warn(
            "docs_site_config",
            "Docs site configuration is missing",
            "Add mkdocs.yml, docusaurus.config.*, book.toml, or VitePress config.",
        )
    }
}

fn check_docs_site_dir(path: &Path) -> Check {
    if path.join("docs").is_dir() || path.join("website").is_dir() {
        pass(
            "docs_site_content",
            "Docs site content directory is present",
        )
    } else {
        warn(
            "docs_site_content",
            "Docs site content directory is missing",
            "Add docs/ or website/ content.",
        )
    }
}

fn check_docs_site_ci(path: &Path) -> Check {
    let workflows = workflow_contents(path);
    if workflows.contains("mkdocs build")
        || workflows.contains("docusaurus build")
        || workflows.contains("mdbook build")
        || workflows.contains("vitepress build")
    {
        pass("docs_site_ci", "Docs site build is present in CI")
    } else {
        warn(
            "docs_site_ci",
            "Docs site build is missing from CI",
            "Add a docs build command to CI.",
        )
    }
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
