# Technical Implementation Recommendations
## Detailed Enhancement Specifications for Hazler Crates

This document provides **specific, actionable technical recommendations** for each crate with code examples, architecture decisions, and implementation guidance.

---

## 1. hazler-core Enhancements

### 1.1 Response Diffing Engine (HIGH PRIORITY)

**Problem:** Cannot detect when pages change over time, missing dynamic content patterns.

**Solution:** Implement SimHash-based response comparison.

**Architecture:**
```rust
// New module: crates/hazler-core/src/differ.rs

pub struct ResponseDiffer {
    baseline: HashMap<Url, SimHash>,
    threshold: f64, // 0.0-1.0, similarity threshold
}

impl ResponseDiffer {
    pub fn new(threshold: f64) -> Self { ... }
    
    pub fn calculate_simhash(content: &str) -> SimHash { ... }
    
    pub fn compare(&self, url: &Url, content: &str) -> DiffResult {
        // Calculate simhash for new content
        // Compare with baseline
        // Return similarity percentage and changes
    }
    
    pub fn set_baseline(&mut self, url: Url, content: &str) { ... }
}

pub struct DiffResult {
    pub similarity: f64,
    pub changed: bool,
    pub added_content: Vec<String>,
    pub removed_content: Vec<String>,
}
```

**Dependencies to add:**
```toml
simhash = "0.4"  # SimHash implementation
diff = "0.1"     # Text diffing
```

**Implementation Steps:**
1. Create `differ.rs` module
2. Integrate SimHash algorithm (use existing crate or implement)
3. Add baseline storage (in-memory HashMap for now)
4. Add diff calculation logic
5. Expose through `Crawler` API
6. Add CLI flag `--diff-baseline <file.json>`

**Tests to add:**
- Test identical content (similarity = 1.0)
- Test completely different content (similarity = 0.0)
- Test minor changes (similarity > 0.9)
- Test large changes (similarity < 0.5)

---

### 1.2 Smart Retry with Circuit Breaker (HIGH PRIORITY)

**Problem:** Gets rate-limited or banned, no intelligent retry strategy.

**Solution:** Implement exponential backoff with circuit breaker pattern.

**Architecture:**
```rust
// New module: crates/hazler-core/src/retry.rs

use std::time::Duration;
use tokio::time::sleep;

pub struct RetryPolicy {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
    jitter: bool,
}

impl RetryPolicy {
    pub fn exponential_backoff() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            jitter: true,
        }
    }
    
    pub async fn execute<F, T, E>(&self, mut f: F) -> Result<T, E>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    {
        for attempt in 0..self.max_attempts {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) if attempt == self.max_attempts - 1 => return Err(e),
                Err(_) => {
                    let delay = self.calculate_delay(attempt);
                    sleep(delay).await;
                }
            }
        }
        unreachable!()
    }
    
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let mut delay = self.base_delay * 2_u32.pow(attempt);
        if delay > self.max_delay {
            delay = self.max_delay;
        }
        if self.jitter {
            delay = self.add_jitter(delay);
        }
        delay
    }
    
    fn add_jitter(&self, delay: Duration) -> Duration {
        // Add random jitter ±25%
        use rand::Rng;
        let jitter = rand::thread_rng().gen_range(-0.25..=0.25);
        delay + delay.mul_f64(jitter)
    }
}

pub struct CircuitBreaker {
    failure_threshold: u32,
    timeout: Duration,
    half_open_attempts: u32,
    state: CircuitState,
}

enum CircuitState {
    Closed,
    Open { opened_at: std::time::Instant },
    HalfOpen { attempts: u32 },
}
```

**Dependencies:**
```toml
rand = "0.8"  # For jitter
```

**Integration:**
Update `hazler-http` client to use retry policy:
```rust
impl HttpClient {
    pub async fn fetch_with_retry(&self, url: &Url) -> Result<Response> {
        let retry_policy = RetryPolicy::exponential_backoff();
        retry_policy.execute(|| {
            Box::pin(self.fetch(url))
        }).await
    }
}
```

---

### 1.3 Priority Queue System (MEDIUM PRIORITY)

**Problem:** Treats all URLs equally, doesn't prioritize interesting endpoints.

**Solution:** Score-based priority queue with heuristics.

