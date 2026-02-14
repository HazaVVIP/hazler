# Hazler Browser Module

The `hazler-browser` crate provides headless browser support for Hazler, enabling it to crawl modern JavaScript-heavy websites and single-page applications (SPAs).

## Features

### 🚀 Core Capabilities
- **Headless Chrome automation** via Chrome DevTools Protocol (CDP)
- **Network request interception** using `Network.requestWillBeSent` events
- **Automatic API discovery** - captures hidden endpoints, auth headers, and payloads
- **JavaScript execution** - renders SPAs like React, Vue, Angular
- **Screenshot capture** - visual verification of pages
- **Cookie management** - session handling
- **Link extraction** - discovers dynamically loaded links

### 💎 Security Features
Perfect for bug bounty hunters and penetration testers:
- **Hidden API endpoint discovery** - finds APIs that never appear in HTML
- **Authentication token capture** - automatically logs Bearer tokens, API keys
- **Payload logging** - records POST/PUT/PATCH request bodies
- **IDOR vulnerability detection** - identifies patterns in API requests
- **GraphQL query capture** - logs queries and mutations

## Architecture

```
hazler-browser/
├── src/
│   ├── lib.rs        # Public API exports
│   ├── browser.rs    # Core browser implementation with CDP
│   ├── types.rs      # Data structures (Config, Results, etc.)
│   └── error.rs      # Error types and handling
```

## Usage

### Basic Example

```rust
use hazler_browser::{Browser, BrowserConfig};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the browser
    let config = BrowserConfig {
        headless: true,
        timeout_secs: 30,
        window_width: 1920,
        window_height: 1080,
        intercept_requests: false,
        screenshot_path: Some("screenshots/".to_string()),
        user_agent: Some("Hazler/0.1.0".to_string()),
        disable_images: true,  // Faster loading
        disable_javascript: false,
    };

    // Launch browser
    let browser = Browser::new(config).await?;

    // Load a page
    let url = Url::parse("https://example.com")?;
    let result = browser.load_page(&url).await?;

    // Access results
    println!("Final URL: {}", result.url);
    println!("Links found: {}", result.links.len());
    println!("Network requests: {}", result.network_requests.len());

    // Check for API calls
    for req in result.network_requests {
        if req.url.contains("/api/") {
            println!("API endpoint: {} {}", req.method, req.url);
            
            // Check for auth headers
            if let Some(auth) = req.headers.get("authorization") {
                println!("  Auth: {}", auth);
            }
            
            // Check payload
            if let Some(payload) = req.post_data {
                println!("  Payload: {}", payload);
            }
        }
    }

    // Close browser
    browser.close().await?;
    
    Ok(())
}
```

### Integration with Hazler Core

```rust
use hazler_core::Config;
use hazler_core::Crawler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .max_depth(3)
        .concurrency(5)
        .headless_browser(true)  // Enable headless browser
        .screenshot_path("screenshots/".to_string())
        .disable_images(true);  // Faster loading

    let mut crawler = Crawler::new(config)?;
    
    // Initialize browser (async operation)
    #[cfg(feature = "browser")]
    crawler.init_browser().await?;

    let url = url::Url::parse("https://example.com")?;
    let result = crawler.crawl(url).await?;

    println!("Crawled {} pages", result.pages.len());
    
    Ok(())
}
```

## Network Request Interception

The browser module uses Chrome DevTools Protocol's `Network.requestWillBeSent` event to capture ALL network activity:

```rust
// Automatically captures:
// - XHR/Fetch requests
// - API endpoints
// - Authentication headers
// - Request payloads
// - GraphQL queries
// - WebSocket connections

let result = browser.load_page(&url).await?;

for req in result.network_requests {
    match req.resource_type.as_str() {
        "XHR" | "Fetch" => {
            println!("🔍 API Request: {} {}", req.method, req.url);
            println!("   Headers: {:?}", req.headers);
            println!("   Payload: {:?}", req.post_data);
        }
        _ => {}
    }
}
```

