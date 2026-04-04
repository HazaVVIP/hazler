# Hazler CLI Reference

Complete reference for all command-line flags, organized by category.

## Synopsis

```
hazler [OPTIONS] <URL>
```

Use `-` as the URL to read targets from stdin (pipeline mode).

---

## Basic Crawl Control

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--max-depth` | `-d` | `3` | Maximum crawl depth from the starting URL |
| `--concurrency` | `-c` | `10` | Number of concurrent HTTP requests |
| `--max-pages` | `-p` | `0` (unlimited) | Maximum number of pages to crawl |
| `--timeout` | `-t` | `10` | Request timeout in seconds |
| `--scope` | | `same-domain` | Domain scope: `strict`, `same-domain`, `subdomains` |
| `--aggressive` | | off | Enable aggressive endpoint discovery (JS extraction, URL variations, API versioning) |
| `--all` | | off | Enable all analysis features (deep crawl, JS extraction, secrets, framework detection, GraphQL) |

---

## Output

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--output-format` | `-o` | `clean` | Output format: `clean`, `json`, `jsonl`, `csv`, `tree`, `urls`, `nuclei`, `ffuf`, `burp`, `openapi`, `postman` |
| `--full-output` | | off | Show full tree view with statistics after crawl completes |
| `--include-body` | | off | Include response body content in output (excluded by default for performance) |
| `--fields` | | all | Comma-separated list of fields to output: `url,status_code,depth,links` |
| `--no-color` | | off | Disable ANSI colours and emoji in terminal output |
| `--plain` | | off | Alias for `--no-color`; useful for log files and CI pipelines |
| `--quiet` | `-q` | off | Suppress the post-crawl summary on stderr |
| `--no-progress` | | off | Suppress the live progress indicator on stderr |
| `--verbose` | `-v` | off | Enable verbose/debug output |

### Output Format Examples

```bash
# Default: print verified 200-range URLs in real time
hazler https://example.com

# Full tree view after crawl
hazler https://example.com --full-output

# Machine-readable JSONL (one object per line)
hazler https://example.com -o jsonl

# Direct Nuclei input
hazler https://example.com -o nuclei | nuclei -t nuclei-templates/

# ffuf wordlist
hazler https://example.com -o ffuf | ffuf -w - -u FUZZ

# Burp Suite import
hazler https://example.com -o burp > targets.txt
```

---

## Export

Exports are written to files after the crawl completes. Multiple exports can be specified.

| Format | Flag | Example |
|--------|------|---------|
| Summary text | `--export summary:FILE` | `--export summary:report.txt` |
| HTML report | `--export html:FILE` | `--export html:report.html` |
| PDF report | `--export pdf:FILE` | `--export pdf:report.pdf` |
| SQLite database | `--export sqlite:FILE` | `--export sqlite:crawl.db` |
| OpenAPI spec | `--export openapi:FILE` | `--export openapi:spec.yaml` |
| Postman collection | `--export postman:FILE` | `--export postman:collection.json` |

```bash
# Multiple exports in one run
hazler https://example.com \
  --export html:report.html \
  --export sqlite:crawl.db \
  --export openapi:spec.yaml
```

---

## Webhook Notifications

| Flag | Description |
|------|-------------|
| `--webhook URL` | Send results to a webhook URL |
| `--webhook-type TYPE` | Override auto-detected type: `slack`, `discord`, `generic` |

Webhook type is auto-detected from the URL:
- Slack: `hooks.slack.com`
- Discord: `discord.com/api/webhooks`
- All others: `generic`

```bash
hazler https://example.com --webhook https://hooks.slack.com/services/T.../B.../...
hazler https://example.com --webhook https://example.com/hook --webhook-type generic
```

---

## Stealth & WAF Evasion

| Flag | Default | Description |
|------|---------|-------------|
| `--no-stealth` | stealth ON | Disable stealth mode (user-agent rotation, Chrome client hints, timing jitter) |
| `--proxy URL` | none | Proxy URL, e.g. `socks5://localhost:1080` or `http://proxy:8080` |
| `--user-agent` | `-u` | `Hazler/<version>` | Custom User-Agent string |

---

## Secret Scanning

| Flag | Default | Description |
|------|---------|-------------|
| `--no-secrets` | secrets ON | Disable automatic secret and credential scanning |

When enabled, Hazler scans all page content for 38+ secret patterns (API keys, tokens, private keys, database strings, etc.) and classifies findings by severity (Critical / High / Medium / Low).

---

## Headless Browser

| Flag | Default | Description |
|------|---------|-------------|
| `--browser` | off | Use Chrome/Chromium headless browser to render pages (requires Chrome) |
| `--screenshot-path DIR` | none | Save screenshots of crawled pages to the given directory |
| `--disable-images` | images ON | Disable image loading in the browser (faster) |

