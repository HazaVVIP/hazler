# Hazler - Next-Generation Intelligent Web Crawler

A fast, efficient, and human-friendly web crawler built in Rust with built-in security features.

## ✨ Key Features

- ✅ **Human-Friendly Output** - Beautiful tree view with colors and clear formatting (default)
- ✅ **Stealth Mode** - WAF evasion enabled by default for better success rates
- ✅ **Secret Scanning** - Automatic detection of API keys, tokens, and credentials (enabled by default)
- ✅ HTTP-only crawling with concurrent request handling
- ✅ HTML parsing and link extraction
- ✅ **JavaScript endpoint discovery** with regex-based extraction
- ✅ **Advanced URL normalization** for better endpoint discovery
- ✅ **Aggressive discovery mode** for security reconnaissance
- ✅ **.frame file support** for endpoint extraction
- ✅ Scope validation (stays within domain)
- ✅ Depth control and multiple output formats
- ✅ Command-line interface with flexible options

## Prerequisites

Before installing Hazler, ensure you have the following dependencies installed:

### Ubuntu/Debian

```bash
sudo apt update && sudo apt install -y build-essential pkg-config libssl-dev
```

### Fedora/RHEL/CentOS

```bash
sudo dnf install -y gcc pkg-config openssl-devel
```

### macOS

```bash
# OpenSSL is typically pre-installed
# If needed, install via Homebrew:
brew install openssl@3
```

### Windows

1. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
2. Install OpenSSL from [Win32OpenSSL](https://slproweb.com/products/Win32OpenSSL.html)

### Rust

Hazler requires Rust 1.70 or later. Install Rust via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify installation:

```bash
rustc --version
cargo --version
```

## Installation

### Quick Install (Recommended)

Use the automated installation script:

```bash
curl -sSf https://raw.githubusercontent.com/HazaVVIP/hazler/main/install.sh | bash
```

Or download and run manually:

```bash
wget https://raw.githubusercontent.com/HazaVVIP/hazler/main/install.sh
chmod +x install.sh
./install.sh
```

### Download Pre-built Binaries

Download the latest release for your platform from the [releases page](https://github.com/HazaVVIP/hazler/releases):

- **Linux (x86_64):** `hazler-linux-x86_64.tar.gz`
- **Linux (aarch64):** `hazler-linux-aarch64.tar.gz`
- **macOS (Intel):** `hazler-macos-x86_64.tar.gz`
- **macOS (Apple Silicon):** `hazler-macos-aarch64.tar.gz`
- **Windows:** `hazler-windows-x86_64.exe.zip`

Extract and verify:

```bash
# Linux/macOS
tar xzf hazler-*.tar.gz
./hazler --version

# Optionally, move to system path
sudo mv hazler /usr/local/bin/
```

### Docker

Run Hazler in a Docker container:

```bash
# Pull the image
docker pull ghcr.io/hazavvip/hazler:latest

# Run a crawl
docker run --rm ghcr.io/hazavvip/hazler:latest https://example.com

# Save output to file
docker run --rm ghcr.io/hazavvip/hazler:latest https://example.com > results.jsonl

# With custom options
docker run --rm ghcr.io/hazavvip/hazler:latest https://example.com -d 2 -c 5 -o json
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/HazaVVIP/hazler.git
cd hazler

# Build in release mode
cargo build --release

# The binary will be located at target/release/hazler
# Optionally, install to system path
cargo install --path crates/hazler-cli
```

### Verify Installation

```bash
hazler --version
```

## Quick Start

### Your First Crawl

Start with a simple crawl of a website:

```bash
hazler https://example.com
```

This will crawl `example.com` with:
- **Human-friendly tree output** with colors and status indicators
- **Stealth mode enabled** for better success rates and WAF evasion
- **Secret scanning enabled** to detect sensitive data leaks
- Default depth of 3 and concurrency of 10

The output will show a beautiful tree view like:
```
🌐 HAZLER CRAWL RESULTS
✓ [200] https://example.com/ (15 links)
  ✓ [200] https://example.com/about (5 links)
  ✓ [200] https://example.com/contact (3 links)
```

### Common Use Cases

#### Comprehensive Security Scan (Recommended)
Perform a full security reconnaissance with all features enabled:
```bash
hazler https://yoursite.com --all
```

This activates:
- Deep crawling (depth 5)
- Aggressive endpoint discovery
- Secret and credential detection
- Framework detection (React, Angular, Vue, etc.)
- API endpoint mapping
- Comprehensive security report

#### Site Audit with Statistics
Crawl your entire site and get detailed statistics:
```bash
hazler https://yoursite.com -d 5 -p 1000 --stats
```

#### Security Audit with HTML Report
Perform a comprehensive security audit and generate an HTML report:
```bash
hazler https://yoursite.com --all --html-report report.html
```

#### Quick Link Check
Check links on a specific page (depth 1):
```bash
hazler https://yoursite.com -d 1 -c 5
```

#### Large Site Crawl with JSON Output
Crawl a large site and save machine-readable output:
```bash
hazler https://example.com -d 4 -c 20 -p 5000 -o json > results.json
```

#### Disable Stealth/Secrets for Speed
If you need faster crawling and don't need stealth or secret scanning:
```bash
hazler https://example.com --no-stealth --no-secrets
```

## Usage

### Basic usage

Crawl a website:

```bash
hazler https://example.com
```

### Advanced options

```bash
hazler [OPTIONS] <URL>

Arguments:
  <URL>  Target URL to crawl

Options:
  -d, --max-depth <MAX_DEPTH>          Maximum crawl depth [default: 3]
  -c, --concurrency <CONCURRENCY>      Number of concurrent requests [default: 10]
  -p, --max-pages <MAX_PAGES>          Maximum number of pages to crawl (0 = unlimited) [default: 0]
  -u, --user-agent <USER_AGENT>        Custom user agent string [default: Hazler/0.1.0]
  -t, --timeout <TIMEOUT>              Request timeout in seconds [default: 10]
  -o, --output-format <OUTPUT_FORMAT>  Output format (json, jsonl, urls, csv, or tree) [default: tree]
      --include-body                   Include response body in output (excluded by default)
      --fields <FIELDS>                Select specific fields to output (comma-separated)
      --aggressive                     Enable aggressive endpoint discovery mode
      --all                            Enable comprehensive scanning mode (deep crawl + secrets + framework detection)
      --stats                          Show crawl statistics with distributions
      --report                         Generate comprehensive summary report
      --html-report <FILE>             Generate HTML report and save to file
      --no-stealth                     Disable stealth mode (enabled by default)
      --no-secrets                     Disable secret scanning (enabled by default)
      --proxy <PROXY>                  Proxy URL (e.g., socks5://localhost:1080, http://proxy:8080)
      --strict-domain                  Only crawl the exact domain (no subdomains)
      --subs                           Allow crawling subdomains
  -v, --verbose                        Verbose output
  -h, --help                           Print help
  -V, --version                        Print version
```

### Examples

Basic crawl with human-friendly output (default):
```bash
hazler https://example.com
```

Comprehensive scan with all features enabled:
```bash
hazler https://example.com --all
```

Crawl with custom depth and concurrency:
```bash
hazler https://example.com -d 2 -c 5
```

Limit to 100 pages:
```bash
hazler https://example.com -p 100
```

Get detailed statistics:
```bash
hazler https://example.com --stats
```

Generate comprehensive report with security findings:
```bash
hazler https://example.com --report
```

Generate HTML report:
```bash
hazler https://example.com --html-report report.html
```

Output as single JSON object for processing:
```bash
hazler https://example.com -o json > results.json
```

Output as JSONL (one JSON object per line):
```bash
hazler https://example.com -o jsonl > results.jsonl
```

Output as URL list:
```bash
hazler https://example.com -o urls > urls.txt
```

Output as CSV:
```bash
hazler https://example.com -o csv > results.csv
```

Disable stealth and secrets for faster crawling:
```bash
hazler https://example.com --no-stealth --no-secrets
```

Use a proxy for requests:
```bash
hazler https://example.com --proxy socks5://localhost:1080
```

Crawl only the exact domain (no subdomains):
```bash
hazler https://example.com --strict-domain
```

Allow crawling subdomains:
```bash
hazler https://example.com --subs
```

Include body content (excluded by default):
```bash
hazler https://example.com --include-body
```

Select specific fields:
```bash
hazler https://example.com --fields url,status_code,depth -o jsonl
```
```bash
hazler https://example.com --report
```

Verbose logging:
```bash
hazler https://example.com -v
```

## Security Reconnaissance Features

Hazler has been enhanced with powerful security reconnaissance capabilities for bug hunting and penetration testing:

### Secret & Credential Detection

Hazler automatically scans all crawled content for sensitive information (enabled by default):

```bash
# Crawl with secret detection (default)
hazler https://target.com

# View secrets in comprehensive report
hazler https://target.com --report
```

**Detects 38+ types of secrets including:**

**Critical Severity:**
- AWS Access Keys and Secret Keys
- GitHub Personal Access Tokens and OAuth Tokens
- Stripe Live Secret Keys
- Google Cloud Service Account credentials
- Private Keys (RSA, SSH, PGP, DSA)
- Database connection strings

**High Severity:**
- Generic API keys and tokens
- Slack tokens and webhooks
- Azure Storage keys
- SendGrid and Mailgun API keys
- Google API Keys
- JWT tokens

**Medium Severity:**
- Internal IP addresses (10.x.x.x, 192.168.x.x, 172.16.x.x)
- OAuth Client IDs and Secrets
- NPM and PyPI tokens

**Low Severity:**
- Email addresses
- Configuration file references (.env, config.json)

Secrets are automatically redacted in output and classified by severity to help prioritize remediation.

### JavaScript Endpoint Discovery

Hazler automatically extracts endpoints from JavaScript files using advanced regex patterns:

```bash
# Crawl and extract JavaScript endpoints
hazler https://target.com --aggressive
```

Supports extraction from:
- **Fetch API calls**: `fetch('/api/users')`
- **XMLHttpRequest**: `.open('GET', '/api/data')`
- **Axios**: `axios.get('/api/posts')`
- **jQuery AJAX**: `$.ajax({url: '/api/items'})`
- **API definitions**: `const endpoint = '/api/v1/users'`
- **Template literals**: `` `/api/${userId}` ``
- **Router configs**: `path: '/admin/dashboard'`
- **GraphQL endpoints**: `graphql: '/graphql'`
- **WebSocket endpoints**: `wss://example.com/socket`

### Framework Detection

Hazler detects modern web frameworks to apply specialized extraction patterns:

**Supported Frameworks:**
- React (including React Router)
- Angular (including routing)
- Vue.js (including Vue Router)
- Next.js (including API routes)
- Nuxt
- Svelte
- Ember
- Backbone

### Aggressive Discovery Mode

Enable comprehensive endpoint discovery with the `--aggressive` flag:

```bash
hazler https://target.com --aggressive -d 3
```

In aggressive mode, Hazler:
- ✅ Applies regex patterns to JavaScript embedded in HTML
- ✅ Generates URL variations (with/without trailing slashes)
- ✅ Tests common file extensions (.json, .xml, .html, .txt)
- ✅ Discovers API version variants (v1, v2, v3)
- ✅ Tests different format parameters (?format=json, ?format=xml)
- ✅ Extracts endpoints from .frame files

### Advanced URL Normalization

Hazler uses intelligent URL normalization to:
- Remove duplicate URLs with different query parameter orders
- Canonicalize URLs for proper deduplication
- Generate endpoint variations for thorough testing
- Handle template variables in URLs (`${id}` → `0`, `{userId}` → `1`)

### Example: Security Audit

Perform a comprehensive security audit of a target:

```bash
# Deep crawl with aggressive discovery
hazler https://target.com \
  --aggressive \
  -d 5 \
  -c 20 \
  -p 10000 \
  --fields url,status_code,content_type \
  > endpoints.jsonl

# Extract just the URLs for further testing
hazler https://target.com --aggressive -o urls > urls.txt
```

### Example: Find API Endpoints

Discover hidden API endpoints:

```bash
# Focus on API discovery
hazler https://api.target.com --aggressive --fields url,links -o json | \
  jq '.pages[] | select(.url | contains("api")) | .url'
```

## Output Formats

Hazler supports multiple output formats to suit different use cases:

### Tree (default)
Human-friendly tree structure with colors showing site hierarchy:
```
🌐 HAZLER CRAWL RESULTS
════════════════════════════════════════════════════════════════════════════════

✓ [200] https://example.com/ (10 links)
  ✓ [200] https://example.com/page1 (5 links)
    ✓ [200] https://example.com/page1/sub (2 links)
  ✓ [200] https://example.com/page2 (3 links)
  ↻ [301] https://example.com/old (0 links)
  ✗ [404] https://example.com/missing (0 links)

════════════════════════════════════════════════════════════════════════════════
```

Features:
- ✓ Color-coded status indicators (green=success, yellow=redirect, red=error)
- Shows link count for each page
- Displays secrets found (if any)
- Visual hierarchy based on crawl depth

### JSONL
Each line is a JSON object representing a crawled page:
```json
{"url":"https://example.com/","status_code":200,"body":"...","headers":{...},"content_type":"text/html","links":[...],"depth":0}
{"url":"https://example.com/page1","status_code":200,"body":"...","headers":{...},"content_type":"text/html","links":[...],"depth":1}
```

### JSON
Single JSON object with all results:
```json
{
  "pages": [...],
  "total_pages": 10,
  "total_urls": 25,
  "errors": [],
  "secret_findings": {
    "total": 5,
    "critical": 2,
    "high": 1,
    "medium": 2,
    "low": 0
  }
}
```

### URLs
Simple list of URLs (one per line):
```
https://example.com/
https://example.com/page1
https://example.com/page2
```

### CSV
Comma-separated values with headers:
```csv
url,status_code,depth,content_type,num_links
"https://example.com/",200,0,"text/html",10
"https://example.com/page1",200,1,"text/html",5
```

## Output Processing Examples

### Using with jq

Extract URLs and status codes:
```bash
hazler https://yoursite.com -o json | jq -r '.pages[] | "\(.url) → \(.status_code)"'
```

Find all 404 errors:
```bash
hazler https://yoursite.com -o json | jq '.pages[] | select(.status_code == 404) | .url'
```

Create a simple sitemap:
```bash
hazler https://yoursite.com -o json | jq -r '.pages[].url' | sort > sitemap.txt
```

Count pages by depth:
```bash
hazler https://yoursite.com -o json | jq '.pages | group_by(.depth) | map({depth: .[0].depth, count: length})'
```

## Troubleshooting

### Build Errors

#### OpenSSL Not Found

**Error:**
```
error: failed to run custom build command for `openssl-sys v0.9.x`
Could not find directory of OpenSSL installation
```

**Solution:**
Install OpenSSL development libraries:
- **Ubuntu/Debian:** `sudo apt install -y pkg-config libssl-dev`
- **Fedora/RHEL:** `sudo dnf install -y pkg-config openssl-devel`
- **macOS:** `brew install openssl@3` (if not already installed)
- **Windows:** Install from [Win32OpenSSL](https://slproweb.com/products/Win32OpenSSL.html)

#### pkg-config Not Found

**Error:**
```
error: failed to run custom build command for `openssl-sys v0.9.x`
Perhaps you need to install pkg-config?
```

**Solution:**
- **Ubuntu/Debian:** `sudo apt install -y pkg-config`
- **Fedora/RHEL:** `sudo dnf install -y pkg-config`
- **macOS:** `brew install pkg-config`

#### Rust Version Too Old

**Error:**
```
error: package requires rustc 1.70 or newer
```

**Solution:**
Update Rust to the latest version:
```bash
rustup update stable
```

### Runtime Issues

#### Command Not Found

**Error:**
```
hazler: command not found
```

**Solution:**
1. If you built from source, use the full path: `./target/release/hazler`
2. Or install to system path: `cargo install --path crates/hazler-cli`
3. Ensure `~/.cargo/bin` is in your PATH:
   ```bash
   echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

#### Connection Timeouts

If you're experiencing connection timeouts, increase the timeout value:
```bash
hazler https://example.com -t 30
```

#### Memory Issues

For very large crawls, limit the number of pages:
```bash
hazler https://example.com -p 10000
```

Or reduce concurrency:
```bash
hazler https://example.com -c 5
```

### Getting Help

If you encounter issues not covered here:

1. Check [GitHub Issues](https://github.com/HazaVVIP/hazler/issues)
2. Search existing issues for similar problems
3. Create a new issue with:
   - Your OS and version
   - Rust version (`rustc --version`)
   - Full error message
   - Steps to reproduce

## Project Structure

```
hazler/
├── Cargo.toml                  # Root workspace manifest
├── README.md                   # This file
├── CONTRIBUTING.md             # Contribution guidelines
├── LICENSE                     # MIT License
├── install.sh                  # Automated installation script
├── Dockerfile                  # Docker image configuration
├── crates/
│   ├── hazler-core/           # Core crawling engine
│   ├── hazler-http/           # HTTP client wrapper
│   ├── hazler-parser/         # HTML parsing
│   ├── hazler-js-parser/      # JavaScript endpoint extraction
│   ├── hazler-secrets/        # Secret & credential detection
│   └── hazler-cli/            # Command-line interface
```

## Development

### Running tests

```bash
cargo test
```

### Running with debug logs

```bash
RUST_LOG=debug cargo run -- https://example.com
```

### eBPF/bpftrace Debugging 🔍

Hazler includes advanced eBPF-based monitoring scripts for deep system-level debugging and performance analysis:

```bash
# Monitor network activity
sudo ./scripts/bpftrace/hazler-trace.sh network hazler https://example.com

# Profile performance
sudo ./scripts/bpftrace/hazler-trace.sh perf hazler https://example.com -d 3

# Security monitoring
sudo ./scripts/bpftrace/hazler-trace.sh security hazler https://target.com

# HTTP tracing
sudo ./scripts/bpftrace/hazler-trace.sh http hazler https://api.example.com
```

**Features:**
- 🌐 Network connection tracking (TCP, DNS, TLS)
- ⚡ Performance profiling (CPU, memory, I/O)
- 🛡️ Security monitoring (suspicious patterns, file access)
- 📊 HTTP request/response tracing
- 📈 Real-time statistics and histograms

See [scripts/bpftrace/README.md](scripts/bpftrace/README.md) for detailed documentation.

**Requirements:** Linux with bpftrace installed (`sudo apt install bpftrace`)

## Roadmap

### Phase 1: MVP ✅
- Basic HTTP crawler ✅
- HTML parsing ✅
- Concurrent crawling ✅
- CLI interface ✅
- Multiple output formats (JSON, JSONL, CSV, Tree, URLs) ✅

### Phase 2: Security Intelligence ✅
- **JavaScript endpoint extraction** ✅
- **Advanced URL normalization** ✅
- **Aggressive discovery mode** ✅
- **Framework detection** ✅ (React, Angular, Vue, Next.js, etc.)
- **Secret scanning** ✅ (38+ patterns for credentials, keys, tokens)
- **.frame file support** ✅
- **Regex-based pattern matching** ✅
- **Template variable replacement** ✅
- **Comprehensive reporting** ✅
- **HTML report generation** ✅

### Phase 3: Enhanced Stealth & Scale (In Progress)
- Full WAF evasion implementation
- Proxy support implementation
- Advanced rate limiting
- Priority queue with scoring
- Content similarity detection (SimHash)
- **Headless browser support** ✅
- **eBPF/bpftrace debugging** ✅
- Distributed crawling (Redis)
- OpenTelemetry integration
- Dashboard

### Phase 4: Polish (Planned)
- robots.txt respect
- Binary releases for all platforms
- Advanced authentication support
- Resume capability for interrupted crawls
- Plugin system for extensibility

## FAQ

### How fast is Hazler?

Hazler can crawl 100+ pages per second with appropriate concurrency settings (e.g., `-c 20`), depending on your network and target server capabilities.

### Does Hazler respect robots.txt?

Not yet. This is planned for a future release. Use responsibly and only crawl sites you have permission to access.

### Can I crawl JavaScript-heavy sites?

Yes! Hazler now includes:
- JavaScript endpoint discovery that extracts API endpoints from JavaScript code
- Framework detection (React, Angular, Vue, Next.js, etc.)
- Specialized extraction patterns for each framework

Use `--aggressive` mode or `--all` mode for the most thorough discovery.

### What is the --all mode?

The `--all` flag enables comprehensive scanning mode, which:
- Increases crawl depth from 3 to 5 (if using default)
- Enables aggressive endpoint discovery
- Activates secret and credential scanning
- Enables framework detection
- Provides comprehensive security reporting

This is the recommended mode for security audits and bug bounty reconnaissance.

### What is aggressive mode?

Aggressive mode (`--aggressive` flag) enables comprehensive endpoint discovery by:
- Extracting endpoints from JavaScript code
- Generating URL variations (trailing slashes, extensions)
- Testing API version variants (v1, v2, v3)
- Parsing .frame files for endpoint definitions
- Applying framework-specific extraction patterns

This is particularly useful for security reconnaissance and bug hunting.

### What types of secrets can Hazler detect?

Hazler detects 38+ types of secrets including:
- AWS keys, GitHub tokens, Stripe keys
- API keys and authentication tokens
- Private keys (RSA, SSH, PGP)
- Database connection strings
- Internal IP addresses and emails

All secrets are classified by severity (Critical, High, Medium, Low) and redacted in output.

### Does Hazler work with .frame files?

Yes! Hazler automatically detects and parses .frame files to extract endpoint definitions.

### How do I limit crawling to specific paths?

Currently, Hazler crawls all pages within the same domain. URL filtering is planned for a future release. As a workaround, you can filter the output with `jq`:
```bash
hazler https://example.com -o json | jq '.pages[] | select(.url | contains("/blog/"))'
```

### Does Hazler store crawl data?

No, Hazler outputs all data to stdout. You can redirect output to a file:
```bash
hazler https://example.com > crawl-results.jsonl
```

### Can I resume an interrupted crawl?

Not yet. Crawl state persistence is planned for Phase 4.

### How do I crawl multiple domains?

Currently, Hazler is designed for single-domain crawls. Run multiple instances for different domains:
```bash
hazler https://site1.com > site1.jsonl &
hazler https://site2.com > site2.jsonl &
wait
```

## Performance Tips

- **Start small:** Test with `-d 1 -p 10` first
- **Increase gradually:** Slowly increase `-c` (concurrency) and `-d` (depth)
- **Monitor resources:** Watch CPU and memory usage
- **Respect servers:** Don't overwhelm target servers; consider `-c 5` for smaller sites
- **Use filters:** Process output with `jq` or similar tools to reduce data size
- **Use aggressive mode wisely:** `--aggressive` generates more requests; use on targets you're authorized to test
- **Exclude body by default:** Body content is excluded by default for performance; use `--include-body` only when needed
- **Disable features for speed:** If you don't need stealth or secret scanning, use `--no-stealth --no-secrets` for faster crawling
- **Use machine-readable formats:** For large crawls, use `-o jsonl` or `-o json` instead of tree format to save on terminal rendering

## New Default Behavior

**Hazler now defaults to human-friendly behavior:**
- 🎨 **Tree output format** - Beautiful, colored tree view (instead of JSONL)
- 🕵️ **Stealth mode enabled** - Better success rates with WAF evasion
- 🔒 **Secret scanning enabled** - Automatic detection of sensitive data leaks

You can disable these features if needed:
```bash
# Traditional machine-readable output
hazler https://example.com -o jsonl

# Disable stealth and secrets for maximum speed
hazler https://example.com --no-stealth --no-secrets
```

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Acknowledgments

Built with:
- [Tokio](https://tokio.rs/) - Async runtime
- [Reqwest](https://github.com/seanmonstar/reqwest) - HTTP client
- [Scraper](https://github.com/causal-agent/scraper) - HTML parsing
- [Clap](https://github.com/clap-rs/clap) - CLI framework
