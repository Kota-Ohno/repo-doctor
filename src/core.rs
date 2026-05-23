use std::path::Path;

use crate::checks::{check_any_file, check_directory_has_file, check_file, check_workflows};
use crate::report::Check;

pub(crate) fn inspect(path: &Path) -> Vec<Check> {
    vec![
        check_any_file(
            path,
            "readme",
            &["README.md", "README", "README.txt"],
            "README is present",
        ),
        check_any_file(
            path,
            "license",
            &[
                "LICENSE",
                "LICENSE.md",
                "LICENSE.txt",
                "LICENSE-MIT",
                "LICENSE-APACHE",
            ],
            "License file is present",
        ),
        check_file(path, "gitignore", ".gitignore", ".gitignore is present"),
        check_workflows(
            path,
            "github_actions",
            "GitHub Actions workflow file is present",
        ),
        check_any_file(
            path,
            "contributing",
            &[
                "CONTRIBUTING.md",
                "docs/CONTRIBUTING.md",
                ".github/CONTRIBUTING.md",
            ],
            "Contribution guide is present",
        ),
        check_any_file(
            path,
            "code_of_conduct",
            &[
                "CODE_OF_CONDUCT.md",
                "docs/CODE_OF_CONDUCT.md",
                ".github/CODE_OF_CONDUCT.md",
            ],
            "Code of conduct is present",
        ),
        check_any_file(
            path,
            "security_policy",
            &["SECURITY.md", "docs/SECURITY.md", ".github/SECURITY.md"],
            "Security policy is present",
        ),
        check_directory_has_file(
            path,
            "issue_templates",
            ".github/ISSUE_TEMPLATE",
            "Issue template is present",
        ),
        check_any_file(
            path,
            "pull_request_template",
            &[
                ".github/pull_request_template.md",
                ".github/PULL_REQUEST_TEMPLATE.md",
                "docs/pull_request_template.md",
                "docs/PULL_REQUEST_TEMPLATE.md",
                "PULL_REQUEST_TEMPLATE.md",
            ],
            "Pull request template is present",
        ),
        check_any_file(
            path,
            "changelog",
            &[
                "CHANGELOG.md",
                "CHANGES.md",
                "RELEASES.md",
                "docs/CHANGELOG.md",
            ],
            "Changelog or release notes are present",
        ),
    ]
}
