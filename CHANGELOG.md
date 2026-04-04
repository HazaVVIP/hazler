# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-04-04

### Added

#### Core Infrastructure
- **State persistence & resume** — Crawl state can be saved to JSON or SQLite and resumed with `--resume`. Auto-save interval configurable via `--auto-save` (default: 60 s).
- **Graceful shutdown** — Ctrl+C saves progress and exits cleanly via `GracefulShutdown` / `ShutdownHandler`.
- **Real-time progress tracking** — Live progress indicator showing valid endpoints found; suppress with `--no-progress`.
- **Retry & circuit breaker** — Exponential backoff with jitter (`--max-retries`), per-domain circuit breaker (`--circuit-breaker`) to prevent cascading failures.
- **Per-domain rate limiting** — Token-bucket rate limiter with adaptive adjustment on HTTP 429 responses (`--rate-limit`).
- **JS confidence threshold** — Filter JavaScript-extracted endpoints by regex reliability score (`--js-confidence`, default: 0.5).
- **Noise filter** — Automatically suppresses repetitive WAF/modified-404 false positives.
- **Clean output mode** — Default output prints only verified 200-range endpoints in real time; use `--full-output` for the full tree view.

#### Headless Browser (`hazler-browser` crate)
- New `hazler-browser` crate powered by [chromiumoxide](https://github.com/mattsse/chromiumoxide).
- `Network.requestWillBeSent` hook captures hidden API calls, auth headers, and JSON payloads that pure HTTP crawling misses.
- Screenshot support (`--screenshot-path`), image disable for speed (`--disable-images`).
- CLI flag: `--browser`.

#### GraphQL Intelligence
- Automatic GraphQL endpoint detection via URL pattern and content-type analysis.
- Full introspection query system extracts types, queries, mutations, and subscriptions.
- Sample query/mutation generation from discovered schema.
- CLI flag: `--graphql-introspect`.

#### Source Map Parsing
- Automatic detection and parsing of JavaScript source maps (`*.js.map`).
- Extracts original source paths and classifies them (admin panels, API routes, auth logic, secrets).
- Framework detection from source paths.
- Enabled by default; disable with `--no-source-maps`.

#### Fuzzing (`hazler-fuzzer` crate)
- New `hazler-fuzzer` crate with a URL mutation engine.
- Generates pluralisation, extension, API-version, and trailing-slash variants.
- Parameter discovery with built-in wordlists (60+ entries each for endpoints, params, files).
- BOLA/IDOR detection via response comparison.
- CLI flags: `--fuzz`, `--fuzz-level` (`minimal` / `default` / `aggressive` / `full`), `--fuzz-crawl`, `--fuzz-output`.

#### Response Diffing Engine
- SimHash algorithm for fuzzy document hashing and near-duplicate detection.
- Response clustering with K-means and DBSCAN algorithms (`--cluster`).
- Smart normalisation strips timestamps, tokens, UUIDs, and session IDs before comparison.
- Baseline mode stores normalised hashes for temporal change detection (`--baseline`, `--compare`, `--diff-threshold`).

#### Authentication Framework
- Comprehensive authentication: Basic, Bearer, Cookie, Header, OAuth2, API Key (`--auth`).
- JSON configuration file for complex auth (form-based login, multi-step flows) via `--auth-file`.
- Secure credential handling (credentials are never logged).

#### Reporting & Export
- **HTML report** — Interactive report with charts and tabbed interface (`--export html:report.html`).
- **PDF report** — Professional PDF via `printpdf` (`--export pdf:report.pdf`).
- **SQLite export** — Full crawl data in a queryable database (`--export sqlite:data.db`).
- **OpenAPI / Swagger spec** — Machine-readable API spec (`--export openapi:spec.yaml`, `-o openapi`).
- **Postman collection** — Ready-to-import collection (`--export postman:collection.json`, `-o postman`).
- **Nuclei / ffuf / Burp output** — Direct integration with popular security tools (`-o nuclei`, `-o ffuf`, `-o burp`).
- **Webhook notifications** — Post results to Slack, Discord, or a generic webhook (`--webhook`).

#### Developer & Operations
- `hazler-browser` and `hazler-fuzzer` crates added to the workspace.
- eBPF/bpftrace monitoring scripts in `scripts/bpftrace/` (network, performance, security, HTTP tracing).
- Interactive wizard mode for first-time users (`--wizard`).
- `--no-color` / `--plain` flags for CI/log-file-friendly output.
- `--quiet` flag suppresses the post-crawl summary.

### Changed
- Default output format changed from `tree` to `clean` (verified 200-range endpoints only, real-time).
- Stealth mode enabled by default (`--no-stealth` to disable).
- Secret scanning enabled by default (`--no-secrets` to disable).
- Source map parsing enabled by default.
- User-agent now reads from `CARGO_PKG_VERSION` at compile time instead of a hardcoded string.

### Removed
- Unused `elasticsearch` alpha dependency removed from `hazler-cli`.

## [0.1.0] - 2025-01-01

### Added
- Initial release of Hazler.
- HTTP crawler with configurable depth, concurrency, and page limits.
- HTML parsing and link extraction (`hazler-parser` crate).
- JavaScript endpoint discovery via regex patterns (`hazler-js-parser` crate).
- Secret and credential scanning with 38+ patterns, severity classification (`hazler-secrets` crate).
- WAF evasion / stealth mode: user-agent rotation, Chrome client hints, request timing jitter.
- Scope validation (same-domain, strict, subdomains).
- Output formats: `json`, `jsonl`, `csv`, `tree`, `urls`, `clean`.
- Command-line interface with Clap v4 (`hazler-cli` crate).
- `Dockerfile` and `install.sh` for quick deployment.
- MIT License.

[Unreleased]: https://github.com/HazaVVIP/hazler/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/HazaVVIP/hazler/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/HazaVVIP/hazler/releases/tag/v0.1.0