**Architecture:**
```rust
// Enhance: crates/hazler-core/src/queue.rs

use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct ScoredUrl {
    pub url: Url,
    pub depth: usize,
    pub score: f64,  // Higher = more interesting
}

impl Ord for ScoredUrl {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher scores first
        other.score.partial_cmp(&self.score).unwrap()
    }
}

pub struct PriorityUrlQueue {
    queue: BinaryHeap<ScoredUrl>,
    visited: HashSet<String>,
}

impl PriorityUrlQueue {
    pub fn calculate_score(url: &Url) -> f64 {
        let mut score = 0.0;
        
        // API endpoints are interesting
        if url.path().contains("/api/") {
            score += 10.0;
        }
        
        // Admin panels are interesting
        if url.path().contains("/admin") || url.path().contains("/dashboard") {
            score += 15.0;
        }
        
        // JSON endpoints are interesting
        if url.path().ends_with(".json") {
            score += 8.0;
        }
        
        // GraphQL is interesting
        if url.path().contains("graphql") {
            score += 12.0;
        }
        
        // Shorter paths are generally more interesting
        let path_depth = url.path().split('/').count();
        score += 5.0 / path_depth as f64;
        
        // Common boring extensions get negative score
        if url.path().ends_with(".jpg") || 
           url.path().ends_with(".png") || 
           url.path().ends_with(".css") {
            score -= 10.0;
        }
        
        score
    }
}
```

**Configuration:**
Add to `Config`:
```rust
pub struct Config {
    // ... existing fields
    pub use_priority_queue: bool,
    pub custom_scoring: Option<Box<dyn Fn(&Url) -> f64>>,
}
```

---

### 1.4 Crawl State Persistence (MEDIUM PRIORITY)

**Problem:** Cannot resume interrupted crawls.

**Solution:** Periodic state snapshots to disk.

**Architecture:**
```rust
// New module: crates/hazler-core/src/persistence.rs

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct CrawlState {
    pub start_url: String,
    pub visited: HashSet<String>,
    pub queue: Vec<(String, usize)>,  // (url, depth)
    pub pages: Vec<Page>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct StateManager {
    path: PathBuf,
}

impl StateManager {
    pub fn new(path: PathBuf) -> Self { ... }
    
    pub fn save(&self, state: &CrawlState) -> Result<()> {
        let json = serde_json::to_string_pretty(state)?;
        std::fs::write(&self.path, json)?;
        Ok(())
    }
    
    pub fn load(&self) -> Result<CrawlState> {
        let json = std::fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&json)?)
    }
    
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}
```

**CLI Integration:**
```bash
# Save state every 100 pages
hazler https://example.com --state-file crawl.state --save-interval 100

# Resume from saved state
hazler https://example.com --resume crawl.state
```

---

## 2. hazler-http Enhancements

### 2.1 Advanced WAF Evasion (CRITICAL PRIORITY)

**Problem:** Gets blocked by Cloudflare, Akamai, and other WAFs.

**Solution:** Comprehensive browser fingerprint emulation.

**Architecture:**
```rust
// New module: crates/hazler-http/src/stealth.rs

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE};
use rand::seq::SliceRandom;

pub struct StealthConfig {
    user_agents: Vec<String>,
    accept_languages: Vec<String>,
    accept_encodings: Vec<String>,
    rotate_headers: bool,
    random_delay: Option<(u64, u64)>,  // (min_ms, max_ms)
}

impl StealthConfig {
    pub fn aggressive() -> Self {
        Self {
            user_agents: Self::browser_user_agents(),
            accept_languages: vec![
                "en-US,en;q=0.9".to_string(),
                "en-GB,en;q=0.9".to_string(),
                "fr-FR,fr;q=0.9".to_string(),
            ],
            accept_encodings: vec![
                "gzip, deflate, br".to_string(),
            ],
            rotate_headers: true,
            random_delay: Some((100, 500)),
        }
    }
    
    fn browser_user_agents() -> Vec<String> {
        vec![
            // Chrome on Windows
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            // Chrome on macOS
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            // Firefox on Windows
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0".to_string(),
            // Safari on macOS
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15".to_string(),
            // Edge on Windows
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0".to_string(),
        ]
    }
    
    pub fn generate_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        
        // Randomize User-Agent
        if let Some(ua) = self.user_agents.choose(&mut rand::thread_rng()) {
            headers.insert(USER_AGENT, HeaderValue::from_str(ua).unwrap());
        }
        
        // Add realistic browser headers
        headers.insert(ACCEPT, HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"
        ));
        
        if let Some(lang) = self.accept_languages.choose(&mut rand::thread_rng()) {
            headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_str(lang).unwrap());
        }
        
        // Chrome-specific headers
        headers.insert("sec-ch-ua", HeaderValue::from_static(
            r#""Not_A Brand";v="8", "Chromium";v="120", "Google Chrome";v="120""#
        ));
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        headers.insert("sec-ch-ua-platform", HeaderValue::from_static(r#""Windows""#));
        headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
        headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
        headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
        headers.insert("upgrade-insecure-requests", HeaderValue::from_static("1"));
        
        headers
    }
    
    pub async fn apply_random_delay(&self) {
        if let Some((min, max)) = self.random_delay {
            use tokio::time::{sleep, Duration};
            use rand::Rng;
            let delay = rand::thread_rng().gen_range(min..=max);
            sleep(Duration::from_millis(delay)).await;
        }
    }
}
```

