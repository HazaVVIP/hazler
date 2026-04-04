# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | ✅ Yes             |
| 0.1.x   | ❌ No (EOL)        |

## Scope

This policy covers **security vulnerabilities in Hazler's own code** — the crawler engine, CLI, HTTP client, parsers, and associated crates in this repository.

### In Scope
- Remote code execution, memory safety issues, or unsafe use of Rust `unsafe` blocks.
- Credential/secret leakage in Hazler's own output or logs.
- Authentication bypass or privilege escalation in Hazler's auth framework.
- Path traversal or arbitrary file write in export/report generation.
- Dependency vulnerabilities with a direct, exploitable impact on Hazler users.

### Out of Scope
- Security issues in websites or APIs that Hazler crawls — Hazler is a security tool and may interact with intentionally defensive systems.
- Theoretical vulnerabilities in transitive dependencies with no demonstrated exploit path against Hazler.
- Issues that require an attacker to already have access to the machine running Hazler.
- Feature requests or non-security bugs (please use [GitHub Issues](https://github.com/HazaVVIP/hazler/issues) for those).

## Reporting a Vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

We prefer private disclosure to give time for a fix before public announcement.

### Preferred Method: GitHub Security Advisories

1. Go to [https://github.com/HazaVVIP/hazler/security/advisories/new](https://github.com/HazaVVIP/hazler/security/advisories/new).
2. Fill in the advisory form with a detailed description, reproduction steps, and potential impact.
3. Submit as a **private** advisory.

### Response SLA

| Event                          | Target Time     |
| ------------------------------ | --------------- |
| Initial acknowledgement        | 7 business days |
| Triage and severity assessment | 14 business days|
| Fix released (critical/high)   | 30 days         |
| Fix released (medium/low)      | 90 days         |
| Public disclosure              | After fix ships |

We will credit reporters in the release notes and CHANGELOG unless you prefer to remain anonymous.

## Security Best Practices for Users

- **Only crawl targets you are authorised to test.** Hazler is designed for security reconnaissance; use it responsibly and in accordance with applicable laws.
- **Protect credential files.** Auth config files (`--auth-file`) and state files contain sensitive data — store them with appropriate file permissions (`chmod 600`).
- **Review generated reports before sharing.** HTML and PDF reports may contain secrets detected during a crawl.
- **Keep Hazler up to date.** Subscribe to [GitHub releases](https://github.com/HazaVVIP/hazler/releases) to be notified of security updates.
