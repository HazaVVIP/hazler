# Hazler Roadmap

This document describes the high-level direction for Hazler. Completed work is tracked in [CHANGELOG.md](CHANGELOG.md).

---

## ✅ Phase 1 — MVP (Released in v0.1.0)

- Basic HTTP crawler with configurable depth, concurrency, and page limits
- HTML parsing and link extraction
- Concurrent crawling with semaphore-based throttling
- CLI interface with flexible output options (JSON, JSONL, CSV, Tree, URLs)
- MIT License, Dockerfile, and install script

## ✅ Phase 2 — Security Intelligence (Released in v0.1.x / v0.2.0)

- JavaScript endpoint discovery with regex-based extraction and confidence scoring
- Advanced URL normalisation for better deduplication
- Aggressive discovery mode (framework-specific extraction, API version variants)
- Framework detection (React, Angular, Vue, Next.js, etc.)
- Secret scanning — 38+ patterns for credentials, keys, tokens (classified by severity)
- `.frame` file support for endpoint extraction
- Comprehensive reporting (HTML, PDF, OpenAPI, Postman, Nuclei, ffuf, Burp)

## ✅ Phase 3 — Stealth, Scale & Resilience (Released in v0.2.0)

- **Headless browser support** — Crawl SPAs (React, Vue, Angular) via CDP/chromiumoxide
- **WAF evasion** — User-agent rotation, Chrome client hints, adaptive timing
- **Smart fuzzing** — URL mutation engine, parameter and endpoint discovery, BOLA/IDOR detection
- **Response diffing** — SimHash, K-means/DBSCAN clustering, baseline comparison
- **Retry & circuit breaker** — Exponential backoff, per-domain failure isolation
- **Per-domain rate limiting** — Token-bucket with adaptive 429 detection
- **State persistence & resume** — JSON and SQLite backends, auto-save
- **Graceful shutdown** — Ctrl+C saves state cleanly
- **Authentication framework** — Basic, Bearer, Cookie, Header, OAuth2, API Key, form login
- **Webhook notifications** — Slack, Discord, generic
- **GraphQL introspection** — Schema extraction from detected GraphQL endpoints
- **Source map parsing** — Original source path extraction and classification
- **eBPF/bpftrace monitoring scripts** — Deep system-level debugging

## 🔄 Phase 4 — Polish & Enterprise (Planned)

- `robots.txt` respect
- Proxy support (SOCKS5, HTTP)
- Distributed crawling with Redis backend
- OpenTelemetry integration for observability
- Real-time dashboard (web UI)
- Advanced authentication: form-based multi-step flows, PKCE OAuth2
- URL-path filtering and include/exclude rules
- Plugin system for extensibility
- `cargo install` / Homebrew / apt package distribution
- Comprehensive integration test suite

---

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