**Integration:**
```rust
impl HttpClient {
    pub fn with_stealth(mut self, config: StealthConfig) -> Self {
        self.stealth_config = Some(config);
        self
    }
    
    pub async fn fetch(&self, url: &Url) -> Result<Response> {
        if let Some(ref stealth) = self.stealth_config {
            stealth.apply_random_delay().await;
            let headers = stealth.generate_headers();
            // Use these headers in request
        }
        // ... rest of fetch logic
    }
}
```

---

### 2.2 Proxy Pool Manager (HIGH PRIORITY)

**Problem:** No proxy support means easy detection and IP banning.

**Solution:** Proxy pool with health checking and rotation.

**Architecture:**
```rust
// New module: crates/hazler-http/src/proxy.rs

use reqwest::Proxy;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub url: String,
    pub proxy_type: ProxyType,
    pub health: ProxyHealth,
}

#[derive(Debug, Clone)]
pub enum ProxyType {
    Http,
    Socks5,
    Https,
}

#[derive(Debug, Clone)]
pub struct ProxyHealth {
    pub success_count: u32,
    pub failure_count: u32,
    pub last_used: Option<std::time::Instant>,
}

pub struct ProxyPool {
    proxies: Arc<RwLock<Vec<ProxyConfig>>>,
    current_index: Arc<RwLock<usize>>,
}

impl ProxyPool {
    pub fn from_file(path: &Path) -> Result<Self> {
        // Read proxy list from file (one per line)
        // Format: socks5://localhost:1080
        let content = std::fs::read_to_string(path)?;
        let proxies = content.lines()
            .filter(|line| !line.is_empty())
            .map(|line| ProxyConfig {
                url: line.to_string(),
                proxy_type: Self::detect_type(line),
                health: ProxyHealth {
                    success_count: 0,
                    failure_count: 0,
                    last_used: None,
                },
            })
            .collect();
        
        Ok(Self {
            proxies: Arc::new(RwLock::new(proxies)),
            current_index: Arc::new(RwLock::new(0)),
        })
    }
    
    pub async fn next_proxy(&self) -> Option<ProxyConfig> {
        let proxies = self.proxies.read().await;
        if proxies.is_empty() {
            return None;
        }
        
        let mut index = self.current_index.write().await;
        let proxy = proxies[*index].clone();
        *index = (*index + 1) % proxies.len();
        
        Some(proxy)
    }
    
    pub async fn mark_success(&self, proxy_url: &str) {
        let mut proxies = self.proxies.write().await;
        if let Some(proxy) = proxies.iter_mut().find(|p| p.url == proxy_url) {
            proxy.health.success_count += 1;
            proxy.health.last_used = Some(std::time::Instant::now());
        }
    }
    
    pub async fn mark_failure(&self, proxy_url: &str) {
        let mut proxies = self.proxies.write().await;
        if let Some(proxy) = proxies.iter_mut().find(|p| p.url == proxy_url) {
            proxy.health.failure_count += 1;
            // Remove proxy if failure rate is too high
            if proxy.health.failure_count > 10 {
                proxies.retain(|p| p.url != proxy_url);
            }
        }
    }
    
    pub async fn health_check(&self) {
        // Test each proxy with a simple request
        // Remove dead proxies
    }
}
```

**CLI Integration:**
```bash
# Single proxy
hazler https://example.com --proxy socks5://localhost:1080

# Proxy pool (round-robin)
hazler https://example.com --proxy-file proxies.txt --rotate-proxy
```

---

### 2.3 Authentication Manager (MEDIUM PRIORITY)

