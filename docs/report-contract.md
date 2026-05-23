# Report Contract

`repo-doctor check --format json` emits `schema_version: 1`.

Within schema version 1:

- Existing top-level fields keep their meaning: `schema_version`, `path`,
  `selected_profiles`, `summary`, and `checks`.
- Existing check fields keep their meaning: `id`, `status`, `severity`,
  `message`, and `remediation`.
- New fields may be added in minor releases.
- `documentation_url` is optional and appears only when a rule has stable
  external or project documentation.
- Rule IDs are stable once released. If a rule must be replaced, keep the old
  rule ID available until a schema version bump or document the migration.
- Text and Markdown output are intended for humans. JSON and SARIF are intended
  for automation.

`summary.score` is an integer percentage based on passed checks divided by
total checks. Disabled rules are removed before summary calculation.
