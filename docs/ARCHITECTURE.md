# Hazler Architecture

This document describes the high-level structure of the Hazler codebase, the relationships between crates, and the data flow for a typical crawl.

---

## Workspace Crates

```
hazler (workspace)
├── hazler-cli          — Binary entry point; orchestrates all crates
├── hazler-core         — Crawler engine, queue, config, persistence, diffing
├── hazler-http         — HTTP client wrapper (reqwest), auth, stealth headers
├── hazler-parser       — HTML link extraction, GraphQL endpoint detection
├── hazler-js-parser    — JavaScript endpoint extraction, source map parsing, .frame files
├── hazler-secrets      — Secret/credential pattern matching and severity classification
├── hazler-browser      — Headless browser integration via chromiumoxide (CDP)
└── hazler-fuzzer       — URL mutation engine, parameter discovery, BOLA/IDOR detection
```

### Dependency Graph

```
hazler-cli
  ├── hazler-core (feature = "browser")
  │     ├── hazler-http
  │     ├── hazler-parser
  │     ├── hazler-js-parser
  │     ├── hazler-secrets
  │     └── hazler-browser  (optional, gated by "browser" feature)
  ├── hazler-http
  ├── hazler-parser
  └── hazler-fuzzer
```

---

## Crawl Data Flow

```
                         ┌─────────────────────────────┐
                         │          hazler-cli          │
                         │  parse args → build Config   │
                         └────────────┬────────────────┘
                                      │
                         ┌────────────▼────────────────┐
                         │         hazler-core          │
                         │   Crawler::new(config)       │
                         │   Crawler::crawl(start_url)  │
                         └──┬─────────────────────┬────┘
                            │                     │
               ┌────────────▼─────┐    ┌──────────▼──────────┐
               │  UrlQueue        │    │  tokio::Semaphore    │
               │  (BFS frontier)  │    │  (concurrency limit) │
               └────────────┬─────┘    └──────────┬──────────┘
                            │                     │
                         ┌──▼─────────────────────▼──┐
                         │   spawn_crawl_task(url)    │
                         │   ┌──────────────────────┐ │
                         │   │  hazler-http          │ │
                         │   │  HttpClient::get(url) │ │
                         │   └──────────┬───────────┘ │
                         │              │              │
                         │   ┌──────────▼───────────┐ │
                         │   │  NoiseFilter         │ │
                         │   │  (suppress WAF/404s) │ │
                         │   └──────────┬───────────┘ │
                         │              │              │
                         │   ┌──────────▼───────────┐ │
                         │   │  hazler-parser        │ │
                         │   │  HtmlParser           │ │
                         │   │  GraphQLParser        │ │
                         │   └──────────┬───────────┘ │
                         │              │              │
                         │   ┌──────────▼───────────┐ │
                         │   │  hazler-js-parser     │ │
                         │   │  JavaScriptParser     │ │
                         │   │  SourceMapParser      │ │
                         │   │  FrameFileParser      │ │
                         │   └──────────┬───────────┘ │
                         │              │              │
                         │   ┌──────────▼───────────┐ │
                         │   │  hazler-secrets       │ │
                         │   │  SecretScanner        │ │
                         │   └──────────┬───────────┘ │
                         │              │              │
                         │   ┌──────────▼───────────┐ │
                         │   │  ScopeValidator       │ │
                         │   │  AdvancedUrlNormalizer│ │
                         │   └──────────┬───────────┘ │
                         └─────────────┼──────────────┘
                                       │
                    ┌──────────────────▼────────────────────┐
                    │  Valid endpoint emitted via mpsc channel│
                    │  ValidEndpoint { url, status, ct }     │
                    └──────────────────┬────────────────────┘
                                       │
                    ┌──────────────────▼────────────────────┐
                    │     hazler-cli display task            │
                    │  prints URL in real time (clean mode)  │
                    └────────────────────────────────────────┘
```

---