**Problem:** Cannot crawl authenticated areas.

**Solution:** Flexible authentication framework.

**Architecture:**
```rust
// New module: crates/hazler-http/src/auth.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    Basic { username: String, password: String },
    Bearer { token: String },
    Cookie { cookies: HashMap<String, String> },
    OAuth { /* OAuth config */ },
    Custom { headers: HashMap<String, String> },
}

pub struct AuthManager {
    method: AuthMethod,
    token_refresh: Option<Box<dyn Fn() -> String>>,
}

impl AuthManager {
    pub fn new(method: AuthMethod) -> Self {
        Self {
            method,
            token_refresh: None,
        }
    }
    
    pub fn apply_to_headers(&self, headers: &mut HeaderMap) {
        match &self.method {
            AuthMethod::Basic { username, password } => {
                let credentials = format!("{}:{}", username, password);
                let encoded = base64::encode(credentials);
                headers.insert(
                    "Authorization",
                    HeaderValue::from_str(&format!("Basic {}", encoded)).unwrap()
                );
            }
            AuthMethod::Bearer { token } => {
                headers.insert(
                    "Authorization",
                    HeaderValue::from_str(&format!("Bearer {}", token)).unwrap()
                );
            }
            AuthMethod::Cookie { cookies } => {
                let cookie_str = cookies.iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("; ");
                headers.insert(
                    "Cookie",
                    HeaderValue::from_str(&cookie_str).unwrap()
                );
            }
            AuthMethod::Custom { headers: custom_headers } => {
                for (key, value) in custom_headers {
                    headers.insert(
                        key.parse().unwrap(),
                        HeaderValue::from_str(value).unwrap()
                    );
                }
            }
            _ => {}
        }
    }
}
```

**CLI Integration:**
```bash
# Basic auth
hazler https://example.com --auth-basic user:pass

# Bearer token
hazler https://example.com --auth-bearer "eyJhbGc..."

# Cookie
hazler https://example.com --auth-cookie "session=abc123"

# Auth file (JSON)
hazler https://example.com --auth-file auth.json
```

---

## 3. hazler-parser Enhancements

### 3.1 GraphQL Intelligence (CRITICAL PRIORITY)

**Problem:** Cannot discover or analyze GraphQL endpoints.

**Solution:** GraphQL detection and introspection.

**Architecture:**
```rust
// New module: crates/hazler-parser/src/graphql.rs

use serde_json::Value;

pub struct GraphQLParser {
    introspection_query: String,
}

impl GraphQLParser {
    pub fn new() -> Self {
        Self {
            introspection_query: r#"
                query IntrospectionQuery {
                  __schema {
                    queryType { name }
                    mutationType { name }
                    types {
                      name
                      kind
                      fields {
                        name
                        type { name kind }
                      }
                    }
                  }
                }
            "#.to_string(),
        }
    }
    
    pub fn detect_graphql(body: &str) -> bool {
        // Check for GraphQL indicators
        body.contains("graphql") ||
        body.contains("__schema") ||
        body.contains("query ") ||
        body.contains("mutation ")
    }
    
    pub async fn introspect(&self, endpoint: &Url) -> Result<GraphQLSchema> {
        // Send introspection query
        // Parse response
        // Extract schema
    }
    
    pub fn generate_queries(&self, schema: &GraphQLSchema) -> Vec<String> {
        // Generate sample queries for each type
        // Useful for fuzzing
    }
}

#[derive(Debug)]
pub struct GraphQLSchema {
    pub types: Vec<GraphQLType>,
    pub queries: Vec<String>,
    pub mutations: Vec<String>,
}
```

**Integration:**
When a URL contains "graphql", automatically try introspection.

---

### 3.2 Multi-Format Parser (HIGH PRIORITY)

**Problem:** Only parses HTML, misses XML/RSS/JSON APIs.

**Solution:** Universal parser supporting multiple formats.

