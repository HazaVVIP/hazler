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

## Installation

### Build from source

```bash
cargo build --release
```

The binary will be located at `target/release/hazler`.

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
  -o, --output-format <OUTPUT_FORMAT>  Output format (json or jsonl) [default: jsonl]
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

Verbose logging:
```bash
hazler https://example.com -v
```

## Output Formats

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

## Project Structure

```
hazler/
├── Cargo.toml                  # Root workspace manifest
├── README.md                   # This file
├── LICENSE                     # MIT License
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

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
