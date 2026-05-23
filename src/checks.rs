use std::path::Path;

use anyhow::Result;

use crate::report::{Check, pass, warn};

pub(crate) fn check_file(
    path: &Path,
    id: &'static str,
    relative: &str,
    pass_message: &str,
) -> Check {
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

pub(crate) fn check_any_file(
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

pub(crate) fn check_workflows(path: &Path, id: &'static str, pass_message: &str) -> Check {
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

pub(crate) fn check_directory_has_file(
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