**Architecture:**
```rust
// Enhance: crates/hazler-parser/src/parser.rs

pub enum ContentType {
    Html,
    Json,
    Xml,
    Rss,
    Atom,
    Plain,
}

impl HtmlParser {
    pub fn parse_universal(&self, content_type: &str, body: &str) -> Vec<Url> {
        match self.detect_content_type(content_type, body) {
            ContentType::Html => self.parse_html(body),
            ContentType::Json => self.parse_json(body),
            ContentType::Xml | ContentType::Rss | ContentType::Atom => self.parse_xml(body),
            ContentType::Plain => self.parse_text(body),
        }
    }
    
    fn parse_json(&self, body: &str) -> Vec<Url> {
        let mut urls = Vec::new();
        
        // Parse JSON and extract all string values that look like URLs
        if let Ok(value) = serde_json::from_str::<Value>(body) {
            self.extract_urls_from_json(&value, &mut urls);
        }
        
        urls
    }
    
    fn extract_urls_from_json(&self, value: &Value, urls: &mut Vec<Url>) {
        match value {
            Value::String(s) => {
                if let Ok(url) = Url::parse(s) {
                    urls.push(url);
                } else if s.starts_with('/') {
                    // Relative URL
                    urls.push(s.clone());
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    self.extract_urls_from_json(item, urls);
                }
            }
            Value::Object(map) => {
                for (_, val) in map {
                    self.extract_urls_from_json(val, urls);
                }
            }
            _ => {}
        }
    }
    
    fn parse_xml(&self, body: &str) -> Vec<Url> {
        // Use quick-xml or similar
        // Extract href, src attributes
        // Extract text content that looks like URLs
    }
}
```

---

### 3.3 Sitemap & robots.txt Parser (MEDIUM PRIORITY)

**Problem:** Doesn't automatically check common discovery files.

**Solution:** Auto-fetch and parse sitemap.xml and robots.txt.

**Architecture:**
```rust
// New module: crates/hazler-parser/src/discovery.rs

pub struct DiscoveryParser;

impl DiscoveryParser {
    pub async fn fetch_robots_txt(base_url: &Url) -> Result<Vec<Url>> {
        let robots_url = base_url.join("/robots.txt")?;
        let response = fetch(robots_url).await?;
        
        let mut urls = Vec::new();
        for line in response.text().await?.lines() {
            if line.starts_with("Sitemap:") {
                if let Some(url_str) = line.split_whitespace().nth(1) {
                    urls.push(Url::parse(url_str)?);
                }
            } else if line.starts_with("Disallow:") || line.starts_with("Allow:") {
                // Extract paths (these are interesting!)
                if let Some(path) = line.split_whitespace().nth(1) {
                    urls.push(base_url.join(path)?);
                }
            }
        }
        
        Ok(urls)
    }
    
    pub async fn fetch_sitemap(sitemap_url: &Url) -> Result<Vec<Url>> {
        // Parse XML sitemap
        // Extract all <loc> URLs
        // Handle sitemap indexes recursively
    }
}
```

**Integration:**
Automatically check robots.txt and sitemap.xml at crawl start.

---

## 4. hazler-js-parser Enhancements

### 4.1 Source Map Parser (CRITICAL PRIORITY)

**Problem:** Missing goldmine of information from source maps.

**Solution:** Automatic source map download and parsing.

**Architecture:**
```rust
// New module: crates/hazler-js-parser/src/sourcemap.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SourceMap {
    pub version: u32,
    pub sources: Vec<String>,
    pub names: Vec<String>,
    pub mappings: String,
    #[serde(rename = "sourcesContent")]
    pub sources_content: Option<Vec<String>>,
}

pub struct SourceMapParser;

impl SourceMapParser {
    pub async fn detect_and_fetch(js_url: &Url, js_content: &str) -> Option<SourceMap> {
        // Check for sourceMappingURL comment
        if let Some(map_url) = Self::extract_map_url(js_content) {
            // Resolve relative URL
            let full_url = js_url.join(&map_url).ok()?;
            
            // Fetch source map
            if let Ok(response) = fetch(full_url).await {
                if let Ok(map) = response.json::<SourceMap>().await {
                    return Some(map);
                }
            }
        }
        
        // Also try common patterns: app.js -> app.js.map
        let map_url = format!("{}.map", js_url);
        if let Ok(map_url) = Url::parse(&map_url) {
            if let Ok(response) = fetch(map_url).await {
                if let Ok(map) = response.json::<SourceMap>().await {
                    return Some(map);
                }
            }
        }
        
        None
    }
    
    fn extract_map_url(js_content: &str) -> Option<String> {
        // Look for: //# sourceMappingURL=app.js.map
        js_content.lines()
            .find(|line| line.contains("sourceMappingURL="))
            .and_then(|line| {
                line.split("sourceMappingURL=").nth(1)
                    .map(|s| s.trim().to_string())
            })
    }
    
    pub fn extract_file_paths(map: &SourceMap) -> Vec<String> {
        // Extract all file paths from sources
        // These reveal project structure!
        map.sources.iter()
            .filter(|s| !s.starts_with("node_modules"))
            .cloned()
            .collect()
    }
    
    pub fn extract_original_code(map: &SourceMap) -> Vec<(String, String)> {
        // If sourcesContent exists, return (filename, content) pairs
        if let Some(ref contents) = map.sources_content {
            map.sources.iter()
                .zip(contents.iter())
                .map(|(name, content)| (name.clone(), content.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }
}
```