## Key Types

### `hazler-core`

| Type | Module | Description |
|------|--------|-------------|
| `Config` | `config.rs` | All crawler settings; builder API |
| `Crawler` | `crawler.rs` | Main crawl orchestrator |
| `CrawlResult` | `types.rs` | Returned after crawl completes; holds all `Page` objects |
| `Page` | `types.rs` | A crawled page with URL, status, links, secrets |
| `ValidEndpoint` | `types.rs` | Real-time endpoint event sent via mpsc channel |
| `UrlQueue` | `queue.rs` | Thread-safe BFS queue with deduplication |
| `ScopeValidator` | `scope.rs` | Checks whether a URL is in-scope |
| `AdvancedUrlNormalizer` | `normalizer.rs` | Canonicalises URLs to prevent duplicate visits |
| `NoiseFilter` | `noise_filter.rs` | Detects and suppresses repetitive WAF/404 patterns |
| `StatePersistence` | `persistence.rs` | JSON or SQLite crawl-state save/load |
| `RetryConfig` | `retry.rs` | Exponential backoff with jitter |
| `CircuitBreaker` | `circuit_breaker.rs` | Per-domain failure isolation |
| `RateLimiter` | `rate_limiter.rs` | Token-bucket with adaptive 429 detection |
| `ResponseDiffer` | `differ/` | SimHash, K-means/DBSCAN clustering, baseline |
| `GracefulShutdown` | `shutdown.rs` | Ctrl+C handler that saves state |
| `ProgressTracker` | `progress.rs` | Real-time statistics |

### `hazler-http`

| Type | Module | Description |
|------|--------|-------------|
| `HttpClient` | `client.rs` | Wraps `reqwest`; adds stealth headers, auth, proxy |
| `AuthConfig` | `client.rs` | Authentication configuration |
| `AuthMethod` | `client.rs` | Enum: Basic, Bearer, ApiKey, Cookie, OAuth2 |

### `hazler-js-parser`

| Type | Module | Description |
|------|--------|-------------|
| `JavaScriptParser` | `parser.rs` | Regex-based endpoint extraction with confidence scoring |
| `SourceMapParser` | `sourcemap.rs` | Parses `*.js.map` files to extract original source paths |
| `FrameFileParser` | `parser.rs` | Parses `.frame` endpoint definition files |

### `hazler-fuzzer`

| Type | Module | Description |
|------|--------|-------------|
| `UrlMutator` | `mutator.rs` | Generates URL variants (plurals, extensions, versions) |
| `ParamDetector` | `params.rs` | Discovers query parameters from built-in wordlists |
| `FuzzerConfig` | `config.rs` | Fuzzing level and mode configuration |

---

## Real-Time Endpoint Channel

Valid endpoints are streamed to the CLI in real time via an unbounded tokio mpsc channel:

```rust
let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ValidEndpoint>();
let crawler = Crawler::new(config)?.with_endpoint_sender(tx);

// Spawned task reads from rx and prints each endpoint as it arrives
tokio::spawn(async move {
    while let Some(ep) = rx.recv().await {
        println!("{} {}", ep.status_code, ep.url);
    }
});

crawler.crawl(start_url).await?;
```

---

## Persistence Backends

Two backends are supported for crawl-state save/resume:

```rust
// JSON backend (default)
let persistence = StatePersistence::json(PathBuf::from("hazler-state.json"));

// SQLite backend
let persistence = StatePersistence::sqlite(PathBuf::from("hazler-state.db"));
```

See also: [docs/RETRY_PERSISTENCE.md](RETRY_PERSISTENCE.md) for in-depth retry and persistence documentation.

---

## Feature Flags

| Crate | Feature | Effect |
|-------|---------|--------|
| `hazler-core` | `browser` | Enables `hazler-browser` dependency for headless CDP crawling |
| `hazler-cli` | (always enabled) | Depends on `hazler-core` with `features = ["browser"]` |
