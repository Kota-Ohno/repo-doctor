# Security Policy

## Supported Versions

The latest `main` branch is supported.

## Reporting A Vulnerability

Please report security issues privately to the repository owner.

Do not open a public issue for suspected vulnerabilities. Include:

- affected version or commit,
- reproduction steps,
- expected impact,
- suggested fix, if known.

## Dependency Review

This template uses:

```bash
cargo audit
cargo deny check
```

Run both before release.