**Integration:**
Automatically check for source maps when parsing JS files.

**CLI Output:**
```
[INFO] Found source map: app.js.map
[INFO] Discovered file structure:
  - src/components/Admin.tsx
  - src/api/endpoints.ts
  - src/utils/auth.ts
```

---

### 4.2 JS Beautifier Integration (HIGH PRIORITY)

**Problem:** Minified JS is hard to analyze.

**Solution:** Integrate JS beautifier.

**Architecture:**
```rust
// Add to: crates/hazler-js-parser/src/parser.rs

pub struct JavaScriptParser {
    // ... existing fields
    beautify: bool,
}

impl JavaScriptParser {
    pub fn beautify_if_minified(&self, js_content: &str) -> String {
        if self.is_minified(js_content) && self.beautify {
            self.beautify_js(js_content)
        } else {
            js_content.to_string()
        }
    }
    
    fn is_minified(&self, js: &str) -> bool {
        // Heuristics:
        // 1. Very long lines (>500 chars)
        // 2. Few newlines
        // 3. No indentation
        
        let lines: Vec<&str> = js.lines().collect();
        if lines.len() < 10 {
            return true; // Probably minified
        }
        
        let avg_line_length = js.len() / lines.len().max(1);
        avg_line_length > 200
    }
    
    fn beautify_js(&self, js: &str) -> String {
        // Option 1: Use external tool (prettier, js-beautify)
        // Call via subprocess
        
        // Option 2: Use Rust crate if available
        // Currently no good Rust beautifier, so use external tool
        
        use std::process::Command;
        
        // Write JS to temp file
        let temp_file = "/tmp/hazler_temp.js";
        std::fs::write(temp_file, js).ok();
        
        // Run prettier (must be installed)
        let output = Command::new("prettier")
            .args(&["--parser", "babel", temp_file])
            .output()
            .ok();
        
        if let Some(output) = output {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            js.to_string()
        }
    }
}
```

**Note:** For production, consider vendoring a JS beautifier or using WASM-based solution.

---

## 5. hazler-secrets Enhancements

### 5.1 Entropy-Based Detection (HIGH PRIORITY)

**Problem:** Only detects known patterns, misses custom secrets.

**Solution:** Shannon entropy analysis.

**Architecture:**
```rust
// Add to: crates/hazler-secrets/src/scanner.rs

use std::collections::HashMap;

impl SecretScanner {
    pub fn scan_with_entropy(&self, text: &str) -> Vec<Finding> {
        let mut findings = self.scan(text); // Existing pattern-based scan
        
        // Add entropy-based findings
        let entropy_findings = self.find_high_entropy_strings(text);
        findings.extend(entropy_findings);
        
        findings
    }
    
    fn find_high_entropy_strings(&self, text: &str) -> Vec<Finding> {
        let mut findings = Vec::new();
        
        // Extract potential secrets (alphanumeric strings 20-64 chars)
        let re = regex::Regex::new(r#"[A-Za-z0-9+/=]{20,64}"#).unwrap();
        
        for mat in re.find_iter(text) {
            let candidate = mat.as_str();
            let entropy = self.calculate_shannon_entropy(candidate);
            
            // High entropy suggests random/encrypted data
            if entropy > 4.5 {
                findings.push(Finding {
                    secret_type: "High-Entropy String".to_string(),
                    severity: Severity::Medium,
                    value: self.redact(candidate),
                    context: Some(self.extract_context(text, mat.start())),
                    entropy: Some(entropy),
                });
            }
        }
        
        findings
    }
    
    fn calculate_shannon_entropy(&self, s: &str) -> f64 {
        let mut frequencies: HashMap<char, usize> = HashMap::new();
        let len = s.len() as f64;
        
        for c in s.chars() {
            *frequencies.entry(c).or_insert(0) += 1;
        }
        
        let mut entropy = 0.0;
        for &count in frequencies.values() {
            let probability = count as f64 / len;
            entropy -= probability * probability.log2();
        }
        
        entropy
    }
    
    fn extract_context(&self, text: &str, pos: usize) -> String {
        // Extract 50 chars before and after
        let start = pos.saturating_sub(50);
        let end = (pos + 50).min(text.len());
        text[start..end].to_string()
    }
}
```

