<div align="center">

# Hazler

**Next-generation web crawler for security reconnaissance.**  
Built in Rust — fast, stealthy, and operator-friendly.

[![CI](https://github.com/HazaVVIP/hazler/actions/workflows/ci.yml/badge.svg)](https://github.com/HazaVVIP/hazler/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/HazaVVIP/hazler?include_prereleases)](https://github.com/HazaVVIP/hazler/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

</div>

---

## Installation

### Pre-built Binaries *(recommended — no Rust required)*

Download the latest release from the **[Releases page](https://github.com/HazaVVIP/hazler/releases)**.

| Platform | Archive |
|----------|---------|
| Linux x86_64 | `hazler-linux-x86_64.tar.gz` |
| Linux ARM64 | `hazler-linux-aarch64.tar.gz` |
| macOS Intel | `hazler-macos-x86_64.tar.gz` |
| macOS Apple Silicon | `hazler-macos-aarch64.tar.gz` |
| Windows x86_64 | `hazler-windows-x86_64.zip` |

```bash
# Linux / macOS
tar xzf hazler-*.tar.gz
sudo mv hazler /usr/local/bin/
hazler --version
```

> Pre-releases (alpha/rc) are also available on the Releases page for early access.

### One-line Installer

```bash
curl -sSf https://raw.githubusercontent.com/HazaVVIP/hazler/main/install.sh | bash
```

### Docker

```bash
docker pull ghcr.io/hazavvip/hazler:latest
docker run --rm ghcr.io/hazavvip/hazler:latest https://example.com
```

### Build from Source

```bash
git clone https://github.com/HazaVVIP/hazler.git
cd hazler
cargo build --release
# binary → target/release/hazler
```

---

## Quick Start

```bash
# Basic crawl — prints verified URLs in real time
hazler https://example.com

# Full security scan (secrets, JS endpoints, GraphQL, source maps)
hazler https://example.com --all

# Crawl a React/Vue/Angular SPA
hazler https://app.example.com --browser

# Fuzz for hidden endpoints
hazler https://example.com --fuzz --fuzz-level aggressive

# Resume an interrupted crawl
hazler https://example.com --auto-save 30
hazler https://example.com --resume hazler-state.json

# Export reports
hazler https://example.com --export html:report.html --export sqlite:crawl.db
```

### Interactive Wizard *(great for first-time users)*

```bash
hazler --wizard
```

---

## Features

| Category | Capabilities |
|----------|-------------|
| **Crawling** | Concurrent HTTP, BFS depth control, scope validation (domain / subdomain) |
| **Stealth** | WAF evasion, user-agent rotation, Chrome client hints, adaptive timing |
| **Discovery** | JS endpoint extraction, source map parsing, GraphQL introspection, `.frame` files |
| **Secret Scanning** | 38+ patterns — AWS keys, tokens, private keys, DB strings (Critical → Low) |
| **Headless Browser** | Chrome/CDP via `--browser`; captures SPA routes, XHR calls, screenshots |
| **Fuzzing** | URL mutations, parameter discovery, BOLA/IDOR detection (`hazler-fuzzer`) |
| **Auth** | Basic, Bearer, Cookie, Header, API Key, OAuth2, form login (`--auth` / `--auth-file`) |
| **Diffing** | SimHash, K-means/DBSCAN clustering, baseline comparison (`--baseline`, `--compare`) |
| **Persistence** | JSON & SQLite state; resume interrupted crawls (`--resume`, `--auto-save`) |
| **Export** | HTML report, PDF, SQLite, OpenAPI, Postman, Nuclei, ffuf, Burp Suite |
| **Webhooks** | Slack, Discord, generic HTTP (`--webhook`) |
| **Rate Control** | Per-domain token-bucket, adaptive 429 detection, circuit breaker |

---

## Common Flags

```
hazler [OPTIONS] <URL>

  -d, --max-depth <N>          Crawl depth [default: 3]
  -c, --concurrency <N>        Concurrent requests [default: 10]
  -p, --max-pages <N>          Page limit (0 = unlimited)
  -t, --timeout <secs>         Request timeout [default: 10]
  -o, --output-format <FMT>    clean | json | jsonl | csv | urls | nuclei | ffuf | burp | openapi | postman
      --all                    Enable all scanning features
      --aggressive             Deep JS / URL variant discovery
      --browser                Headless Chrome for SPAs
      --fuzz                   Smart endpoint fuzzing
      --graphql-introspect     GraphQL schema extraction
      --auth <METHOD:VALUE>    Authentication (basic / bearer / apikey / cookie)
      --export <TYPE:FILE>     Export report (html / pdf / sqlite / openapi / postman)
      --resume <FILE>          Resume from saved state
      --auto-save <secs>       Periodic state save interval
      --no-stealth             Disable WAF evasion
      --no-secrets             Disable secret scanning
      --proxy <URL>            Proxy (socks5:// or http://)
  -w, --wizard                 Interactive setup wizard
  -v, --verbose                Debug output
```

Full reference → [`docs/CLI.md`](docs/CLI.md)

---

## Pipeline Mode

```bash
# Read targets from stdin
cat targets.txt | hazler - -o urls

# Bug bounty pipeline
subfinder -d target.com | httpx -silent | hazler - --all -o nuclei | nuclei -t templates/
```

---

## Output Formats

| Format | Description |
|--------|-------------|
| `clean` *(default)* | Live stream of verified 200-range URLs |
| `json` / `jsonl` | Structured data for scripting / jq |
| `urls` | Plain URL list |
| `nuclei` / `ffuf` / `burp` | Direct tool integration |
| `openapi` / `postman` | API spec export |
| `--export html:FILE` | Interactive HTML report |
| `--export pdf:FILE` | PDF report |
| `--export sqlite:FILE` | Queryable database |

---

## Architecture

8 focused Rust crates in a workspace:

```
hazler-core       Core crawler engine, persistence, diffing
hazler-http       HTTP client, authentication, stealth headers
hazler-parser     HTML link extraction, GraphQL detection
hazler-js-parser  JS endpoint & source map extraction
hazler-secrets    Credential pattern matching
hazler-browser    Headless Chrome via CDP (chromiumoxide)
hazler-fuzzer     URL mutation, parameter discovery
hazler-cli        CLI entry point
```

Full diagram → [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)

---

## Development

```bash
cargo test --workspace          # Run all tests
cargo fmt --check               # Check formatting
cargo clippy -- -D warnings     # Lint
cargo build --release           # Release build
```

---

## Legal & Ethics

Hazler is a security research tool. **Only crawl targets you are authorised to test.**  
See [SECURITY.md](SECURITY.md) for responsible disclosure and [CONTRIBUTING.md](CONTRIBUTING.md) to contribute.

MIT License — see [LICENSE](LICENSE).