```bash
# Crawl a React SPA
hazler https://app.example.com --browser

# With screenshots
hazler https://app.example.com --browser --screenshot-path screenshots/
```

---

## GraphQL

| Flag | Default | Description |
|------|---------|-------------|
| `--graphql-introspect` | off | Run introspection queries on detected GraphQL endpoints |

---

## Source Maps

| Flag | Default | Description |
|------|---------|-------------|
| `--no-source-maps` | parsing ON | Disable source map detection and path extraction |

---

## Fuzzing

| Flag | Default | Description |
|------|---------|-------------|
| `--fuzz` | off | Enable smart URL fuzzing |
| `--fuzz-level LEVEL` | `default` | Fuzzing depth: `off`, `minimal`, `default`, `aggressive`, `full` |
| `--fuzz-crawl` | off | Fetch each fuzzed URL and merge results into the crawl |
| `--fuzz-output FILE` | none | Write fuzzed URLs to a file (one per line) |

Fuzz levels:
- `minimal` — trailing-slash and simple extension variants
- `default` — URL mutations (pluralisation, versioning)
- `aggressive` — mutations + parameter discovery
- `full` — all of the above + endpoint wordlist

```bash
# Generate fuzzed URLs and write them to a file
hazler https://example.com --fuzz --fuzz-level aggressive --fuzz-output fuzzed.txt

# Fuzz and crawl all variants
hazler https://example.com --fuzz --fuzz-level full --fuzz-crawl
```

---

## Response Diffing & Baseline

| Flag | Default | Description |
|------|---------|-------------|
| `--baseline FILE` | none | Save normalised response hashes to a JSON file |
| `--compare FILE` | none | Compare current responses against a saved baseline |
| `--diff-threshold N` | `0.85` | Similarity threshold (0.0–1.0) for detecting changes |
| `--cluster MODE` | `off` | Response clustering: `off`, `auto`, `kmeans:N`, `dbscan:epsilon,minpts` |

```bash
# Save baseline
hazler https://example.com --baseline baseline.json

# Compare on next run
hazler https://example.com --compare baseline.json

# Cluster responses with K-means
hazler https://example.com --cluster kmeans:10
```

---

## Authentication

### Inline Authentication

| Flag | Example |
|------|---------|
| `--auth basic:USER:PASS` | `--auth basic:admin:s3cret` |
| `--auth bearer:TOKEN` | `--auth bearer:eyJhbG...` |
| `--auth apikey:KEY` | `--auth apikey:my-api-key-123` |
| `--auth cookie:NAME=VALUE` | `--auth cookie:session=abc123` |

### File-based Authentication

```bash
hazler https://example.com --auth-file credentials.json
```

Example `credentials.json` (Bearer):
```json
{
  "method": "bearer",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

See the `examples/` directory for auth configuration templates (`auth-basic.json`, `auth-bearer.json`, `auth-cookie.json`, `auth-apikey.json`, `auth-oauth2.json`).

---

## State Persistence & Resume

| Flag | Default | Description |
|------|---------|-------------|
| `--auto-save N` | `60` | Save crawl state every N seconds (0 = disabled) |
| `--resume FILE` | none | Resume crawl from a saved state file |
| `--persist-sqlite` | off | Use SQLite backend instead of JSON for state files |

```bash
# Start a long crawl with auto-save
hazler https://example.com -d 5 --auto-save 30

# Resume after interruption
hazler https://example.com --resume hazler-state.json
```

---

## Reliability & Rate Control

| Flag | Default | Description |
|------|---------|-------------|
| `--max-retries N` | `3` | Maximum retry attempts per request (exponential backoff) |
| `--circuit-breaker` | off | Enable per-domain circuit breaker (stops requests to repeatedly failing domains) |
| `--rate-limit N` | `10` | Maximum requests per second per domain |
| `--progress N` | `5` | Progress reporting interval in seconds |

---

## JavaScript Confidence

| Flag | Default | Description |
|------|---------|-------------|
| `--js-confidence N` | `0.5` | Minimum confidence score (0.0–1.0) for JS-extracted endpoints |

Endpoints extracted from JavaScript by regex patterns are assigned a reliability score. Higher values mean fewer but more accurate endpoints; lower values discover more endpoints at the cost of false positives.

---

## Interactive Wizard

```bash
hazler --wizard
```

Guides you through URL, depth, pages, secrets, and output format interactively. Ideal for first-time users.

---

## Pipeline Mode

Use `-` as the URL to read from stdin:

```bash
# Crawl all URLs from a file
cat targets.txt | hazler - -o urls

# Subdomain enumeration pipeline
subfinder -d example.com | httpx -silent | hazler - --aggressive -o urls
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `RUST_LOG` | Set log level: `error`, `warn`, `info`, `debug`, `trace` |
| `NO_COLOR` | Disable ANSI color output (equivalent to `--no-color`) |