---

## 6. hazler-cli Enhancements

### 6.1 Tool Integration Formats (CRITICAL PRIORITY)

**Problem:** Output not compatible with popular security tools.

**Solution:** Add specialized output formats.

**Architecture:**
```rust
// Add to: crates/hazler-cli/src/output.rs

pub enum OutputFormat {
    Json,
    Jsonl,
    Tree,
    Csv,
    Urls,
    // New formats:
    Nuclei,
    Ffuf,
    Burp,
    Caido,
}

impl OutputWriter {
    pub fn write_nuclei_format(&self, result: &CrawlResult) -> Result<()> {
        // Nuclei template format
        println!("id: hazler-crawl");
        println!("info:");
        println!("  name: Hazler Discovered Endpoints");
        println!("  author: hazler");
        println!("  severity: info");
        println!("http:");
        for page in &result.pages {
            println!("  - method: GET");
            println!("    path:");
            println!("      - {}", page.url);
        }
        Ok(())
    }
    
    pub fn write_ffuf_format(&self, result: &CrawlResult) -> Result<()> {
        // Wordlist for ffuf
        let mut paths: HashSet<String> = HashSet::new();
        
        for page in &result.pages {
            if let Ok(url) = Url::parse(&page.url) {
                paths.insert(url.path().to_string());
            }
        }
        
        for path in paths {
            println!("{}", path);
        }
        Ok(())
    }
    
    pub fn write_burp_format(&self, result: &CrawlResult) -> Result<()> {
        // Burp Suite XML format
        println!(r#"<?xml version="1.0"?>"#);
        println!(r#"<items burpVersion="2023.12">"#);
        
        for page in &result.pages {
            println!(r#"  <item>"#);
            println!(r#"    <url>{}</url>"#, page.url);
            println!(r#"    <status>{}</status>"#, page.status_code);
            println!(r#"    <mimetype>{}</mimetype>"#, page.content_type.as_deref().unwrap_or(""));
            println!(r#"  </item>"#);
        }
        
        println!(r#"</items>"#);
        Ok(())
    }
}
```

**CLI Usage:**
```bash
# Generate Nuclei template
hazler https://example.com -o nuclei > endpoints.yaml
nuclei -t endpoints.yaml

# Generate ffuf wordlist
hazler https://example.com -o ffuf > paths.txt
ffuf -w paths.txt -u https://target.com/FUZZ

# Generate Burp import
hazler https://example.com -o burp > burp-import.xml
```

---

### 6.2 Pipeline Mode (HIGH PRIORITY)

**Problem:** Cannot integrate into Unix pipelines.

**Solution:** Accept stdin, output to stdout line-by-line.

**Architecture:**
```rust
// Add to: crates/hazler-cli/src/main.rs

async fn run_pipeline_mode(config: Config) -> Result<()> {
    use tokio::io::{stdin, AsyncBufReadExt, BufReader};
    
    let stdin = stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    
    // Process each URL from stdin
    while let Some(line) = lines.next_line().await? {
        let url = match Url::parse(&line.trim()) {
            Ok(url) => url,
            Err(e) => {
                eprintln!("Invalid URL: {}: {}", line, e);
                continue;
            }
        };
        
        // Crawl the URL
        let crawler = Crawler::new(config.clone())?;
        let result = crawler.crawl(url).await?;
        
        // Output results immediately (streaming)
        for page in result.pages {
            println!("{}", page.url);
        }
    }
    
    Ok(())
}
```

**CLI Usage:**
```bash
# Read URLs from stdin
cat urls.txt | hazler --pipeline | grep api

# Chain with other tools
echo "https://example.com" | hazler --pipeline -o jsonl | jq '.url'

# Combine with nuclei
hazler https://example.com -o urls | nuclei -l -
```

---

## 7. New Crate: hazler-browser

### Purpose
Headless browser integration for JavaScript-heavy sites.