## Configuration Options

### BrowserConfig

```rust
pub struct BrowserConfig {
    /// Run in headless mode (default: true)
    pub headless: bool,
    
    /// Request timeout in seconds (default: 30)
    pub timeout_secs: u64,
    
    /// Window width (default: 1920)
    pub window_width: u32,
    
    /// Window height (default: 1080)
    pub window_height: u32,
    
    /// Enable network request interception (experimental)
    pub intercept_requests: bool,
    
    /// Path to save screenshots (optional)
    pub screenshot_path: Option<String>,
    
    /// Custom user agent (optional)
    pub user_agent: Option<String>,
    
    /// Disable images for faster loading
    pub disable_images: bool,
    
    /// Disable JavaScript (not recommended)
    pub disable_javascript: bool,
}
```

## Data Structures

### NetworkRequest

Captured network request with full details:

```rust
pub struct NetworkRequest {
    pub url: String,              // Request URL
    pub method: String,           // HTTP method (GET, POST, etc.)
    pub headers: HashMap<String, String>,  // All headers
    pub post_data: Option<String>,  // Request payload
    pub resource_type: String,    // XHR, Fetch, Document, etc.
    pub request_id: String,       // Unique request ID
    pub timestamp: f64,           // Request timestamp
}
```

### PageLoadResult

Complete page load result:

```rust
pub struct PageLoadResult {
    pub url: Url,                    // Final URL (after redirects)
    pub status_code: u16,            // HTTP status
    pub links: Vec<String>,          // Extracted links
    pub title: Option<String>,       // Page title
    pub screenshot_data: Option<Vec<u8>>,  // Screenshot bytes
    pub cookies: Vec<Cookie>,        // Page cookies
    pub network_requests: Vec<NetworkRequest>,  // All requests
}
```

## Requirements

- Chrome or Chromium browser installed on the system
- Chrome must be available in PATH or specified via environment variable

## Performance Tips

1. **Disable images** for faster page loading:
   ```rust
   config.disable_images = true;
   ```

2. **Reduce timeout** for faster failures:
   ```rust
   config.timeout_secs = 10;
   ```

3. **Use headless mode** (default) for better performance:
   ```rust
   config.headless = true;
   ```

4. **Wait time**: Browser waits 3 seconds after navigation for dynamic content. Adjust in code if needed.

## Troubleshooting

### Chrome not found
```
Error: Failed to launch browser: Could not find Chrome executable
```

**Solution**: Install Chrome/Chromium or set `CHROME_PATH` environment variable:
```bash
export CHROME_PATH=/path/to/chrome
```

### Timeouts
```
Navigation timeout after 30 seconds
```

**Solution**: Increase timeout or check network connection:
```rust
config.timeout_secs = 60;
```

### Memory usage
For large crawls, the browser can consume significant memory. Consider:
- Crawling in batches
- Reducing concurrency
- Closing and reopening browser periodically

## Security Considerations

- Browser runs with `--no-sandbox` flag for Docker compatibility
- Be cautious when crawling untrusted sites
- Review captured credentials before logging/storing
- Screenshots may contain sensitive information

## Testing

Run tests (requires Chrome installed):
```bash
# Run all tests including browser tests
cargo test --features browser

# Skip browser tests (don't require Chrome)
cargo test --workspace
```

The main browser test is marked `#[ignore]` and requires Chrome to be installed.

## Future Enhancements

- [ ] Response body capture
- [ ] Advanced JavaScript evaluation
- [ ] Cookie injection
- [ ] Custom CDP commands
- [ ] Multiple browser instances
- [ ] Browser pool management
- [ ] Request/response modification
- [ ] WebSocket message capture

## License

MIT License - See LICENSE file for details

## Contributing

Contributions welcome! Please read CONTRIBUTING.md for guidelines.
