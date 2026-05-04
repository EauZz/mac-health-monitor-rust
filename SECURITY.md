# Security Policy

## Supported Versions

Only the latest `main` branch is currently supported.

## Reporting A Vulnerability

Please open a private security advisory on GitHub if available, or contact the maintainer directly.

Do not open a public issue for vulnerabilities that expose local files, tokens, command-line arguments, or private account data.

## Local Data Boundaries

Mac Health Monitor Rust is local-first. It should not transmit system metrics, process lists, OpenUsage cache data, or LLM usage data to remote services.

Security-sensitive changes should preserve these boundaries:

- no remote telemetry by default;
- no transcript reading;
- no API key/token collection;
- no privileged helper without a separate explicit review.