### Architecture
```rust
// New crate: crates/hazler-browser/

use chromiumoxide::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::network::EventRequestWillBeSent;

pub struct HeadlessCrawler {
    browser: Browser,
    intercept_requests: bool,
}

impl HeadlessCrawler {
    pub async fn new() -> Result<Self> {
        let (browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .request_timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to launch browser: {}", e))?
        ).await?;
        
        // Spawn browser handler
        tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });
        
        Ok(Self {
            browser,
            intercept_requests: true,
        })
    }
    
    pub async fn crawl_spa(&self, url: &Url) -> Result<Vec<String>> {
        let page = self.browser.new_page(url.as_str()).await?;
        
        // Enable request interception to capture XHR/Fetch
        let mut requests = Vec::new();
        
        if self.intercept_requests {
            page.event_listener::<EventRequestWillBeSent>().await?;
            // Collect all network requests
        }
        
        // Wait for page to load
        page.wait_for_navigation().await?;
        
        // Extract all links
        let links = page.evaluate(r#"
            Array.from(document.querySelectorAll('a'))
                .map(a => a.href)
        "#).await?;
        
        Ok(requests)
    }
    
    pub async fn screenshot(&self, url: &Url, path: &Path) -> Result<()> {
        let page = self.browser.new_page(url.as_str()).await?;
        page.wait_for_navigation().await?;
        page.screenshot(chromiumoxide::page::ScreenshotParams::builder().build())
            .await?
            .save(path)?;
        Ok(())
    }
}
```

### Dependencies
```toml
[package]
name = "hazler-browser"
version = "0.1.0"
edition = "2021"

[dependencies]
chromiumoxide = "0.5"  # Or fantoccini for WebDriver approach
tokio = "1.35"
anyhow = "1.0"
url = "2.5"
```

### CLI Integration
```bash
# Use headless mode
hazler https://spa-app.com --headless

# Headless with screenshot
hazler https://example.com --headless --screenshot screenshots/
```

---

## Implementation Timeline

### Week 1-2: Critical Gaps
- [ ] hazler-http: Advanced WAF evasion
- [ ] hazler-cli: Tool integration formats (Nuclei, ffuf, Burp)
- [ ] hazler-cli: Pipeline mode

### Week 3-5: Browser Support
- [ ] Create hazler-browser crate
- [ ] Integrate chromiumoxide
- [ ] Add request interception
- [ ] CLI integration

### Week 6-7: Parser Enhancements
- [ ] hazler-parser: GraphQL intelligence
- [ ] hazler-parser: Multi-format parsing
- [ ] hazler-js-parser: Source map parser

### Week 8-9: Advanced Features
- [ ] hazler-core: Response diffing
- [ ] hazler-core: Smart retry/circuit breaker
- [ ] hazler-secrets: Entropy detection

### Week 10-12: Polish & Testing
- [ ] Comprehensive testing
- [ ] Performance optimization
- [ ] Documentation updates
- [ ] Example workflows

---

## Testing Strategy

### Unit Tests
Each module should have tests covering:
- Happy path
- Error cases
- Edge cases
- Performance benchmarks

### Integration Tests
- Full crawl workflows
- Tool chain integration (Nuclei, ffuf)
- Browser mode end-to-end
- Authentication flows

### Performance Tests
- Benchmark crawl speed (pages/sec)
- Memory usage under load
- Concurrent request handling
- Large-scale crawls (10k+ pages)

### Security Tests
- WAF evasion effectiveness
- Secret detection accuracy
- False positive rate
- Privacy (no data leaks)

---

## Success Criteria

### Phase 1 (Q1 2026)
- ✅ Headless browser working
- ✅ WAF evasion >90% success rate
- ✅ Tool integration (3+ formats)
- ✅ GraphQL support
- ✅ Source map parsing

### Phase 2 (Q2 2026)
- ✅ Response diffing functional
- ✅ Entropy-based secret detection
- ✅ Authentication framework
- ✅ All tests passing
- ✅ Documentation complete

### Phase 3 (Q3 2026)
- ✅ Production-ready stability
- ✅ Performance: 200+ pages/sec
- ✅ Community adoption: 1000+ stars
- ✅ Integration into bug bounty workflows

---

## Conclusion

These technical recommendations provide a **clear path** to transform Hazler into a **top-tier security reconnaissance tool**. The focus is on:

1. **Critical gaps** (browser, WAF, integrations) that are table-stakes
2. **Unique differentiators** (diffing, entropy, fuzzing) that set Hazler apart
3. **Practical implementation** with code examples and timelines

By following this roadmap, Hazler will go from "stable but ordinary" to **"indispensable for bug bounty hunters and penetration testers."**
