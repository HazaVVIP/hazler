# Hazler - Next-Generation Intelligent Web Crawler

A fast, efficient web crawler built in Rust.

## Features (Phase 1 MVP)

- ✅ HTTP-only crawling
- ✅ Concurrent request handling with configurable concurrency
- ✅ HTML parsing and link extraction
- ✅ Scope validation (stays within domain)
- ✅ Depth control
- ✅ JSONL output format
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

This will crawl `example.com` with default settings (depth: 3, concurrency: 10) and output results in JSONL format.

### Common Use Cases

#### Site Audit
Crawl your entire site to discover all pages:
```bash
hazler https://yoursite.com -d 5 -p 1000 -o json > site-audit.json
```

#### Quick Link Check
Check links on a specific page (depth 1):
```bash
hazler https://yoursite.com -d 1 -c 5
```

#### Large Site Crawl
Crawl a large site with high concurrency:
```bash
hazler https://example.com -d 4 -c 20 -p 5000
```

#### Custom User Agent
Use a custom user agent string:
```bash
hazler https://example.com -u "MyBot/1.0 (compatible; +https://mysite.com)"
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
  -o, --output-format <OUTPUT_FORMAT>  Output format (json, jsonl, urls, csv, or tree) [default: jsonl]
      --include-body                   Include response body in output (excluded by default to prevent flooding)
      --fields <FIELDS>                Select specific fields to output (comma-separated)
      --stats                          Show crawl statistics
      --report                         Generate summary report
  -v, --verbose                        Verbose output
  -h, --help                           Print help
  -V, --version                        Print version
```

### Examples

Crawl with custom depth and concurrency:
```bash
hazler https://example.com -d 2 -c 5
```

Limit to 100 pages:
```bash
hazler https://example.com -p 100
```

Output as single JSON object:
```bash
hazler https://example.com -o json
```

Output as URL list:
```bash
hazler https://example.com -o urls
```

Output as CSV:
```bash
hazler https://example.com -o csv > results.csv
```

Output as tree structure:
```bash
hazler https://example.com -o tree
```

Include body content in output (body is excluded by default):
```bash
hazler https://example.com --include-body
```

Select specific fields:
```bash
hazler https://example.com --fields url,status_code,depth
```

Show statistics:
```bash
hazler https://example.com --stats
```

Generate full report:
```bash
hazler https://example.com --report
```

Verbose logging:
```bash
hazler https://example.com -v
```

## Output Formats

Hazler supports multiple output formats to suit different use cases:

### JSONL (default)
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
  "errors": []
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

### Tree
Visual tree structure showing site hierarchy:
```
✓ [200] https://example.com/ (10 links)
  ✓ [200] https://example.com/page1 (5 links)
    ✓ [200] https://example.com/page1/sub (2 links)
  ✓ [200] https://example.com/page2 (3 links)
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
├── LICENSE                     # MIT License
├── install.sh                  # Automated installation script
├── crates/
│   ├── hazler-core/           # Core crawling engine
│   ├── hazler-http/           # HTTP client wrapper
│   ├── hazler-parser/         # HTML parsing
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

## Roadmap

### Phase 1: MVP (Current) ✅
- Basic HTTP crawler
- HTML parsing
- Concurrent crawling
- CLI interface
- JSONL output

### Phase 2: Intelligence (Planned)
- Priority queue with scoring
- URL pattern detection
- Content similarity detection (SimHash)
- Headless browser support
- JavaScript endpoint extraction

### Phase 3: Scale (Planned)
- Distributed crawling (Redis)
- Advanced SPA handling
- OpenTelemetry integration
- Dashboard
- Multiple output formats (HAR, SQLite, GraphML)

### Phase 4: Polish (Planned)
- Comprehensive documentation
- Binary releases
- Docker images
- Security audit

## FAQ

### How fast is Hazler?

Hazler can crawl 100+ pages per second with appropriate concurrency settings (e.g., `-c 20`), depending on your network and target server capabilities.

### Does Hazler respect robots.txt?

Not yet. This is planned for a future release. Use responsibly and only crawl sites you have permission to access.

### Can I crawl JavaScript-heavy sites?

Currently, Hazler only processes static HTML. Support for JavaScript rendering via headless browsers is planned for Phase 2.

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
